use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub page_id: String,
    pub title: String,
    pub entity_type_id: Option<String>,
    pub snippet: Option<String>,
}

#[tauri::command]
pub async fn update_search_index(
    managed: State<'_, ManagedDb>,
    page_id: String,
    title: String,
    text_content: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    // Upsert into backing table
    sqlx::query(
        "INSERT INTO pages_fts_content (page_id, title, text_content)
         VALUES (?, ?, ?)
         ON CONFLICT(page_id) DO UPDATE SET title = excluded.title, text_content = excluded.text_content",
    )
    .bind(&page_id)
    .bind(&title)
    .bind(&text_content)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // Rebuild FTS index for this row
    let rowid: Option<i64> = sqlx::query_scalar(
        "SELECT rowid FROM pages_fts_content WHERE page_id = ?",
    )
    .bind(&page_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(rid) = rowid {
        // Delete old FTS entry and reinsert
        sqlx::query("INSERT INTO pages_fts(pages_fts, rowid, title, text_content) VALUES('delete', ?, ?, ?)")
            .bind(rid)
            .bind(&title)
            .bind(&text_content)
            .execute(&pool)
            .await
            .ok(); // Ignore error if entry didn't exist

        sqlx::query("INSERT INTO pages_fts(rowid, title, text_content) VALUES(?, ?, ?)")
            .bind(rid)
            .bind(&title)
            .bind(&text_content)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn search_pages(
    managed: State<'_, ManagedDb>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Primary: simple LIKE search on page titles (always reliable)
    let like_query = format!("%{}%", query.trim());
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, title, entity_type_id FROM pages WHERE title LIKE ? ORDER BY title LIMIT 20",
    )
    .bind(&like_query)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut seen: std::collections::HashSet<String> = rows.iter().map(|r| r.0.clone()).collect();
    let mut results: Vec<(String, String, Option<String>, Option<String>)> = rows
        .into_iter()
        .map(|r| (r.0, r.1, r.2, None))
        .collect();

    // Secondary: FTS content search (finds matches in page body text)
    let fts_query = format!("{}*", query.trim());
    let fts_rows = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "SELECT fc.page_id, p.title, p.entity_type_id, snippet(pages_fts, 1, '<b>', '</b>', '...', 32) as snippet
         FROM pages_fts
         JOIN pages_fts_content fc ON fc.rowid = pages_fts.rowid
         JOIN pages p ON p.id = fc.page_id
         WHERE pages_fts MATCH ?
         ORDER BY rank
         LIMIT 20",
    )
    .bind(&fts_query)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for r in fts_rows {
        if !seen.contains(&r.0) {
            seen.insert(r.0.clone());
            results.push((r.0, r.1, r.2, Some(r.3)));
        }
    }

    Ok(results
        .into_iter()
        .map(|row| SearchResult {
            page_id: row.0,
            title: row.1,
            entity_type_id: row.2,
            snippet: row.3,
        })
        .collect())
}
