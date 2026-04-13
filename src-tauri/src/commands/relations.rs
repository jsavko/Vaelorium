use crate::db::{self, ManagedDb};
use crate::sync::journal::{self, active_sync_session, emit_for_row, load_row_via_schema};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationType {
    pub id: String,
    pub name: String,
    pub inverse_name: Option<String>,
    pub color: Option<String>,
    pub is_builtin: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relation {
    pub id: String,
    pub source_page_id: String,
    pub target_page_id: String,
    pub relation_type_id: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageRelation {
    pub id: String,
    pub page_id: String,
    pub page_title: String,
    pub page_icon: Option<String>,
    pub page_entity_type_id: Option<String>,
    pub relation_type_id: String,
    pub relation_label: String,
    pub description: Option<String>,
    pub direction: String,
}

#[tauri::command]
pub async fn list_relation_types(managed: State<'_, ManagedDb>) -> Result<Vec<RelationType>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, bool, String)>(
        "SELECT id, name, inverse_name, color, is_builtin, created_at FROM relation_types ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| RelationType {
        id: r.0, name: r.1, inverse_name: r.2, color: r.3, is_builtin: r.4, created_at: r.5,
    }).collect())
}

#[tauri::command]
pub async fn create_relation_type(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    name: String,
    inverse_name: Option<String>,
    color: Option<String>,
) -> Result<RelationType, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO relation_types (id, name, inverse_name, color, is_builtin, created_at) VALUES (?, ?, ?, ?, FALSE, ?)")
        .bind(&id).bind(&name).bind(&inverse_name).bind(&color).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.relation_types, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(RelationType { id, name, inverse_name, color, is_builtin: false, created_at: now })
}

#[tauri::command]
pub async fn create_relation(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    source_page_id: String,
    target_page_id: String,
    relation_type_id: String,
    description: Option<String>,
) -> Result<Relation, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO relations (id, source_page_id, target_page_id, relation_type_id, description, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&source_page_id).bind(&target_page_id).bind(&relation_type_id).bind(&description).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.relations, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(Relation { id, source_page_id, target_page_id, relation_type_id, description, created_at: now })
}

#[tauri::command]
pub async fn delete_relation(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.relations, &id).await.map_err(|e| e.to_string())?)
    } else { None };
    sqlx::query("DELETE FROM relations WHERE id = ?")
        .bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.relations, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();
    Ok(())
}

#[tauri::command]
pub async fn get_page_relations(managed: State<'_, ManagedDb>, page_id: String) -> Result<Vec<PageRelation>, String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Outgoing relations (this page is source)
    let outgoing = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String, String, Option<String>)>(
        "SELECT r.id, p.id, p.title, p.icon, p.entity_type_id, r.relation_type_id, rt.name, r.description
         FROM relations r
         JOIN pages p ON p.id = r.target_page_id
         JOIN relation_types rt ON rt.id = r.relation_type_id
         WHERE r.source_page_id = ?",
    )
    .bind(&page_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // Incoming relations (this page is target)
    let incoming = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String, Option<String>, Option<String>)>(
        "SELECT r.id, p.id, p.title, p.icon, p.entity_type_id, r.relation_type_id, rt.inverse_name, r.description
         FROM relations r
         JOIN pages p ON p.id = r.source_page_id
         JOIN relation_types rt ON rt.id = r.relation_type_id
         WHERE r.target_page_id = ?",
    )
    .bind(&page_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut results: Vec<PageRelation> = outgoing.into_iter().map(|r| PageRelation {
        id: r.0, page_id: r.1, page_title: r.2, page_icon: r.3, page_entity_type_id: r.4,
        relation_type_id: r.5, relation_label: r.6, description: r.7, direction: "outgoing".to_string(),
    }).collect();

    results.extend(incoming.into_iter().map(|r| PageRelation {
        id: r.0, page_id: r.1, page_title: r.2, page_icon: r.3, page_entity_type_id: r.4,
        relation_type_id: r.5, relation_label: r.6.unwrap_or_default(), description: r.7, direction: "incoming".to_string(),
    }));

    results.sort_by(|a, b| a.page_title.cmp(&b.page_title));
    Ok(results)
}

#[tauri::command]
pub async fn list_all_relations(managed: State<'_, ManagedDb>) -> Result<Vec<Relation>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, source_page_id, target_page_id, relation_type_id, description, created_at FROM relations",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| Relation {
        id: r.0, source_page_id: r.1, target_page_id: r.2, relation_type_id: r.3, description: r.4, created_at: r.5,
    }).collect())
}
