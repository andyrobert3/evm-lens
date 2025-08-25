use async_trait::async_trait;

use crate::storage::layout::{Provenance, StorageEntry, StorageLayout, StorageType};
use crate::storage::resolver::StorageLayoutResolver;

/// Minimal placeholder for a metadata-based resolver.
/// v0.3 aims for conservative approach: try to parse embedded metadata if present.
pub struct MetadataResolver;

#[async_trait]
impl StorageLayoutResolver for MetadataResolver {
    async fn resolve(&self, _input: &[u8]) -> color_eyre::Result<Option<StorageLayout>> {
        // For v0.3.0, we keep this minimal. Real implementation would attempt to decode
        // compiler metadata and extract storage layout. Returning None means fallback.
        let _ = _input;
        Ok(None)
    }
}

#[allow(dead_code)]
fn layout_from_metadata_example() -> StorageLayout {
    let mut l = StorageLayout::new();
    l.add_entry(StorageEntry {
        slot: 0,
        offset: None,
        size: None,
        r#type: StorageType::Uint { bits: 256 },
        label: Some("example".to_string()),
        provenance: Provenance::CompilerMetadata,
    });
    l
}
