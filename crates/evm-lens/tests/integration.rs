use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Helper to get the evm-lens binary command
fn evm_lens_cmd() -> Command {
    Command::cargo_bin("evm-lens").expect("Failed to find evm-lens binary")
}

/// Sample bytecode for testing
const SAMPLE_BYTECODE: &str = "60ff61abcd00";
const SAMPLE_BYTECODE_WITH_PREFIX: &str = "0x60ff61abcd00";
const INVALID_HEX: &str = "60gg";

#[test]
fn test_hex_input_valid_bytecode() {
    let mut cmd = evm_lens_cmd();
    cmd.arg(SAMPLE_BYTECODE);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EVM BYTECODE DISASSEMBLY"))
        .stdout(predicate::str::contains("PUSH1"))
        .stdout(predicate::str::contains("PUSH2"))
        .stdout(predicate::str::contains("STOP"))
        .stdout(predicate::str::contains("3 opcodes total"));
}

#[test]
fn test_hex_input_invalid_characters() {
    let mut cmd = evm_lens_cmd();
    cmd.arg(INVALID_HEX);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Invalid hex characters found"));
}

#[test]
fn test_stdin_input_valid_bytecode() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--stdin");
    cmd.write_stdin(SAMPLE_BYTECODE_WITH_PREFIX);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EVM BYTECODE DISASSEMBLY"))
        .stdout(predicate::str::contains("PUSH1"));
}

#[test]
fn test_stdin_input_empty() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--stdin");
    cmd.write_stdin("");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("No input provided via stdin"));
}

#[test]
fn test_no_arguments_reads_stdin() {
    let mut cmd = evm_lens_cmd();
    cmd.write_stdin(SAMPLE_BYTECODE);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EVM BYTECODE DISASSEMBLY"))
        .stdout(predicate::str::contains("PUSH1"));
}

#[test]
fn test_file_input_valid_bytecode() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "{}", SAMPLE_BYTECODE).expect("Failed to write to temp file");

    let mut cmd = evm_lens_cmd();
    cmd.arg("--file").arg(temp_file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EVM BYTECODE DISASSEMBLY"))
        .stdout(predicate::str::contains("PUSH1"));
}

#[test]
fn test_file_input_nonexistent_file() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--file").arg("/nonexistent/file.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Failed to read file"));
}

#[tokio::test]
async fn test_address_input_valid_contract() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": format!("0x{}", SAMPLE_BYTECODE),
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = evm_lens_cmd();
    cmd.arg("--address")
        .arg("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        .arg("--rpc")
        .arg(mock_server.uri());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EVM BYTECODE DISASSEMBLY"))
        .stdout(predicate::str::contains("PUSH1"));
}

#[tokio::test]
async fn test_address_input_no_contract_code() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": "0x",
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = evm_lens_cmd();
    cmd.arg("--address")
        .arg("0x742d35Cc6634C0532925a3b8D56f3a1f0b9CF81b")
        .arg("--rpc")
        .arg(mock_server.uri());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("has no contract code"));
}

#[tokio::test]
async fn test_address_input_network_error() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--address")
        .arg("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        .arg("--rpc")
        .arg("http://localhost:1"); // Non-existent endpoint

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Failed to send RPC request"));
}

#[test]
fn test_address_input_invalid_address() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--address")
        .arg("invalid_address")
        .arg("--rpc")
        .arg("https://eth.llamarpc.com");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Invalid address"));
}

#[test]
fn test_conflicting_arguments() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--stdin").arg("--file").arg("test.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_help_output() {
    let mut cmd = evm_lens_cmd();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "A colorful EVM bytecode disassembler",
        ))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains("--address"))
        .stdout(predicate::str::contains("--rpc"));
}
