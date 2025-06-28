use colored::*;
use evm_lens_core::{abi, OpCode};
use std::{collections::HashMap, sync::Arc};

pub fn opcode_to_string(opcode: OpCode) -> &'static str {
    match opcode {
        OpCode::STOP => "STOP",
        OpCode::ADD => "ADD",
        OpCode::MUL => "MUL",
        OpCode::SUB => "SUB",
        OpCode::DIV => "DIV",
        OpCode::SDIV => "SDIV",
        OpCode::MOD => "MOD",
        OpCode::SMOD => "SMOD",
        OpCode::ADDMOD => "ADDMOD",
        OpCode::MULMOD => "MULMOD",
        OpCode::EXP => "EXP",
        OpCode::SIGNEXTEND => "SIGNEXTEND",
        OpCode::LT => "LT",
        OpCode::GT => "GT",
        OpCode::SLT => "SLT",
        OpCode::SGT => "SGT",
        OpCode::EQ => "EQ",
        OpCode::ISZERO => "ISZERO",
        OpCode::AND => "AND",
        OpCode::OR => "OR",
        OpCode::XOR => "XOR",
        OpCode::NOT => "NOT",
        OpCode::BYTE => "BYTE",
        OpCode::SHL => "SHL",
        OpCode::SHR => "SHR",
        OpCode::SAR => "SAR",
        OpCode::KECCAK256 => "KECCAK256",
        OpCode::ADDRESS => "ADDRESS",
        OpCode::BALANCE => "BALANCE",
        OpCode::ORIGIN => "ORIGIN",
        OpCode::CALLER => "CALLER",
        OpCode::CALLVALUE => "CALLVALUE",
        OpCode::CALLDATALOAD => "CALLDATALOAD",
        OpCode::CALLDATASIZE => "CALLDATASIZE",
        OpCode::CALLDATACOPY => "CALLDATACOPY",
        OpCode::CODESIZE => "CODESIZE",
        OpCode::CODECOPY => "CODECOPY",
        OpCode::GASPRICE => "GASPRICE",
        OpCode::EXTCODESIZE => "EXTCODESIZE",
        OpCode::EXTCODECOPY => "EXTCODECOPY",
        OpCode::RETURNDATASIZE => "RETURNDATASIZE",
        OpCode::RETURNDATACOPY => "RETURNDATACOPY",
        OpCode::EXTCODEHASH => "EXTCODEHASH",
        OpCode::BLOCKHASH => "BLOCKHASH",
        OpCode::COINBASE => "COINBASE",
        OpCode::TIMESTAMP => "TIMESTAMP",
        OpCode::NUMBER => "NUMBER",
        OpCode::DIFFICULTY => "DIFFICULTY",
        OpCode::GASLIMIT => "GASLIMIT",
        OpCode::CHAINID => "CHAINID",
        OpCode::SELFBALANCE => "SELFBALANCE",
        OpCode::BASEFEE => "BASEFEE",
        OpCode::BLOBHASH => "BLOBHASH",
        OpCode::BLOBBASEFEE => "BLOBBASEFEE",
        OpCode::POP => "POP",
        OpCode::MLOAD => "MLOAD",
        OpCode::MSTORE => "MSTORE",
        OpCode::MSTORE8 => "MSTORE8",
        OpCode::SLOAD => "SLOAD",
        OpCode::SSTORE => "SSTORE",
        OpCode::JUMP => "JUMP",
        OpCode::JUMPI => "JUMPI",
        OpCode::PC => "PC",
        OpCode::MSIZE => "MSIZE",
        OpCode::GAS => "GAS",
        OpCode::JUMPDEST => "JUMPDEST",
        OpCode::TLOAD => "TLOAD",
        OpCode::TSTORE => "TSTORE",
        OpCode::MCOPY => "MCOPY",
        OpCode::PUSH0 => "PUSH0",
        OpCode::PUSH1 => "PUSH1",
        OpCode::PUSH2 => "PUSH2",
        OpCode::PUSH3 => "PUSH3",
        OpCode::PUSH4 => "PUSH4",
        OpCode::PUSH5 => "PUSH5",
        OpCode::PUSH6 => "PUSH6",
        OpCode::PUSH7 => "PUSH7",
        OpCode::PUSH8 => "PUSH8",
        OpCode::PUSH9 => "PUSH9",
        OpCode::PUSH10 => "PUSH10",
        OpCode::PUSH11 => "PUSH11",
        OpCode::PUSH12 => "PUSH12",
        OpCode::PUSH13 => "PUSH13",
        OpCode::PUSH14 => "PUSH14",
        OpCode::PUSH15 => "PUSH15",
        OpCode::PUSH16 => "PUSH16",
        OpCode::PUSH17 => "PUSH17",
        OpCode::PUSH18 => "PUSH18",
        OpCode::PUSH19 => "PUSH19",
        OpCode::PUSH20 => "PUSH20",
        OpCode::PUSH21 => "PUSH21",
        OpCode::PUSH22 => "PUSH22",
        OpCode::PUSH23 => "PUSH23",
        OpCode::PUSH24 => "PUSH24",
        OpCode::PUSH25 => "PUSH25",
        OpCode::PUSH26 => "PUSH26",
        OpCode::PUSH27 => "PUSH27",
        OpCode::PUSH28 => "PUSH28",
        OpCode::PUSH29 => "PUSH29",
        OpCode::PUSH30 => "PUSH30",
        OpCode::PUSH31 => "PUSH31",
        OpCode::PUSH32 => "PUSH32",
        OpCode::DUP1 => "DUP1",
        OpCode::DUP2 => "DUP2",
        OpCode::DUP3 => "DUP3",
        OpCode::DUP4 => "DUP4",
        OpCode::DUP5 => "DUP5",
        OpCode::DUP6 => "DUP6",
        OpCode::DUP7 => "DUP7",
        OpCode::DUP8 => "DUP8",
        OpCode::DUP9 => "DUP9",
        OpCode::DUP10 => "DUP10",
        OpCode::DUP11 => "DUP11",
        OpCode::DUP12 => "DUP12",
        OpCode::DUP13 => "DUP13",
        OpCode::DUP14 => "DUP14",
        OpCode::DUP15 => "DUP15",
        OpCode::DUP16 => "DUP16",
        OpCode::SWAP1 => "SWAP1",
        OpCode::SWAP2 => "SWAP2",
        OpCode::SWAP3 => "SWAP3",
        OpCode::SWAP4 => "SWAP4",
        OpCode::SWAP5 => "SWAP5",
        OpCode::SWAP6 => "SWAP6",
        OpCode::SWAP7 => "SWAP7",
        OpCode::SWAP8 => "SWAP8",
        OpCode::SWAP9 => "SWAP9",
        OpCode::SWAP10 => "SWAP10",
        OpCode::SWAP11 => "SWAP11",
        OpCode::SWAP12 => "SWAP12",
        OpCode::SWAP13 => "SWAP13",
        OpCode::SWAP14 => "SWAP14",
        OpCode::SWAP15 => "SWAP15",
        OpCode::SWAP16 => "SWAP16",
        OpCode::LOG0 => "LOG0",
        OpCode::LOG1 => "LOG1",
        OpCode::LOG2 => "LOG2",
        OpCode::LOG3 => "LOG3",
        OpCode::LOG4 => "LOG4",
        OpCode::CREATE => "CREATE",
        OpCode::CALL => "CALL",
        OpCode::CALLCODE => "CALLCODE",
        OpCode::RETURN => "RETURN",
        OpCode::DELEGATECALL => "DELEGATECALL",
        OpCode::CREATE2 => "CREATE2",
        OpCode::STATICCALL => "STATICCALL",
        OpCode::REVERT => "REVERT",
        OpCode::INVALID => "INVALID",
        OpCode::SELFDESTRUCT => "SELFDESTRUCT",
        _ => "UNKNOWN",
    }
}

