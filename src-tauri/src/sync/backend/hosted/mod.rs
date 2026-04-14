//! Vaelorium Cloud (`BackendKind::Hosted`) — hosted encrypted backup tier.
//!
//! Talks to `https://cloud.vaelorium.com` over a custom HTTPS protocol
//! (see `~/Projects/vaelorium-cloud/docs/m5-app-integration-brief.md`).
//! Fully zero-knowledge: the cloud stores only opaque ciphertext blobs
//! and has no way to decrypt them. Blob encryption is the existing M7
//! per-Tome Argon2id + ChaCha20-Poly1305 path (`sync::crypto`).
//!
//! This module adds **account-level** crypto — a second Argon2id pass
//! that derives an `auth_hash` (sent to the server) and an `enc_key`
//! (unwraps a server-stored `account_key`). That `account_key` is never
//! used to encrypt user content; it's reserved for future
//! recovery-phrase-based password reset (M6+).

pub mod crypto;
pub mod protocol;
pub mod backend;

pub use backend::HostedBackend;
pub use protocol::{Client as HostedClient, CloudAccountInfo};
