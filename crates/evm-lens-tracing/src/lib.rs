// fetch tx data
// run inside revm
// custom backend to fetch block info and storage slots
// output tracing info
// display stuff

use alloy::{
    network::Ethereum,
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::Transaction,
    transports::{RpcError, TransportErrorKind},
};

pub struct Tracer<P: Provider> {
    provider: P,
}

#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to connect rpc provider : {0}")]
    Connection(String),
    #[error("{0}")]
    Io(#[from] RpcError<TransportErrorKind>),
    #[error("Invalid transaction")]
    Invalid,
}

pub type TracingResult<T> = Result<T, TracingError>;

impl<T: Provider> Tracer<T> {
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
}

pub async fn create_provider(url: &str) -> TracingResult<impl Provider> {
    ProviderBuilder::new()
        .connect(url)
        .await
        .map_err(|err| TracingError::Connection(err.to_string()))
}
