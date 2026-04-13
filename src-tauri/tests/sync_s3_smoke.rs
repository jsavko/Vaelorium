//! One-shot end-to-end smoke test against a real S3-compatible service.
//!
//! Ignored by default — requires environment variables to be set. Run with:
//! ```
//! S3_ENDPOINT=https://s3.us-west-000.backblazeb2.com \
//! S3_REGION=us-west-000 \
//! S3_BUCKET=PhoenixTest \
//! S3_ACCESS_KEY=... \
//! S3_SECRET_KEY=... \
//! cargo test --test sync_s3_smoke -- --ignored --nocapture
//! ```
//!
//! Validates: backend init, real upload of encrypted ops + snapshot, remote
//! object listing, round-trip via sync_tome_once with a second simulated
//! device on the same bucket prefix.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;

use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tempfile::TempDir;
use ulid::Ulid;
use uuid::Uuid;

use vaelorium_lib::sync::backend::s3::{S3Backend, S3Config};
use vaelorium_lib::sync::backend::SyncBackend;
use vaelorium_lib::sync::crypto::{generate_salt, KeyMaterial};
use vaelorium_lib::sync::journal::{self, insert_op, record_op};
use vaelorium_lib::sync::state::{BackendKind, SyncConfig};
use vaelorium_lib::sync::sync_tome_once;

const TOME_ID: &str = "s3-smoke-test-tome";
const PASSPHRASE: &str = "phoenix-test-passphrase-xyz";

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("env var {key} must be set"))
}

fn s3_config_from_env(prefix: &str) -> S3Config {
    S3Config {
        endpoint: Some(env("S3_ENDPOINT")),
        region: env("S3_REGION"),
        bucket: env("S3_BUCKET"),
        access_key: env("S3_ACCESS_KEY"),
        secret_key: env("S3_SECRET_KEY"),
        prefix: Some(prefix.to_string()),
    }
}

async fn make_tome() -> (TempDir, SqlitePool) {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("smoke.tome");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let opts = SqliteConnectOptions::from_str(&url)
        .unwrap()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(opts)
        .await
        .unwrap();
    apply_all_migrations(&pool).await;
    (tmpdir, pool)
}

async fn apply_all_migrations(pool: &SqlitePool) {
    let migrations: &[(&str, &str)] = &[
        ("001_wiki_engine", include_str!("../migrations/001_wiki_engine.sql")),
        ("002_entity_types", include_str!("../migrations/002_entity_types.sql")),
        ("003_tome_metadata", include_str!("../migrations/003_tome_metadata.sql")),
        ("004_images", include_str!("../migrations/004_images.sql")),
        ("005_relations", include_str!("../migrations/005_relations.sql")),
        ("006_maps", include_str!("../migrations/006_maps.sql")),
        ("007_timelines", include_str!("../migrations/007_timelines.sql")),
        ("008_boards", include_str!("../migrations/008_boards.sql")),
        ("009_sync", include_str!("../migrations/009_sync.sql")),
        ("010_sync_app_global", include_str!("../migrations/010_sync_app_global.sql")),
        ("011_device_name_app_global", include_str!("../migrations/011_device_name_app_global.sql")),
    ];
    for (_name, sql) in migrations {
        for stmt in split_sql(sql) {
            sqlx::query(&stmt).execute(pool).await.unwrap();
        }
    }
}

fn split_sql(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") { continue; }
        for ch in trimmed.chars() {
            if ch == '(' { depth += 1; }
            if ch == ')' { depth -= 1; }
        }
        if !current.is_empty() { current.push(' '); }
        current.push_str(trimmed);
        if trimmed.ends_with(';') && depth == 0 {
            let s = current.trim_end_matches(';').trim().to_string();
            if !s.is_empty() { out.push(s); }
            current.clear();
        }
    }
    out
}

