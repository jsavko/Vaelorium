//! Sync journal — typed mutation operations and the local-side recording API.
//!
//! Each op records exactly one row mutation. User-facing edits that touch
//! multiple rows (e.g. `reorder_pages` updating 10 sort_orders) share a single
//! `transaction_id` so conflict detection and rollback can reason about them
//! as a unit.
//!
//! Ops are JSON-serialized for storage in `sync_journal_local` and for upload
//! to backends (after encryption). The format is versioned via
//! [`crate::sync::SCHEMA_VERSION`] so future format changes can be detected.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Executor, Sqlite};
use std::collections::BTreeMap;
use ulid::Ulid;
use uuid::Uuid;

use super::SyncResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpKind {
    Insert,
    Update,
    Delete,
}

impl OpKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpKind::Insert => "insert",
            OpKind::Update => "update",
            OpKind::Delete => "delete",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "insert" => Some(OpKind::Insert),
            "update" => Some(OpKind::Update),
            "delete" => Some(OpKind::Delete),
            _ => None,
        }
    }
}

/// One row-level mutation. The atomic unit of sync exchange.
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
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn touched_fields(&self) -> impl Iterator<Item = &String> {
        self.fields.keys().chain(self.prev_fields.keys().filter(|k| !self.fields.contains_key(*k)))
    }
}

/// Builder helpers ----------------------------------------------------------

/// Build an Insert op from the fully-populated row.
pub fn insert_op(
    device_id: Uuid,
    transaction_id: Ulid,
    table: impl Into<String>,
    row_id: impl Into<String>,
    fields: BTreeMap<String, Option<JsonValue>>,
) -> Op {
    Op {
        op_id: Ulid::new(),
        device_id,
        table: table.into(),
        row_id: row_id.into(),
        kind: OpKind::Insert,
        fields,
        prev_fields: BTreeMap::new(),
        schema_version: super::SCHEMA_VERSION,
        timestamp: Utc::now(),
        transaction_id,
    }
}

/// Build an Update op containing only the fields that actually changed.
/// `before` and `after` should be full-row snapshots (caller responsibility).
pub fn update_op(
    device_id: Uuid,
    transaction_id: Ulid,
    table: impl Into<String>,
    row_id: impl Into<String>,
    before: &BTreeMap<String, Option<JsonValue>>,
    after: &BTreeMap<String, Option<JsonValue>>,
) -> Option<Op> {
    let mut fields = BTreeMap::new();
    let mut prev_fields = BTreeMap::new();

    // Look at every key present in either side.
    let mut all_keys: Vec<&String> = before.keys().chain(after.keys()).collect();
    all_keys.sort();
    all_keys.dedup();

    for k in all_keys {
        let b = before.get(k).cloned().unwrap_or(None);
        let a = after.get(k).cloned().unwrap_or(None);
        if b != a {
            fields.insert(k.clone(), a);
            prev_fields.insert(k.clone(), b);
        }
    }

    if fields.is_empty() {
        return None;
    }

    Some(Op {
        op_id: Ulid::new(),
        device_id,
        table: table.into(),
        row_id: row_id.into(),
        kind: OpKind::Update,
        fields,
        prev_fields,
        schema_version: super::SCHEMA_VERSION,
        timestamp: Utc::now(),
        transaction_id,
    })
}

/// Build a Delete op carrying the full pre-deletion row in `prev_fields`.
pub fn delete_op(
    device_id: Uuid,
    transaction_id: Ulid,
    table: impl Into<String>,
    row_id: impl Into<String>,
    before: BTreeMap<String, Option<JsonValue>>,
) -> Op {
    Op {
        op_id: Ulid::new(),
        device_id,
        table: table.into(),
        row_id: row_id.into(),
        kind: OpKind::Delete,
        fields: BTreeMap::new(),
        prev_fields: before,
        schema_version: super::SCHEMA_VERSION,
        timestamp: Utc::now(),
        transaction_id,
    }
}

/// Read all the columns named in `schema` for `row_id` and return as a field
/// map. Used by [`emit_for_row`] to capture before/after state.
pub async fn load_row_via_schema(
    executor: &mut sqlx::SqliteConnection,
    schema: &super::registry::TableSchema,
    row_id: &str,
) -> SyncResult<std::collections::BTreeMap<String, Option<JsonValue>>> {
    use sqlx::Row;

    let cols = schema.columns.join(", ");
    let sql = format!(
        "SELECT {} FROM {} WHERE {} = ?",
        cols, schema.name, schema.primary_key
    );
    let row_opt = sqlx::query(&sql)
        .bind(row_id)
        .fetch_optional(executor)
        .await?;
    let row = row_opt.ok_or_else(|| {
        super::SyncError::Journal(format!("{}: row '{}' not found", schema.name, row_id))
    })?;

    let mut out = std::collections::BTreeMap::new();
    for col in schema.columns.iter() {
        out.insert(col.to_string(), read_col_as_json(&row, col));
    }
    Ok(out)
}

