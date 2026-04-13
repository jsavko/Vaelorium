//! M7 Sync integration tests — the multi-device test harness.
//!
//! Each test simulates 2–3 devices: independent SQLite Tomes pointing at a
//! shared on-disk filesystem backend. Devices mutate their local DB through
//! helper functions that mirror `commands/pages.rs` SQL but emit ops via
//! [`vaelorium_lib::sync::journal::record_op`]. Each test exercises one of
//! the 6 scenarios from the Phase 2 plan.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{json, Value};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{Row, SqliteConnection};
use std::str::FromStr;
use tempfile::TempDir;
use ulid::Ulid;
use uuid::Uuid;

use vaelorium_lib::sync::backend::filesystem::FilesystemBackend;
use vaelorium_lib::sync::backend::SyncBackend;
use vaelorium_lib::sync::crypto::{generate_salt, KeyMaterial};
use vaelorium_lib::sync::journal::{
    self, delete_op, insert_op, record_op, update_op, Op, OpKind,
};
use vaelorium_lib::sync::snapshot;
use vaelorium_lib::sync::state::{BackendKind, SyncConfig};
use vaelorium_lib::sync::{sync_tome_once, SyncOutcome};

const TOME_ID: &str = "tome-test";
const PASSPHRASE: &str = "correct horse battery staple";

/// One simulated device.
struct Device {
    name: String,
    pool: SqlitePool,
    device_id: Uuid,
    db_path: PathBuf,
    _tmpdir: TempDir,
}

impl Device {
    async fn new(name: &str) -> Self {
        let tmpdir = TempDir::new().unwrap();
        let db_path = tmpdir.path().join(format!("{}.tome", name));
        let pool = open_with_migrations(&db_path).await;
        let device_id = Uuid::new_v4();
        Self {
            name: name.to_string(),
            pool,
            device_id,
            db_path,
            _tmpdir: tmpdir,
        }
    }

    /// Like [`Device::new`] but bootstraps from `from_snapshot_bytes` first
    /// (used for the "fresh device receives snapshot" scenario).
    async fn new_from_bytes(name: &str, snapshot_bytes: &[u8]) -> Self {
        let tmpdir = TempDir::new().unwrap();
        let db_path = tmpdir.path().join(format!("{}.tome", name));
        tokio::fs::write(&db_path, snapshot_bytes).await.unwrap();
        let pool = open_with_migrations(&db_path).await;
        let device_id = Uuid::new_v4();
        Self {
            name: name.to_string(),
            pool,
            device_id,
            db_path,
            _tmpdir: tmpdir,
        }
    }

