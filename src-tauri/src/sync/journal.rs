//! Sync journal — typed mutation operations.
//!
//! Each op records exactly one row mutation. User-facing edits that touch
//! multiple rows (e.g. renaming an entity that updates 10 wiki link rows)
//! share a single `transaction_id` so conflict detection and rollback can
//! reason about them as a unit.
//!
//! Ops are serialized to JSON for storage in `sync_journal_local` and for
//! upload to backends (after encryption). The format is versioned via
//! [`crate::sync::SCHEMA_VERSION`] so future format changes can be detected.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpKind {
    Insert,
    Update,
    Delete,
}

/// One row-level mutation. The atomic unit of sync exchange.
///
/// `fields` captures the new state for [`OpKind::Insert`] / [`OpKind::Update`];
/// for [`OpKind::Delete`] it is empty. `prev_fields` captures the pre-mutation
/// state of every field present in `fields` — used during conflict detection
/// to determine whether two ops touched truly overlapping data or just shared
/// a row by accident.
///
/// `None` in either map means "field absent" (e.g. an optional column was
/// previously NULL or has been cleared); `Some(value)` is the explicit value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Op {
    pub op_id: Ulid,
    pub device_id: Uuid,
    pub table: String,
    pub row_id: String,
    pub kind: OpKind,
    pub fields: BTreeMap<String, Option<JsonValue>>,
    pub prev_fields: BTreeMap<String, Option<JsonValue>>,
    pub schema_version: u32,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: Ulid,
}

impl Op {
    /// Serialize to canonical JSON bytes for storage / encryption.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes (typically after decryption).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Return the set of field names this op modified.
    pub fn touched_fields(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_op() -> Op {
        let mut fields = BTreeMap::new();
        fields.insert("title".to_string(), Some(json!("New title")));
        fields.insert("content".to_string(), Some(json!("Updated content")));
        fields.insert("archived".to_string(), Some(json!(false)));

        let mut prev_fields = BTreeMap::new();
        prev_fields.insert("title".to_string(), Some(json!("Old title")));
        prev_fields.insert("content".to_string(), Some(json!("Original content")));
        prev_fields.insert("archived".to_string(), Some(json!(false)));

        Op {
            op_id: Ulid::new(),
            device_id: Uuid::new_v4(),
            table: "pages".to_string(),
            row_id: "page-123".to_string(),
            kind: OpKind::Update,
            fields,
            prev_fields,
            schema_version: 1,
            timestamp: Utc::now(),
            transaction_id: Ulid::new(),
        }
    }

    #[test]
    fn op_roundtrip_serialization() {
        let op = sample_op();
        let bytes = op.to_bytes().expect("serialize");
        let restored = Op::from_bytes(&bytes).expect("deserialize");

        assert_eq!(restored.op_id, op.op_id);
        assert_eq!(restored.device_id, op.device_id);
        assert_eq!(restored.table, op.table);
        assert_eq!(restored.row_id, op.row_id);
        assert_eq!(restored.kind, op.kind);
        assert_eq!(restored.fields, op.fields);
        assert_eq!(restored.prev_fields, op.prev_fields);
        assert_eq!(restored.schema_version, op.schema_version);
        assert_eq!(restored.transaction_id, op.transaction_id);
    }

    #[test]
    fn op_kind_serializes_lowercase() {
        let op = Op {
            kind: OpKind::Insert,
            ..sample_op()
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""kind":"insert""#));
    }

    #[test]
    fn touched_fields_returns_modified_field_names() {
        let op = sample_op();
        let touched: Vec<&String> = op.touched_fields().collect();
        assert_eq!(touched.len(), 3);
        assert!(touched.iter().any(|f| f.as_str() == "title"));
        assert!(touched.iter().any(|f| f.as_str() == "content"));
        assert!(touched.iter().any(|f| f.as_str() == "archived"));
    }

    #[test]
    fn delete_op_has_empty_fields() {
        let op = Op {
            kind: OpKind::Delete,
            fields: BTreeMap::new(),
            ..sample_op()
        };
        assert_eq!(op.touched_fields().count(), 0);
        let bytes = op.to_bytes().unwrap();
        let restored = Op::from_bytes(&bytes).unwrap();
        assert_eq!(restored.kind, OpKind::Delete);
        assert!(restored.fields.is_empty());
    }
}