/// Read a single column from a sqlx Row and convert to `Option<JsonValue>`.
/// Tries common SQLite column types in order and falls back to `None`.
fn read_col_as_json(row: &sqlx::sqlite::SqliteRow, col: &str) -> Option<JsonValue> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<Option<String>, _>(col) {
        return v.map(JsonValue::String);
    }
    if let Ok(v) = row.try_get::<Option<i64>, _>(col) {
        return v.map(|n| JsonValue::from(n));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(col) {
        return v.and_then(|n| serde_json::Number::from_f64(n).map(JsonValue::Number));
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(col) {
        return v.map(JsonValue::Bool);
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(col) {
        // Encode BLOB as base64 string. (Used when registry is reading a binary
        // column; the apply path will decode it.)
        return v.map(|bytes| {
            use base64::{engine::general_purpose, Engine as _};
            JsonValue::String(general_purpose::STANDARD.encode(bytes))
        });
    }
    None
}

/// All-in-one helper for op-emission inside a mutation transaction.
///
/// - `before`: the pre-mutation row state for Update/Delete (caller fetched
///   it before running the SQL). For Insert pass `None`.
/// - For Insert/Update, the post-mutation row is fetched here.
/// - No-op when `session` is None (sync not configured).
pub async fn emit_for_row(
    executor: &mut sqlx::SqliteConnection,
    schema: &super::registry::TableSchema,
    row_id: &str,
    kind: OpKind,
    transaction_id: Ulid,
    before: Option<std::collections::BTreeMap<String, Option<JsonValue>>>,
    session: Option<(&str, Uuid)>,
) -> SyncResult<()> {
    let Some((tome_id, device_id)) = session else { return Ok(()) };

    let op = match kind {
        OpKind::Insert => {
            let after = load_row_via_schema(executor, schema, row_id).await?;
            insert_op(device_id, transaction_id, schema.name, row_id, after)
        }
        OpKind::Update => {
            let before = before.ok_or_else(|| {
                super::SyncError::Journal("emit_for_row(Update) requires before state".into())
            })?;
            let after = load_row_via_schema(executor, schema, row_id).await?;
            match update_op(device_id, transaction_id, schema.name, row_id, &before, &after) {
                Some(op) => op,
                None => return Ok(()), // nothing actually changed
            }
        }
        OpKind::Delete => {
            let before = before.ok_or_else(|| {
                super::SyncError::Journal("emit_for_row(Delete) requires before state".into())
            })?;
            delete_op(device_id, transaction_id, schema.name, row_id, before)
        }
    };

    record_op(executor, &op, tome_id).await
}

/// Persist an op into `sync_journal_local`. Atomic with the caller's transaction.
pub async fn record_op<'e, E>(executor: E, op: &Op, tome_id: &str) -> SyncResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let payload = serde_json::json!({
        "fields": &op.fields,
        "prev_fields": &op.prev_fields,
        "device_id": op.device_id.to_string(),
    })
    .to_string();

    sqlx::query(
        r#"INSERT INTO sync_journal_local
             (op_id, tome_id, transaction_id, table_name, row_id,
              op_kind, op_data, schema_version, timestamp)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(op.op_id.to_string())
    .bind(tome_id)
    .bind(op.transaction_id.to_string())
    .bind(&op.table)
    .bind(&op.row_id)
    .bind(op.kind.as_str())
    .bind(payload)
    .bind(op.schema_version as i64)
    .bind(op.timestamp.to_rfc3339())
    .execute(executor)
    .await?;

    Ok(())
}

/// Convenience for callers in the live app: check whether sync is configured
/// for any Tome in this DB; return the tome_id if so, otherwise `None`.
/// Each `.tome` SQLite has at most one sync_config row (one Tome per file).
pub async fn active_tome_id(pool: &sqlx::SqlitePool) -> SyncResult<Option<String>> {
    let row: Option<(String, i64)> = sqlx::query_as(
        "SELECT tome_id, enabled FROM sync_config LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|(id, enabled)| if enabled != 0 { Some(id) } else { None }))
}

/// Returns `(tome_id, device_id)` when sync is enabled, or `None`. Mutation
/// command code calls this once per request and uses the result to decide
/// whether to record ops.
pub async fn active_sync_session(pool: &sqlx::SqlitePool) -> SyncResult<Option<(String, Uuid)>> {
    let row: Option<(String, i64, String)> = sqlx::query_as(
        "SELECT tome_id, enabled, device_id FROM sync_config LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some((id, enabled, device_str)) if enabled != 0 => {
            let dev = Uuid::parse_str(&device_str)
                .map_err(|e| super::SyncError::Journal(format!("bad device_id: {e}")))?;
            Some((id, dev))
        }
        _ => None,
    })
}