    async fn enable_sync(&self, salt: &[u8]) {
        let cfg = SyncConfig {
            tome_id: TOME_ID.to_string(),
            enabled: true,
            backend_type: BackendKind::Filesystem,
            backend_config: json!({}),
            passphrase_salt: salt.to_vec(),
            device_id: self.device_id,
            device_name: self.name.clone(),
            schema_version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        cfg.save(&self.pool).await.unwrap();
    }

    async fn sync(&self, key: &KeyMaterial, backend: &dyn SyncBackend) -> SyncOutcome {
        sync_tome_once(&self.pool, TOME_ID, key, backend).await.unwrap()
    }

    // ----- mutation helpers (mirror commands/pages.rs SQL + emit ops) -----

    async fn create_page(&self, title: &str, parent_id: Option<&str>) -> String {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let tx_id = Ulid::new();
        let mut tx = self.pool.begin().await.unwrap();

        let max_sort: Option<i64> =
            sqlx::query_scalar("SELECT MAX(sort_order) FROM pages WHERE parent_id IS ?")
                .bind(parent_id)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        let sort_order = max_sort.unwrap_or(0) + 1;

        sqlx::query(
            "INSERT INTO pages (id, title, icon, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at)
             VALUES (?, ?, NULL, ?, ?, NULL, 'private', ?, ?)",
        )
        .bind(&id).bind(title).bind(parent_id).bind(sort_order).bind(&now).bind(&now)
        .execute(&mut *tx).await.unwrap();

        let mut fields = BTreeMap::new();
        fields.insert("title".to_string(), Some(json!(title)));
        fields.insert("icon".to_string(), None);
        fields.insert("parent_id".to_string(), parent_id.map(|p| json!(p)).or(Some(Value::Null)));
        fields.insert("sort_order".to_string(), Some(json!(sort_order)));
        fields.insert("entity_type_id".to_string(), None);
        fields.insert("visibility".to_string(), Some(json!("private")));
        fields.insert("created_at".to_string(), Some(json!(now)));
        fields.insert("updated_at".to_string(), Some(json!(now)));

        let op = insert_op(self.device_id, tx_id, "pages", &id, fields);
        record_op(&mut *tx, &op, TOME_ID).await.unwrap();
        tx.commit().await.unwrap();
        id
    }

    async fn update_page_field(&self, page_id: &str, field: &str, value: Value) {
        let now = chrono::Utc::now().to_rfc3339();
        let tx_id = Ulid::new();
        let mut tx = self.pool.begin().await.unwrap();

        let before = load_page_fields(&mut *tx, page_id).await;
        let sql = format!("UPDATE pages SET {} = ?, updated_at = ? WHERE id = ?", field);
        let q = bind_value(sqlx::query(&sql), &value);
        q.bind(&now).bind(page_id).execute(&mut *tx).await.unwrap();

        let after = load_page_fields(&mut *tx, page_id).await;
        if let Some(op) = update_op(self.device_id, tx_id, "pages", page_id, &before, &after) {
            record_op(&mut *tx, &op, TOME_ID).await.unwrap();
        }
        tx.commit().await.unwrap();
    }

    async fn delete_page(&self, page_id: &str) {
        let tx_id = Ulid::new();
        let mut tx = self.pool.begin().await.unwrap();
        let before = load_page_fields(&mut *tx, page_id).await;
        sqlx::query("DELETE FROM pages WHERE id = ?")
            .bind(page_id).execute(&mut *tx).await.unwrap();
        let op = delete_op(self.device_id, tx_id, "pages", page_id, before);
        record_op(&mut *tx, &op, TOME_ID).await.unwrap();
        tx.commit().await.unwrap();
    }

    async fn page_title(&self, page_id: &str) -> Option<String> {
        sqlx::query_scalar("SELECT title FROM pages WHERE id = ?")
            .bind(page_id).fetch_optional(&self.pool).await.unwrap()
    }

    async fn page_icon(&self, page_id: &str) -> Option<String> {
        sqlx::query_scalar::<_, Option<String>>("SELECT icon FROM pages WHERE id = ?")
            .bind(page_id).fetch_optional(&self.pool).await.unwrap().flatten()
    }

    async fn page_count(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM pages")
            .fetch_one(&self.pool).await.unwrap()
    }

    async fn conflict_count(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_conflicts")
            .fetch_one(&self.pool).await.unwrap()
    }
}

async fn open_with_migrations(path: &PathBuf) -> SqlitePool {
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let opts = SqliteConnectOptions::from_str(&url).unwrap()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = SqlitePoolOptions::new().max_connections(2).connect_with(opts).await.unwrap();
    apply_all_migrations(&pool).await;
    pool
}

async fn open_existing(path: &PathBuf) -> SqlitePool {
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let opts = SqliteConnectOptions::from_str(&url).unwrap()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .pragma("foreign_keys", "ON");
    SqlitePoolOptions::new().max_connections(2).connect_with(opts).await.unwrap()
}

async fn apply_all_migrations(pool: &SqlitePool) {
    // Reuse the test-side splitter from sync::state tests' style.
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

async fn load_page_fields(
    tx: &mut SqliteConnection,
    page_id: &str,
) -> BTreeMap<String, Option<Value>> {
    let row = sqlx::query(
        "SELECT title, icon, featured_image_path, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at FROM pages WHERE id = ?",
    )
    .bind(page_id)
    .fetch_one(tx)
    .await
    .unwrap();

    let mut m = BTreeMap::new();
    m.insert("title".into(), Some(json!(row.get::<String, _>("title"))));
    m.insert("icon".into(), row.get::<Option<String>, _>("icon").map(|v| json!(v)));
    m.insert("featured_image_path".into(), row.get::<Option<String>, _>("featured_image_path").map(|v| json!(v)));
    m.insert("parent_id".into(), row.get::<Option<String>, _>("parent_id").map(|v| json!(v)));
    m.insert("sort_order".into(), Some(json!(row.get::<i64, _>("sort_order"))));
    m.insert("entity_type_id".into(), row.get::<Option<String>, _>("entity_type_id").map(|v| json!(v)));
    m.insert("visibility".into(), Some(json!(row.get::<String, _>("visibility"))));
    m.insert("created_at".into(), Some(json!(row.get::<String, _>("created_at"))));
    m.insert("updated_at".into(), Some(json!(row.get::<String, _>("updated_at"))));
    m
}

fn bind_value<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    v: &'q Value,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        Value::String(s) => q.bind(s),
        Value::Number(n) if n.is_i64() => q.bind(n.as_i64().unwrap()),
        Value::Bool(b) => q.bind(*b),
        Value::Null => q.bind(Option::<String>::None),
        _ => panic!("unsupported value type in test bind: {:?}", v),
    }
}

