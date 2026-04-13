//! One-shot measurement of snapshot compressibility. Runs only with --ignored.
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tempfile::TempDir;

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

#[tokio::test]
#[ignore]
async fn measure_snapshot_compression() {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(opts).await.unwrap();

    let migrations: &[&str] = &[
        include_str!("../migrations/001_wiki_engine.sql"),
        include_str!("../migrations/002_entity_types.sql"),
        include_str!("../migrations/003_tome_metadata.sql"),
        include_str!("../migrations/004_images.sql"),
        include_str!("../migrations/005_relations.sql"),
        include_str!("../migrations/006_maps.sql"),
        include_str!("../migrations/007_timelines.sql"),
        include_str!("../migrations/008_boards.sql"),
        include_str!("../migrations/009_sync.sql"),
    ];
    for sql in migrations {
        for stmt in split_sql(sql) {
            sqlx::query(&stmt).execute(&pool).await.unwrap();
        }
    }

    // Insert varying amounts of content to simulate real Tomes
    for scenario in ["empty", "small", "medium", "large"] {
        let n = match scenario { "empty" => 0, "small" => 10, "medium" => 500, "large" => 5000, _ => 0 };
        for i in 0..n {
            let id = format!("page-{i}");
            let content = format!("Page content for row {i}. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.");
            sqlx::query("INSERT INTO pages (id, title, sort_order, visibility, created_at, updated_at) VALUES (?, ?, ?, 'private', datetime('now'), datetime('now'))")
                .bind(&id).bind(&content).bind(i as i64).execute(&pool).await.unwrap();
        }

        let snap_path = tmp.path().join(format!("snap-{scenario}.db"));
        sqlx::query(&format!("VACUUM INTO '{}'", snap_path.display())).execute(&pool).await.unwrap();
        let raw = tokio::fs::read(&snap_path).await.unwrap();

        // gzip level 6 (default)
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&raw).unwrap();
        let gz_bytes = gz.finish().unwrap();

        // gzip level 9 (max)
        let mut gz9 = GzEncoder::new(Vec::new(), Compression::best());
        gz9.write_all(&raw).unwrap();
        let gz9_bytes = gz9.finish().unwrap();

        println!("\n=== {scenario} ({n} pages) ===");
        println!("  raw:       {:>10} bytes ({:.1} KB)", raw.len(), raw.len() as f64 / 1024.0);
        println!("  gzip-6:    {:>10} bytes ({:.1} KB)  {:.1}% of raw", gz_bytes.len(), gz_bytes.len() as f64 / 1024.0, 100.0 * gz_bytes.len() as f64 / raw.len() as f64);
        println!("  gzip-9:    {:>10} bytes ({:.1} KB)  {:.1}% of raw", gz9_bytes.len(), gz9_bytes.len() as f64 / 1024.0, 100.0 * gz9_bytes.len() as f64 / raw.len() as f64);
    }
}
