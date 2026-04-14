//! Per-Tome stable identity.
//!
//! `tome_metadata` stores a `tome_uuid` key — a per-Tome UUID generated on
//! first access and never changed. This UUID is used as the bucket prefix
//! source (`tomes/{uuid}/`) so the same Tome produces the same prefix on
//! every device, regardless of where the `.tome` file lives locally.
//!
//! **Format (post-M5):** canonical hyphenated `8-4-4-4-12` UUIDv4
//! lowercase. Vaelorium Cloud's validator requires this exact form; the
//! filesystem and S3 backends accept any stable string, so unifying on
//! hyphenated keeps all three backends speaking the same dialect and
//! lets users migrate between them without protocol friction.
//!
//! **Migration note:** before M5 the stored value was the simple
//! (32-char, unhyphenated) form. On first read post-upgrade, any
//! 32-char hex stored value is rewritten in place to hyphenated —
//! existing Filesystem/S3 bucket prefixes orphan (under a 32-char
//! folder); users re-enable sync to start fresh under the hyphenated
//! prefix. This was an explicit trade-off: unifying across platforms
//! over preserving legacy bucket contents.

use sqlx::SqlitePool;
use uuid::Uuid;

const KEY: &str = "tome_uuid";

/// Return the Tome's stable UUID in canonical hyphenated form,
/// generating one if absent and migrating any legacy simple-form
/// stored value in place.
///
/// Idempotent: parallel callers will all observe the same value
/// thanks to SQLite's row-level serialization on `INSERT OR IGNORE`
/// + the in-place normalization on read.
pub async fn get_or_create_uuid(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let candidate = Uuid::new_v4().hyphenated().to_string();
    sqlx::query("INSERT OR IGNORE INTO tome_metadata (key, value) VALUES (?, ?)")
        .bind(KEY)
        .bind(&candidate)
        .execute(pool)
        .await?;

    let stored: String = sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = ?")
        .bind(KEY)
        .fetch_one(pool)
        .await?;

    let normalized = normalize(&stored);
    if normalized != stored {
        log::info!(
            "tome_uuid migration: rewriting {stored} → {normalized} (legacy simple form)"
        );
        sqlx::query("UPDATE tome_metadata SET value = ? WHERE key = ?")
            .bind(&normalized)
            .bind(KEY)
            .execute(pool)
            .await?;
    }
    Ok(normalized)
}

/// Convert any accepted UUID form to canonical hyphenated lowercase.
/// Passes through unrecognized shapes unchanged so corrupt values
/// surface loudly in the next consumer instead of silently coercing.
fn normalize(raw: &str) -> String {
    let t = raw.trim();
    if t.len() == 36 && t.as_bytes().iter().filter(|&&b| b == b'-').count() == 4 {
        return t.to_ascii_lowercase();
    }
    if t.len() == 32 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        let s = t.to_ascii_lowercase();
        return format!(
            "{}-{}-{}-{}-{}",
            &s[0..8],
            &s[8..12],
            &s[12..16],
            &s[16..20],
            &s[20..32]
        );
    }
    t.to_string()
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
    async fn generates_hyphenated_uuid_on_first_call() {
        let pool = test_pool().await;
        let uuid = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(uuid.len(), 36, "hyphenated UUID is 36 chars");
        assert_eq!(
            uuid.chars().filter(|&c| c == '-').count(),
            4,
            "four hyphens in 8-4-4-4-12 layout"
        );
    }

    #[tokio::test]
    async fn stable_across_calls() {
        let pool = test_pool().await;
        let a = get_or_create_uuid(&pool).await.unwrap();
        let b = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn migrates_legacy_simple_form_in_place() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO tome_metadata (key, value) VALUES ('tome_uuid', 'abcdef0123456789abcdef0123456789')")
            .execute(&pool)
            .await
            .unwrap();
        let uuid = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(uuid, "abcdef01-2345-6789-abcd-ef0123456789");
        // Stored value is now the hyphenated form.
        let stored: String =
            sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'tome_uuid'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, "abcdef01-2345-6789-abcd-ef0123456789");
    }

    #[tokio::test]
    async fn preserves_already_hyphenated_value() {
        let pool = test_pool().await;
        sqlx::query("INSERT INTO tome_metadata (key, value) VALUES ('tome_uuid', 'abcdef01-2345-6789-abcd-ef0123456789')")
            .execute(&pool)
            .await
            .unwrap();
        let uuid = get_or_create_uuid(&pool).await.unwrap();
        assert_eq!(uuid, "abcdef01-2345-6789-abcd-ef0123456789");
    }
}
