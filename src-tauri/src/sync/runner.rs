//! Background sync runner.
//!
//! One Tokio task per app process. Wakes up on either:
//! - the slow tick (every 5 minutes) to catch remote-side changes, or
//! - a `nudge` from a mutation path, debounced ~10s, to push local edits.
//!
//! Cancelled when the runner's `CancellationToken` is triggered (sync disable
//! or app close).

use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio::time::{sleep, Instant};

use super::backend::SyncBackend;
use super::engine::sync_tome_once;
use super::session::SessionState;
use super::state::SyncRuntimeState;
use crate::db::ManagedDb;

const POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const NUDGE_DEBOUNCE: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, serde::Serialize)]
pub struct SyncStatusEvent {
    pub tome_id: String,
    pub state: &'static str, // "syncing" | "idle" | "error"
    pub ops_uploaded: u32,
    pub ops_applied: u32,
    pub conflicts_created: u32,
    pub error: Option<String>,
}

pub fn start(app: AppHandle, managed: ManagedDb, session: SessionState) {
    // Use Tauri's async runtime (which is Tokio under the hood) so this
    // works when called from the synchronous setup hook, which doesn't
    // have a Tokio runtime context the way #[tokio::main] does.
    tauri::async_runtime::spawn(async move {
        run_loop(app, managed, session).await;
    });
}

async fn run_loop(app: AppHandle, managed: ManagedDb, session: SessionState) {
    let nudge: Arc<Notify> = session.nudge_notify.clone();
    loop {
        // Wait for either the slow tick or a nudge (with debounce).
        tokio::select! {
            _ = sleep(POLL_INTERVAL) => {},
            _ = nudge.notified() => {
                // Drain rapid follow-up nudges so we sync once after a burst.
                let until = Instant::now() + NUDGE_DEBOUNCE;
                loop {
                    let remaining = until.saturating_duration_since(Instant::now());
                    if remaining.is_zero() { break; }
                    tokio::select! {
                        _ = sleep(remaining) => break,
                        _ = nudge.notified() => continue,
                    }
                }
            }
        }

        // Snapshot the session, the pool, and run a sync if we can.
        let Some(active) = session.current().await else { continue };
        let Some(pool) = managed.read().await.clone() else { continue };

        let backend: Box<dyn SyncBackend + Send + Sync> =
            match crate::commands::sync::build_backend(active.backend_kind, &active.backend_config).await {
                Ok(b) => b,
                Err(e) => {
                    emit_error(&app, &active.tome_id, format!("backend init failed: {e}"));
                    continue;
                }
            };

        let _ = app.emit(
            "sync:status",
            SyncStatusEvent {
                tome_id: active.tome_id.clone(),
                state: "syncing",
                ops_uploaded: 0,
                ops_applied: 0,
                conflicts_created: 0,
                error: None,
            },
        );

        match sync_tome_once(&pool, &active.tome_id, &active.key, backend.as_ref()).await {
            Ok(outcome) => {
                let _ = app.emit(
                    "sync:status",
                    SyncStatusEvent {
                        tome_id: active.tome_id.clone(),
                        state: if outcome.error.is_some() { "error" } else { "idle" },
                        ops_uploaded: outcome.ops_uploaded,
                        ops_applied: outcome.ops_applied,
                        conflicts_created: outcome.conflicts_created,
                        error: outcome.error.clone(),
                    },
                );
            }
            Err(e) => {
                let msg = e.to_string();
                // Persist the error to sync_state so the UI can read it across restarts.
                if let Ok(mut state) = SyncRuntimeState::load(&pool, &active.tome_id).await {
                    state.last_error = Some(msg.clone());
                    let _ = state.save(&pool).await;
                }
                emit_error(&app, &active.tome_id, msg);
            }
        }

        // Loop continues — yields back to the select.
    }
}

fn emit_error(app: &AppHandle, tome_id: &str, msg: String) {
    let _ = app.emit(
        "sync:status",
        SyncStatusEvent {
            tome_id: tome_id.to_string(),
            state: "error",
            ops_uploaded: 0,
            ops_applied: 0,
            conflicts_created: 0,
            error: Some(msg),
        },
    );
}

/// Convenience: pull the SessionState from the AppHandle.
pub fn session_state(app: &AppHandle) -> SessionState {
    app.state::<SessionState>().inner().clone()
}
