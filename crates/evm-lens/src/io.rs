use color_eyre::{Result, eyre::eyre};
use ethereum_types::Address;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone)]
pub enum Source {
    Stdin,
    File(PathBuf),
    OnChain { address: Address, rpc: Url },
}

/// Fetches bytecode from the specified source and returns it as a vector of bytes.
///
/// # Arguments
///
/// * `source` - The source to fetch bytecode from, which can be:
///   * `Source::Stdin` - Read hex-encoded bytecode from standard input
///   * `Source::File` - Read hex-encoded bytecode from a file
///   * `Source::OnChain` - Fetch bytecode from an Ethereum contract address via RPC
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok(Vec<u8>)` - The decoded bytecode as a byte vector
/// * `Err` - If reading fails, input is empty/invalid, or RPC request fails
///
/// # Errors
///
/// This function will return an error if:
/// * Reading from stdin/file fails
/// * The input is empty
/// * The hex decoding fails
/// * The RPC request fails when fetching on-chain bytecode
pub async fn fetch_bytes(source: Source) -> Result<Vec<u8>> {
    match source {
        Source::Stdin => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| eyre!("Failed to read from stdin: {}", e))?;

            let trimmed = buffer.trim();
            if trimmed.is_empty() {
                return Err(eyre!("No input provided via stdin"));
            }

            decode_hex(trimmed)
        }

        Source::File(path) => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| eyre!("Failed to read file {:?}: {}", path, e))?;

            let trimmed = content.trim();
            if trimmed.is_empty() {
                return Err(eyre!("File {:?} is empty", path));
            }

            decode_hex(trimmed)
        }

        Source::OnChain { address, rpc } => fetch_on_chain_bytecode(address, rpc).await,
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
    let cleaned = s.trim().trim_start_matches("0x");

    if cleaned.is_empty() {
        return Err(eyre!("Empty hex string provided"));
    }

    if cleaned.len() % 2 != 0 {
        return Err(eyre!(
            "Invalid hex string length ({}). Hex strings must have an even number of characters",
            cleaned.len()
        ));
    }

    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(eyre!(
            "Invalid hex characters found. Only 0-9, a-f, and A-F are allowed"
        ));
    }

    hex::decode(cleaned).map_err(|e| eyre!("Failed to decode hex string: {}", e))
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<String>,
    id: u32,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
}

/// Fetches contract bytecode from an Ethereum node via JSON-RPC.
///
/// # Arguments
///
/// * `address` - The Ethereum address of the contract to fetch bytecode from
/// * `rpc_url` - The URL of the Ethereum JSON-RPC endpoint
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok(Vec<u8>)` - The contract bytecode as a byte vector
/// * `Err` - If the RPC request fails, the address has no code, or the response is invalid
///
/// # Errors
///
/// This function will return an error if:
/// * The RPC request fails to send
/// * The RPC response cannot be parsed
/// * The RPC response contains an error
/// * The address has no contract code (is an EOA or empty contract)
/// * The returned bytecode cannot be hex decoded
async fn fetch_on_chain_bytecode(address: Address, rpc_url: Url) -> Result<Vec<u8>> {
    let client = Client::new();

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        method: "eth_getCode",
        params: vec![format!("{:#x}", address), "latest".to_string()],
        id: 1,
    };

    let response = client
        .post(rpc_url.clone())
        .json(&request)
        .send()
        .await
        .map_err(|e| eyre!("Failed to send RPC request to {}: {}", rpc_url, e))?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .map_err(|e| eyre!("Failed to parse RPC response: {}", e))?;

    if let Some(error) = rpc_response.error {
        return Err(eyre!("RPC error: {}", error));
    }

    let hex_code = rpc_response
        .result
        .ok_or_else(|| eyre!("Missing result in RPC response"))?;

    if hex_code == "0x" {
        return Err(eyre!(
            "Address {:#x} has no contract code (might be an EOA or empty contract)",
            address
        ));
    }

    decode_hex(&hex_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_valid() {
        assert_eq!(decode_hex("60FF").unwrap(), vec![0x60, 0xFF]);
        assert_eq!(decode_hex("0x60FF").unwrap(), vec![0x60, 0xFF]);
        assert_eq!(decode_hex("  0x60FF  ").unwrap(), vec![0x60, 0xFF]);
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("").is_err());
        assert!(decode_hex("0x").is_err());
        assert!(decode_hex("60F").is_err()); // Odd length
        assert!(decode_hex("60GG").is_err()); // Invalid hex
    }

    #[test]
    fn test_decode_hex_case_insensitive() {
        assert_eq!(decode_hex("60ff").unwrap(), vec![0x60, 0xFF]);
        assert_eq!(decode_hex("60FF").unwrap(), vec![0x60, 0xFF]);
        assert_eq!(decode_hex("60Ff").unwrap(), vec![0x60, 0xFF]);
    }
}
