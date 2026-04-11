use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityTypeField {
    pub id: String,
    pub entity_type_id: String,
    pub name: String,
    pub field_type: String,
    pub sort_order: i64,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub options: Option<String>,
    pub reference_type_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ReorderFieldMove {
    pub id: String,
    pub sort_order: i64,
}

#[tauri::command]
pub async fn list_entity_type_fields(
    managed: State<'_, ManagedDb>,
    entity_type_id: String,
) -> Result<Vec<EntityTypeField>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, i64, bool, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, entity_type_id, name, field_type, sort_order, is_required,
                default_value, options, reference_type_id, created_at
         FROM entity_type_fields
         WHERE entity_type_id = ?
         ORDER BY sort_order",
    )
    .bind(&entity_type_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| EntityTypeField {
            id: row.0,
            entity_type_id: row.1,
            name: row.2,
            field_type: row.3,
            sort_order: row.4,
            is_required: row.5,
            default_value: row.6,
            options: row.7,
            reference_type_id: row.8,
            created_at: row.9,
        })
        .collect())
}

#[tauri::command]
pub async fn create_entity_type_field(
    managed: State<'_, ManagedDb>,
    entity_type_id: String,
    name: String,
    field_type: String,
    options: Option<String>,
    is_required: Option<bool>,
    default_value: Option<String>,
    reference_type_id: Option<String>,
) -> Result<EntityTypeField, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let is_required = is_required.unwrap_or(false);

    // Get next sort order for this type
    let max_sort: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM entity_type_fields WHERE entity_type_id = ?",
    )
    .bind(&entity_type_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let sort_order = max_sort.unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&entity_type_id)
    .bind(&name)
    .bind(&field_type)
    .bind(sort_order)
    .bind(is_required)
    .bind(&default_value)
    .bind(&options)
    .bind(&reference_type_id)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(EntityTypeField {
        id,
        entity_type_id,
        name,
        field_type,
        sort_order,
        is_required,
        default_value,
        options,
        reference_type_id,
        created_at: now,
    })
}

#[tauri::command]
pub async fn update_entity_type_field(
    managed: State<'_, ManagedDb>,
    id: String,
    name: Option<String>,
    field_type: Option<String>,
    is_required: Option<bool>,
    default_value: Option<String>,
    options: Option<String>,
    reference_type_id: Option<String>,
) -> Result<EntityTypeField, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let mut updates = Vec::new();

    if name.is_some() { updates.push("name = ?"); }
    if field_type.is_some() { updates.push("field_type = ?"); }
    if is_required.is_some() { updates.push("is_required = ?"); }
    if default_value.is_some() { updates.push("default_value = ?"); }
    if options.is_some() { updates.push("options = ?"); }
    if reference_type_id.is_some() { updates.push("reference_type_id = ?"); }

    if updates.is_empty() {
        // Nothing to update, just return the field
        let row = sqlx::query_as::<_, (String, String, String, String, i64, bool, Option<String>, Option<String>, Option<String>, String)>(
            "SELECT id, entity_type_id, name, field_type, sort_order, is_required,
                    default_value, options, reference_type_id, created_at
             FROM entity_type_fields WHERE id = ?",
        )
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Field not found".to_string())?;

        return Ok(EntityTypeField {
            id: row.0, entity_type_id: row.1, name: row.2, field_type: row.3,
            sort_order: row.4, is_required: row.5, default_value: row.6,
            options: row.7, reference_type_id: row.8, created_at: row.9,
        });
    }

    let query_str = format!("UPDATE entity_type_fields SET {} WHERE id = ?", updates.join(", "));
    let mut query = sqlx::query(&query_str);

    if let Some(ref v) = name { query = query.bind(v); }
    if let Some(ref v) = field_type { query = query.bind(v); }
    if let Some(v) = is_required { query = query.bind(v); }
    if let Some(ref v) = default_value { query = query.bind(v); }
    if let Some(ref v) = options { query = query.bind(v); }
    if let Some(ref v) = reference_type_id { query = query.bind(v); }
    query = query.bind(&id);

    query.execute(&pool).await.map_err(|e| e.to_string())?;

    // Fetch and return updated field
    let row = sqlx::query_as::<_, (String, String, String, String, i64, bool, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, entity_type_id, name, field_type, sort_order, is_required,
                default_value, options, reference_type_id, created_at
         FROM entity_type_fields WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Field not found".to_string())?;

    Ok(EntityTypeField {
        id: row.0, entity_type_id: row.1, name: row.2, field_type: row.3,
        sort_order: row.4, is_required: row.5, default_value: row.6,
        options: row.7, reference_type_id: row.8, created_at: row.9,
    })
}

#[tauri::command]
pub async fn delete_entity_type_field(
    managed: State<'_, ManagedDb>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("DELETE FROM entity_type_fields WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_entity_type_fields(
    managed: State<'_, ManagedDb>,
    moves: Vec<ReorderFieldMove>,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    for m in &moves {
        sqlx::query("UPDATE entity_type_fields SET sort_order = ? WHERE id = ?")
            .bind(m.sort_order)
            .bind(&m.id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
