//! M7 Sync — opt-in, per-Tome synchronization.
//!
//! Architecture (see `.claude/brainstorms/2026-04-12-m7-sync-architecture.md`
//! and `.claude/plans/2026-04-12-m7-sync-mvp.md` for full context):
//!
//! - Hybrid model: encrypted snapshot baseline + append-only encrypted delta journal
//! - End-to-end encrypted (per-account passphrase, Argon2id KDF, ChaCha20-Poly1305 AEAD)
//! - Pluggable backends (filesystem, S3-compatible)
//! - Conflicts resolved inline, side-by-side, per-field — no silent LWW
//!
//! Phase 1 (this commit): module spine — op format, encryption, backend trait,
//! filesystem backend, sync state tables. No op interception or sync runner yet.

pub mod backend;
pub mod conflict;
pub mod crypto;
pub mod engine;
pub mod journal;
pub mod keychain;
pub mod registry;
pub mod runner;
pub mod session;
pub mod snapshot;
pub mod state;

pub use engine::{sync_tome_once, SyncOutcome};
pub use session::SessionState;

use thiserror::Error;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("backend error: {0}")]
    Backend(#[from] backend::BackendError),

    #[error("crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),

    #[error("journal error: {0}")]
    Journal(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("schema version mismatch: op was emitted at v{op}, this client is v{client}")]
    SchemaVersionMismatch { op: u32, client: u32 },
}

pub type SyncResult<T> = Result<T, SyncError>;
