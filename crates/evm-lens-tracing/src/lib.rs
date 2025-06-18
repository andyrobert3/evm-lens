// fetch tx data
// fetch block
// run all tx until before the target tx
// use that as the db state
// wrap tracer inspector
// run inside revm
// output tracing info
// collect and segregate traces to it's individual calls
// display stuff

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::Ethereum,
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::{Block, Transaction},
    transports::{RpcError, TransportErrorKind},
};
use revm::{database::{AlloyDB, CacheDB, StateBuilder}, database_interface::WrapDatabaseAsync};

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


        todo!()
    }
}

pub async fn create_provider(url: &str) -> TracingResult<impl Provider> {
    ProviderBuilder::new()
        .connect(url)
        .await
        .map_err(|err| TracingError::Connection(err.to_string()))
}
