use crate::db::{self, DbPool, ManagedDb};
use crate::sync::journal::{
    self, active_sync_session, emit_for_row, insert_op, load_row_via_schema, record_op,
};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub featured_image_path: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub entity_type_id: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageTreeNode {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub entity_type_id: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub children_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreatePageInput {
    pub title: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub entity_type_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePageInput {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i64>,
    pub visibility: Option<String>,
    pub featured_image_path: Option<String>,
    pub entity_type_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderMove {
    pub id: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
}

#[tauri::command]
pub async fn create_page(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    input: CreatePageInput,
) -> Result<Page, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Get max sort_order for the parent
    let max_sort: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM pages WHERE parent_id IS ?",
    )
    .bind(&input.parent_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let sort_order = max_sort.unwrap_or(0) + 1;

    // Use a transaction to ensure page + content are created atomically
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO pages (id, title, icon, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'private', ?, ?)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.icon)
    .bind(&input.parent_id)
    .bind(sort_order)
    .bind(&input.entity_type_id)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Create empty Yjs document
    let empty_yjs: Vec<u8> = vec![];
    sqlx::query("INSERT INTO page_content (page_id, yjs_state, yjs_version) VALUES (?, ?, 0)")
        .bind(&id)
        .bind(&empty_yjs)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Sync: emit insert op for the new page (atomic with the row writes).
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.pages, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    get_page_by_pool(&pool, &id).await
}

/// Internal helper that fetches a page by id using a pool reference directly.
async fn get_page_by_pool(pool: &DbPool, id: &str) -> Result<Page, String> {
    sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, i64, Option<String>, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, title, icon, featured_image_path, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at, created_by, updated_by FROM pages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .map(|row| Page {
        id: row.0,
        title: row.1,
        icon: row.2,
        featured_image_path: row.3,
        parent_id: row.4,
        sort_order: row.5,
        entity_type_id: row.6,
        visibility: row.7,
        created_at: row.8,
        updated_at: row.9,
        created_by: row.10,
        updated_by: row.11,
    })
    .ok_or_else(|| "Page not found".to_string())
}

#[tauri::command]
pub async fn get_page(managed: State<'_, ManagedDb>, id: String) -> Result<Page, String> {
    let pool = db::get_pool(managed.inner()).await?;
    get_page_by_pool(&pool, &id).await
}

#[tauri::command]
pub async fn update_page(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    id: String,
    input: UpdatePageInput,
) -> Result<Page, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let now = chrono::Utc::now().to_rfc3339();

    // Build dynamic update query
    let mut updates = vec!["updated_at = ?".to_string()];
    let mut has_field = false;

    if input.title.is_some() { updates.push("title = ?".to_string()); has_field = true; }
    if input.icon.is_some() { updates.push("icon = ?".to_string()); has_field = true; }
    if input.parent_id.is_some() { updates.push("parent_id = ?".to_string()); has_field = true; }
    if input.sort_order.is_some() { updates.push("sort_order = ?".to_string()); has_field = true; }
    if input.visibility.is_some() { updates.push("visibility = ?".to_string()); has_field = true; }
    if input.featured_image_path.is_some() { updates.push("featured_image_path = ?".to_string()); has_field = true; }
    if input.entity_type_id.is_some() { updates.push("entity_type_id = ?".to_string()); has_field = true; }

    if !has_field {
        return get_page_by_pool(&pool, &id).await;
    }

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.pages, &id).await.map_err(|e| e.to_string())?)
    } else {
        None
    };

    let query_str = format!("UPDATE pages SET {} WHERE id = ?", updates.join(", "));
    let mut query = sqlx::query(&query_str).bind(&now);

    if let Some(ref title) = input.title { query = query.bind(title); }
    if let Some(ref icon) = input.icon { query = query.bind(icon); }
    if let Some(ref parent_id) = input.parent_id { query = query.bind(parent_id); }
    if let Some(sort_order) = input.sort_order { query = query.bind(sort_order); }
    if let Some(ref visibility) = input.visibility { query = query.bind(visibility); }
    if let Some(ref fip) = input.featured_image_path { query = query.bind(fip); }
    if let Some(ref etid) = input.entity_type_id { query = query.bind(etid); }

    query.bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.pages, &id, journal::OpKind::Update, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    get_page_by_pool(&pool, &id).await
}

