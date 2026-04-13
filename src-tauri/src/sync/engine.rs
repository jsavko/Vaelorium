//! The sync runner: one full sync cycle per call.
//!
//! Phase 2 implements the engine for the `pages` and `page_content` tables.
//! Other tables follow the same shape (Phase 4).

use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use ulid::Ulid;

use super::backend::{BackendError, SyncBackend};
use super::conflict::Conflict;
use super::crypto::{self, KeyMaterial};
use super::journal::{self, Op, OpKind, StoredOp};
use super::snapshot::{self, SnapshotInfo, should_snapshot};
use super::state::SyncRuntimeState;
use super::{SyncError, SyncResult, SCHEMA_VERSION};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncOutcome {
    pub ops_uploaded: u32,
    pub ops_applied: u32,
    pub conflicts_created: u32,
    pub snapshot_taken: Option<String>,
    pub error: Option<String>,
}

/// Run one sync cycle: upload local journal, download remote journal, apply
/// remote ops with conflict detection, take a snapshot if triggered.
pub async fn sync_tome_once(
    pool: &SqlitePool,
    tome_id: &str,
    key: &KeyMaterial,
    backend: &dyn SyncBackend,
) -> SyncResult<SyncOutcome> {
    let mut outcome = SyncOutcome::default();
    let mut state = SyncRuntimeState::load(pool, tome_id).await?;

    // 1. Upload local pending ops.
    let to_upload = journal::pending_ops(pool, tome_id, state.last_uploaded_op_id.as_deref()).await?;
    for stored in &to_upload {
        let bytes = stored.op.to_bytes()?;
        let ciphertext = crypto::encrypt(key, &bytes)?;
        let key_path = format!("journal/{}.op.enc", stored.op.op_id);
        backend.put_object(&key_path, &ciphertext).await?;
        outcome.ops_uploaded += 1;
        state.last_uploaded_op_id = Some(stored.op.op_id.to_string());
    }

    // 2. List remote journal newer than what we've applied.
    let remote_objects = backend.list_prefix("journal").await?;
    let mut remote_ops: Vec<(String, Op)> = Vec::new();
    for obj in remote_objects {
        let Some(name) = obj.key.rsplit('/').next() else { continue };
        let Some(id_str) = name.strip_suffix(".op.enc") else { continue };
        if let Some(last) = &state.last_applied_op_id {
            if id_str <= last.as_str() {
                continue;
            }
        }
        let (ciphertext, _etag) = backend.get_object(&obj.key).await?;
        let plaintext = crypto::decrypt(key, &ciphertext)?;
        let op = Op::from_bytes(&plaintext)?;

        // Skip our own ops we just uploaded — they'd be no-ops (we already have them).
        // Detect via op_id present in our local journal.
        let is_local: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sync_journal_local WHERE op_id = ?)",
        )
        .bind(op.op_id.to_string())
        .fetch_one(pool)
        .await?;
        if is_local {
            // Advance pointer past our own ops too.
            state.last_applied_op_id = Some(op.op_id.to_string());
            continue;
        }

        // Schema version guard.
        if op.schema_version > SCHEMA_VERSION {
            state.last_error = Some(format!(
                "remote op {} requires schema v{}, this client is v{}",
                op.op_id, op.schema_version, SCHEMA_VERSION
            ));
            // Stop here — don't apply newer ops out of order.
            state.last_sync_at = Some(Utc::now());
            state.save(pool).await?;
            outcome.error = state.last_error.clone();
            return Ok(outcome);
        }

        remote_ops.push((id_str.to_string(), op));
    }

    // Sort by op_id (ULID is time-ordered + monotonic per device, ascending).
    remote_ops.sort_by(|a, b| a.0.cmp(&b.0));

    // 3. Apply remote ops with conflict detection.
    for (id_str, op) in remote_ops {
        let conflicts = detect_conflicts(pool, tome_id, &op).await?;
        for conflict in &conflicts {
            persist_conflict(pool, tome_id, conflict).await?;
            outcome.conflicts_created += 1;
        }

        // Filter conflicted fields out of the op before applying.
        let conflicted_fields: std::collections::HashSet<String> =
            conflicts.iter().map(|c| c.field.clone()).collect();
        let filtered_op = filter_fields(&op, &conflicted_fields);

        if filtered_op.fields.is_empty()
            && filtered_op.kind != OpKind::Delete
            && filtered_op.kind != OpKind::Insert
        {
            // Pure-conflict update with nothing left to apply: nothing to do.
        } else {
            apply_op(pool, &filtered_op).await?;
            outcome.ops_applied += 1;
        }

        state.last_applied_op_id = Some(id_str);
    }

    // 4. Snapshot if triggered.
    let pending_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_journal_local WHERE tome_id = ?")
            .bind(tome_id)
            .fetch_one(pool)
            .await?;
    let pending_bytes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(LENGTH(op_data)), 0) FROM sync_journal_local WHERE tome_id = ?",
    )
    .bind(tome_id)
    .fetch_one(pool)
    .await?;

    if should_snapshot(&state, pending_bytes as u64, pending_count as usize) {
        match snapshot::take_snapshot(pool, key, backend).await {
            Ok(SnapshotInfo { snapshot_id, .. }) => {
                state.last_snapshot_id = Some(snapshot_id.to_string());
                outcome.snapshot_taken = Some(snapshot_id.to_string());
                // Prune local journal entries strictly older than this snapshot.
                let _ = journal::prune_journal(pool, tome_id, &snapshot_id.to_string()).await?;
            }
            Err(e) => {
                // Snapshot failure shouldn't fail the whole sync — log and continue.
                state.last_error = Some(format!("snapshot failed: {e}"));
            }
        }
    }

    state.last_sync_at = Some(Utc::now());
    state.last_error = outcome.error.clone();
    state.save(pool).await?;
    Ok(outcome)
}

