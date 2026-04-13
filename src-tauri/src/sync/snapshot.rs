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

use std::path::Path;
use ulid::Ulid;

use super::backend::SyncBackend;
use super::crypto::{self, KeyMaterial};
use super::state::SyncRuntimeState;
use super::SyncResult;

/// Trigger thresholds (kept in code rather than config for now; revisit if
/// users start asking to tune them).
pub const SNAPSHOT_BYTES_THRESHOLD: u64 = 5 * 1024 * 1024; // 5 MB of pending journal
pub const SNAPSHOT_OPS_THRESHOLD: usize = 5_000;

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

    // Open the snapshot file briefly to clear sync_* tables, then close.
    {
        let snapshot_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", tmp_db.display()))
            .await?;
        sqlx::query("DELETE FROM sync_journal_local").execute(&snapshot_pool).await?;
        sqlx::query("DELETE FROM sync_state").execute(&snapshot_pool).await?;
        sqlx::query("DELETE FROM sync_config").execute(&snapshot_pool).await?;
        sqlx::query("DELETE FROM sync_conflicts").execute(&snapshot_pool).await?;
        sqlx::query("VACUUM").execute(&snapshot_pool).await?;
        snapshot_pool.close().await;
    }

    let bytes = tokio::fs::read(&tmp_db).await?;
    let size_bytes = bytes.len() as u64;
    let ciphertext = crypto::encrypt(key, &bytes)?;
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
    let bytes = crypto::decrypt(key, &ciphertext)?;
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

/// Should we take a new snapshot now? Phase 2 evaluates the size + ops
/// thresholds; weekly and on-close paths are added when the runner calls
/// this from those code paths.
pub fn should_snapshot(state: &SyncRuntimeState, journal_bytes: u64, journal_count: usize) -> bool {
    // First-ever snapshot: always.
    if state.last_snapshot_id.is_none() {
        return true;
    }
    journal_bytes >= SNAPSHOT_BYTES_THRESHOLD || journal_count >= SNAPSHOT_OPS_THRESHOLD
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
}
