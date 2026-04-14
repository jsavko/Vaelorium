//! Per-Tome sync configuration & runtime state.
//!
//! [`SyncConfig`] is per-Tome — just `enabled` flag + device identity. The
//! backend credentials and passphrase salt are app-global and live in
//! `sync-backend.json` + the OS keychain (see `sync::app_backend`).
//!
//! [`SyncRuntimeState`] — engine-managed: where we are in the journal
//! exchange, last sync time, last error. Mutates on every sync cycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::SyncResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Filesystem,
    S3,
    /// Vaelorium Cloud — hosted paid tier at cloud.vaelorium.com.
    Hosted,
}

impl BackendKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendKind::Filesystem => "filesystem",
            BackendKind::S3 => "s3",
            BackendKind::Hosted => "hosted",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "filesystem" => Some(BackendKind::Filesystem),
            "s3" => Some(BackendKind::S3),
            "hosted" => Some(BackendKind::Hosted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub tome_id: String,
    pub enabled: bool,
    /// Copy of the app-global device_id, stamped at sync_enable time so
    /// journal::active_sync_session can read it via a simple pool query
    /// without plumbing SessionState through every mutation command.
    pub device_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncRuntimeState {
    pub tome_id: String,
    pub last_uploaded_op_id: Option<String>,
    pub last_applied_op_id: Option<String>,
    pub last_snapshot_id: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl SyncConfig {
    pub async fn load(pool: &SqlitePool, tome_id: &str) -> SyncResult<Option<Self>> {
        let row: Option<SyncConfigRow> = sqlx::query_as(
            r#"SELECT tome_id, enabled, device_id, created_at, updated_at
               FROM sync_config WHERE tome_id = ?"#,
        )
        .bind(tome_id)
        .fetch_optional(pool)
        .await?;

        row.map(SyncConfig::try_from).transpose()
    }

    /// List every Tome with sync_config row (enabled or not). Used by the
    /// runner to iterate enabled Tomes.
    pub async fn list_all(pool: &SqlitePool) -> SyncResult<Vec<Self>> {
        let rows: Vec<SyncConfigRow> = sqlx::query_as(
            r#"SELECT tome_id, enabled, device_id, created_at, updated_at
               FROM sync_config ORDER BY created_at"#,
        )
        .fetch_all(pool)
        .await?;
        rows.into_iter().map(SyncConfig::try_from).collect()
    }

    pub async fn save(&self, pool: &SqlitePool) -> SyncResult<()> {
        sqlx::query(
            r#"INSERT INTO sync_config
                 (tome_id, enabled, device_id, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(tome_id) DO UPDATE SET
                 enabled    = excluded.enabled,
                 device_id  = excluded.device_id,
                 updated_at = excluded.updated_at"#,
        )
        .bind(&self.tome_id)
        .bind(self.enabled as i32)
        .bind(self.device_id.to_string())
        .bind(self.created_at.to_rfc3339())
        .bind(self.updated_at.to_rfc3339())
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl SyncRuntimeState {
    pub async fn load(pool: &SqlitePool, tome_id: &str) -> SyncResult<Self> {
        let row: Option<SyncStateRow> = sqlx::query_as(
            r#"SELECT tome_id, last_uploaded_op_id, last_applied_op_id,
                      last_snapshot_id, last_sync_at, last_error
               FROM sync_state WHERE tome_id = ?"#,
        )
        .bind(tome_id)
        .fetch_optional(pool)
        .await?;

        Ok(row
            .map(SyncRuntimeState::from)
            .unwrap_or_else(|| SyncRuntimeState {
                tome_id: tome_id.to_string(),
                ..Default::default()
            }))
    }

    pub async fn save(&self, pool: &SqlitePool) -> SyncResult<()> {
        sqlx::query(
            r#"INSERT INTO sync_state
                 (tome_id, last_uploaded_op_id, last_applied_op_id,
                  last_snapshot_id, last_sync_at, last_error)
               VALUES (?, ?, ?, ?, ?, ?)
               ON CONFLICT(tome_id) DO UPDATE SET
                 last_uploaded_op_id = excluded.last_uploaded_op_id,
                 last_applied_op_id  = excluded.last_applied_op_id,
                 last_snapshot_id    = excluded.last_snapshot_id,
                 last_sync_at        = excluded.last_sync_at,
                 last_error          = excluded.last_error"#,
        )
        .bind(&self.tome_id)
        .bind(self.last_uploaded_op_id.as_deref())
        .bind(self.last_applied_op_id.as_deref())
        .bind(self.last_snapshot_id.as_deref())
        .bind(self.last_sync_at.map(|t| t.to_rfc3339()))
        .bind(self.last_error.as_deref())
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SyncConfigRow {
    tome_id: String,
    enabled: i64,
    device_id: String,
    created_at: String,
    updated_at: String,
}

impl TryFrom<SyncConfigRow> for SyncConfig {
    type Error = super::SyncError;
    fn try_from(r: SyncConfigRow) -> Result<Self, Self::Error> {
        Ok(SyncConfig {
            tome_id: r.tome_id,
            enabled: r.enabled != 0,
            device_id: Uuid::parse_str(&r.device_id)
                .map_err(|e| super::SyncError::Journal(format!("bad device_id: {e}")))?,
            created_at: parse_rfc3339(&r.created_at)?,
            updated_at: parse_rfc3339(&r.updated_at)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct SyncStateRow {
    tome_id: String,
    last_uploaded_op_id: Option<String>,
    last_applied_op_id: Option<String>,
    last_snapshot_id: Option<String>,
    last_sync_at: Option<String>,
    last_error: Option<String>,
}

impl From<SyncStateRow> for SyncRuntimeState {
    fn from(r: SyncStateRow) -> Self {
        SyncRuntimeState {
            tome_id: r.tome_id,
            last_uploaded_op_id: r.last_uploaded_op_id,
            last_applied_op_id: r.last_applied_op_id,
            last_snapshot_id: r.last_snapshot_id,
            last_sync_at: r.last_sync_at.and_then(|s| parse_rfc3339(&s).ok()),
            last_error: r.last_error,
        }
    }
}

fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, super::SyncError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| super::SyncError::Journal(format!("bad timestamp '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    fn split_sql(sql: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = String::new();
        let mut depth: i32 = 0;
        for line in sql.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            for ch in trimmed.chars() {
                if ch == '(' { depth += 1; }
                if ch == ')' { depth -= 1; }
            }
            if !current.is_empty() { current.push(' '); }
            current.push_str(trimmed);
            if trimmed.ends_with(';') && depth == 0 {
                let stmt = current.trim_end_matches(';').trim().to_string();
                if !stmt.is_empty() { out.push(stmt); }
                current.clear();
            }
        }
        out
    }

    async fn make_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        // Apply 009 then 010 to mirror prod migration order.
        for sql in [
            include_str!("../../migrations/009_sync.sql"),
            include_str!("../../migrations/010_sync_app_global.sql"),
            include_str!("../../migrations/011_device_name_app_global.sql"),
        ] {
            for stmt in split_sql(sql) {
                sqlx::query(&stmt).execute(&pool).await.unwrap();
            }
        }
        pool
    }

    fn sample_config() -> SyncConfig {
        SyncConfig {
            tome_id: "tome-test".to_string(),
            enabled: true,
            device_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn config_save_and_load_roundtrip() {
        let pool = make_pool().await;
        let cfg = sample_config();
        cfg.save(&pool).await.unwrap();

        let loaded = SyncConfig::load(&pool, "tome-test").await.unwrap().unwrap();
        assert_eq!(loaded.tome_id, cfg.tome_id);
        assert_eq!(loaded.enabled, cfg.enabled);
        assert_eq!(loaded.device_id, cfg.device_id);
    }

    #[tokio::test]
    async fn config_load_returns_none_for_unknown_tome() {
        let pool = make_pool().await;
        let result = SyncConfig::load(&pool, "nope").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn config_save_upserts() {
        let pool = make_pool().await;
        let mut cfg = sample_config();
        cfg.save(&pool).await.unwrap();
        let new_id = Uuid::new_v4();
        cfg.device_id = new_id;
        cfg.save(&pool).await.unwrap();

        let loaded = SyncConfig::load(&pool, "tome-test").await.unwrap().unwrap();
        assert_eq!(loaded.device_id, new_id);
    }

    #[tokio::test]
    async fn runtime_state_load_creates_default_for_new_tome() {
        let pool = make_pool().await;
        let state = SyncRuntimeState::load(&pool, "fresh-tome").await.unwrap();
        assert_eq!(state.tome_id, "fresh-tome");
        assert!(state.last_uploaded_op_id.is_none());
        assert!(state.last_applied_op_id.is_none());
    }

    #[tokio::test]
    async fn runtime_state_save_and_load_roundtrip() {
        let pool = make_pool().await;
        let now = Utc::now();
        let state = SyncRuntimeState {
            tome_id: "tome-test".to_string(),
            last_uploaded_op_id: Some("01HJK000000000000000000000".to_string()),
            last_applied_op_id: Some("01HJK000000000000000000001".to_string()),
            last_snapshot_id: Some("01HJK000000000000000000002".to_string()),
            last_sync_at: Some(now),
            last_error: None,
        };
        state.save(&pool).await.unwrap();

        let loaded = SyncRuntimeState::load(&pool, "tome-test").await.unwrap();
        assert_eq!(loaded.last_uploaded_op_id, state.last_uploaded_op_id);
        assert_eq!(loaded.last_applied_op_id, state.last_applied_op_id);
        assert_eq!(loaded.last_snapshot_id, state.last_snapshot_id);
        assert_eq!(
            loaded.last_sync_at.unwrap().timestamp(),
            state.last_sync_at.unwrap().timestamp()
        );
    }

    #[test]
    fn backend_kind_roundtrip() {
        assert_eq!(BackendKind::from_str("filesystem"), Some(BackendKind::Filesystem));
        assert_eq!(BackendKind::from_str("s3"), Some(BackendKind::S3));
        assert_eq!(BackendKind::from_str("hosted"), Some(BackendKind::Hosted));
        assert_eq!(BackendKind::from_str("nope"), None);
        assert_eq!(BackendKind::Filesystem.as_str(), "filesystem");
        assert_eq!(BackendKind::S3.as_str(), "s3");
        assert_eq!(BackendKind::Hosted.as_str(), "hosted");
    }
}
