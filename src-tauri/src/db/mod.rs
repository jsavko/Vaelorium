use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use tauri::{AppHandle, Manager};

pub mod migrations;

pub type DbPool = SqlitePool;

pub async fn init_db(app: &AppHandle) -> Result<DbPool, Box<dyn std::error::Error>> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("vaelorium.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    log::info!("Database path: {}", db_path.display());

    // Configure per-connection options
    // WAL mode enables concurrent readers + single writer
    // busy_timeout prevents SQLITE_BUSY errors under contention
    // foreign_keys enforced on every connection
    let options = SqliteConnectOptions::from_str(&db_url)?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .create_if_missing(true)
        .pragma("foreign_keys", "ON")
        .pragma("synchronous", "NORMAL")
        .pragma("cache_size", "-20000")  // 20MB cache
        .pragma("temp_store", "MEMORY");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_with(options)
        .await?;

    // Run migrations
    migrations::run(&pool).await?;

    log::info!("Database initialized successfully");
    Ok(pool)
}
