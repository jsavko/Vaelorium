//! Unencrypted bucket-root metadata object (`sync-meta.json`).
//!
//! Holds the passphrase salt so a fresh device can derive the same
//! Argon2id key from the same passphrase instead of generating its own
//! salt and failing to decrypt existing remote data.
//!
//! `salt` is intentionally not secret — Argon2id is strong enough that an
//! attacker with the salt still needs to guess the passphrase. Keeping it
//! unencrypted means new devices bootstrap with passphrase alone.
//!
//! The meta is **app-global** for a backend: one bucket / folder hosts
//! one or more Tomes, namespaced under `tomes/{tome_id}/`, all using the
//! same key derived from this salt.

use crate::sync::backend::{BackendError, SyncBackend};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const META_KEY: &str = "sync-meta.json";
pub const META_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMeta {
    pub version: u32,
    /// Base64-encoded Argon2id salt (16 bytes).
    pub salt_b64: String,
    pub created_at: DateTime<Utc>,
}

impl RemoteMeta {
    pub fn new(salt: &[u8]) -> Self {
        Self {
            version: META_VERSION,
            salt_b64: B64.encode(salt),
            created_at: Utc::now(),
        }
    }

    pub fn salt(&self) -> Result<Vec<u8>, String> {
        B64.decode(&self.salt_b64)
            .map_err(|e| format!("corrupt sync-meta.json (bad salt_b64): {e}"))
    }
}

/// Fetch the bucket's sync-meta.json. `Ok(None)` when fresh bucket.
pub async fn fetch(
    backend: &(dyn SyncBackend + Send + Sync),
) -> Result<Option<RemoteMeta>, String> {
    match backend.get_object(META_KEY).await {
        Ok((bytes, _etag)) => {
            let meta: RemoteMeta = serde_json::from_slice(&bytes)
                .map_err(|e| format!("corrupt sync-meta.json: {e}"))?;
            Ok(Some(meta))
        }
        Err(BackendError::NotFound(_)) => Ok(None),
        Err(e) => Err(format!("fetching sync-meta.json: {e}")),
    }
}

pub async fn put(
    backend: &(dyn SyncBackend + Send + Sync),
    meta: &RemoteMeta,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(meta)
        .map_err(|e| format!("serializing sync-meta.json: {e}"))?;
    backend
        .put_object(META_KEY, &bytes)
        .await
        .map_err(|e| format!("writing sync-meta.json: {e}"))?;
    Ok(())
}
