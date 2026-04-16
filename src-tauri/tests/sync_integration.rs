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

    async fn enable_sync(&self, _salt: &[u8]) {
        // _salt parameter retained for backward compatibility with multi-device
        // test setup; in the app-global model the salt lives in
        // sync-backend.json (out of scope for these per-pool tests).
        let cfg = SyncConfig {
            tome_id: TOME_ID.to_string(),
            enabled: true,
            device_id: self.device_id,
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
        ("010_sync_app_global", include_str!("../migrations/010_sync_app_global.sql")),
        ("011_device_name_app_global", include_str!("../migrations/011_device_name_app_global.sql")),
        ("012_sync_activity", include_str!("../migrations/012_sync_activity.sql")),
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
// Registry — non-pages table sync (entity_types).
// ============================================================================
#[tokio::test]
async fn registry_entity_types_propagate() {
    use vaelorium_lib::sync::journal::{emit_for_row, OpKind};
    use vaelorium_lib::sync::registry::TABLES;

    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    // d1 creates an entity_type via the registry helpers.
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut tx = d1.pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO entity_types (id, name, icon, color, is_builtin, sort_order, created_at, updated_at)
         VALUES (?, 'Faction', '⚔', '#B85C5C', FALSE, 1, ?, ?)",
    )
    .bind(&id).bind(&now).bind(&now)
    .execute(&mut *tx).await.unwrap();
    let session = (TOME_ID, d1.device_id);
    emit_for_row(&mut *tx, &TABLES.entity_types, &id, OpKind::Insert, Ulid::new(), None, Some(session))
        .await.unwrap();
    tx.commit().await.unwrap();

    // Sync both devices.
    d1.sync(&key, &backend).await;
    let outcome = d2.sync(&key, &backend).await;
    assert!(outcome.ops_applied >= 1);

    // d2 should now have the entity_type.
    let name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM entity_types WHERE id = ?",
    )
    .bind(&id).fetch_optional(&d2.pool).await.unwrap();
    assert_eq!(name.as_deref(), Some("Faction"));

    // Verify other columns came through.
    let icon: Option<String> = sqlx::query_scalar("SELECT icon FROM entity_types WHERE id = ?")
        .bind(&id).fetch_optional(&d2.pool).await.unwrap();
    assert_eq!(icon.as_deref(), Some("⚔"));
}

// ============================================================================
// Registry — boards table (the largest of the registry-wired tables).
// ============================================================================
#[tokio::test]
async fn registry_boards_propagate() {
    use vaelorium_lib::sync::journal::{emit_for_row, OpKind};
    use vaelorium_lib::sync::registry::TABLES;

    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    // d1 creates a board + a card on it via raw SQL + emit_for_row.
    let board_id = Uuid::new_v4().to_string();
    let card_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let session = (TOME_ID, d1.device_id);

    let mut tx = d1.pool.begin().await.unwrap();
    sqlx::query("INSERT INTO boards (id, name, sort_order, created_at, updated_at) VALUES (?, 'Plot Board', 0, ?, ?)")
        .bind(&board_id).bind(&now).bind(&now)
        .execute(&mut *tx).await.unwrap();
    emit_for_row(&mut *tx, &TABLES.boards, &board_id, OpKind::Insert, Ulid::new(), None, Some(session))
        .await.unwrap();

    sqlx::query("INSERT INTO board_cards (id, board_id, page_id, content, x, y, width, height, color, created_at) VALUES (?, ?, NULL, 'A clue', 100, 100, 200, 120, NULL, ?)")
        .bind(&card_id).bind(&board_id).bind(&now)
        .execute(&mut *tx).await.unwrap();
    emit_for_row(&mut *tx, &TABLES.board_cards, &card_id, OpKind::Insert, Ulid::new(), None, Some(session))
        .await.unwrap();
    tx.commit().await.unwrap();

    d1.sync(&key, &backend).await;
    let outcome = d2.sync(&key, &backend).await;
    assert!(outcome.ops_applied >= 2);

    let board_name: Option<String> = sqlx::query_scalar("SELECT name FROM boards WHERE id = ?")
        .bind(&board_id).fetch_optional(&d2.pool).await.unwrap();
    assert_eq!(board_name.as_deref(), Some("Plot Board"));

    let card_content: Option<String> = sqlx::query_scalar("SELECT content FROM board_cards WHERE id = ?")
        .bind(&card_id).fetch_optional(&d2.pool).await.unwrap();
    assert_eq!(card_content.as_deref(), Some("A clue"));
}