/// All pending local ops with `op_id > after`, ascending. Used by the engine
/// to upload journal tails.
pub async fn pending_ops(
    pool: &sqlx::SqlitePool,
    tome_id: &str,
    after: Option<&str>,
) -> SyncResult<Vec<StoredOp>> {
    let after_clause = after.unwrap_or("");
    let rows: Vec<StoredOpRow> = sqlx::query_as(
        r#"SELECT op_id, transaction_id, table_name, row_id, op_kind,
                  op_data, schema_version, timestamp
           FROM sync_journal_local
           WHERE tome_id = ? AND op_id > ?
           ORDER BY op_id ASC"#,
    )
    .bind(tome_id)
    .bind(after_clause)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(StoredOp::try_from).collect()
}

/// Drop journal entries with `op_id <= up_to`. Used after a snapshot is taken.
pub async fn prune_journal(
    pool: &sqlx::SqlitePool,
    tome_id: &str,
    up_to: &str,
) -> SyncResult<u64> {
    let res = sqlx::query(
        "DELETE FROM sync_journal_local WHERE tome_id = ? AND op_id <= ?",
    )
    .bind(tome_id)
    .bind(up_to)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// Reconstructed op pulled from the local journal.
#[derive(Debug, Clone)]
pub struct StoredOp {
    pub op: Op,
}

#[derive(sqlx::FromRow)]
struct StoredOpRow {
    op_id: String,
    transaction_id: String,
    table_name: String,
    row_id: String,
    op_kind: String,
    op_data: String,
    schema_version: i64,
    timestamp: String,
}

impl TryFrom<StoredOpRow> for StoredOp {
    type Error = super::SyncError;
    fn try_from(r: StoredOpRow) -> Result<Self, Self::Error> {
        let op_id = Ulid::from_string(&r.op_id)
            .map_err(|e| super::SyncError::Journal(format!("bad op_id: {e}")))?;
        let transaction_id = Ulid::from_string(&r.transaction_id)
            .map_err(|e| super::SyncError::Journal(format!("bad transaction_id: {e}")))?;
        let kind = OpKind::from_str(&r.op_kind)
            .ok_or_else(|| super::SyncError::Journal(format!("bad op_kind: {}", r.op_kind)))?;
        let payload: serde_json::Value = serde_json::from_str(&r.op_data)?;

        let fields: BTreeMap<String, Option<JsonValue>> = payload
            .get("fields")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let prev_fields: BTreeMap<String, Option<JsonValue>> = payload
            .get("prev_fields")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let device_id_str = payload
            .get("device_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| super::SyncError::Journal("missing device_id in op_data".to_string()))?;
        let device_id = Uuid::parse_str(device_id_str)
            .map_err(|e| super::SyncError::Journal(format!("bad device_id: {e}")))?;
        let timestamp = DateTime::parse_from_rfc3339(&r.timestamp)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| super::SyncError::Journal(format!("bad timestamp: {e}")))?;

        Ok(StoredOp {
            op: Op {
                op_id,
                device_id,
                table: r.table_name,
                row_id: r.row_id,
                kind,
                fields,
                prev_fields,
                schema_version: r.schema_version as u32,
                timestamp,
                transaction_id,
            },
        })
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
        assert_eq!(restored.fields, op.fields);
        assert_eq!(restored.prev_fields, op.prev_fields);
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
    fn delete_op_carries_full_prev_state() {
        let mut prev = BTreeMap::new();
        prev.insert("title".to_string(), Some(json!("doomed")));
        prev.insert("icon".to_string(), Some(json!("☠")));

        let op = delete_op(Uuid::new_v4(), Ulid::new(), "pages", "p-1", prev.clone());
        assert_eq!(op.kind, OpKind::Delete);
        assert!(op.fields.is_empty());
        assert_eq!(op.prev_fields, prev);
    }

    #[test]
    fn update_op_only_includes_changed_fields() {
        let mut before = BTreeMap::new();
        before.insert("title".to_string(), Some(json!("Old")));
        before.insert("icon".to_string(), Some(json!("📜")));
        before.insert("untouched".to_string(), Some(json!("same")));

        let mut after = BTreeMap::new();
        after.insert("title".to_string(), Some(json!("New")));
        after.insert("icon".to_string(), Some(json!("📜")));
        after.insert("untouched".to_string(), Some(json!("same")));

        let op = update_op(Uuid::new_v4(), Ulid::new(), "pages", "p-1", &before, &after).unwrap();
        assert_eq!(op.fields.len(), 1);
        assert_eq!(op.fields.get("title"), Some(&Some(json!("New"))));
        assert_eq!(op.prev_fields.get("title"), Some(&Some(json!("Old"))));
    }

    #[test]
    fn update_op_returns_none_when_nothing_changed() {
        let mut state = BTreeMap::new();
        state.insert("title".to_string(), Some(json!("Same")));
        let op = update_op(Uuid::new_v4(), Ulid::new(), "pages", "p-1", &state, &state);
        assert!(op.is_none());
    }
}
