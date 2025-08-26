use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provenance {
    CompilerMetadata,
    HeuristicTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageType {
    /// Unknown or not confidently inferred
    Unknown,
    /// Basic solidity-like types (conservative)
    Uint {
        bits: u16,
    },
    Int {
        bits: u16,
    },
    Bool,
    Address,
    BytesFixed {
        size: u8,
    },
    BytesDynamic,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageEntry {
    pub slot: u128,
    /// Optional byte offset for packed fields (0..31). If None, occupies full slot.
    pub offset: Option<u8>,
    /// Size in bytes if known when packed; None for full slot/unknown.
    pub size: Option<u8>,
    pub r#type: StorageType,
    pub label: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageLayout {
    /// Map by slot to entries (supports packed fields).
    pub entries: Vec<StorageEntry>,
}

impl StorageLayout {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: StorageEntry) {
        self.entries.push(entry);
    }
}