// ============================================================================
// Registry — M:N page_tags pivot via the special apply path.
// ============================================================================
#[tokio::test]
async fn registry_page_tags_pivot_propagates() {
    use vaelorium_lib::sync::journal::{insert_op, record_op};
    use std::collections::BTreeMap;

    let backend_dir = shared_backend_dir();
    let backend = make_backend(&backend_dir).await;
    let salt = generate_salt();
    let key = make_key(&salt);

    let d1 = Device::new("d1").await;
    let d2 = Device::new("d2").await;
    d1.enable_sync(&salt).await;
    d2.enable_sync(&salt).await;

    // d1 creates a page + a tag, then associates them (M:N pivot row).
    let page_id = d1.create_page("Tagged Page", None).await;

    let tag_id = Uuid::new_v4().to_string();
    let mut tx = d1.pool.begin().await.unwrap();
    sqlx::query("INSERT INTO tags (id, name, color) VALUES (?, 'mystery', '#5C7AB8')")
        .bind(&tag_id).execute(&mut *tx).await.unwrap();
    let mut tag_fields = BTreeMap::new();
    tag_fields.insert("name".into(), Some(serde_json::json!("mystery")));
    tag_fields.insert("color".into(), Some(serde_json::json!("#5C7AB8")));
    let tag_op = insert_op(d1.device_id, Ulid::new(), "tags", &tag_id, tag_fields);
    record_op(&mut *tx, &tag_op, TOME_ID).await.unwrap();

    sqlx::query("INSERT INTO page_tags (page_id, tag_id) VALUES (?, ?)")
        .bind(&page_id).bind(&tag_id).execute(&mut *tx).await.unwrap();
    let composite = format!("{}|{}", page_id, tag_id);
    let mut pt_fields = BTreeMap::new();
    pt_fields.insert("page_id".into(), Some(serde_json::json!(page_id)));
    pt_fields.insert("tag_id".into(), Some(serde_json::json!(tag_id)));
    let pt_op = insert_op(d1.device_id, Ulid::new(), "page_tags", &composite, pt_fields);
    record_op(&mut *tx, &pt_op, TOME_ID).await.unwrap();
    tx.commit().await.unwrap();

    d1.sync(&key, &backend).await;
    d2.sync(&key, &backend).await;

    // d2 should now have the page, the tag, and the association.
    let association: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM page_tags WHERE page_id = ? AND tag_id = ?",
    )
    .bind(&page_id).bind(&tag_id).fetch_optional(&d2.pool).await.unwrap();
    assert!(association.is_some(), "M:N pivot row should exist on d2");
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
        device_id: Uuid::new_v4(),
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