fn shared_backend_dir() -> TempDir {
    TempDir::new().unwrap()
}

async fn make_backend(dir: &TempDir) -> FilesystemBackend {
    FilesystemBackend::new(dir.path()).await.unwrap()
}

fn make_key(salt: &[u8]) -> KeyMaterial {
    KeyMaterial::derive(PASSPHRASE, salt).unwrap()
}

// ============================================================================
// Scenario A — clean sync.
// ============================================================================
#[tokio::test]
async fn scenario_a_clean_sync_propagates_inserts() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    let p1 = d1.create_page("Aelinor", None).await;
    let p2 = d1.create_page("Westmarch", None).await;
    let _p3 = d1.create_page("The Crooked Spire", None).await;
    assert_eq!(d1.page_count().await, 3);

    let outcome = d1.sync(&key, &backend).await;
    assert_eq!(outcome.ops_uploaded, 3);
    assert_eq!(outcome.conflicts_created, 0);

    let outcome = d2.sync(&key, &backend).await;
    assert_eq!(outcome.ops_applied, 3);
    assert_eq!(d2.page_count().await, 3);
    assert_eq!(d2.page_title(&p1).await.as_deref(), Some("Aelinor"));
    assert_eq!(d2.page_title(&p2).await.as_deref(), Some("Westmarch"));
}

// ============================================================================
// Scenario B — sequential edits.
// ============================================================================
#[tokio::test]
async fn scenario_b_sequential_edits_propagate() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    let p = d1.create_page("Original", None).await;
    d1.sync(&key, &backend).await;
    d2.sync(&key, &backend).await;
    assert_eq!(d2.page_title(&p).await.as_deref(), Some("Original"));

    d2.update_page_field(&p, "title", json!("Edited on d2")).await;
    d2.sync(&key, &backend).await;
    d1.sync(&key, &backend).await;
    assert_eq!(d1.page_title(&p).await.as_deref(), Some("Edited on d2"));
}

