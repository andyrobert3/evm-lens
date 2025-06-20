use revm::bytecode::{Bytecode, opcode::OPCODE_INFO};

#[derive(Debug)]
pub struct Stats {
    pub byte_len: usize,
    pub opcode_count: usize,
    pub max_stack_depth: usize,
}

#[derive(Debug)]
pub enum StatsError {
    UnknownOpcode(u8),
}

impl std::fmt::Display for StatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatsError::UnknownOpcode(opcode) => {
                write!(f, "Unknown opcode: 0x{:02x}", opcode)
            }
        }
    }
}

impl std::error::Error for StatsError {}

pub fn compute_stats(bytecode: &Bytecode) -> Result<Stats, StatsError> {
    // Count the number of opcodes
    let opcode_count = compute_opcode_count(bytecode);

    // Get total byte length
    let byte_len = get_byte_len(bytecode);

    // Track PUSH / POP depth
    let max_stack_depth = compute_max_stack_depth(bytecode)?;

    Ok(Stats {
        byte_len,
        opcode_count,
        max_stack_depth,
    })
}

fn compute_opcode_count(bytecode: &Bytecode) -> usize {
    let iter = bytecode.iter_opcodes();
    iter.count()
}

fn get_byte_len(bytecode: &Bytecode) -> usize {
    bytecode.bytecode().as_ref().len()
}

fn compute_max_stack_depth(bytecode: &Bytecode) -> Result<usize, StatsError> {
    let mut iter = bytecode.iter_opcodes();
    let mut max_depth: i32 = 0;
    let mut depth: i32 = 0;

    while let Some(opcode) = iter.peek_opcode() {
        let opcode_info = OPCODE_INFO[opcode.get() as usize];

        match opcode_info {
            Some(opcode_info) => {
                depth += opcode_info.io_diff() as i32;
            }
            None => {
                // If the opcode is not found, it's an invalid opcode
                return Err(StatsError::UnknownOpcode(opcode.get()));
            }
        }

        max_depth = max_depth.max(depth);
        iter.next();
    }

    Ok(max_depth as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::primitives::Bytes;

    #[test]
    fn test_simple_bytecode_stats() {
        // PUSH1 0xFF, STOP
        let bytes = hex::decode("60FF00").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 3);
        assert_eq!(stats.opcode_count, 2);
        assert_eq!(stats.max_stack_depth, 1); // PUSH1 adds 1 to stack
    }

    #[test]
    fn test_complex_bytecode_stats() {
        // PUSH1 0x01, PUSH1 0x02, ADD, STOP
        let bytes = hex::decode("600160020100").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 6);
        assert_eq!(stats.opcode_count, 4);
        assert_eq!(stats.max_stack_depth, 2); // Max depth when both PUSH1s are on stack
    }

    #[test]
    fn test_stack_operations() {
        // PUSH1 0x01, PUSH1 0x02, DUP1, SWAP1, ADD, STOP
        let bytes = hex::decode("6001600280900100").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 8);
        assert_eq!(stats.opcode_count, 6);
        assert_eq!(stats.max_stack_depth, 3); // DUP1 increases stack depth to 3
    }

    #[test]
    fn test_memory_operations() {
        // PUSH1 0x20, PUSH1 0x00, MSTORE, PUSH1 0x00, MLOAD, STOP
        let bytes = hex::decode("602060005260005100").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 9);
        assert_eq!(stats.opcode_count, 6);
        assert_eq!(stats.max_stack_depth, 2); // Max when PUSH1 values are on stack
    }

    #[test]
    fn test_push_operations() {
        // PUSH1 0xFF, PUSH2 0xABCD, PUSH32 (32 bytes of data), STOP
        let mut bytes = vec![0x60, 0xFF]; // PUSH1 0xFF
        bytes.extend_from_slice(&[0x61, 0xAB, 0xCD]); // PUSH2 0xABCD
        bytes.push(0x7F); // PUSH32
        bytes.extend_from_slice(&[0xFF; 32]); // 32 bytes of 0xFF
        bytes.push(0x00); // STOP

        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 39); // 2 + 3 + 1 + 32 + 1 = 39
        assert_eq!(stats.opcode_count, 4);
        assert_eq!(stats.max_stack_depth, 3); // All three PUSH operations on stack
    }

    #[test]
    fn test_arithmetic_operations() {
        // PUSH1 0x05, PUSH1 0x03, ADD, PUSH1 0x02, MUL, STOP
        let bytes = hex::decode("600560030160020200").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 9);
        assert_eq!(stats.opcode_count, 6);
        assert_eq!(stats.max_stack_depth, 2); // Max depth when two values are on stack
    }

    #[test]
    fn test_single_opcode() {
        // Just STOP
        let bytes = hex::decode("00").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 1);
        assert_eq!(stats.opcode_count, 1);
        assert_eq!(stats.max_stack_depth, 0); // STOP doesn't affect stack
    }

    #[test]
    fn test_large_bytecode() {
        // Create a larger bytecode with many operations
        let mut bytes = Vec::new();

        // Add 20 PUSH1 operations (reduced from 50 for simpler test)
        for i in 0..20 {
            bytes.push(0x60); // PUSH1
            bytes.push(i as u8); // value
        }
        bytes.push(0x00); // STOP

        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let stats = compute_stats(&bytecode).unwrap();
        assert_eq!(stats.byte_len, 41); // 20 * 2 + 1
        assert_eq!(stats.opcode_count, 21); // 20 PUSH1s + 1 STOP
        assert_eq!(stats.max_stack_depth, 20); // All PUSH1s accumulate on stack
    }

    #[test]
    fn test_compute_opcode_count() {
        let bytes = hex::decode("60FF61ABCD00").unwrap(); // PUSH1, PUSH2, STOP
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let count = compute_opcode_count(&bytecode);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_byte_len() {
        let bytes = hex::decode("60FF61ABCD00").unwrap();
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let len = get_byte_len(&bytecode);
        assert_eq!(len, 6);
    }

    #[test]
    fn test_compute_max_stack_depth() {
        let bytes = hex::decode("60FF00").unwrap(); // PUSH1 0xFF, STOP
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let depth = compute_max_stack_depth(&bytecode).unwrap();
        assert_eq!(depth, 1);
    }

    #[test]
    fn test_zero_stack_depth() {
        let bytes = hex::decode("00").unwrap(); // Just STOP
        let bytecode = Bytecode::new_raw_checked(Bytes::from(bytes)).unwrap();

        let depth = compute_max_stack_depth(&bytecode).unwrap();
        assert_eq!(depth, 0);
    }

    #[test]
    fn test_error_display() {
        let error = StatsError::UnknownOpcode(0xFF);
        assert_eq!(format!("{}", error), "Unknown opcode: 0xff");
    }

    #[test]
    fn test_stats_struct_access() {
        let stats = Stats {
            byte_len: 10,
            opcode_count: 5,
            max_stack_depth: 3,
        };

        assert_eq!(stats.byte_len, 10);
        assert_eq!(stats.opcode_count, 5);
        assert_eq!(stats.max_stack_depth, 3);
    }
}