/// Scenario G — multi-Tome recovery discovery on a fresh device.
///
/// Two devices each push a snapshot for a different `tome_uuid` into the
/// shared backend (under `tomes/<uuid>/snapshots/...`). A third device,
/// starting from no local DB, calls `list_tome_snapshots` against the raw
/// backend to discover both Tomes, then `restore_snapshot_by_key` to pull
/// down one of them. Verifies the restored DB contains the originating
/// device's pages.
#[tokio::test]
async fn scenario_g_multi_tome_recovery_discovery() {
    use std::sync::Arc;
    use vaelorium_lib::sync::backend::prefixed::PrefixedBackend;

    let backend_dir = shared_backend_dir();
    let raw_backend = make_backend(&backend_dir).await;
    let raw_arc: Arc<dyn SyncBackend + Send + Sync> = Arc::new(raw_backend);
    let salt = generate_salt();
    let key = make_key(&salt);

    let uuid_alpha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let uuid_beta = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    // Device A syncs into tomes/<uuid_alpha>/...
    let d_a = Device::new("d_a").await;
    d_a.enable_sync(&salt).await;
    d_a.create_page("Alpha One", None).await;
    d_a.create_page("Alpha Two", None).await;
    let alpha_backend =
        PrefixedBackend::new(raw_arc.clone(), format!("tomes/{}", uuid_alpha));
    d_a.sync(&key, &alpha_backend).await;

    // Device B syncs into tomes/<uuid_beta>/...
    let d_b = Device::new("d_b").await;
    d_b.enable_sync(&salt).await;
    d_b.create_page("Beta One", None).await;
    let beta_backend =
        PrefixedBackend::new(raw_arc.clone(), format!("tomes/{}", uuid_beta));
    d_b.sync(&key, &beta_backend).await;

    // Fresh device C: discover what Tomes are out there.
    let summaries = snapshot::list_tome_snapshots(raw_arc.as_ref())
        .await
        .expect("list_tome_snapshots");
    assert_eq!(summaries.len(), 2, "should discover both Tomes");
    let alpha_summary = summaries
        .iter()
        .find(|s| s.tome_uuid == uuid_alpha)
        .expect("alpha discovered");
    let beta_summary = summaries
        .iter()
        .find(|s| s.tome_uuid == uuid_beta)
        .expect("beta discovered");
    assert!(!alpha_summary.snapshot_id.is_empty());
    assert!(!beta_summary.snapshot_id.is_empty());

    // C restores Alpha by full key.
    let c_tmpdir = TempDir::new().unwrap();
    let c_path = c_tmpdir.path().join("c.tome");
    let alpha_key = format!(
        "tomes/{}/snapshots/{}.snap.enc",
        uuid_alpha, alpha_summary.snapshot_id
    );
    snapshot::restore_snapshot_by_key(&alpha_key, &key, raw_arc.as_ref(), &c_path)
        .await
        .expect("restore_snapshot_by_key");

    let c_pool = open_existing(&c_path).await;
    let titles: Vec<String> =
        sqlx::query_scalar("SELECT title FROM pages ORDER BY title")
            .fetch_all(&c_pool)
            .await
            .unwrap();
    assert_eq!(titles, vec!["Alpha One".to_string(), "Alpha Two".to_string()]);
}

/// Scenario H — sync_activity log accrues + retains last 100.
#[tokio::test]
async fn scenario_h_activity_log_accrues_and_retains() {
    use vaelorium_lib::sync::activity;
    use vaelorium_lib::sync::SyncOutcome;

    let d = Device::new("d_act").await;

    // Three diverse outcomes.
    let ok_outcome = SyncOutcome { ops_uploaded: 2, ops_applied: 1, conflicts_created: 0, snapshot_taken: None, error: None };
    activity::record(&d.pool, TOME_ID, chrono::Utc::now(), 30, "success", Some(&ok_outcome), None).await;
    activity::record(&d.pool, TOME_ID, chrono::Utc::now(), 12, "error", None, Some("simulated network down")).await;
    let snap_outcome = SyncOutcome { ops_uploaded: 0, ops_applied: 0, conflicts_created: 0, snapshot_taken: Some("01ABCDEFG".into()), error: None };
    activity::record(&d.pool, TOME_ID, chrono::Utc::now(), 200, "success", Some(&snap_outcome), None).await;

    let rows = activity::list(&d.pool, 100).await.unwrap();
    assert_eq!(rows.len(), 3);
    let outcomes: Vec<&str> = rows.iter().map(|r| r.outcome.as_str()).collect();
    assert!(outcomes.contains(&"error"));
    assert!(outcomes.iter().filter(|o| **o == "success").count() == 2);
    let snap_row = rows.iter().find(|r| r.snapshot_taken.is_some()).expect("snapshot row");
    assert_eq!(snap_row.snapshot_taken.as_deref(), Some("01ABCDEFG"));

    // Retention: push 102 more, total cap should be 100.
    for _ in 0..102 {
        activity::record(&d.pool, TOME_ID, chrono::Utc::now(), 1, "success", Some(&ok_outcome), None).await;
    }
    let rows = activity::list(&d.pool, 200).await.unwrap();
    assert_eq!(rows.len(), 100, "retention must cap at 100 per tome");
}

