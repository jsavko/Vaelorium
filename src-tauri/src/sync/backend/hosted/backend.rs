//! `SyncBackend` impl over the hosted cloud HTTPS protocol.
//!
//! Unlike `S3Backend` + `FilesystemBackend`, hosted does NOT wrap a
//! separate prefix layer — the tome UUID is baked into each URL path
//! (`/v1/tomes/<uuid>/...`). The runner still uses `PrefixedBackend`
//! over it so the engine stays prefix-agnostic; we just return stripped
//! keys from `list_prefix`.

use super::protocol::Client;
use crate::sync::backend::{BackendError, ObjectInfo, SyncBackend};
use async_trait::async_trait;
use chrono::TimeZone;

pub struct HostedBackend {
    http: Client,
    tome_uuid: String,
    token: String,
}

impl HostedBackend {
    pub fn new(http: Client, tome_uuid: String, token: String) -> Self {
        // Cloud validator requires the hyphenated 8-4-4-4-12 UUID form.
        // The app's `tome_metadata.tome_uuid` is stored as the simple
        // (32-char unhyphenated) form because the filesystem + S3 bucket
        // layouts use it that way (`tomes/<32hex>/...`). Hyphenate here
        // at the URL boundary so the app storage format stays stable
        // and cloud requests pass validation.
        let canonical = hyphenate_uuid(&tome_uuid);
        Self {
            http,
            tome_uuid: canonical,
            token,
        }
    }
}

/// Accept either the hyphenated or simple UUID form and emit the
/// canonical hyphenated `8-4-4-4-12` form. Falls back to the input
/// string if it's neither shape (no-op, lets the server reject it).
fn hyphenate_uuid(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() == 36 && trimmed.as_bytes().iter().filter(|&&b| b == b'-').count() == 4 {
        return trimmed.to_ascii_lowercase();
    }
    // Simple form: 32 lowercase hex chars, no hyphens.
    if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        let s = trimmed.to_ascii_lowercase();
        return format!(
            "{}-{}-{}-{}-{}",
            &s[0..8],
            &s[8..12],
            &s[12..16],
            &s[16..20],
            &s[20..32]
        );
    }
    trimmed.to_ascii_lowercase()
}

fn obj_info(key: String, size: u64, etag: String, last_modified_ms: i64) -> ObjectInfo {
    let last_modified = chrono::Utc
        .timestamp_millis_opt(last_modified_ms)
        .single()
        .unwrap_or_else(chrono::Utc::now);
    ObjectInfo {
        key,
        size,
        etag,
        last_modified,
    }
}

#[async_trait]
impl SyncBackend for HostedBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<String, BackendError> {
        let meta = self
            .http
            .put_object(&self.token, &self.tome_uuid, key, data.to_vec())
            .await?;
        Ok(meta.etag)
    }

    async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), BackendError> {
        self.http.get_object(&self.token, &self.tome_uuid, key).await
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<ObjectInfo>, BackendError> {
        let metas = self
            .http
            .list_prefix(&self.token, &self.tome_uuid, prefix)
            .await?;
        Ok(metas
            .into_iter()
            .map(|m| obj_info(m.key.unwrap_or_default(), m.size, m.etag, m.last_modified))
            .collect())
    }

    async fn delete_object(&self, key: &str) -> Result<(), BackendError> {
        self.http
            .delete_object(&self.token, &self.tome_uuid, key)
            .await
    }

    async fn head_object(&self, key: &str) -> Result<ObjectInfo, BackendError> {
        let meta = self
            .http
            .head_object(&self.token, &self.tome_uuid, key)
            .await?;
        Ok(obj_info(
            meta.key.unwrap_or_else(|| key.to_string()),
            meta.size,
            meta.etag,
            meta.last_modified,
        ))
    }

    async fn atomic_swap(
        &self,
        key: &str,
        expected_etag: Option<&str>,
        data: &[u8],
    ) -> Result<String, BackendError> {
        // Cloud protocol only supports CAS on sync-meta.json — the engine
        // exclusively calls atomic_swap there, but guard anyway.
        if !key.ends_with("sync-meta.json") && key != "sync-meta" {
            return Err(BackendError::Other(format!(
                "hosted backend only supports atomic_swap on sync-meta, got: {key}"
            )));
        }
        let meta = self
            .http
            .atomic_swap_meta(&self.token, &self.tome_uuid, expected_etag, data.to_vec())
            .await?;
        Ok(meta.etag)
    }
}

#[cfg(test)]
mod tests {
    use super::hyphenate_uuid;

    #[test]
    fn hyphenates_simple_form() {
        assert_eq!(
            hyphenate_uuid("abcdef0123456789abcdef0123456789"),
            "abcdef01-2345-6789-abcd-ef0123456789"
        );
    }

    #[test]
    fn preserves_hyphenated_form() {
        let s = "abcdef01-2345-6789-abcd-ef0123456789";
        assert_eq!(hyphenate_uuid(s), s);
    }

    #[test]
    fn lowercases_uppercase_input() {
        assert_eq!(
            hyphenate_uuid("ABCDEF0123456789ABCDEF0123456789"),
            "abcdef01-2345-6789-abcd-ef0123456789"
        );
    }

    #[test]
    fn passes_through_invalid_shapes_for_server_rejection() {
        assert_eq!(hyphenate_uuid("not-a-uuid"), "not-a-uuid");
    }
}
