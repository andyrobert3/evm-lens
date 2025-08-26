use async_trait::async_trait;
use revm::bytecode::OpCode;

use crate::disassemble;
use crate::storage::layout::{Provenance, StorageEntry, StorageLayout, StorageType};
use crate::storage::resolver::StorageLayoutResolver;

/// Heuristic that scans for PUSH <const> followed by SLOAD/SSTORE.
/// If found, treat the const as a direct slot index. Types are Unknown.
pub struct HeuristicResolver;

#[async_trait]
impl StorageLayoutResolver for HeuristicResolver {
    async fn resolve(&self, input: &[u8]) -> color_eyre::Result<Option<StorageLayout>> {
        if input.is_empty() {
            return Ok(Some(StorageLayout::new()));
        }

        let ops = match disassemble(input) {
            Ok(v) => v,
            Err(_) => return Ok(Some(StorageLayout::new())),
        };

        let mut layout = StorageLayout::new();

        // Windowed scan: PUSH[1..32] followed within small window by SLOAD/SSTORE
        let window = 6usize;

        for (idx, (_pos, op)) in ops.iter().enumerate() {
            let push_n = match *op {
                OpCode::PUSH0 => Some(0u8),
                OpCode::PUSH1 => Some(1),
                OpCode::PUSH2 => Some(2),
                OpCode::PUSH3 => Some(3),
                OpCode::PUSH4 => Some(4),
                OpCode::PUSH5 => Some(5),
                OpCode::PUSH6 => Some(6),
                OpCode::PUSH7 => Some(7),
                OpCode::PUSH8 => Some(8),
                OpCode::PUSH9 => Some(9),
                OpCode::PUSH10 => Some(10),
                OpCode::PUSH11 => Some(11),
                OpCode::PUSH12 => Some(12),
                OpCode::PUSH13 => Some(13),
                OpCode::PUSH14 => Some(14),
                OpCode::PUSH15 => Some(15),
                OpCode::PUSH16 => Some(16),
                OpCode::PUSH17 => Some(17),
                OpCode::PUSH18 => Some(18),
                OpCode::PUSH19 => Some(19),
                OpCode::PUSH20 => Some(20),
                OpCode::PUSH21 => Some(21),
                OpCode::PUSH22 => Some(22),
                OpCode::PUSH23 => Some(23),
                OpCode::PUSH24 => Some(24),
                OpCode::PUSH25 => Some(25),
                OpCode::PUSH26 => Some(26),
                OpCode::PUSH27 => Some(27),
                OpCode::PUSH28 => Some(28),
                OpCode::PUSH29 => Some(29),
                OpCode::PUSH30 => Some(30),
                OpCode::PUSH31 => Some(31),
                OpCode::PUSH32 => Some(32),
                _ => None,
            };

            if push_n.is_none() {
                continue;
            }

            // Look ahead up to `window` ops for SLOAD/SSTORE as a conservative signal
            let mut found = false;
            for j in 1..=window {
                if let Some((_, next_op)) = ops.get(idx + j) {
                    match *next_op {
                        OpCode::SLOAD | OpCode::SSTORE => {
                            found = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if !found {
                continue;
            }

            // Extract pushed immediate as big-endian integer from underlying byte slice.
            // We conservatively re-scan input bytes at the recorded opcode position to get immediates.
            // For v0.3 simplicity, we approximate by reading from the raw hex window preceding the op.
            let (pos, _op) = ops[idx];
            // The immediate begins at pos+1 and spans push_n bytes; guard bounds.
            let n = push_n.unwrap() as usize;
            if pos + 1 + n > input.len() {
                continue;
            }
            let imm = &input[pos + 1..pos + 1 + n];
            // Convert to u128 if fits; skip otherwise
            if n > 16 {
                continue;
            }
            let mut slot: u128 = 0;
            // Shift left by 8 bits for each byte
            for &b in imm {
                slot = (slot << 8) | b as u128;
            }

            // Deduplicate per slot
            if layout.entries.iter().any(|e| e.slot == slot) {
                continue;
            }
            layout.add_entry(StorageEntry {
                slot,
                offset: None,
                size: None,
                r#type: StorageType::Unknown,
                label: None,
                provenance: Provenance::HeuristicTrace,
            });
        }

        Ok(Some(layout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_push1_followed_by_sload() {
        // 60 00 54 00 => PUSH1 0x00; SLOAD; STOP
        let resolver = HeuristicResolver;
        let bytes = hex::decode("60005400").unwrap();
        let layout = resolver.resolve(&bytes).await.unwrap().unwrap();
        assert_eq!(layout.entries.len(), 1);
        assert_eq!(layout.entries[0].slot, 0);
    }

    #[tokio::test]
    async fn detects_push2_followed_by_sstore() {
        // 61 ab cd 55 00 => PUSH2 0xABCD; SSTORE; STOP
        let resolver = HeuristicResolver;
        let bytes = hex::decode("61abcd5500").unwrap();
        let layout = resolver.resolve(&bytes).await.unwrap().unwrap();
        assert_eq!(layout.entries.len(), 1);
        assert_eq!(layout.entries[0].slot, 0xABCD);
    }

    #[tokio::test]
    async fn no_detection_beyond_window() {
        // PUSH1 0x00, then 7 non-storage ops (DUP1), then SLOAD => beyond window=6
        // 60 00 80 80 80 80 80 80 80 54 00
        let resolver = HeuristicResolver;
        let bytes = hex::decode("6000808080808080805400").unwrap();
        let layout = resolver.resolve(&bytes).await.unwrap().unwrap();
        assert_eq!(layout.entries.len(), 0);
    }

    #[tokio::test]
    async fn dedup_same_slot_multiple_occurrences() {
        // PUSH1 0x01; SLOAD; PUSH1 0x01; SSTORE; STOP
        // 60 01 54 60 01 55 00
        let resolver = HeuristicResolver;
        let bytes = hex::decode("60015460015500").unwrap();
        let layout = resolver.resolve(&bytes).await.unwrap().unwrap();
        assert_eq!(layout.entries.len(), 1);
        assert_eq!(layout.entries[0].slot, 1);
    }

    #[tokio::test]
    async fn skip_push32_immediate() {
        // 7f followed by 32 bytes of 0x00, then SLOAD => n>16 should skip
        let resolver = HeuristicResolver;
        let bytes = hex::decode(&format!("7f{}54", "00".repeat(32))).unwrap();
        let layout = resolver.resolve(&bytes).await.unwrap().unwrap();
        assert_eq!(layout.entries.len(), 0);
    }
}
