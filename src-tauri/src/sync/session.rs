//! In-memory session state for sync.
//!
//! Holds the unlocked [`KeyMaterial`] for the currently-syncing Tome, plus a
//! `Notify` handle that mutation paths use to nudge the runner into syncing
//! soon. Everything here is process-local — never persisted.

use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;

use super::crypto::KeyMaterial;

#[derive(Clone)]
pub struct SyncSession {
    pub tome_id: String,
    pub device_id: Uuid,
    pub key: Arc<KeyMaterial>,
    pub backend_kind: super::state::BackendKind,
    pub backend_config: serde_json::Value,
}

#[derive(Clone, Default)]
pub struct SessionState {
    inner: Arc<RwLock<Option<SyncSession>>>,
    /// Mutation paths call `nudge_notify.notify_one()` after committing a
    /// write. The runner waits on this with a debounce so it syncs ~10s
    /// after the last change.
    pub nudge_notify: Arc<Notify>,
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn current(&self) -> Option<SyncSession> {
        self.inner.read().await.clone()
    }

    pub async fn set(&self, session: SyncSession) {
        let mut g = self.inner.write().await;
        *g = Some(session);
    }

    pub async fn clear(&self) {
        let mut g = self.inner.write().await;
        *g = None;
    }

    pub fn nudge(&self) {
        self.nudge_notify.notify_one();
    }
}
