use revm::{
    bytecode::{Bytecode, OpCode},
    primitives::Bytes,
};

pub mod stats;
pub use stats::{Stats, StatsError, compute_stats};

#[derive(Debug)]
pub enum DisassemblyError {
    InvalidBytecode(String),
    EmptyBytecode,
    MalformedInstruction { position: usize, byte: u8 },
}

impl std::fmt::Display for DisassemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisassemblyError::InvalidBytecode(msg) => write!(f, "Invalid bytecode: {}", msg),
            DisassemblyError::EmptyBytecode => write!(f, "Bytecode is empty"),
            DisassemblyError::MalformedInstruction { position, byte } => {
                write!(
                    f,
                    "Malformed instruction at position {}: invalid opcode 0x{:02x}",
                    position, byte
                )
            }
        }
    }
}

impl std::error::Error for DisassemblyError {}

/// Disassembles EVM bytecode into a sequence of opcodes with their positions.
///
/// Takes a byte slice containing raw EVM bytecode and returns a vector of tuples,
/// where each tuple contains:
/// - The position of the opcode in the bytecode (usize)
/// - The opcode itself (OpCode)
///
/// # Arguments
///
/// * `bytes` - A slice of bytes containing the raw EVM bytecode
///
/// # Returns
///
/// * `Ok(Vec<(usize, OpCode)>)` - A vector of tuples containing the position and opcode for each instruction
/// * `Err(DisassemblyError)` - If the bytecode is invalid
///
/// # Example
///
/// ```
/// use evm_lens_core::disassemble;
/// use revm::bytecode::OpCode;
///
/// let bytecode = hex::decode("60FF").unwrap(); // PUSH1 0xFF
/// let ops = disassemble(&bytecode).unwrap();
/// assert_eq!(ops[0].0, 0); // Position 0
/// assert_eq!(ops[0].1, OpCode::PUSH1); // PUSH1 opcode
/// ```
pub fn disassemble(bytes: &[u8]) -> Result<Vec<(usize, OpCode)>, DisassemblyError> {
    if bytes.is_empty() {
        return Err(DisassemblyError::EmptyBytecode);
    }

    let bytecode = match Bytecode::new_raw_checked(Bytes::from(bytes.to_vec())) {
        Ok(bytecode) => bytecode,
        Err(e) => return Err(DisassemblyError::InvalidBytecode(e.to_string())),
    };

    let mut result: Vec<(usize, OpCode)> = Vec::new();
    let mut bytecode_iter = bytecode.iter_opcodes();

    while let Some(opcode) = bytecode_iter.peek_opcode() {
        result.push((bytecode_iter.position(), opcode));
        bytecode_iter.next();
    }

    if result.is_empty() {
        return Err(DisassemblyError::InvalidBytecode(
            "No valid opcodes found".to_string(),
        ));
    }

    Ok(result)
}

