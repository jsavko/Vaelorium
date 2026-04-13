//! Local sync activity log — per-device history of "did the last sync run?"
//!
//! Not synced across devices (snapshot strip + no journal emission); each
//! device tracks its own. Retention: latest 100 rows per Tome, pruned
//! inline on insert.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;

use super::SyncOutcome;

const RETENTION: i64 = 100;

#[derive(Debug, Clone, Serialize)]
pub struct ActivityRow {
    pub id: i64,
    pub tome_id: String,
    pub started_at: String,
    pub duration_ms: i64,
    pub ops_uploaded: i64,
    pub ops_applied: i64,
    pub conflicts_created: i64,
    pub snapshot_taken: Option<String>,
    pub outcome: String,
    pub error: Option<String>,
}

/// Insert one activity row + prune. Best-effort: any failure is logged
/// but never propagated, since logging must not break a real sync cycle.
pub async fn record(
    pool: &SqlitePool,
    tome_id: &str,
    started_at: DateTime<Utc>,
    duration_ms: i64,
    outcome_kind: &str,
    outcome: Option<&SyncOutcome>,
    error: Option<&str>,
) {
    let (uploaded, applied, conflicts, snapshot) = match outcome {
        Some(o) => (
            o.ops_uploaded as i64,
            o.ops_applied as i64,
            o.conflicts_created as i64,
            o.snapshot_taken.clone(),
        ),
        None => (0, 0, 0, None),
    };
    let insert = sqlx::query(
        "INSERT INTO sync_activity
           (tome_id, started_at, duration_ms, ops_uploaded, ops_applied,
            conflicts_created, snapshot_taken, outcome, error)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(tome_id)
    .bind(started_at.to_rfc3339())
    .bind(duration_ms)
    .bind(uploaded)
    .bind(applied)
    .bind(conflicts)
    .bind(snapshot)
    .bind(outcome_kind)
    .bind(error)
    .execute(pool)
    .await;
    if let Err(e) = insert {
        log::warn!("sync_activity insert failed: {e}");
        return;
    }
    let prune = sqlx::query(
        "DELETE FROM sync_activity
         WHERE tome_id = ?
           AND id NOT IN (
             SELECT id FROM sync_activity
             WHERE tome_id = ? ORDER BY id DESC LIMIT ?
           )",
    )
    .bind(tome_id)
    .bind(tome_id)
    .bind(RETENTION)
    .execute(pool)
    .await;
    if let Err(e) = prune {
        log::warn!("sync_activity prune failed: {e}");
    }
}

pub async fn list(pool: &SqlitePool, limit: i64) -> Result<Vec<ActivityRow>, sqlx::Error> {
    let rows: Vec<(i64, String, String, i64, i64, i64, i64, Option<String>, String, Option<String>)> =
        sqlx::query_as(
            "SELECT id, tome_id, started_at, duration_ms, ops_uploaded, ops_applied,
                    conflicts_created, snapshot_taken, outcome, error
             FROM sync_activity ORDER BY id DESC LIMIT ?",
        )
        .bind(limit.max(1))
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| ActivityRow {
            id: r.0,
            tome_id: r.1,
            started_at: r.2,
            duration_ms: r.3,
            ops_uploaded: r.4,
            ops_applied: r.5,
            conflicts_created: r.6,
            snapshot_taken: r.7,
            outcome: r.8,
            error: r.9,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE sync_activity (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tome_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                ops_uploaded INTEGER NOT NULL DEFAULT 0,
                ops_applied INTEGER NOT NULL DEFAULT 0,
                conflicts_created INTEGER NOT NULL DEFAULT 0,
                snapshot_taken TEXT,
                outcome TEXT NOT NULL,
                error TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn record_inserts_a_row() {
        let pool = fresh_pool().await;
        record(&pool, "t1", Utc::now(), 42, "success", None, None).await;
        let rows = list(&pool, 100).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tome_id, "t1");
        assert_eq!(rows[0].outcome, "success");
        assert_eq!(rows[0].duration_ms, 42);
    }

    #[tokio::test]
    async fn retention_caps_at_100_per_tome() {
        let pool = fresh_pool().await;
        for _ in 0..105 {
            record(&pool, "t1", Utc::now(), 1, "success", None, None).await;
        }
        let rows = list(&pool, 200).await.unwrap();
        assert_eq!(rows.len(), 100);
    }

    #[tokio::test]
    async fn retention_per_tome_independent() {
        let pool = fresh_pool().await;
        for _ in 0..50 {
            record(&pool, "t1", Utc::now(), 1, "success", None, None).await;
        }
        for _ in 0..50 {
            record(&pool, "t2", Utc::now(), 1, "success", None, None).await;
        }
        let rows = list(&pool, 200).await.unwrap();
        assert_eq!(rows.len(), 100);
        assert_eq!(rows.iter().filter(|r| r.tome_id == "t1").count(), 50);
        assert_eq!(rows.iter().filter(|r| r.tome_id == "t2").count(), 50);
    }
}