// ============================================================================
// Snapshot cursor preservation tests
// ============================================================================

/// take_snapshot preserves last_applied_op_id and last_uploaded_op_id as
/// journal cursors, and embeds the new snapshot_id — enabling efficient
/// journal-tail-only replay on restore.
#[tokio::test]
async fn snapshot_preserves_cursor_fields() {
    use vaelorium_lib::sync::backend::prefixed::PrefixedBackend;
    use vaelorium_lib::sync::state::SyncRuntimeState;

    let backend_dir = shared_backend_dir();
    let raw_backend = make_backend(&backend_dir).await;
    let raw_arc: std::sync::Arc<dyn SyncBackend + Send + Sync> = std::sync::Arc::new(raw_backend);
    let salt = generate_salt();
    let key = make_key(&salt);

    let tome_uuid = "cccccccccccccccccccccccccccccccc";
    let prefixed = PrefixedBackend::new(raw_arc.clone(), format!("tomes/{tome_uuid}"));

    let d = Device::new("snap_cursor").await;
    d.enable_sync(&salt).await;
    d.create_page("Before snapshot", None).await;
    d.sync(&key, &prefixed).await;

    // Verify sync_state has cursors before snapshot.
    let state_before = SyncRuntimeState::load(&d.pool, TOME_ID).await.unwrap();
    assert!(state_before.last_applied_op_id.is_some(), "should have cursor after sync");
    assert!(state_before.last_uploaded_op_id.is_some(), "should have upload cursor after sync");
    let applied_id = state_before.last_applied_op_id.clone().unwrap();
    let uploaded_id = state_before.last_uploaded_op_id.clone().unwrap();

    // Take a snapshot (force it by creating another page so there's a pending op).
    d.create_page("Trigger snapshot", None).await;
    let snap_info = snapshot::take_snapshot(&d.pool, &key, &prefixed).await.unwrap();

    // Decrypt the snapshot and inspect sync_state.
    let snap_key = format!("snapshots/{}.snap.enc", snap_info.snapshot_id);
    let snap_tmpdir = tempfile::TempDir::new().unwrap();
    let snap_path = snap_tmpdir.path().join("peek.tome");
    snapshot::restore_snapshot_to_file(
        &snap_info.snapshot_id.to_string(),
        &key,
        &prefixed,
        &snap_path,
    )
    .await
    .unwrap();
    let snap_pool = open_existing(&snap_path).await;

    // Cursor fields should survive.
    let row: Option<(String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT tome_id, last_uploaded_op_id, last_applied_op_id, last_snapshot_id, last_sync_at, last_error FROM sync_state",
        )
        .fetch_optional(&snap_pool)
        .await
        .unwrap();
    let row = row.expect("sync_state row should exist in snapshot");
    assert_eq!(row.0, "__snapshot__", "tome_id should be sentinel");
    assert_eq!(row.1.as_deref(), Some(uploaded_id.as_str()), "last_uploaded_op_id preserved");
    // last_applied_op_id may have advanced past the pre-snapshot value (sync
    // applies the device's own uploaded ops), but it should exist.
    assert!(row.2.is_some(), "last_applied_op_id preserved");
    assert_eq!(
        row.3.as_deref(),
        Some(snap_info.snapshot_id.to_string().as_str()),
        "last_snapshot_id should be the new snapshot"
    );
    assert!(row.4.is_none(), "last_sync_at should be stripped");
    assert!(row.5.is_none(), "last_error should be stripped");

    // sync_config should be gone.
    let cfg_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_config")
        .fetch_one(&snap_pool)
        .await
        .unwrap();
    assert_eq!(cfg_count, 0, "sync_config should be stripped from snapshot");

    snap_pool.close().await;
}

