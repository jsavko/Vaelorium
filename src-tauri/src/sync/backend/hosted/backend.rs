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
        Self {
            http,
            tome_uuid,
            token,
        }
    }
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
