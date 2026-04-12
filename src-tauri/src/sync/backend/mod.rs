//! Sync backend abstraction.
//!
//! Backends are passive ciphertext object stores. The sync engine pushes and
//! pulls encrypted blobs (snapshots, ops, metadata, conflict descriptors) by
//! key. Backends never decrypt anything — they don't have the key.
//!
//! Two implementations land in M7 MVP:
//! - [`filesystem::FilesystemBackend`] — a local directory (also the test substrate)
//! - S3-compatible — Phase 4
//!
//! Etags carry version information for compare-and-swap atomic writes
//! (essential for the snapshot pointer and meta blob). For the filesystem
//! backend etags are SHA-256 hex of the object contents; for S3 they're the
//! standard ETag header.

pub mod filesystem;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("object not found: {0}")]
    NotFound(String),

    #[error("etag mismatch on atomic swap (key {key}, expected {expected}, found {found})")]
    EtagMismatch {
        key: String,
        expected: String,
        found: String,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("backend internal error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
}

/// Async object-store interface. All operations are atomic at the object level.
#[async_trait]
pub trait SyncBackend: Send + Sync {
    /// Upload `data` under `key`, overwriting any existing object.
    /// Returns the new etag.
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<String, BackendError>;

    /// Download `key`. Returns `(data, etag)`.
    async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), BackendError>;

    /// List objects whose key starts with `prefix`. Sorted by key ascending.
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<ObjectInfo>, BackendError>;

    /// Remove `key`. NotFound is treated as success (idempotent).
    async fn delete_object(&self, key: &str) -> Result<(), BackendError>;

    /// Cheap metadata fetch for `key` (no body download).
    async fn head_object(&self, key: &str) -> Result<ObjectInfo, BackendError>;

    /// Compare-and-swap. Writes `data` only if the current object's etag
    /// matches `expected_etag` (or the object doesn't exist when
    /// `expected_etag` is `None`). Returns the new etag.
    ///
    /// Used for snapshot-pointer / meta blob updates where two devices
    /// racing must not silently overwrite each other.
    async fn atomic_swap(
        &self,
        key: &str,
        expected_etag: Option<&str>,
        data: &[u8],
    ) -> Result<String, BackendError>;
}