#[tauri::command]
pub async fn delete_page(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.pages, &id).await.map_err(|e| e.to_string())?)
    } else {
        None
    };

    sqlx::query("DELETE FROM pages WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.pages, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_pages(managed: State<'_, ManagedDb>) -> Result<Vec<Page>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, i64, Option<String>, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, title, icon, featured_image_path, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at, created_by, updated_by FROM pages ORDER BY sort_order",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| Page {
            id: row.0,
            title: row.1,
            icon: row.2,
            featured_image_path: row.3,
            parent_id: row.4,
            sort_order: row.5,
            entity_type_id: row.6,
            visibility: row.7,
            created_at: row.8,
            updated_at: row.9,
            created_by: row.10,
            updated_by: row.11,
        })
        .collect())
}

#[tauri::command]
pub async fn list_pages_by_type(
    managed: State<'_, ManagedDb>,
    entity_type_id: String,
) -> Result<Vec<Page>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, i64, Option<String>, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, title, icon, featured_image_path, parent_id, sort_order, entity_type_id, visibility, created_at, updated_at, created_by, updated_by FROM pages WHERE entity_type_id = ? ORDER BY title",
    )
    .bind(&entity_type_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| Page {
            id: row.0,
            title: row.1,
            icon: row.2,
            featured_image_path: row.3,
            parent_id: row.4,
            sort_order: row.5,
            entity_type_id: row.6,
            visibility: row.7,
            created_at: row.8,
            updated_at: row.9,
            created_by: row.10,
            updated_by: row.11,
        })
        .collect())
}

#[tauri::command]
pub async fn get_page_tree(managed: State<'_, ManagedDb>) -> Result<Vec<PageTreeNode>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, i64)>(
        "SELECT p.id, p.title, p.icon, p.entity_type_id, p.parent_id, p.sort_order FROM pages p ORDER BY p.sort_order",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // Count children for each page
    let child_counts = sqlx::query_as::<_, (String, i64)>(
        "SELECT parent_id, COUNT(*) FROM pages WHERE parent_id IS NOT NULL GROUP BY parent_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let count_map: std::collections::HashMap<String, i64> = child_counts.into_iter().collect();

    Ok(rows
        .into_iter()
        .map(|row| PageTreeNode {
            children_count: count_map.get(&row.0).copied().unwrap_or(0),
            id: row.0,
            title: row.1,
            icon: row.2,
            entity_type_id: row.3,
            parent_id: row.4,
            sort_order: row.5,
        })
        .collect())
}

#[tauri::command]
pub async fn save_page_content(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    page_id: String,
    yjs_state: Vec<u8>,
) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};

    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check page exists first to avoid FK violation
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pages WHERE id = ?)")
        .bind(&page_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Page '{}' not found", page_id));
    }

    sqlx::query(
        "INSERT INTO page_content (page_id, yjs_state, yjs_version)
         VALUES (?, ?, 0)
         ON CONFLICT(page_id) DO UPDATE SET yjs_state = excluded.yjs_state, yjs_version = yjs_version + 1",
    )
    .bind(&page_id)
    .bind(&yjs_state)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Update page timestamp
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE pages SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if let Some((tome_id, device_id)) = sync_session {
        let tx_id = Ulid::new();
        let yjs_b64 = general_purpose::STANDARD.encode(&yjs_state);
        let mut fields = BTreeMap::new();
        fields.insert("yjs_state".into(), Some(json!(yjs_b64)));
        // page_content is upsert; treat as Update (apply path uses INSERT OR REPLACE).
        let op = insert_op(device_id, tx_id, "page_content", &page_id, fields);
        record_op(&mut *tx, &op, &tome_id).await.map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_page_content(
    managed: State<'_, ManagedDb>,
    page_id: String,
) -> Result<Vec<u8>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let result: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT yjs_state FROM page_content WHERE page_id = ?",
    )
    .bind(&page_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.unwrap_or_default())
}

#[tauri::command]
pub async fn reorder_pages(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    moves: Vec<ReorderMove>,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let tx_id = Ulid::new(); // all moves share one transaction id

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));

    for m in moves {
        let before = if sync_session.is_some() {
            Some(load_row_via_schema(&mut *tx, &TABLES.pages, &m.id).await.map_err(|e| e.to_string())?)
        } else {
            None
        };

        sqlx::query("UPDATE pages SET parent_id = ?, sort_order = ? WHERE id = ?")
            .bind(&m.parent_id)
            .bind(m.sort_order)
            .bind(&m.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        emit_for_row(&mut *tx, &TABLES.pages, &m.id, journal::OpKind::Update, tx_id, before, session_ref)
            .await.map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
