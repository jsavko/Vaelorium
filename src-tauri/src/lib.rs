mod commands;
mod db;

use db::DbPool;
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

            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async {
                let pool = db::init_db(&handle)
                    .await
                    .expect("Failed to initialize database");
                handle.manage(pool);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Pages
            commands::pages::create_page,
            commands::pages::get_page,
            commands::pages::update_page,
            commands::pages::delete_page,
            commands::pages::list_pages,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
