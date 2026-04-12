//! Conflict descriptors.
//!
//! When two devices modify the same `(table, row, field)` between common
//! ancestors, neither write wins automatically. Both values are recorded as
//! a `Conflict` for the user to resolve via the inline ConflictResolver UI
//! (Phase 3).
//!
//! The descriptor is symmetric — there's no "winner" until the user picks one.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_id: Ulid,
    pub table: String,
    pub row_id: String,
    pub field: String,

    /// Value as it stands locally. `None` means the local op deleted the field
    /// (or set it to NULL).
    pub local_value: Option<JsonValue>,

    /// Value coming from the remote op. Same `None` semantics.
    pub remote_value: Option<JsonValue>,

    pub local_op_id: Ulid,
    pub remote_op_id: Ulid,
    pub detected_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn conflict_roundtrip_serialization() {
        let conflict = Conflict {
            conflict_id: Ulid::new(),
            table: "pages".to_string(),
            row_id: "page-1".to_string(),
            field: "title".to_string(),
            local_value: Some(json!("Laptop title")),
            remote_value: Some(json!("Desktop title")),
            local_op_id: Ulid::new(),
            remote_op_id: Ulid::new(),
            detected_at: Utc::now(),
        };
        let bytes = serde_json::to_vec(&conflict).unwrap();
        let restored: Conflict = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.field, "title");
        assert_eq!(restored.local_value, Some(json!("Laptop title")));
        assert_eq!(restored.remote_value, Some(json!("Desktop title")));
    }

    #[test]
    fn conflict_with_field_deletion_local_side() {
        let conflict = Conflict {
            conflict_id: Ulid::new(),
            table: "pages".to_string(),
            row_id: "page-1".to_string(),
            field: "subtitle".to_string(),
            local_value: None, // local cleared the field
            remote_value: Some(json!("remote-set value")),
            local_op_id: Ulid::new(),
            remote_op_id: Ulid::new(),
            detected_at: Utc::now(),
        };
        let bytes = serde_json::to_vec(&conflict).unwrap();
        let restored: Conflict = serde_json::from_slice(&bytes).unwrap();
        assert!(restored.local_value.is_none());
        assert_eq!(restored.remote_value, Some(json!("remote-set value")));
    }
}
