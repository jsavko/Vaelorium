//! Unencrypted bucket-root metadata object (`sync-meta.json`).
//!
//! Holds the passphrase salt so that a second device / freshly-configured
//! Tome can derive the same key from the same passphrase instead of
//! generating its own salt and failing to decrypt existing remote data.
//!
//! `salt` is intentionally not secret — Argon2id is strong enough that an
//! attacker with the salt still needs to guess the passphrase. Keeping it
//! unencrypted means new devices can bootstrap without any out-of-band
//! channel beyond the passphrase itself.

use crate::sync::backend::{BackendError, SyncBackend};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const META_KEY: &str = "sync-meta.json";
pub const META_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMeta {
    pub version: u32,
    /// Base64-encoded Argon2id salt (16 bytes).
    pub salt_b64: String,
    /// The tome_id that first enabled sync against this bucket. Used to
    /// refuse enabling a *different* Tome on the same bucket (which would
    /// scramble the snapshot/journal namespace).
    pub tome_id: String,
    pub created_at: DateTime<Utc>,
}

impl RemoteMeta {
    pub fn new(salt: &[u8], tome_id: &str) -> Self {
        Self {
            version: META_VERSION,
            salt_b64: B64.encode(salt),
            tome_id: tome_id.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn salt(&self) -> Result<Vec<u8>, String> {
        B64.decode(&self.salt_b64)
            .map_err(|e| format!("corrupt sync-meta.json (bad salt_b64): {e}"))
    }
}

/// Fetch the bucket's sync-meta.json. Returns `Ok(None)` if the object
/// doesn't exist yet (fresh bucket).
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

/// Write the bucket's sync-meta.json (overwrites any existing).
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
