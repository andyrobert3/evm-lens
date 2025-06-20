// fetch tx data
// fetch block
// run all tx until before the target tx
// use that as the db state
// wrap tracer inspector
// run inside revm
// output tracing info
// collect and segregate traces to it's individual calls
// display stuff

use std::io::Write;

use alloy::{
    consensus::Transaction,
    eips::{BlockId, BlockNumberOrTag},
    network::Ethereum,
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::{Block, BlockTransactions, Transaction},
    transports::{RpcError, TransportErrorKind},
};
use revm::{
    database::{AlloyDB, CacheDB, StateBuilder}, database_interface::WrapDatabaseAsync, inspector::inspectors::TracerEip3155, primitives::U256, Context, MainBuilder, MainContext
};

use crate::sort::SortMarker;

pub mod item;


pub mod sort {

    pub struct Sorted;
    pub struct Unsorted;
    
    pub trait SortMarker {}
    impl SortMarker for Sorted{}
    impl SortMarker for Unsorted{}
    
}

/// used to collect traces from inspector
#[derive(Clone)]
pub struct Traces<S: sort::SortMarker>{
    buff : Vec<>
}

pub enum TraceKind {
Summary()
}


impl<S:SortMarker> Write for Traces<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // handles new line being written by the tracer
        // we don't actually need the new line since we're writing to memory
        if buf.len() == 1 {
            return Ok(1)
        }




        
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
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

    async fn fetch_tx_data(&self, hash: TxHash) -> TracingResult<Transaction> {
        self.provider
            .get_transaction_by_hash(hash)
            .await
            .map_err(|err| TracingError::Io(err))?
            .ok_or(TracingError::Invalid)
    }

    async fn fetch_block_full(&self, block: BlockNumberOrTag) -> TracingResult<Block> {
        self.provider
            .get_block_by_number(block)
            .full()
            .await
            .map_err(|err| TracingError::Io(err))?
            .ok_or(TracingError::Invalid)
    }

    pub async fn trace(&self, hash: TxHash) -> TracingResult<()> {
        let chain_id = self.provider.get_chain_id().await?;
        let tx = self.fetch_tx_data(hash).await?;

        let Some(block) = &tx.block_number else {
            return Err(TracingError::Other(String::from(
                "Can't trace pending transaction",
            )));
        };
        let block_ident = BlockNumberOrTag::Number(block.to_owned());
        let block = self.fetch_block_full(block_ident).await?;

        let state_db = AlloyDB::new(self.provider.clone(), BlockId::Number(block_ident));
        let state_db = WrapDatabaseAsync::new(state_db).ok_or(TracingError::Other(format!(
            "for some reason no tokio rt is found :("
        )))?;
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

            
                let mut evm = ctx.build_mainnet_with_inspector(TracerEip3155::new(Box::new(writer)));

        let BlockTransactions::Full(transactions) = block.transactions else {
            return Err(TracingError::Invalid);
        };

        for tx in transactions {
            // Construct the file writer to write the trace to
            let tx_number = tx.transaction_index.unwrap_or_default();

            let tx = TxEnv {
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
        }

        todo!()
    }

    async fn inspect() {}
}

pub async fn create_provider(url: &str) -> TracingResult<impl Provider> {
    ProviderBuilder::new()
        .connect(url)
        .await
        .map_err(|err| TracingError::Connection(err.to_string()))
}
