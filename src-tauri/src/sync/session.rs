//! In-memory session state for sync.
//!
//! Holds the unlocked app-global [`KeyMaterial`] used by every Tome that
//! opts into backup, plus a `Notify` handle that mutation paths use to
//! nudge the runner. Process-local; never persisted.

use std::sync::Arc;
use tokio::sync::{Notify, RwLock};

use super::crypto::KeyMaterial;

/// The unlocked app-global session: the derived key for whatever backup
/// destination is configured in `sync-backend.json`. Per-Tome
/// `device_id` lives on `SyncConfig` rows in the per-Tome DB.
#[derive(Clone)]
pub struct SyncSession {
    pub key: Arc<KeyMaterial>,
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
