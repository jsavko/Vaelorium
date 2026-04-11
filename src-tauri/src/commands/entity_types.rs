use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityType {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_builtin: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn list_entity_types(pool: State<'_, DbPool>) -> Result<Vec<EntityType>, String> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, bool, i64, String, String)>(
        "SELECT id, name, icon, color, is_builtin, sort_order, created_at, updated_at
         FROM entity_types ORDER BY sort_order, name",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| EntityType {
            id: row.0,
            name: row.1,
            icon: row.2,
            color: row.3,
            is_builtin: row.4,
            sort_order: row.5,
            created_at: row.6,
            updated_at: row.7,
        })
        .collect())
}

#[tauri::command]
pub async fn get_entity_type(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<EntityType, String> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, bool, i64, String, String)>(
        "SELECT id, name, icon, color, is_builtin, sort_order, created_at, updated_at
         FROM entity_types WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Entity type not found".to_string())?;

    Ok(EntityType {
        id: row.0,
        name: row.1,
        icon: row.2,
        color: row.3,
        is_builtin: row.4,
        sort_order: row.5,
        created_at: row.6,
        updated_at: row.7,
    })
}

#[tauri::command]
pub async fn create_entity_type(
    pool: State<'_, DbPool>,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<EntityType, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Get next sort order
    let max_sort: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM entity_types",
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let sort_order = max_sort.unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO entity_types (id, name, icon, color, is_builtin, sort_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, FALSE, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&icon)
    .bind(&color)
    .bind(sort_order)
    .bind(&now)
    .bind(&now)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(EntityType {
        id,
        name,
        icon,
        color,
        is_builtin: false,
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_entity_type(
    pool: State<'_, DbPool>,
    id: String,
    name: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Result<EntityType, String> {
    let now = chrono::Utc::now().to_rfc3339();

    let mut updates = vec!["updated_at = ?".to_string()];
    let mut has_name = false;
    let mut has_icon = false;
    let mut has_color = false;

    if name.is_some() {
        updates.push("name = ?".to_string());
        has_name = true;
    }
    if icon.is_some() {
        updates.push("icon = ?".to_string());
        has_icon = true;
    }
    if color.is_some() {
        updates.push("color = ?".to_string());
        has_color = true;
    }

    let query_str = format!("UPDATE entity_types SET {} WHERE id = ?", updates.join(", "));
    let mut query = sqlx::query(&query_str).bind(&now);

    if has_name {
        query = query.bind(&name);
    }
    if has_icon {
        query = query.bind(&icon);
    }
    if has_color {
        query = query.bind(&color);
    }
    query = query.bind(&id);

    query.execute(pool.inner()).await.map_err(|e| e.to_string())?;

    get_entity_type(pool, id).await
}

#[tauri::command]
pub async fn delete_entity_type(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    // Prevent deleting built-in types
    let is_builtin: bool = sqlx::query_scalar(
        "SELECT is_builtin FROM entity_types WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Entity type not found".to_string())?;

    if is_builtin {
        return Err("Cannot delete built-in entity types".to_string());
    }

    // Clear entity_type_id from pages using this type
    sqlx::query("UPDATE pages SET entity_type_id = NULL WHERE entity_type_id = ?")
        .bind(&id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    // Delete the type (cascades to fields and values)
    sqlx::query("DELETE FROM entity_types WHERE id = ?")
        .bind(&id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
