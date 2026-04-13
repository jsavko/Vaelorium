//! Background sync runner.
//!
//! One Tokio task per app process. Wakes up on either:
//! - the slow tick (every 5 minutes) to catch remote-side changes, or
//! - a `nudge` from a mutation path, debounced ~10s, to push local edits.
//!
//! With app-global backup, each tick iterates every Tome that has
//! `sync_config.enabled = 1` for the currently-open Tome's DB.

use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio::time::{sleep, Instant};

use super::engine::sync_tome_once;
use super::session::SessionState;
use super::state::{SyncConfig, SyncRuntimeState};
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
    tauri::async_runtime::spawn(async move {
        run_loop(app, managed, session).await;
    });
}

async fn run_loop(app: AppHandle, managed: ManagedDb, session: SessionState) {
    let nudge: Arc<Notify> = session.nudge_notify.clone();
    loop {
        tokio::select! {
            _ = sleep(POLL_INTERVAL) => {},
            _ = nudge.notified() => {
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

        // Need an unlocked session AND an open Tome DB.
        let Some(active) = session.current().await else { continue };
        let Some(pool) = managed.read().await.clone() else { continue };

        // Pull every Tome flagged enabled in the active Tome's DB.
        let configs = match SyncConfig::list_all(&pool).await {
            Ok(v) => v.into_iter().filter(|c| c.enabled).collect::<Vec<_>>(),
            Err(e) => {
                log::warn!("runner: list_all failed: {e}");
                continue;
            }
        };

        for cfg in configs {
            let backend = match crate::commands::sync::build_tome_backend(&app, &pool).await {
                Ok(b) => b,
                Err(e) => {
                    emit_error(&app, &cfg.tome_id, format!("backend init failed: {e}"));
                    continue;
                }
            };

            let _ = app.emit(
                "sync:status",
                SyncStatusEvent {
                    tome_id: cfg.tome_id.clone(),
                    state: "syncing",
                    ops_uploaded: 0,
                    ops_applied: 0,
                    conflicts_created: 0,
                    error: None,
                },
            );

            let started = chrono::Utc::now();
            let t0 = std::time::Instant::now();
            match sync_tome_once(&pool, &cfg.tome_id, &active.key, backend.as_ref()).await {
                Ok(outcome) => {
                    let dur = t0.elapsed().as_millis() as i64;
                    let outcome_kind = if outcome.error.is_some() { "error" } else { "success" };
                    super::activity::record(
                        &pool,
                        &cfg.tome_id,
                        started,
                        dur,
                        outcome_kind,
                        Some(&outcome),
                        outcome.error.as_deref(),
                    )
                    .await;
                    let _ = app.emit(
                        "sync:status",
                        SyncStatusEvent {
                            tome_id: cfg.tome_id.clone(),
                            state: if outcome.error.is_some() { "error" } else { "idle" },
                            ops_uploaded: outcome.ops_uploaded,
                            ops_applied: outcome.ops_applied,
                            conflicts_created: outcome.conflicts_created,
                            error: outcome.error.clone(),
                        },
                    );
                }
                Err(e) => {
                    let dur = t0.elapsed().as_millis() as i64;
                    let msg = e.to_string();
                    super::activity::record(
                        &pool,
                        &cfg.tome_id,
                        started,
                        dur,
                        "error",
                        None,
                        Some(&msg),
                    )
                    .await;
                    if let Ok(mut state) = SyncRuntimeState::load(&pool, &cfg.tome_id).await {
                        state.last_error = Some(msg.clone());
                        let _ = state.save(&pool).await;
                    }
                    emit_error(&app, &cfg.tome_id, msg);
                }
            }
        }
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

pub fn session_state(app: &AppHandle) -> SessionState {
    app.state::<SessionState>().inner().clone()
}
