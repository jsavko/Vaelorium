use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod migrations;

pub type DbPool = SqlitePool;
pub type ManagedDb = Arc<RwLock<Option<DbPool>>>;

/// Create a new ManagedDb with no active connection.
pub fn create_managed_db() -> ManagedDb {
    Arc::new(RwLock::new(None))
}

/// Open a SQLite database at the given path and run all migrations.
pub async fn open_database(path: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_url = format!("sqlite:{}?mode=rwc", path);

    log::info!("Opening database: {}", path);

    let options = SqliteConnectOptions::from_str(&db_url)?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .create_if_missing(true)
        .pragma("foreign_keys", "ON")
        .pragma("synchronous", "NORMAL")
        .pragma("cache_size", "-20000")
        .pragma("temp_store", "MEMORY");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_with(options)
        .await?;

    // Run migrations
    migrations::run(&pool).await?;

    log::info!("Database opened successfully: {}", path);
    Ok(pool)
}

/// Helper to get the pool from ManagedDb, returning an error if no Tome is open.
pub async fn get_pool(managed: &ManagedDb) -> Result<DbPool, String> {
    let guard = managed.read().await;
    guard
        .clone()
        .ok_or_else(|| "No Tome is currently open".to_string())
}
