mod app_state;
mod commands;
mod db;
pub mod sync;

use db::ManagedDb;
use tauri::Manager;

/// Migrate legacy vaelorium.db (pre-Tomes) to a .vaelorium file on first launch.
async fn migrate_legacy_db(app: &tauri::AppHandle, managed: &ManagedDb) {
    let app_data = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let legacy_path = app_data.join("vaelorium.db");
    if !legacy_path.exists() {
        return;
    }

    // Check if we already have recent tomes — if so, migration was already done
    let state = app_state::load_app_state(app);
    if !state.recent_tomes.is_empty() {
        return;
    }

    log::info!("Found legacy database, migrating to Tome format...");

    // Copy to a .vaelorium file in Documents or alongside the old DB
    let docs_dir = app
        .path()
        .document_dir()
        .unwrap_or_else(|_| app_data.clone());

    let tome_dir = docs_dir.join("Vaelorium Tomes");
    std::fs::create_dir_all(&tome_dir).ok();

    let tome_path = tome_dir.join("My Campaign.tome");
    let tome_path_str = tome_path.to_string_lossy().to_string();

    if let Err(e) = std::fs::copy(&legacy_path, &tome_path) {
        log::error!("Failed to copy legacy DB: {}", e);
        return;
    }

    // Open the copied tome and run migrations (adds tome_metadata table)
    match db::open_database(&tome_path_str).await {
        Ok(pool) => {
            // Seed tome metadata
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("INSERT OR IGNORE INTO tome_metadata (key, value) VALUES ('name', 'My Campaign')")
                .execute(&pool)
                .await
                .ok();
            sqlx::query("INSERT OR IGNORE INTO tome_metadata (key, value) VALUES ('created_at', ?)")
                .bind(&now)
                .execute(&pool)
                .await
                .ok();

            // Set as active pool
            {
                let mut guard = managed.write().await;
                *guard = Some(pool);
            }

            // Add to recent tomes
            app_state::add_recent_tome(app, &tome_path_str, "My Campaign", None);

            log::info!("Legacy database migrated to: {}", tome_path_str);
        }
        Err(e) => {
            log::error!("Failed to open migrated tome: {}", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_fs::init())?;
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

            // Manage a ManagedDb with no active connection — Tome Picker will open a Tome
            let managed_db = db::create_managed_db();

            // Auto-migrate legacy vaelorium.db to a .vaelorium Tome file
            let handle = app.handle().clone();
            let managed_clone = managed_db.clone();
            tauri::async_runtime::block_on(async {
                migrate_legacy_db(&handle, &managed_clone).await;
            });

            app.manage(managed_db.clone());

            // Sync session state + background runner.
            let session = sync::SessionState::new();
            app.manage(session.clone());
            sync::runner::start(app.handle().clone(), managed_db, session);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tomes
            commands::tomes::get_app_state,
            commands::tomes::create_tome,
            commands::tomes::open_tome,
            commands::tomes::close_tome,
            commands::tomes::get_tome_metadata,
            commands::tomes::update_tome_metadata,
            // Pages
            commands::pages::create_page,
            commands::pages::get_page,
            commands::pages::update_page,
            commands::pages::delete_page,
            commands::pages::list_pages,
            commands::pages::list_pages_by_type,
            commands::pages::get_page_tree,
            commands::pages::save_page_content,
            commands::pages::get_page_content,
            commands::pages::reorder_pages,
            // Wiki Links
            commands::wiki_links::save_wiki_links,
            commands::wiki_links::get_backlinks,
            // Search
            commands::search::update_search_index,
            commands::search::search_pages,
            // Tags
            commands::tags::create_tag,
            commands::tags::list_tags,
            commands::tags::add_tag_to_page,
            commands::tags::remove_tag_from_page,
            commands::tags::get_page_tags,
            // Versions
            commands::versions::create_version,
            commands::versions::list_versions,
            commands::versions::get_version_snapshot,
            // Entity Types
            commands::entity_types::list_entity_types,
            commands::entity_types::get_entity_type,
            commands::entity_types::create_entity_type,
            commands::entity_types::update_entity_type,
            commands::entity_types::delete_entity_type,
            // Entity Fields
            commands::entity_fields::list_entity_type_fields,
            commands::entity_fields::create_entity_type_field,
            commands::entity_fields::update_entity_type_field,
            commands::entity_fields::delete_entity_type_field,
            commands::entity_fields::reorder_entity_type_fields,
            // Images
            commands::images::upload_image,
            commands::images::upload_image_data,
            commands::images::get_image,
            commands::images::delete_image,
            commands::images::list_images,
            // Export/Import
            commands::export::export_tome_json,
            commands::export::export_tome_markdown,
            commands::import_data::import_markdown_folder,
            commands::import_data::import_json,
            // Backup (app-global backend)
            commands::backup::backup_configure,
            commands::backup::backup_disconnect,
            commands::backup::backup_status,
            commands::backup::backup_unlock,
            commands::backup::backup_try_auto_unlock,
            commands::backup::backup_set_device_name,
            commands::backup::backup_list_restorable_tomes,
            commands::backup::backup_restore_tome,
            // Sync (per-Tome opt-in)
            commands::sync::sync_enable,
            commands::sync::sync_disable,
            commands::sync::sync_now,
            commands::sync::sync_status,
            commands::sync::sync_take_snapshot,
            commands::sync::sync_list_conflicts,
            commands::sync::sync_resolve_conflict,
            // Boards
            commands::boards::create_board,
            commands::boards::list_boards,
            commands::boards::delete_board,
            commands::boards::create_card,
            commands::boards::update_card,
            commands::boards::delete_card,
            commands::boards::get_board_cards,
            commands::boards::create_connector,
            commands::boards::delete_connector,
            commands::boards::get_board_connectors,
            // Maps
            commands::maps::create_map,
            commands::maps::list_maps,
            commands::maps::get_map,
            commands::maps::delete_map,
            commands::maps::create_pin,
            commands::maps::update_pin,
            commands::maps::delete_pin,
            commands::maps::get_map_pins,
            // Timelines
            commands::timelines::create_timeline,
            commands::timelines::list_timelines,
            commands::timelines::delete_timeline,
            commands::timelines::create_timeline_event,
            commands::timelines::update_timeline_event,
            commands::timelines::delete_timeline_event,
            commands::timelines::get_timeline_events,
            // Relations
            commands::relations::list_relation_types,
            commands::relations::create_relation_type,
            commands::relations::create_relation,
            commands::relations::delete_relation,
            commands::relations::get_page_relations,
            commands::relations::list_all_relations,
            // Field Values
            commands::field_values::get_page_field_values,
            commands::field_values::set_field_value,
            commands::field_values::delete_field_value,
            commands::field_values::query_pages_by_field,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