pub fn categorize_opcode(opcode_str: &str) -> ColoredString {
    match opcode_str {
        // Stack operations - Green
        op if op.starts_with("PUSH") => op.bright_green().bold(),
        op if op.starts_with("POP") => op.green(),
        op if op.starts_with("DUP") => op.green(),
        op if op.starts_with("SWAP") => op.green(),

        // Arithmetic - Yellow
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "ADDMOD" | "MULMOD" => {
            opcode_str.bright_yellow().bold()
        }
        "LT" | "GT" | "SLT" | "SGT" | "EQ" | "ISZERO" => opcode_str.yellow(),

        // Memory operations - Blue
        "MLOAD" | "MSTORE" | "MSTORE8" | "MSIZE" | "MCOPY" => opcode_str.bright_blue().bold(),

        // Storage operations - Magenta
        "SLOAD" | "SSTORE" => opcode_str.bright_magenta().bold(),

        // Crypto/Hash - Cyan
        "KECCAK256" => opcode_str.bright_cyan().bold(),

        // Control flow - Red
        "JUMP" | "JUMPI" | "JUMPDEST" => opcode_str.bright_red().bold(),
        "CALL" | "CALLCODE" | "DELEGATECALL" | "STATICCALL" => opcode_str.red().bold(),
        "CREATE" | "CREATE2" => opcode_str.red(),

        // End operations - White
        "STOP" | "RETURN" | "REVERT" | "SELFDESTRUCT" => opcode_str.bright_white().bold(),

        // Default - Normal
        _ => opcode_str.normal(),
    }
}

