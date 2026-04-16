//! Snapshot creation, encryption, and bootstrap.
//!
//! A snapshot is an encrypted, point-in-time copy of the entire `.tome` SQLite
//! database produced via `VACUUM INTO`. Snapshots provide:
//! - Fast bootstrap for a new device (download one snapshot + replay journal
//!   tail — much faster than replaying the full journal from origin).
//! - A garbage-collection horizon: journal entries older than the latest
//!   snapshot can be pruned both locally and remotely.
//!
//! Phase 2 ships:
//! - `take_snapshot` — VACUUM INTO → encrypt → upload as `snapshots/<ulid>.snap.enc`.
//! - `restore_snapshot_to_file` — download → decrypt → write bytes to a local
//!   path. The caller is responsible for opening it as a fresh DB. Phase 5
//!   adds a hot-swap-into-running-Tome path; not needed for tests.
//! - `should_snapshot` — trigger evaluator (weekly OR ~5MB / 5000 ops OR Tome
//!   close OR manual). Phase 2 only enforces the size+count thresholds; the
//!   weekly and on-close paths land when the runner that calls this exists
//!   (Phase 3) and the user-facing manual button (Phase 3) wires through.

use std::io::{Read, Write};
use std::path::Path;
use ulid::Ulid;

use super::backend::SyncBackend;
use super::crypto::{self, KeyMaterial};
use super::state::SyncRuntimeState;
use super::SyncResult;

/// Compress + decompress helpers. gzip level 6 (default) is effectively
/// identical to level 9 on SQLite dumps — the input is dominated by
/// page-aligned empty space — so we stick with the default for speed.
/// Measured at design time: ~97% reduction on typical Tomes.
fn gzip_compress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = flate2::write::GzEncoder::new(Vec::with_capacity(data.len() / 20), flate2::Compression::default());
    enc.write_all(data)?;
    enc.finish()
}

fn gzip_decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut dec = flate2::read::GzDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out)?;
    Ok(out)
}

/// Trigger thresholds (kept in code rather than config for now; revisit if
/// users start asking to tune them).
pub const SNAPSHOT_BYTES_THRESHOLD: u64 = 1024 * 1024; // 1 MB of pending journal
pub const SNAPSHOT_OPS_THRESHOLD: usize = 5_000;
/// Maximum age of the last snapshot before a new one is forced, regardless
/// of journal size/count. Keeps the journal tail short for faster restores.
pub const SNAPSHOT_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 3600); // 1 week

#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub snapshot_id: Ulid,
    pub size_bytes: u64,
    pub etag: String,
}

/// Take a snapshot of `tome_pool`'s database. Uses `VACUUM INTO` to produce
/// a defragmented, point-in-time copy without locking ongoing writers.
pub async fn take_snapshot(
    pool: &sqlx::SqlitePool,
    key: &KeyMaterial,
    backend: &dyn SyncBackend,
) -> SyncResult<SnapshotInfo> {
    let snapshot_id = Ulid::new();

    // VACUUM INTO writes to an absolute path. Use a tmp file in the system temp dir.
    let tmpdir = tempfile::tempdir()?;
    let tmp_db = tmpdir.path().join(format!("snapshot-{}.db", snapshot_id));
    let path_str = tmp_db.to_string_lossy().replace('\'', "''");

    // Note: we DELETE local sync_journal_local rows from the snapshot copy
    // before reading bytes, since those rows are local-only state (each device
    // has its own pending queue). sync_config and sync_state are similarly
    // local; clear them too. The destination .tome on a fresh device will
    // re-populate from its own setup.
    sqlx::query(&format!("VACUUM INTO '{}'", path_str))
        .execute(pool)
        .await?;

    // Open the snapshot file briefly to clear device-local sync tables while
    // preserving the journal cursors so a restoring device can replay only the
    // tail instead of the full journal.
    {
        let snapshot_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", tmp_db.display()))
            .await?;
        sqlx::query("DELETE FROM sync_journal_local").execute(&snapshot_pool).await?;
        // Preserve last_applied_op_id and last_uploaded_op_id as journal
        // cursors for restore-time replay. Replace the row with a minimal
        // one carrying only the cursor fields + this snapshot's ID (so the
        // restoring device doesn't immediately re-snapshot).
        let cursor_row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT last_uploaded_op_id, last_applied_op_id FROM sync_state LIMIT 1",
        )
        .fetch_optional(&snapshot_pool)
        .await?;
        sqlx::query("DELETE FROM sync_state").execute(&snapshot_pool).await?;
        if let Some((last_uploaded, last_applied)) = cursor_row {
            sqlx::query(
                "INSERT INTO sync_state (tome_id, last_uploaded_op_id, last_applied_op_id, last_snapshot_id, last_sync_at, last_error)
                 VALUES ('__snapshot__', ?, ?, ?, NULL, NULL)",
            )
            .bind(last_uploaded)
            .bind(last_applied)
            .bind(snapshot_id.to_string())
            .execute(&snapshot_pool)
            .await?;
        }
        sqlx::query("DELETE FROM sync_config").execute(&snapshot_pool).await?;
        sqlx::query("DELETE FROM sync_conflicts").execute(&snapshot_pool).await?;
        // sync_activity is a per-device log; fresh-restored DBs shouldn't
        // inherit the source device's history. Best-effort (table may not
        // exist on snapshots taken before migration 012).
        let _ = sqlx::query("DELETE FROM sync_activity").execute(&snapshot_pool).await;
        sqlx::query("VACUUM").execute(&snapshot_pool).await?;
        snapshot_pool.close().await;
    }

    let bytes = tokio::fs::read(&tmp_db).await?;
    let size_bytes = bytes.len() as u64;
    // gzip then encrypt (order matters: compressing ciphertext is worthless).
    let compressed = gzip_compress(&bytes).map_err(|e| super::SyncError::Io(e))?;
    let ciphertext = crypto::encrypt(key, &compressed)?;
    let key_path = format!("snapshots/{}.snap.enc", snapshot_id);
    let etag = backend.put_object(&key_path, &ciphertext).await?;

    Ok(SnapshotInfo {
        snapshot_id,
        size_bytes,
        etag,
    })
}

