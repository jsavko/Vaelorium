use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentTome {
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub last_opened: String,
    /// Per-Tome stable UUID from `tome_metadata.tome_uuid`. Populated
    /// when the Tome is opened or created (M5 added this so TomePicker
    /// can cross-reference recent cards against the backup listing
    /// to show a cloud badge and to de-dup restore entries). `None`
    /// on legacy entries written before this field existed.
    #[serde(default)]
    pub tome_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    pub recent_tomes: Vec<RecentTome>,
}

fn app_state_path(app: &AppHandle) -> PathBuf {
    let app_data = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    std::fs::create_dir_all(&app_data).ok();
    app_data.join("app_state.json")
}

pub fn load_app_state(app: &AppHandle) -> AppState {
    let path = app_state_path(app);
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppState::default(),
    }
}

pub fn save_app_state(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let path = app_state_path(app);
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_recent_tome(
    app: &AppHandle,
    path: &str,
    name: &str,
    description: Option<&str>,
    tome_uuid: Option<&str>,
) {
    let mut state = load_app_state(app);
    // Remove existing entry for this path
    state.recent_tomes.retain(|t| t.path != path);
    // Add at the front
    state.recent_tomes.insert(
        0,
        RecentTome {
            path: path.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            last_opened: chrono::Utc::now().to_rfc3339(),
            tome_uuid: tome_uuid.map(|s| s.to_string()),
        },
    );
    // Keep at most 10
    state.recent_tomes.truncate(10);
    save_app_state(app, &state).ok();
}