fn prioritize_signatures(signatures: &[abi::SigInfo]) -> Vec<String> {
    let mut sig_texts: Vec<String> = signatures.iter().map(|s| s.text.clone()).collect();
    
    // Define priority patterns for common/standard functions
    let priority_patterns = [
        // Standard ERC20 functions
        "totalSupply()",
        "balanceOf(address)",
        "transfer(address,uint256)",
        "transferFrom(address,address,uint256)",
        "approve(address,uint256)",
        "allowance(address,address)",
        
        // Common contract functions
        "name()",
        "symbol()",
        "decimals()",
        "owner()",
        "mint(address,uint256)",
        "burn(uint256)",
    ];
    
    // Sort signatures by priority
    sig_texts.sort_by(|a, b| {
        let a_priority = get_signature_priority(a, &priority_patterns);
        let b_priority = get_signature_priority(b, &priority_patterns);
        
        // Lower priority number means higher importance
        a_priority.cmp(&b_priority).then_with(|| a.cmp(b))
    });
    
    sig_texts
}

fn get_signature_priority(sig: &str, priority_patterns: &[&str]) -> usize {
    // Check for exact matches first (highest priority)
    for (index, pattern) in priority_patterns.iter().enumerate() {
        if sig == *pattern {
            return index;
        }
    }
    
    // Penalize obvious obfuscated/random functions (low priority)
    if is_likely_obfuscated(sig) {
        return priority_patterns.len() * 2;
    }
    
    // Default priority for unknown but potentially legitimate functions
    priority_patterns.len()
}

fn is_likely_obfuscated(sig: &str) -> bool {
    let function_name = sig.split('(').next().unwrap_or("");
    
    // Common patterns in obfuscated function names
    let obfuscation_patterns = [
        "_SIMON", "watch_tg_", "join_tg_", "func_", "many_msg_", "workMyDireful", "_haha_", "_invmru_",
    ];
    
    obfuscation_patterns.iter().any(|pattern| function_name.contains(pattern))
}

pub fn print_opcode(
    position: usize,
    opcode: OpCode,
    bytecode: &[u8],
    resolved_sigs: Option<&HashMap<abi::Selector, Arc<[abi::SigInfo]>>>,
) {
    let opcode_str = opcode_to_string(opcode);
    let colored_opcode = categorize_opcode(opcode_str);

    let abi_comment = get_abi_comment(position, opcode, bytecode, resolved_sigs);

    println!(
        "{} {} {}{}",
        format!("{:04x}", position).bright_black(),
        "│".bright_black(),
        colored_opcode,
        abi_comment
    );
}

fn get_abi_comment(
    position: usize,
    opcode: OpCode,
    bytecode: &[u8],
    resolved_sigs: Option<&HashMap<abi::Selector, Arc<[abi::SigInfo]>>>,
) -> String {
    if opcode != OpCode::PUSH4 {
        return String::new();
    }

    // PUSH4 data starts at position + 1 and is 4 bytes long
    if position + 4 >= bytecode.len() {
        return String::new();
    }

    let selector_bytes = &bytecode[position + 1..position + 5];
    if selector_bytes.len() != 4 {
        return String::new();
    }

    let mut selector = [0u8; 4];
    selector.copy_from_slice(selector_bytes);
    
    if let Some(resolved) = resolved_sigs {
        if let Some(infos) = resolved.get(&selector) {
            if !infos.is_empty() {
                let prioritized_sigs = prioritize_signatures(infos);
                let first_sig = prioritized_sigs.get(0).unwrap_or(&String::new()).clone();

                return format!("  # 0x{} → {}", hex::encode(selector), first_sig)
                    .bright_cyan()
                    .to_string();
            }
        }
    }

    String::new()
} 