async fn configure_sync(pool: &SqlitePool, _salt: &[u8]) -> Uuid {
    let device_id = Uuid::new_v4();
    let cfg = SyncConfig {
        tome_id: TOME_ID.to_string(),
        enabled: true,
        device_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    cfg.save(pool).await.unwrap();
    device_id
}

// ============================================================================
// S3 smoke test — runs only when env vars are set + --ignored flag given.
// ============================================================================
#[tokio::test]
#[ignore]
async fn s3_real_backend_round_trip() {
    // Unique prefix per run so successive runs don't interfere / pollute.
    let run_id = Ulid::new().to_string();
    let prefix = format!("vaelorium-smoke/{run_id}");
    println!("\n=== S3 SMOKE TEST ===");
    println!("Endpoint: {}", env("S3_ENDPOINT"));
    println!("Region:   {}", env("S3_REGION"));
    println!("Bucket:   {}", env("S3_BUCKET"));
    println!("Prefix:   {prefix}");
    println!();

    // 1. Build backend and validate connectivity via a harmless list.
    let backend = S3Backend::new(s3_config_from_env(&prefix))
        .await
        .expect("backend init failed");
    println!("✅ Backend initialized (connection succeeded — auth NOT yet validated)");

    let initial = backend.list_prefix("").await.expect("list_prefix failed");
    println!("✅ list_prefix succeeded — auth is valid. {} existing objects under prefix.", initial.len());

    // 2. Create a Tome, configure sync, derive key.
    let (_d1_tmp, d1_pool) = make_tome().await;
    let salt = generate_salt();
    let key = KeyMaterial::derive(PASSPHRASE, &salt).unwrap();
    let d1_device = configure_sync(&d1_pool, &salt).await;
    println!("✅ Tome created + sync configured (device: {d1_device})");

    // 3. Mutate: create a page, record the op.
    let page_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut tx = d1_pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO pages (id, title, icon, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at)
         VALUES (?, 'Smoke Test Page', NULL, NULL, 1, NULL, 'private', ?, ?)",
    )
    .bind(&page_id).bind(&now).bind(&now)
    .execute(&mut *tx).await.unwrap();
    let mut fields = BTreeMap::new();
    fields.insert("title".to_string(), Some(json!("Smoke Test Page")));
    fields.insert("sort_order".to_string(), Some(json!(1)));
    fields.insert("visibility".to_string(), Some(json!("private")));
    fields.insert("created_at".to_string(), Some(json!(now)));
    fields.insert("updated_at".to_string(), Some(json!(now)));
    let op = insert_op(d1_device, Ulid::new(), "pages", &page_id, fields);
    record_op(&mut *tx, &op, TOME_ID).await.unwrap();
    tx.commit().await.unwrap();
    println!("✅ Local mutation + op recorded");

    // 4. Run one sync cycle. Uploads op + takes first-ever snapshot.
    let outcome = sync_tome_once(&d1_pool, TOME_ID, &key, &backend)
        .await
        .expect("sync_tome_once failed");
    println!(
        "✅ Sync complete: {} uploaded, {} applied, {} conflicts, snapshot={:?}, error={:?}",
        outcome.ops_uploaded, outcome.ops_applied, outcome.conflicts_created,
        outcome.snapshot_taken, outcome.error
    );
    assert_eq!(outcome.ops_uploaded, 1, "expected the new op to upload");
    assert!(outcome.error.is_none());

    // 5. List what's actually in the bucket under our prefix.
    let after = backend.list_prefix("").await.expect("list_prefix after sync failed");
    println!("\n📦 Objects in bucket under {prefix}:");
    for obj in &after {
        println!("  {} ({} bytes, etag {})", obj.key, obj.size, obj.etag);
    }
    assert!(after.len() >= 2, "expected at least 1 snapshot + 1 op blob");
    let has_snapshot = after.iter().any(|o| o.key.contains("snapshots/"));
    let has_journal = after.iter().any(|o| o.key.contains("journal/"));
    assert!(has_snapshot, "no snapshot found under prefix");
    assert!(has_journal, "no journal entry found under prefix");
    println!("✅ Snapshot + journal blobs present on remote");

    // 6. Simulate a second device: fresh Tome, same salt, same backend → pull.
    let (_d2_tmp, d2_pool) = make_tome().await;
    let _d2_device = configure_sync(&d2_pool, &salt).await;
    let outcome2 = sync_tome_once(&d2_pool, TOME_ID, &key, &backend)
        .await
        .expect("second-device sync failed");
    println!(
        "\n✅ Second device sync: {} uploaded, {} applied, {} conflicts, snapshot={:?}",
        outcome2.ops_uploaded, outcome2.ops_applied, outcome2.conflicts_created,
        outcome2.snapshot_taken
    );
    assert!(outcome2.ops_applied >= 1, "second device should have applied the page op");

    let title: Option<String> = sqlx::query_scalar("SELECT title FROM pages WHERE id = ?")
        .bind(&page_id).fetch_optional(&d2_pool).await.unwrap();
    assert_eq!(title.as_deref(), Some("Smoke Test Page"));
    println!("✅ Page propagated to second device with correct title");

    // 7. Cleanup: wipe the prefix so successive test runs start clean.
    println!("\n🧹 Cleaning up {} objects...", after.len());
    for obj in &after {
        let _ = backend.delete_object(&obj.key.replace(&format!("{prefix}/"), "")).await;
    }
    // The second sync may have written a snapshot too; sweep again.
    let final_pass = backend.list_prefix("").await.unwrap_or_default();
    for obj in &final_pass {
        let _ = backend.delete_object(&obj.key.replace(&format!("{prefix}/"), "")).await;
    }
    println!("✅ Cleanup complete");
    println!("\n=== SMOKE TEST PASSED ===\n");
}
