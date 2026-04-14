//! Per-Tome stable identity.
//!
//! `tome_metadata` stores a `tome_uuid` key — a per-Tome UUID generated on
//! first access and never changed. This UUID is used as the bucket prefix
//! source (`tomes/{uuid}/`) so the same Tome produces the same prefix on
//! every device, regardless of where the `.tome` file lives locally.
//!
//! Before this module existed, the prefix was derived from the local file
//! path, which meant device A's `/home/james/Tomes/Foo.tome` and device B's
//! `C:\Users\james\Tomes\Foo.tome` hashed to different prefixes —
//! defeating any multi-device restore workflow.

use sqlx::SqlitePool;
use uuid::Uuid;

const KEY: &str = "tome_uuid";

/// Return the Tome's stable UUID, generating one if absent.
///
/// Idempotent: parallel callers will all observe the same value thanks to
/// SQLite's row-level serialization on `INSERT OR IGNORE`.
pub async fn get_or_create_uuid(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let candidate = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT OR IGNORE INTO tome_metadata (key, value) VALUES (?, ?)")
        .bind(KEY)
        .bind(&candidate)
        .execute(pool)
        .await?;

    let stored: String = sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = ?")
        .bind(KEY)
        .fetch_one(pool)
        .await?;
    Ok(stored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE tome_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn generates_on_first_call() {
        let pool = test_pool().await;
        let uuid = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(uuid.len(), 32, "simple UUID is 32 hex chars");
    }

    #[tokio::test]
    async fn stable_across_calls() {
        let pool = test_pool().await;
        let a = get_or_create_uuid(&pool).await.unwrap();
        let b = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(a, b, "must return the same UUID on repeat calls");
    }

    #[tokio::test]
    async fn preserves_existing_value() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO tome_metadata (key, value) VALUES ('tome_uuid', 'preexisting')")
            .execute(&pool)
            .await
            .unwrap();
        let uuid = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(uuid, "preexisting");
    }
}