// ============================================================================
// Scenario C — concurrent edits, disjoint fields → auto-merge, no conflict.
// ============================================================================
#[tokio::test]
async fn scenario_c_disjoint_field_edits_auto_merge() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    let p = d1.create_page("Common", None).await;
    d1.sync(&key, &backend).await;
    d2.sync(&key, &backend).await;

    // Both go offline. d1 edits title; d2 edits icon.
    d1.update_page_field(&p, "title", json!("Title from d1")).await;
    d2.update_page_field(&p, "icon", json!("📜")).await;

    // Both come back, sync.
    d1.sync(&key, &backend).await;
    let outcome = d2.sync(&key, &backend).await;
    assert_eq!(outcome.conflicts_created, 0);
    d1.sync(&key, &backend).await;

    assert_eq!(d1.page_title(&p).await.as_deref(), Some("Title from d1"));
    assert_eq!(d1.page_icon(&p).await.as_deref(), Some("📜"));
    assert_eq!(d2.page_title(&p).await.as_deref(), Some("Title from d1"));
    assert_eq!(d2.page_icon(&p).await.as_deref(), Some("📜"));
    assert_eq!(d1.conflict_count().await, 0);
    assert_eq!(d2.conflict_count().await, 0);
}

// ============================================================================
// Scenario D — concurrent edits, overlapping field → conflict.
// ============================================================================
#[tokio::test]
async fn scenario_d_overlapping_field_edits_create_conflict() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    let p = d1.create_page("Original", None).await;
    d1.sync(&key, &backend).await;
    d2.sync(&key, &backend).await;

    // Both go offline. Both edit title.
    d1.update_page_field(&p, "title", json!("Laptop title")).await;
    d2.update_page_field(&p, "title", json!("Desktop title")).await;

    // d1 syncs first (uploads its op). d2 syncs second — sees d1's op while
    // having its own pending → conflict on title field.
    d1.sync(&key, &backend).await;
    let outcome = d2.sync(&key, &backend).await;
    assert_eq!(outcome.conflicts_created, 1, "expected 1 conflict on title");

    // Local title on d2 should still be d2's choice (we didn't auto-resolve).
    assert_eq!(d2.page_title(&p).await.as_deref(), Some("Desktop title"));

    // The conflict descriptor should record both sides.
    let row = sqlx::query("SELECT field_name, local_value, remote_value FROM sync_conflicts LIMIT 1")
        .fetch_one(&d2.pool).await.unwrap();
    let field: String = row.get("field_name");
    assert_eq!(field, "title");
    let local_val: Option<String> = row.get("local_value");
    let remote_val: Option<String> = row.get("remote_value");
    // Stored as JSON-encoded values: e.g. "\"Desktop title\""
    assert!(local_val.unwrap().contains("Desktop title"));
    assert!(remote_val.unwrap().contains("Laptop title"));
}

// ============================================================================
// Scenario E — long offline catch-up.
// ============================================================================
#[tokio::test]
async fn scenario_e_long_offline_catch_up() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    // d1 makes 20 edits while d2 is offline.
    let mut page_ids = Vec::new();
    for i in 0..20 {
        let id = d1.create_page(&format!("Page {i:02}"), None).await;
        page_ids.push(id);
    }
    d1.sync(&key, &backend).await;

    // d2 syncs once and catches up with all 20.
    let outcome = d2.sync(&key, &backend).await;
    assert_eq!(outcome.ops_applied, 20);
    assert_eq!(d2.page_count().await, 20);
    for (i, id) in page_ids.iter().enumerate() {
        assert_eq!(d2.page_title(id).await.as_deref(), Some(format!("Page {i:02}").as_str()));
    }
}

// ============================================================================
// Defensive — schema mismatch parks instead of crashing.
// ============================================================================
#[tokio::test]
async fn defense_schema_version_mismatch_is_parked() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    // Manually upload a "future-version" op directly to the backend.
    let future_op = Op {
        op_id: Ulid::new(),
        device_id: Uuid::new_v4(),
        table: "pages".to_string(),
        row_id: "future-page".to_string(),
        kind: OpKind::Insert,
        fields: BTreeMap::from([("title".to_string(), Some(json!("From the future")))]),
        prev_fields: BTreeMap::new(),
        schema_version: 999,
        timestamp: chrono::Utc::now(),
        transaction_id: Ulid::new(),
    };
    let bytes = future_op.to_bytes().unwrap();
    let cipher = vaelorium_lib::sync::crypto::encrypt(&key, &bytes).unwrap();
    backend.put_object(&format!("journal/{}.op.enc", future_op.op_id), &cipher).await.unwrap();

    let d1 = Device::new("d1").await;
    d1.enable_sync(&salt).await;

    let outcome = sync_tome_once(&d1.pool, TOME_ID, &key, &backend).await.unwrap();
    assert!(outcome.error.is_some(), "future-schema op should surface error");
    assert_eq!(d1.page_count().await, 0, "future-schema op must not be applied");
}

