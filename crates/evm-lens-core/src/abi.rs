use std::{collections::HashMap, num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};
use tokio::time::sleep;

use async_trait::async_trait;
use log::error;
use lru::LruCache;
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use tokio::{fs, sync::RwLock, task};

/// Maximum number of selectors kept in the in‑memory LRU.
const DEFAULT_CACHE_SIZE: usize = 5_000;

/// Short alias for a 4‑byte function selector.
pub type Selector = [u8; 4];

/// Information about one human‑readable function signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigInfo {
    /// e.g. "transfer(address,uint256)"
    pub text: String,
    pub selector: Selector,
}

/// Errors that can occur during resolution.
#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    #[error("network error")]
    Network(#[from] ReqwestError),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("task join error")]
    Join(#[from] task::JoinError),
}

/// Async interface for mapping a 4‑byte selector → signatures.
#[async_trait]
pub trait SelectorResolver: Send + Sync {
    async fn resolve(&self, selector: Selector) -> Result<Arc<[SigInfo]>, ResolveError>;
}

/// “Do‑nothing” implementation, returns an empty slice.
pub struct NullResolver;

#[async_trait]
impl SelectorResolver for NullResolver {
    async fn resolve(&self, _selector: Selector) -> Result<Arc<[SigInfo]>, ResolveError> {
        Ok(Arc::from([]))
    }
}

/// LRU‑cached + on‑disk‑persisted resolver that falls back to 4byte.directory.
#[derive(Clone)]
pub struct CompositeResolver {
    cache: Arc<RwLock<LruCache<Selector, Arc<[SigInfo]>>>>,
    cache_file: PathBuf,
    client: Client,
    base_url: String,
}

#[derive(Deserialize)]
struct FourByteEntry {
    text_signature: String,
}

#[derive(Deserialize)]
struct FourByteResponse {
    #[serde(rename = "results")]
    results: Vec<FourByteEntry>,
}

impl CompositeResolver {
    /// Creates a new resolver. Will eagerly load the on‑disk JSON cache if present.
    pub async fn new(cache_file: PathBuf, base_url: Option<String>) -> Self {
        let map: HashMap<String, Vec<String>> = fs::read_to_string(&cache_file)
            .await
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let mut lru = LruCache::new(NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("non‑zero"));
        for (hex_sel, sigs) in map {
            if let Ok(bytes) = hex::decode(hex_sel) {
                if bytes.len() == 4 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(&bytes);
                    let arc = Arc::from(
                        sigs.into_iter()
                            .map(|text| SigInfo {
                                text,
                                selector: arr,
                            })
                            .collect::<Vec<_>>(),
                    );
                    lru.put(arr, arc);
                }
            }
        }

        Self {
            cache: Arc::new(RwLock::new(lru)),
            cache_file,
            client: Client::builder()
                .user_agent("evm-lens (+https://github.com/andyrobert3/evm-lens)")
                .build()
                .expect("reqwest client"),
            base_url: base_url.unwrap_or_else(|| "https://www.4byte.directory".to_string()),
        }
    }

    /// Write the current cache map back to disk (atomic rename).
    async fn persist(&self) -> Result<(), ResolveError> {
        // Snapshot the cache with a read‑lock then serialize outside the lock.
        let snapshot: HashMap<String, Vec<String>> = {
            let cache = self.cache.read().await;
            cache
                .iter()
                .map(|(sel, sigs)| {
                    let key = hex::encode(sel);
                    let val = sigs.iter().map(|s| s.text.clone()).collect();
                    (key, val)
                })
                .collect()
        };

        let tmp = self.cache_file.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(&snapshot)?;

        // Use a blocking task for disk I/O.
        let tmp_clone = tmp.clone();
        let cache_file = self.cache_file.clone();
        task::spawn_blocking(move || {
            std::fs::write(&tmp_clone, &json)?;
            std::fs::rename(&tmp_clone, &cache_file)?;
            Ok::<(), std::io::Error>(())
        })
        .await??;

        Ok(())
    }

    /// Query 4byte.directory. Returns an empty vec if the selector is unknown or the
    /// request fails.
    async fn fetch_remote(&self, selector: Selector) -> Result<Arc<[SigInfo]>, ResolveError> {
        let url = format!(
            "{}/api/v1/signatures/?hex_signature=0x{}",
            self.base_url,
            hex::encode(selector)
        );

        let body = self
            .client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?
            .error_for_status()?
            .json::<FourByteResponse>()
            .await?;

        // Add rate limiting delay after successful request
        sleep(Duration::from_millis(200)).await;

        let sigs: Vec<SigInfo> = body
            .results
            .into_iter()
            .map(|e| SigInfo {
                text: e.text_signature,
                selector,
            })
            .collect();

        Ok(Arc::from(sigs))
    }
}