/// Detect per-field conflicts between an incoming remote op and any unsynced
/// local ops touching the same row. Returns one [`Conflict`] per conflicted
/// field. A conflict exists when both sides modified the same field but to
/// different values.
/// Bookkeeping columns that are touched on every write (timestamps, audit
/// identities) and so would conflict on every concurrent edit, even when the
/// user-meaningful change was on disjoint fields. We treat these as
/// last-write-wins silently.
fn is_meta_field(field: &str) -> bool {
    matches!(
        field,
        "updated_at" | "created_at" | "updated_by" | "created_by"
    )
}

async fn detect_conflicts(
    pool: &SqlitePool,
    tome_id: &str,
    remote: &Op,
) -> SyncResult<Vec<Conflict>> {
    // Only conflicting if the local side has unsynced ops touching this row.
    let local_ops = journal::pending_ops(pool, tome_id, None).await?;
    let local_for_row: Vec<&StoredOp> = local_ops
        .iter()
        .filter(|s| s.op.table == remote.table && s.op.row_id == remote.row_id)
        .collect();
    if local_for_row.is_empty() {
        return Ok(Vec::new());
    }

    let mut conflicts = Vec::new();
    let now = Utc::now();

    // Build a map of local field state from the latest local op per field.
    use std::collections::HashMap;
    let mut local_field_state: HashMap<String, (Option<serde_json::Value>, Ulid)> = HashMap::new();
    for stored in &local_for_row {
        for (k, v) in &stored.op.fields {
            local_field_state.insert(k.clone(), (v.clone(), stored.op.op_id));
        }
        // For deletes, every prev_field is now "absent" locally.
        if stored.op.kind == OpKind::Delete {
            for k in stored.op.prev_fields.keys() {
                local_field_state.insert(k.clone(), (None, stored.op.op_id));
            }
        }
    }

    // Compare remote's intended new values against the local current state.
    for (field, remote_value) in &remote.fields {
        if is_meta_field(field) {
            continue; // bookkeeping field — let LWW play out silently
        }
        if let Some((local_value, local_op_id)) = local_field_state.get(field) {
            if local_value != remote_value {
                conflicts.push(Conflict {
                    conflict_id: Ulid::new(),
                    table: remote.table.clone(),
                    row_id: remote.row_id.clone(),
                    field: field.clone(),
                    local_value: local_value.clone(),
                    remote_value: remote_value.clone(),
                    local_op_id: *local_op_id,
                    remote_op_id: remote.op_id,
                    detected_at: now,
                });
            }
        }
    }

    Ok(conflicts)
}

