use serde::{Deserialize, Serialize};

use super::layout::{Provenance, StorageEntry, StorageLayout, StorageType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffStatus {
    Same,
    Added,
    Removed,
    TypeChanged,
    PackingChanged,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityGrade {
    Ok,
    Risk,
    Break,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffEntry {
    pub slot: u128,
    pub old: Option<StorageEntry>,
    pub new: Option<StorageEntry>,
    pub status: DiffStatus,
    pub grade: SeverityGrade,
    pub provenance_old: Option<Provenance>,
    pub provenance_new: Option<Provenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summary {
    pub added: usize,
    pub removed: usize,
    pub type_changed: usize,
    pub packing_changed: usize,
    pub same: usize,
    pub max_grade: SeverityGrade,
}

fn grade_for(status: &DiffStatus) -> SeverityGrade {
    match status {
        DiffStatus::Same => SeverityGrade::Ok,
        DiffStatus::Added => SeverityGrade::Ok,
        DiffStatus::Removed => SeverityGrade::Break,
        DiffStatus::TypeChanged => SeverityGrade::Break,
        DiffStatus::PackingChanged => SeverityGrade::Risk,
    }
}

fn is_same_type(a: &StorageType, b: &StorageType) -> bool {
    a == b
}

fn packing_key(e: &StorageEntry) -> (Option<u8>, Option<u8>) {
    (e.offset, e.size)
}

pub fn diff_layouts(old: &StorageLayout, new: &StorageLayout) -> (Vec<DiffEntry>, Summary) {
    // Group by slot, keeping a single representative per slot for conservative policy.
    // If multiple packed fields exist, we compare by type and packing attributes; if any differ, mark PackingChanged.

    use std::collections::BTreeMap;

    let mut old_map: BTreeMap<u128, Vec<&StorageEntry>> = BTreeMap::new();
    for e in &old.entries {
        old_map.entry(e.slot).or_default().push(e);
    }

    let mut new_map: BTreeMap<u128, Vec<&StorageEntry>> = BTreeMap::new();
    for e in &new.entries {
        new_map.entry(e.slot).or_default().push(e);
    }

    let mut all_slots: BTreeMap<u128, ()> = BTreeMap::new();
    for s in old_map.keys() {
        all_slots.insert(*s, ());
    }
    for s in new_map.keys() {
        all_slots.insert(*s, ());
    }

    let mut diffs: Vec<DiffEntry> = Vec::new();
    let mut summary = Summary {
        added: 0,
        removed: 0,
        type_changed: 0,
        packing_changed: 0,
        same: 0,
        max_grade: SeverityGrade::Ok,
    };

    for slot in all_slots.keys().copied() {
        let old_entries = old_map.get(&slot).cloned().unwrap_or_default();
        let new_entries = new_map.get(&slot).cloned().unwrap_or_default();

        let status = match (old_entries.is_empty(), new_entries.is_empty()) {
            (true, false) => DiffStatus::Added,
            (false, true) => DiffStatus::Removed,
            (true, true) => continue,
            (false, false) => {
                // Compare conservatively
                // If any type differs across matched pairs, TypeChanged.
                // Else if packing differs (offset/size counts differ), PackingChanged.
                let mut type_changed = false;
                let mut packing_changed = false;

                let min_len = old_entries.len().min(new_entries.len());
                for i in 0..min_len {
                    if !is_same_type(&old_entries[i].r#type, &new_entries[i].r#type) {
                        type_changed = true;
                        break;
                    }
                    if packing_key(old_entries[i]) != packing_key(new_entries[i]) {
                        packing_changed = true;
                    }
                }
                if !type_changed {
                    if old_entries.len() != new_entries.len() {
                        packing_changed = true;
                    }
                }

                if type_changed {
                    DiffStatus::TypeChanged
                } else if packing_changed {
                    DiffStatus::PackingChanged
                } else {
                    DiffStatus::Same
                }
            }
        };

        let grade = grade_for(&status);
        summary.max_grade = summary.max_grade.max(grade);
        match status {
            DiffStatus::Same => summary.same += 1,
            DiffStatus::Added => summary.added += 1,
            DiffStatus::Removed => summary.removed += 1,
            DiffStatus::TypeChanged => summary.type_changed += 1,
            DiffStatus::PackingChanged => summary.packing_changed += 1,
        }

        let old_rep = old_entries.first().cloned().cloned();
        let new_rep = new_entries.first().cloned().cloned();

        diffs.push(DiffEntry {
            slot,
            old: old_rep.clone(),
            new: new_rep.clone(),
            status,
            grade,
            provenance_old: old_rep.as_ref().map(|e| e.provenance),
            provenance_new: new_rep.as_ref().map(|e| e.provenance),
        });
    }

    (diffs, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        slot: u128,
        ty: StorageType,
        offset: Option<u8>,
        size: Option<u8>,
        prov: Provenance,
    ) -> StorageEntry {
        StorageEntry {
            slot,
            offset,
            size,
            r#type: ty,
            label: None,
            provenance: prov,
        }
    }

    #[test]
    fn added_and_removed() {
        let old = StorageLayout {
            entries: vec![entry(
                0,
                StorageType::Unknown,
                None,
                None,
                Provenance::HeuristicTrace,
            )],
        };
        let new = StorageLayout {
            entries: vec![entry(
                1,
                StorageType::Unknown,
                None,
                None,
                Provenance::HeuristicTrace,
            )],
        };
        let (_diffs, summary) = diff_layouts(&old, &new);
        assert_eq!(summary.added, 1);
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.max_grade, SeverityGrade::Break);
    }

    #[test]
    fn type_changed_breaks() {
        let old = StorageLayout {
            entries: vec![entry(
                0,
                StorageType::Uint { bits: 256 },
                None,
                None,
                Provenance::CompilerMetadata,
            )],
        };
        let new = StorageLayout {
            entries: vec![entry(
                0,
                StorageType::Address,
                None,
                None,
                Provenance::CompilerMetadata,
            )],
        };
        let (_diffs, summary) = diff_layouts(&old, &new);
        assert_eq!(summary.type_changed, 1);
        assert_eq!(summary.max_grade, SeverityGrade::Break);
    }

    #[test]
    fn packing_changed_risk() {
        let old = StorageLayout {
            entries: vec![entry(
                0,
                StorageType::Uint { bits: 256 },
                Some(0),
                Some(32),
                Provenance::CompilerMetadata,
            )],
        };
        let new = StorageLayout {
            entries: vec![entry(
                0,
                StorageType::Uint { bits: 256 },
                Some(1),
                Some(31),
                Provenance::CompilerMetadata,
            )],
        };
        let (_diffs, summary) = diff_layouts(&old, &new);
        assert_eq!(summary.packing_changed, 1);
        assert_eq!(summary.max_grade, SeverityGrade::Risk);
    }

    #[test]
    fn same_ok() {
        let e = entry(
            2,
            StorageType::Bool,
            None,
            None,
            Provenance::CompilerMetadata,
        );
        let old = StorageLayout {
            entries: vec![e.clone()],
        };
        let new = StorageLayout { entries: vec![e] };
        let (_diffs, summary) = diff_layouts(&old, &new);
        assert_eq!(summary.same, 1);
        assert_eq!(summary.max_grade, SeverityGrade::Ok);
    }
}
