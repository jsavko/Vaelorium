mod app_state;
mod commands;
mod db;

use tauri::Manager;

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

            // Manage a ManagedDb with no active connection — Tome Picker will open a Tome
            let managed_db = db::create_managed_db();
            app.manage(managed_db);

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
            // Field Values
            commands::field_values::get_page_field_values,
            commands::field_values::set_field_value,
            commands::field_values::delete_field_value,
            commands::field_values::query_pages_by_field,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
