// fetch tx data
// run inside revm
// custom backend to fetch block info and storage slots
// output tracing info
// display stuff

use alloy::{
    network::Ethereum,
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
};

pub struct Tracer<P: Provider> {
    provider: P,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum TracingError {
    #[error("Failed to connect rpc provider : {0}")]
    Connection(String),
    #[error("Failed to fetch data from provider: {0}")]
    Io(String),
}

pub type TracingResult<T> = Result<T, TracingError>;
// pub type TxHash = Fix

impl<T: Provider> Tracer<T> {
    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    async fn fetch_tx_data(&self, hash: TxHash) {
        let tx = self.provider.get_transaction_by_hash(hash).await;
    }
}

pub async fn create_provider(url: &str) -> TracingResult<impl Provider> {
    ProviderBuilder::new()
        .connect(url)
        .await
        .map_err(|err| TracingError::Connection(err.to_string()))
}
