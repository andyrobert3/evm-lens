
use std::{
    fmt::Debug,
    io::Write,
    sync::{Arc, Mutex},
};

use alloy::{
    consensus::Transaction,
    eips::{BlockId, BlockNumberOrTag},
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::{Block, BlockTransactions, Transaction as RpcTransaction},
    transports::{RpcError, TransportErrorKind},
};
use revm::{
    Context, ExecuteCommitEvm, InspectEvm, MainBuilder, MainContext,
    context::TxEnv,
    database::{AlloyDB, CacheDB, StateBuilder},
    database_interface::WrapDatabaseAsync,
    inspector::inspectors::TracerEip3155,
    primitives::TxKind,
};

use crate::{
    item::{OrderedTrace, TraceKind},
    sort::SortMarker,
};

pub mod item;

pub mod sort {
    pub trait SortMarker {
        fn sort(&mut self);
    }
}

/// used to collect traces from inspector
#[derive(Debug)]
pub struct Traces<S: sort::SortMarker> {
    buff: Arc<Mutex<Vec<S>>>,
}

impl<S: sort::SortMarker> Traces<S> {
    pub fn into_inner(self) -> Vec<S> {
        let buff = Arc::into_inner(self.buff).unwrap();
        buff.into_inner().unwrap()
    }
}

impl<S: Clone + SortMarker> Clone for Traces<S> {
    fn clone(&self) -> Self {
        Traces {
            buff: self.buff.clone(),
        }
    }
}

impl SortMarker for Traces<TraceKind> {
    fn sort(&mut self) {
        let mut inner = self.buff.lock().unwrap();

        let mut tmp = vec![];

        for unordered_trace in inner.iter() {
            let TraceKind::Output(trace) = unordered_trace.to_owned() else {
                continue;
            };

            tmp.push(trace);
        }

        let ordered = OrderedTrace::sort(tmp);
        let buff = ordered
            .into_iter()
            .map(TraceKind::Ordered)
            .collect::<Vec<_>>();

        *inner = buff
    }
}

impl<S: sort::SortMarker> Default for Traces<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: sort::SortMarker> Traces<S> {
    pub fn new() -> Self {
        Self {
            buff: Default::default(),
        }
    }
}

/// as long as you implement [SortMarker] and [TryFrom<&[u8]>] you're good
impl<S> Write for Traces<S>
where
    S: Debug + SortMarker + for<'a> TryFrom<&'a [u8]>,
    for<'a> <S as TryFrom<&'a [u8]>>::Error: Debug,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // handles new line being written by the tracer
        // we don't actually need the new line since we're writing to memory
        if buf.len() == 1 {
            return Ok(1);
        }

        let trace = S::try_from(buf).expect("internal serialization must succeed");

        let mut buff = self
            .buff
            .lock()
            .expect("no other thread should hold the lock");

        buff.push(trace);

        Ok(buf.len())
    }

    /// we dont write stuff onto disk so this'll be a noop
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Tracer<P>
where
    P: Provider + Clone,
{
    provider: P,
}

#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to connect rpc provider : {0}")]
    Connection(String),
    #[error("{0}")]
    Io(#[from] RpcError<TransportErrorKind>),
    #[error("Invalid transaction or block")]
    Invalid,
    #[error("{0}")]
    Other(String),
}

pub type TracingResult<T> = Result<T, TracingError>;

impl<T> Tracer<T>
where
    T: Provider + Clone,
{
    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    async fn fetch_tx_data(&self, hash: TxHash) -> TracingResult<RpcTransaction> {
        self.provider
            .get_transaction_by_hash(hash)
            .await
            .map_err(TracingError::Io)?
            .ok_or(TracingError::Invalid)
    }

    async fn fetch_block_full(&self, block: BlockNumberOrTag) -> TracingResult<Block> {
        self.provider
            .get_block_by_number(block)
            .full()
            .await
            .map_err(TracingError::Io)?
            .ok_or(TracingError::Invalid)
    }

    pub async fn trace(&self, hash: TxHash) -> TracingResult<Vec<TraceKind>> {
        let chain_id = self.provider.get_chain_id().await?;
        let tx = self.fetch_tx_data(hash.to_owned()).await?;

        let Some(block) = &tx.block_number else {
            return Err(TracingError::Other(String::from(
                "Can't trace pending transaction",
            )));
        };
        let block_ident = BlockNumberOrTag::Number(block.to_owned());
        let block = self.fetch_block_full(block_ident).await?;

        let state_db = AlloyDB::new(self.provider.clone(), BlockId::Number(block_ident));
        let state_db = WrapDatabaseAsync::new(state_db).ok_or(TracingError::Other(
            "for some reason no tokio rt is found :(".to_string(),
        ))?;
        let state_db = CacheDB::new(state_db);
        let mut state = StateBuilder::new_with_database(state_db).build();

        let ctx = Context::mainnet()
            .with_db(&mut state)
            .modify_block_chained(|b| {
                b.number = block.header.number;
                b.beneficiary = block.header.beneficiary;
                b.timestamp = block.header.timestamp;

                b.difficulty = block.header.difficulty;
                b.gas_limit = block.header.gas_limit;
                b.basefee = block.header.base_fee_per_gas.unwrap_or_default();
            })
            .modify_cfg_chained(|c| {
                c.chain_id = chain_id;
            });

        // fokin ugly
        let mut buff_writer = Box::new(Traces::<item::TraceKind>::new());

        let mut evm = ctx.build_mainnet_with_inspector(TracerEip3155::new(buff_writer.clone()));

        let BlockTransactions::Full(transactions) = block.transactions else {
            return Err(TracingError::Invalid);
        };

        for tx in transactions {
            let reconstructed_tx = TxEnv {
                caller: tx.inner.signer(),
                gas_limit: tx.gas_limit(),
                gas_price: tx.gas_price().unwrap_or(tx.inner.max_fee_per_gas()),
                value: tx.value(),
                data: tx.input().to_owned(),
                gas_priority_fee: tx.max_priority_fee_per_gas(),
                chain_id: Some(chain_id),
                nonce: tx.nonce(),
                access_list: tx.access_list().cloned().unwrap_or_default(),
                kind: match tx.to() {
                    Some(to_address) => TxKind::Call(to_address),
                    None => TxKind::Create,
                },
                ..Default::default()
            };

            // inspect the tx if it's the same hash
            if tx.info().hash.unwrap() == hash {
                let _ = evm.inspect_with_tx(reconstructed_tx);
                break;
            } else {
                let _ = evm.transact_commit(reconstructed_tx);
            }
        }

        // we need to explicitly drop evm here so that the traces doesn't have any reference associated into it
        drop(evm);

        let mut traces = buff_writer;
        SortMarker::sort(&mut *traces);
        let traces = traces.into_inner();

        Ok(traces)
    }
}

pub async fn create_provider(url: &str) -> TracingResult<impl Provider> {
    ProviderBuilder::new()
        .connect(url)
        .await
        .map_err(|err| TracingError::Connection(err.to_string()))
}
