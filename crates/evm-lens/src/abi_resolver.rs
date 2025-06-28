use evm_lens_core::{abi::{self, SelectorResolver}, OpCode};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub async fn resolve_selectors(
    ops: &[(usize, OpCode)],
    bytes: &[u8],
) -> color_eyre::Result<HashMap<abi::Selector, Arc<[abi::SigInfo]>>> {
    let cache_dir = get_cache_directory();
    tokio::fs::create_dir_all(&cache_dir).await?;
    let cache_file = cache_dir.join("selectors.json");
    
    let base_url = std::env::var("EVM_LENS_4BYTE_URL").ok();
    let resolver = abi::CompositeResolver::new(cache_file, base_url).await;
    let selectors_to_resolve = collect_push4_selectors(ops, bytes);
    
    let mut resolved_map = HashMap::new();
    
    // Sequential resolution with rate limiting
    for selector in selectors_to_resolve {
        match resolver.resolve(selector).await {
            Ok(infos) if !infos.is_empty() => {
                resolved_map.insert(selector, infos);
            }
            _ => {} // Ignore errors and empty results
        }
    }

    Ok(resolved_map)
}

fn get_cache_directory() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
        })
        .join("evm-lens")
}

fn collect_push4_selectors(ops: &[(usize, OpCode)], bytes: &[u8]) -> Vec<abi::Selector> {
    let mut selectors = Vec::new();
    
    for (position, opcode) in ops {
        if *opcode == OpCode::PUSH4 {
            if let Some(selector) = extract_selector_from_push4(*position, bytes) {
                selectors.push(selector);
            }
        }
    }
    
    selectors
}

fn extract_selector_from_push4(position: usize, bytes: &[u8]) -> Option<abi::Selector> {
    if position + 4 >= bytes.len() {
        return None;
    }

    let selector_bytes = &bytes[position + 1..position + 5];
    if selector_bytes.len() != 4 {
        return None;
    }

    let mut selector = [0u8; 4];
    selector.copy_from_slice(selector_bytes);
    Some(selector)
} 