#[async_trait]
impl SelectorResolver for CompositeResolver {
    async fn resolve(&self, selector: Selector) -> Result<Arc<[SigInfo]>, ResolveError> {
        // Fast path – LRU hit
        if let Some(hit) = self.cache.read().await.peek(&selector).cloned() {
            return Ok(hit);
        }

        // Miss → fetch
        let sigs = self.fetch_remote(selector).await?;

        if !sigs.is_empty() {
            self.cache.write().await.put(selector, sigs.clone());
            // Fire‑and‑forget persistence so we don't block the caller.
            let slf = self.clone();
            tokio::spawn(async move {
                if let Err(e) = slf.persist().await {
                    error!("persisting cache: {e}");
                }
            });
        }

        Ok(sigs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use serde_json::json;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_null_resolver() {
        let resolver = NullResolver;
        let selector = [0xde, 0xad, 0xbe, 0xef];
        let result = resolver.resolve(selector).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_composite_resolver_cache_miss_and_hit() {
        let selector = [0xca, 0xfe, 0xba, 0xbe];
        let sig_text = "cafeBabe()";
        let mock_body = json!({
            "count": 1,
            "next": null,
            "previous": null,
            "results": [{ "text_signature": sig_text }]
        });

        let mut server = mockito::Server::new_async().await;
        let mock_api = server
            .mock("GET", "/api/v1/signatures/?hex_signature=0xcafebabe")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_body.to_string())
            .create_async()
            .await;

        let temp_dir = tempdir().unwrap();
        let cache_file = temp_dir.path().join("test_cache.json");

        let resolver = CompositeResolver::new(cache_file, Some(server.url())).await;

        // 1. Cache Miss - should hit the mock server
        let result = resolver.resolve(selector).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, sig_text);
        assert_eq!(result[0].selector, selector);
        mock_api.assert_async().await;

        // 2. Cache Hit - should NOT hit the mock server again
        let result_cached = resolver.resolve(selector).await.unwrap();
        assert_eq!(result, result_cached);
        mock_api.assert_async().await; // Assertions are for the total number of hits
    }

    #[tokio::test]
    async fn test_composite_resolver_loads_from_disk() {
        let selector = [0xfe, 0xed, 0xfa, 0xce];
        let sig_text = "feedFace(uint256)";
        let cache_content = json!({
            "feedface": [sig_text]
        });

        let temp_dir = tempdir().unwrap();
        let cache_file = temp_dir.path().join("preloaded_cache.json");
        fs::write(&cache_file, cache_content.to_string())
            .await
            .unwrap();

        let mut server = mockito::Server::new_async().await;
        // The mock should NOT be called if the cache is loaded correctly.
        let mock_api = server
            .mock("GET", "/api/v1/signatures/?hex_signature=0xfeedface")
            .with_status(500) // Expect an error if we miss
            .expect(0)
            .create_async()
            .await;

        let resolver = CompositeResolver::new(cache_file, Some(server.url())).await;
        let result = resolver.resolve(selector).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, sig_text);
        mock_api.assert_async().await;
    }

    #[tokio::test]
    async fn test_composite_resolver_network_error() {
        let selector = [0xba, 0xad, 0xf0, 0x0d];

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/v1/signatures/?hex_signature=0xbaadf00d")
            .with_status(500) // Simulate a server error
            .create_async()
            .await;

        let temp_dir = tempdir().unwrap();
        let cache_file = temp_dir.path().join("error_cache.json");

        let resolver = CompositeResolver::new(cache_file, Some(server.url())).await;

        let result = resolver.resolve(selector).await;
        assert!(matches!(result, Err(ResolveError::Network(_))));
    }
}