/// Computes and returns statistics for the given bytecode.
///
/// This function takes raw bytecode bytes and returns comprehensive statistics
/// including byte length, opcode count, maximum stack depth, and other metrics.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes containing the raw EVM bytecode
///
/// # Returns
///
/// * `Ok(Stats)` - Statistics about the bytecode
/// * `Err(DisassemblyError)` - If the bytecode is invalid
///
/// # Example
///
/// ```
/// use evm_lens_core::get_stats;
///
/// let bytecode = hex::decode("60FF600101").unwrap(); // PUSH1 0xFF, PUSH1 0x01, ADD
/// let stats = get_stats(&bytecode).unwrap();
/// println!("Bytecode length: {} bytes", stats.byte_len);
/// println!("Number of opcodes: {}", stats.opcode_count);
/// ```
pub fn get_stats(bytes: &[u8]) -> Result<Stats, DisassemblyError> {
    if bytes.is_empty() {
        return Err(DisassemblyError::EmptyBytecode);
    }

    let bytecode = match Bytecode::new_raw_checked(Bytes::from(bytes.to_vec())) {
        Ok(bytecode) => bytecode,
        Err(e) => return Err(DisassemblyError::InvalidBytecode(e.to_string())),
    };

    compute_stats(&bytecode).map_err(|e| match e {
        StatsError::UnknownOpcode(opcode) => DisassemblyError::MalformedInstruction {
            position: 0, // We don't have position info from stats error
            byte: opcode,
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push1_push2_stop() {
        let bytes = hex::decode("60FF61ABCD00").unwrap(); // PUSH1 0xFF, PUSH2 0xABCD, STOP
        let ops = disassemble(&bytes).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].0, 0);
        assert_eq!(ops[0].1, OpCode::PUSH1);
        assert_eq!(ops[1].0, 2);
        assert_eq!(ops[1].1, OpCode::PUSH2);
        assert_eq!(ops[2].0, 5);
        assert_eq!(ops[2].1, OpCode::STOP);
    }

    #[test]
    fn memory_operations() {
        // PUSH1 0x20, PUSH1 0x00, MSTORE, PUSH1 0x00, MLOAD, STOP
        let bytes = hex::decode("602060005260005100").unwrap();
        let ops = disassemble(&bytes).unwrap();
        assert_eq!(ops.len(), 6);
        assert_eq!(ops[0], (0, OpCode::PUSH1)); // PUSH1 32
        assert_eq!(ops[1], (2, OpCode::PUSH1)); // PUSH1 0  
        assert_eq!(ops[2], (4, OpCode::MSTORE)); // MSTORE (store to memory)
        assert_eq!(ops[3], (5, OpCode::PUSH1)); // PUSH1 0
        assert_eq!(ops[4], (7, OpCode::MLOAD)); // MLOAD (load from memory)
        assert_eq!(ops[5], (8, OpCode::STOP)); // STOP
    }

    #[test]
    fn stack_operations() {
        // PUSH1 0x01, PUSH1 0x02, DUP1, SWAP1, ADD, STOP
        let bytes = hex::decode("6001600280900100").unwrap();
        let ops = disassemble(&bytes).unwrap();
        assert_eq!(ops.len(), 6);
        assert_eq!(ops[0], (0, OpCode::PUSH1)); // PUSH1 1
        assert_eq!(ops[1], (2, OpCode::PUSH1)); // PUSH1 2
        assert_eq!(ops[2], (4, OpCode::DUP1)); // DUP1 (duplicate top stack item)
        assert_eq!(ops[3], (5, OpCode::SWAP1)); // SWAP1 (swap top 2 stack items)
        assert_eq!(ops[4], (6, OpCode::ADD)); // ADD
        assert_eq!(ops[5], (7, OpCode::STOP)); // STOP
    }

    #[test]
    fn storage_and_crypto() {
        // PUSH1 0x42, PUSH1 0x00, SSTORE, PUSH1 0x00, SLOAD, KECCAK256, STOP
        let bytes = hex::decode("60426000556000542000").unwrap();
        let ops = disassemble(&bytes).unwrap();
        assert_eq!(ops.len(), 7);
        assert_eq!(ops[0], (0, OpCode::PUSH1)); // PUSH1 0x42
        assert_eq!(ops[1], (2, OpCode::PUSH1)); // PUSH1 0
        assert_eq!(ops[2], (4, OpCode::SSTORE)); // SSTORE (store to storage)
        assert_eq!(ops[3], (5, OpCode::PUSH1)); // PUSH1 0  
        assert_eq!(ops[4], (7, OpCode::SLOAD)); // SLOAD (load from storage)
        assert_eq!(ops[5], (8, OpCode::KECCAK256)); // KECCAK256 (hash function)
        assert_eq!(ops[6], (9, OpCode::STOP)); // STOP
    }

    #[test]
    fn empty_bytecode_error() {
        let bytes = vec![];
        let result = disassemble(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            DisassemblyError::EmptyBytecode => {} // Expected
            _ => panic!("Expected EmptyBytecode error"),
        }
    }

    #[test]
    fn test_get_stats() {
        // PUSH1 0xFF, PUSH1 0x01, ADD, STOP
        let bytes = hex::decode("60FF60010100").unwrap();
        let stats = get_stats(&bytes).unwrap();

        assert_eq!(stats.byte_len, 6);
        assert_eq!(stats.opcode_count, 4);
        assert!(stats.max_stack_depth > 0);
    }

    #[test]
    fn test_stats_with_empty_bytecode() {
        let bytes = vec![];
        let result = get_stats(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            DisassemblyError::EmptyBytecode => {} // Expected
            _ => panic!("Expected EmptyBytecode error"),
        }
    }
}