/// Restore + journal replay: a snapshot is taken, then more ops are added.
/// Restoring the snapshot and replaying the journal tail yields all data.
/// Verifies the cursor makes the replay efficient (only tail ops applied).
#[tokio::test]
async fn restore_replays_journal_tail_with_cursor() {
    use vaelorium_lib::sync::backend::prefixed::PrefixedBackend;
    use vaelorium_lib::sync::state::SyncRuntimeState;

    let backend_dir = shared_backend_dir();
    let raw_backend = make_backend(&backend_dir).await;
    let raw_arc: std::sync::Arc<dyn SyncBackend + Send + Sync> = std::sync::Arc::new(raw_backend);
    let salt = generate_salt();
    let key = make_key(&salt);

    let tome_uuid = "dddddddddddddddddddddddddddddd";
    let prefixed = PrefixedBackend::new(raw_arc.clone(), format!("tomes/{tome_uuid}"));

    // Device 1: create pages, sync (triggers snapshot), create more, sync again.
    let d1 = Device::new("d1_replay").await;
    d1.enable_sync(&salt).await;
    for i in 0..3 {
        d1.create_page(&format!("Pre-snap {i}"), None).await;
    }
    let outcome1 = d1.sync(&key, &prefixed).await;
    assert!(outcome1.snapshot_taken.is_some(), "first sync should snapshot");
    let snapshot_id = outcome1.snapshot_taken.unwrap();

    // Post-snapshot: create 2 more pages and sync (uploads journal ops).
    let post_id_1 = d1.create_page("Post-snap Alpha", None).await;
    let post_id_2 = d1.create_page("Post-snap Beta", None).await;
    d1.sync(&key, &prefixed).await;

    // Simulate restore: download snapshot, rename cursor, replay.
    let restore_tmpdir = tempfile::TempDir::new().unwrap();
    let restore_path = restore_tmpdir.path().join("restored.tome");
    snapshot::restore_snapshot_to_file(&snapshot_id, &key, &prefixed, &restore_path)
        .await
        .unwrap();
    let restore_pool = open_existing(&restore_path).await;

    // Verify pre-snapshot pages present.
    let pre_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&restore_pool)
        .await
        .unwrap();
    assert_eq!(pre_count, 3, "snapshot should have 3 pre-snap pages");

    // Rename cursor from '__snapshot__' to the tome_id (mirrors what restore.rs does).
    sqlx::query("UPDATE sync_state SET tome_id = ? WHERE tome_id = '__snapshot__'")
        .bind(TOME_ID)
        .execute(&restore_pool)
        .await
        .unwrap();

    // Check cursor was loaded correctly.
    let state = SyncRuntimeState::load(&restore_pool, TOME_ID).await.unwrap();
    assert!(
        state.last_applied_op_id.is_some(),
        "cursor should exist from snapshot"
    );

    // Replay journal tail.
    let replay_outcome =
        sync_tome_once(&restore_pool, TOME_ID, &key, &prefixed).await.unwrap();

    // Only the 2 post-snapshot ops should be applied (not the 3 pre-snapshot ones).
    assert_eq!(
        replay_outcome.ops_applied, 2,
        "should replay only the 2 post-snapshot ops, not all 5"
    );

    // All 5 pages should now be present.
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&restore_pool)
        .await
        .unwrap();
    assert_eq!(total, 5, "restored tome should have all 5 pages after replay");

    // Verify specific post-snapshot pages exist.
    let post_alpha: Option<String> =
        sqlx::query_scalar("SELECT title FROM pages WHERE id = ?")
            .bind(&post_id_1)
            .fetch_optional(&restore_pool)
            .await
            .unwrap();
    assert_eq!(post_alpha.as_deref(), Some("Post-snap Alpha"));

    restore_pool.close().await;
}