// ============================================================================
// Defensive — wrong passphrase fails to decrypt remote ops.
// ============================================================================
#[tokio::test]
async fn defense_wrong_passphrase_cannot_decrypt() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let correct_key = make_key(&salt);
    let wrong_key = KeyMaterial::derive("wrong passphrase", &salt).unwrap();

    let d1 = Device::new("d1").await;
    d1.enable_sync(&salt).await;
    d1.create_page("Secret", None).await;
    d1.sync(&correct_key, &backend).await;

    let d2 = Device::new("d2").await;
    d2.enable_sync(&salt).await;
    let result = sync_tome_once(&d2.pool, TOME_ID, &wrong_key, &backend).await;
    assert!(result.is_err(), "wrong-passphrase sync must fail");
    assert_eq!(d2.page_count().await, 0);
}

// ============================================================================
// Scenario F — bootstrap from snapshot.
// ============================================================================
#[tokio::test]
async fn scenario_f_snapshot_bootstrap_for_fresh_device() {
    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    d1.enable_sync(&salt).await;

    // d1 creates pages and syncs (this writes ops to the backend AND triggers
    // the first-ever snapshot, since last_snapshot_id is None).
    for i in 0..5 {
        d1.create_page(&format!("Pre-snap {i}"), None).await;
    }
    let outcome = d1.sync(&key, &backend).await;
    assert!(outcome.snapshot_taken.is_some(), "first sync should snapshot");
    let snapshot_id = outcome.snapshot_taken.unwrap();

    // d1 creates more pages after the snapshot.
    let _post1 = d1.create_page("Post-snap A", None).await;
    let _post2 = d1.create_page("Post-snap B", None).await;
    d1.sync(&key, &backend).await;

    // Fresh d3: download + decrypt the snapshot, write to its tome path,
    // then open and sync to apply the post-snapshot journal tail.
    let d3_tmpdir = TempDir::new().unwrap();
    let d3_path = d3_tmpdir.path().join("d3.tome");
    snapshot::restore_snapshot_to_file(&snapshot_id, &key, &backend, &d3_path)
        .await.unwrap();
    // Open without re-running migrations — the snapshot already has the schema.
    let d3_pool = open_existing(&d3_path).await;

    // Verify the snapshot restored the 5 pre-snapshot pages.
    let pre_snap_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&d3_pool).await.unwrap();
    assert_eq!(pre_snap_count, 5, "snapshot should contain pre-snap pages");

    // Configure d3 sync state: it has the snapshot, so last_applied should
    // skip ahead to the snapshot point. For the simple test, just enable sync
    // and run — the engine treats unseen ops as new.
    let cfg = SyncConfig {
        tome_id: TOME_ID.to_string(),
        enabled: true,
        backend_type: BackendKind::Filesystem,
        backend_config: json!({}),
        passphrase_salt: salt.to_vec(),
        device_id: Uuid::new_v4(),
        device_name: "d3".to_string(),
        schema_version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    cfg.save(&d3_pool).await.unwrap();
    let outcome = sync_tome_once(&d3_pool, TOME_ID, &key, &backend).await.unwrap();

    // After sync, d3 should have all 7 pages: 5 from snapshot + 2 from journal tail.
    // (The 5 ops are also present in the journal but they'd be no-ops when
    // applied on top of an INSERT OR REPLACE — fine.)
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&d3_pool).await.unwrap();
    assert_eq!(total, 7, "expected snapshot pages + journal-tail pages");
    assert!(outcome.ops_applied >= 2);
}
