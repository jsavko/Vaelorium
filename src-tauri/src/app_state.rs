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
    /// Whether this Tome has `sync_config.enabled = 1` in its own
    /// SQLite DB. Mirrored here so TomePicker can decide whether to
    /// draw the cloud badge and whether to offer the Tome in the
    /// restore list (stop-sync → Tome re-appears for deletion)
    /// without opening each Tome's DB. Kept in sync by
    /// `sync_enable` / `sync_disable` / `open_tome` / restore flows.
    #[serde(default)]
    pub sync_enabled: bool,
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
    sync_enabled: bool,
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
            sync_enabled,
        },
    );
    // Keep at most 10
    state.recent_tomes.truncate(10);
    save_app_state(app, &state).ok();
}

/// Update the `sync_enabled` flag on an existing recent_tomes entry
/// matched by `path`. No-op if the path isn't in the list — the next
/// `add_recent_tome` will carry the correct flag forward. Called
/// from `sync_enable` / `sync_disable` so TomePicker's cloud badge
/// flips in-session without waiting for Tome close.
pub fn set_sync_enabled_for_path(app: &AppHandle, path: &str, sync_enabled: bool) {
    let mut state = load_app_state(app);
    let mut changed = false;
    for t in state.recent_tomes.iter_mut() {
        if t.path == path && t.sync_enabled != sync_enabled {
            t.sync_enabled = sync_enabled;
            changed = true;
        }
    }
    if changed {
        save_app_state(app, &state).ok();
    }
}