/// When restoring an old snapshot that has no cursor (pre-fix), the engine
/// falls back to replaying the full journal — all ops are applied.
#[tokio::test]
async fn restore_without_cursor_replays_full_journal() {
    use vaelorium_lib::sync::backend::prefixed::PrefixedBackend;

    let backend_dir = shared_backend_dir();
    let raw_backend = make_backend(&backend_dir).await;
    let raw_arc: std::sync::Arc<dyn SyncBackend + Send + Sync> = std::sync::Arc::new(raw_backend);
    let salt = generate_salt();
    let key = make_key(&salt);

    let tome_uuid = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    let prefixed = PrefixedBackend::new(raw_arc.clone(), format!("tomes/{tome_uuid}"));

    let d1 = Device::new("d1_nocursor").await;
    d1.enable_sync(&salt).await;
    for i in 0..3 {
        d1.create_page(&format!("Page {i}"), None).await;
    }
    let outcome1 = d1.sync(&key, &prefixed).await;
    assert!(outcome1.snapshot_taken.is_some());
    let snapshot_id = outcome1.snapshot_taken.unwrap();

    // Post-snapshot pages.
    d1.create_page("Post A", None).await;
    d1.create_page("Post B", None).await;
    d1.sync(&key, &prefixed).await;

    // Restore snapshot then simulate old format: DELETE the cursor row.
    let restore_tmpdir = tempfile::TempDir::new().unwrap();
    let restore_path = restore_tmpdir.path().join("old_restored.tome");
    snapshot::restore_snapshot_to_file(&snapshot_id, &key, &prefixed, &restore_path)
        .await
        .unwrap();
    let restore_pool = open_existing(&restore_path).await;

    // Simulate old snapshot: no sync_state row at all.
    sqlx::query("DELETE FROM sync_state")
        .execute(&restore_pool)
        .await
        .unwrap();

    // Replay: with no cursor, engine lists the full journal.
    let replay_outcome =
        sync_tome_once(&restore_pool, TOME_ID, &key, &prefixed).await.unwrap();

    // All 5 ops replayed (3 pre-snap are idempotent no-ops content-wise but
    // still count as applied).
    assert!(
        replay_outcome.ops_applied >= 2,
        "should apply at least the 2 post-snapshot ops"
    );

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&restore_pool)
        .await
        .unwrap();
    assert_eq!(total, 5, "all pages should be present after full replay");

    restore_pool.close().await;
}

/// When there are no journal ops after the snapshot, replay is a fast no-op.
#[tokio::test]
async fn restore_no_ops_after_snapshot_is_noop() {
    use vaelorium_lib::sync::backend::prefixed::PrefixedBackend;

    let backend_dir = shared_backend_dir();
    let raw_backend = make_backend(&backend_dir).await;
    let raw_arc: std::sync::Arc<dyn SyncBackend + Send + Sync> = std::sync::Arc::new(raw_backend);
    let salt = generate_salt();
    let key = make_key(&salt);

    let tome_uuid = "ffffffffffffffffffffffffffffffff";
    let prefixed = PrefixedBackend::new(raw_arc.clone(), format!("tomes/{tome_uuid}"));

    let d1 = Device::new("d1_noop").await;
    d1.enable_sync(&salt).await;
    d1.create_page("Only page", None).await;
    let outcome1 = d1.sync(&key, &prefixed).await;
    assert!(outcome1.snapshot_taken.is_some());
    let snapshot_id = outcome1.snapshot_taken.unwrap();

    // No post-snapshot ops — restore and replay immediately.
    let restore_tmpdir = tempfile::TempDir::new().unwrap();
    let restore_path = restore_tmpdir.path().join("noop_restored.tome");
    snapshot::restore_snapshot_to_file(&snapshot_id, &key, &prefixed, &restore_path)
        .await
        .unwrap();
    let restore_pool = open_existing(&restore_path).await;

    sqlx::query("UPDATE sync_state SET tome_id = ? WHERE tome_id = '__snapshot__'")
        .bind(TOME_ID)
        .execute(&restore_pool)
        .await
        .unwrap();

    let replay_outcome =
        sync_tome_once(&restore_pool, TOME_ID, &key, &prefixed).await.unwrap();

    assert_eq!(replay_outcome.ops_applied, 0, "no ops to replay");
    assert_eq!(replay_outcome.conflicts_created, 0, "no conflicts");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages")
        .fetch_one(&restore_pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "only the pre-snapshot page should exist");

    restore_pool.close().await;
}