/// Download `snapshot_id` from the backend, decrypt, and write the raw
/// SQLite bytes to `dest_path`. Caller opens that path as a new pool.
pub async fn restore_snapshot_to_file(
    snapshot_id: &str,
    key: &KeyMaterial,
    backend: &dyn SyncBackend,
    dest_path: &Path,
) -> SyncResult<()> {
    let key_path = format!("snapshots/{}.snap.enc", snapshot_id);
    let (ciphertext, _etag) = backend.get_object(&key_path).await?;
    let compressed = crypto::decrypt(key, &ciphertext)?;
    let bytes = gzip_decompress(&compressed).map_err(|e| super::SyncError::Io(e))?;
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(dest_path, bytes).await?;
    Ok(())
}

/// Per-Tome snapshot summary used by recovery flow on a fresh device.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TomeSnapshotSummary {
    pub tome_uuid: String,
    pub snapshot_id: String,
    pub size_bytes: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// Discover all Tomes that have at least one snapshot under this backend.
/// Returns the latest snapshot per Tome (ULID order = chronological).
///
/// The backend passed in must be the **raw** (non-prefixed) backend so we
/// can see across all `tomes/<uuid>/` subtrees.
pub async fn list_tome_snapshots(
    raw_backend: &dyn SyncBackend,
) -> SyncResult<Vec<TomeSnapshotSummary>> {
    use std::collections::HashMap;
    let infos = raw_backend.list_prefix("tomes").await?;
    let mut by_tome: HashMap<String, super::backend::ObjectInfo> = HashMap::new();
    for info in infos {
        // Expected key shape: tomes/<uuid>/snapshots/<ulid>.snap.enc
        let parts: Vec<&str> = info.key.split('/').collect();
        if parts.len() != 4 || parts[0] != "tomes" || parts[2] != "snapshots" {
            continue;
        }
        if !parts[3].ends_with(".snap.enc") {
            continue;
        }
        let uuid = parts[1].to_string();
        // ULID file names sort lexicographically = chronologically.
        match by_tome.get(&uuid) {
            Some(existing) if existing.key >= info.key => {}
            _ => {
                by_tome.insert(uuid, info);
            }
        }
    }
    let mut out: Vec<TomeSnapshotSummary> = by_tome
        .into_iter()
        .map(|(tome_uuid, info)| {
            let stem = info
                .key
                .rsplit('/')
                .next()
                .and_then(|f| f.strip_suffix(".snap.enc"))
                .unwrap_or("")
                .to_string();
            TomeSnapshotSummary {
                tome_uuid,
                snapshot_id: stem,
                size_bytes: info.size,
                last_modified: info.last_modified,
            }
        })
        .collect();
    out.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(out)
}

/// Restore a snapshot identified by its full key `tomes/<uuid>/snapshots/<ulid>.snap.enc`
/// from the raw backend. Decrypts and writes raw SQLite bytes to `dest_path`.
pub async fn restore_snapshot_by_key(
    full_key: &str,
    key: &KeyMaterial,
    raw_backend: &dyn SyncBackend,
    dest_path: &Path,
) -> SyncResult<()> {
    let (ciphertext, _etag) = raw_backend.get_object(full_key).await?;
    let compressed = crypto::decrypt(key, &ciphertext)?;
    let bytes = gzip_decompress(&compressed).map_err(|e| super::SyncError::Io(e))?;
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(dest_path, bytes).await?;
    Ok(())
}

/// List snapshots on the backend in chronological (= ULID) order.
pub async fn list_snapshots(backend: &dyn SyncBackend) -> SyncResult<Vec<String>> {
    let infos = backend.list_prefix("snapshots").await?;
    Ok(infos
        .into_iter()
        .filter_map(|i| {
            let name = i.key.rsplit('/').next()?;
            let stem = name.strip_suffix(".snap.enc")?;
            Some(stem.to_string())
        })
        .collect())
}

