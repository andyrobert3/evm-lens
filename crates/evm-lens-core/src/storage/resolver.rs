use async_trait::async_trait;

use super::layout::StorageLayout;

#[async_trait]
pub trait StorageLayoutResolver: Send + Sync {
    async fn resolve(&self, input: &[u8]) -> color_eyre::Result<Option<StorageLayout>>;
}

/// Composite resolver that tries multiple resolvers in order and returns the first non-None
/// layout. If none succeed, returns an empty layout.
pub struct CompositeResolver {
    resolvers: Vec<Box<dyn StorageLayoutResolver>>,
}

impl CompositeResolver {
    pub fn new(resolvers: Vec<Box<dyn StorageLayoutResolver>>) -> Self {
        Self { resolvers }
    }

    pub async fn resolve(&self, input: &[u8]) -> color_eyre::Result<StorageLayout> {
        for r in &self.resolvers {
            if let Some(l) = r.resolve(input).await? {
                return Ok(l);
            }
        }
        Ok(StorageLayout::new())
    }
}