async fn persist_conflict(pool: &SqlitePool, tome_id: &str, c: &Conflict) -> SyncResult<()> {
    let local_json = c
        .local_value
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    let remote_json = c
        .remote_value
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    sqlx::query(
        r#"INSERT INTO sync_conflicts
             (conflict_id, tome_id, table_name, row_id, field_name,
              local_value, remote_value, local_op_id, remote_op_id, detected_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(c.conflict_id.to_string())
    .bind(tome_id)
    .bind(&c.table)
    .bind(&c.row_id)
    .bind(&c.field)
    .bind(local_json)
    .bind(remote_json)
    .bind(c.local_op_id.to_string())
    .bind(c.remote_op_id.to_string())
    .bind(c.detected_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

fn filter_fields(op: &Op, exclude: &std::collections::HashSet<String>) -> Op {
    let mut filtered = op.clone();
    filtered.fields.retain(|k, _| !exclude.contains(k));
    filtered.prev_fields.retain(|k, _| !exclude.contains(k));
    filtered
}

/// Apply a remote op against the local DB. Phase 2 supports `pages` and
/// `page_content` only. Other tables added in Phase 4.
///
/// Crucially, this writes raw SQL — it does NOT call [`journal::record_op`],
/// so applying a remote op doesn't loop back into the journal.
async fn apply_op(pool: &SqlitePool, op: &Op) -> SyncResult<()> {
    match op.table.as_str() {
        "pages" => apply_pages_op(pool, op).await,
        "page_content" => apply_page_content_op(pool, op).await,
        other => Err(SyncError::Journal(format!(
            "apply_op: unknown table '{other}' in Phase 2"
        ))),
    }
}

async fn apply_pages_op(pool: &SqlitePool, op: &Op) -> SyncResult<()> {
    use serde_json::Value;
    fn s(v: &Option<Value>) -> Option<String> {
        v.as_ref().and_then(|x| x.as_str().map(|s| s.to_string()))
    }
    fn i(v: &Option<Value>) -> Option<i64> {
        v.as_ref().and_then(|x| x.as_i64())
    }

    match op.kind {
        OpKind::Insert => {
            sqlx::query(
                r#"INSERT OR REPLACE INTO pages
                     (id, title, icon, featured_image_path, parent_id,
                      sort_order, entity_type_id, visibility,
                      created_at, updated_at, created_by, updated_by)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&op.row_id)
            .bind(s(op.fields.get("title").unwrap_or(&None)).unwrap_or_default())
            .bind(s(op.fields.get("icon").unwrap_or(&None)))
            .bind(s(op.fields.get("featured_image_path").unwrap_or(&None)))
            .bind(s(op.fields.get("parent_id").unwrap_or(&None)))
            .bind(i(op.fields.get("sort_order").unwrap_or(&None)).unwrap_or(0))
            .bind(s(op.fields.get("entity_type_id").unwrap_or(&None)))
            .bind(s(op.fields.get("visibility").unwrap_or(&None)).unwrap_or_else(|| "private".to_string()))
            .bind(s(op.fields.get("created_at").unwrap_or(&None)).unwrap_or_else(|| Utc::now().to_rfc3339()))
            .bind(s(op.fields.get("updated_at").unwrap_or(&None)).unwrap_or_else(|| Utc::now().to_rfc3339()))
            .bind(s(op.fields.get("created_by").unwrap_or(&None)))
            .bind(s(op.fields.get("updated_by").unwrap_or(&None)))
            .execute(pool)
            .await?;
        }
        OpKind::Update => {
            for (field, value) in &op.fields {
                let sql = format!("UPDATE pages SET {} = ? WHERE id = ?", field);
                let q = match value {
                    Some(v) if v.is_string() => sqlx::query(&sql).bind(v.as_str().unwrap()),
                    Some(v) if v.is_i64() => sqlx::query(&sql).bind(v.as_i64().unwrap()),
                    Some(v) if v.is_boolean() => sqlx::query(&sql).bind(v.as_bool().unwrap()),
                    Some(_) => {
                        return Err(SyncError::Journal(format!(
                            "unsupported field value type for pages.{field}"
                        )))
                    }
                    None => sqlx::query(&sql).bind(Option::<String>::None),
                };
                q.bind(&op.row_id).execute(pool).await?;
            }
        }
        OpKind::Delete => {
            sqlx::query("DELETE FROM pages WHERE id = ?")
                .bind(&op.row_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

async fn apply_page_content_op(pool: &SqlitePool, op: &Op) -> SyncResult<()> {
    match op.kind {
        OpKind::Insert | OpKind::Update => {
            let yjs_state = op
                .fields
                .get("yjs_state")
                .and_then(|v| v.as_ref())
                .and_then(|v| {
                    // Stored as base64 string in JSON (BLOB serializer choice for Phase 2).
                    v.as_str().map(|s| {
                        use base64::{engine::general_purpose, Engine as _};
                        general_purpose::STANDARD.decode(s).unwrap_or_default()
                    })
                })
                .unwrap_or_default();

            sqlx::query(
                r#"INSERT INTO page_content (page_id, yjs_state, yjs_version)
                   VALUES (?, ?, 0)
                   ON CONFLICT(page_id) DO UPDATE SET
                     yjs_state = excluded.yjs_state,
                     yjs_version = page_content.yjs_version + 1"#,
            )
            .bind(&op.row_id)
            .bind(&yjs_state)
            .execute(pool)
            .await?;
        }
        OpKind::Delete => {
            sqlx::query("DELETE FROM page_content WHERE page_id = ?")
                .bind(&op.row_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// Backend-not-found is treated as no-op for operations that read on a fresh
/// device; the engine should tolerate empty backends gracefully.
#[allow(dead_code)]
fn ignore_not_found<T: Default>(r: Result<T, BackendError>) -> Result<T, BackendError> {
    match r {
        Ok(v) => Ok(v),
        Err(BackendError::NotFound(_)) => Ok(T::default()),
        Err(e) => Err(e),
    }
}