/// Should we take a new snapshot now? Triggers on:
/// 1. First-ever snapshot (no `last_snapshot_id`).
/// 2. Pending journal exceeds size threshold (1 MB).
/// 3. Pending journal exceeds op-count threshold (5,000).
/// 4. Last snapshot is older than `SNAPSHOT_MAX_AGE` (1 week) — keeps the
///    journal tail short for faster restores on low-activity Tomes.
pub fn should_snapshot(state: &SyncRuntimeState, journal_bytes: u64, journal_count: usize) -> bool {
    // First-ever snapshot: always.
    if state.last_snapshot_id.is_none() {
        return true;
    }
    if journal_bytes >= SNAPSHOT_BYTES_THRESHOLD || journal_count >= SNAPSHOT_OPS_THRESHOLD {
        return true;
    }
    // Weekly trigger: ULID embeds a millisecond timestamp in its first 48
    // bits. If the last snapshot is older than SNAPSHOT_MAX_AGE, force a
    // new one even if the journal is tiny.
    if let Some(ref snap_id) = state.last_snapshot_id {
        if let Ok(ulid) = Ulid::from_string(snap_id) {
            let snap_ms = ulid.timestamp_ms();
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            if now_ms.saturating_sub(snap_ms) >= SNAPSHOT_MAX_AGE.as_millis() as u64 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_snapshot_first_time_returns_true() {
        let state = SyncRuntimeState::default();
        assert!(should_snapshot(&state, 0, 0));
    }

    #[test]
    fn should_snapshot_under_thresholds_returns_false() {
        let mut state = SyncRuntimeState::default();
        state.last_snapshot_id = Some(Ulid::new().to_string());
        assert!(!should_snapshot(&state, 1000, 10));
    }

    #[test]
    fn should_snapshot_above_size_threshold_returns_true() {
        let mut state = SyncRuntimeState::default();
        state.last_snapshot_id = Some(Ulid::new().to_string());
        assert!(should_snapshot(&state, SNAPSHOT_BYTES_THRESHOLD + 1, 10));
    }

    #[test]
    fn should_snapshot_above_ops_threshold_returns_true() {
        let mut state = SyncRuntimeState::default();
        state.last_snapshot_id = Some(Ulid::new().to_string());
        assert!(should_snapshot(&state, 0, SNAPSHOT_OPS_THRESHOLD));
    }

    #[test]
    fn should_snapshot_weekly_trigger_fires_for_old_snapshot() {
        let mut state = SyncRuntimeState::default();
        // Create a ULID from 8 days ago.
        let eight_days_ago_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - 8 * 24 * 3600 * 1000;
        let old_ulid = Ulid::from_parts(eight_days_ago_ms, 0);
        state.last_snapshot_id = Some(old_ulid.to_string());
        // Below both size and count thresholds, but old enough to trigger.
        assert!(should_snapshot(&state, 100, 1));
    }

    #[test]
    fn should_snapshot_weekly_trigger_does_not_fire_for_recent() {
        let mut state = SyncRuntimeState::default();
        // Fresh ULID = just now.
        state.last_snapshot_id = Some(Ulid::new().to_string());
        assert!(!should_snapshot(&state, 100, 1));
    }

    #[tokio::test]
    async fn list_tome_snapshots_groups_by_uuid_and_picks_latest() {
        use crate::sync::backend::filesystem::FilesystemBackend;
        let dir = tempfile::tempdir().unwrap();
        let backend = FilesystemBackend::new(dir.path().to_path_buf()).await.unwrap();

        // Two Tomes, two snapshots each.
        let uuid_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let uuid_b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let snap_a_old = Ulid::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let snap_a_new = Ulid::new();
        let snap_b = Ulid::new();
        backend
            .put_object(&format!("tomes/{uuid_a}/snapshots/{snap_a_old}.snap.enc"), b"x")
            .await
            .unwrap();
        backend
            .put_object(&format!("tomes/{uuid_a}/snapshots/{snap_a_new}.snap.enc"), b"x")
            .await
            .unwrap();
        backend
            .put_object(&format!("tomes/{uuid_b}/snapshots/{snap_b}.snap.enc"), b"x")
            .await
            .unwrap();
        // Decoy: a journal entry should be ignored.
        backend
            .put_object(&format!("tomes/{uuid_a}/journal/{snap_a_old}.op.enc"), b"x")
            .await
            .unwrap();

        let mut summaries = list_tome_snapshots(&backend).await.unwrap();
        summaries.sort_by(|a, b| a.tome_uuid.cmp(&b.tome_uuid));
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].tome_uuid, uuid_a);
        assert_eq!(summaries[0].snapshot_id, snap_a_new.to_string());
        assert_eq!(summaries[1].tome_uuid, uuid_b);
        assert_eq!(summaries[1].snapshot_id, snap_b.to_string());
    }

    #[tokio::test]
    async fn list_tome_snapshots_returns_empty_for_fresh_backend() {
        use crate::sync::backend::filesystem::FilesystemBackend;
        let dir = tempfile::tempdir().unwrap();
        let backend = FilesystemBackend::new(dir.path().to_path_buf()).await.unwrap();
        let summaries = list_tome_snapshots(&backend).await.unwrap();
        assert!(summaries.is_empty());
    }
}
