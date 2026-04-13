//! Prefixed wrapper for any [`SyncBackend`].
//!
//! Used to namespace per-Tome data under `tomes/{tome_id}/` in a shared
//! app-global backend, so multiple Tomes can coexist in one bucket
//! without colliding on snapshot/journal keys. The wrapper transparently
//! prepends the prefix on every write/read and strips it off keys
//! returned by `list_prefix`, so engine code can stay prefix-agnostic.

use super::{BackendError, ObjectInfo, SyncBackend};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrefixedBackend {
    inner: Arc<dyn SyncBackend + Send + Sync>,
    prefix: String,
}

impl PrefixedBackend {
    pub fn new(inner: Arc<dyn SyncBackend + Send + Sync>, prefix: impl Into<String>) -> Self {
        let mut prefix = prefix.into();
        // Normalize: no leading or trailing slash; we always join with '/'.
        while prefix.ends_with('/') {
            prefix.pop();
        }
        Self { inner, prefix }
    }

    fn full(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix, key)
        }
    }

    fn strip<'a>(&self, key: &'a str) -> &'a str {
        if self.prefix.is_empty() {
            return key;
        }
        let with_slash = format!("{}/", self.prefix);
        key.strip_prefix(&with_slash).unwrap_or(key)
    }
}

#[async_trait]
impl SyncBackend for PrefixedBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<String, BackendError> {
        self.inner.put_object(&self.full(key), data).await
    }

    async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), BackendError> {
        self.inner.get_object(&self.full(key)).await
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<ObjectInfo>, BackendError> {
        let infos = self.inner.list_prefix(&self.full(prefix)).await?;
        Ok(infos
            .into_iter()
            .map(|i| ObjectInfo {
                key: self.strip(&i.key).to_string(),
                size: i.size,
                etag: i.etag,
                last_modified: i.last_modified,
            })
            .collect())
    }

    async fn delete_object(&self, key: &str) -> Result<(), BackendError> {
        self.inner.delete_object(&self.full(key)).await
    }

    async fn head_object(&self, key: &str) -> Result<ObjectInfo, BackendError> {
        let mut info = self.inner.head_object(&self.full(key)).await?;
        info.key = self.strip(&info.key).to_string();
        Ok(info)
    }

    async fn atomic_swap(
        &self,
        key: &str,
        expected_etag: Option<&str>,
        data: &[u8],
    ) -> Result<String, BackendError> {
        self.inner
            .atomic_swap(&self.full(key), expected_etag, data)
            .await
    }
}

/// Build the canonical Tome prefix.
pub fn tome_prefix(tome_id: &str) -> String {
    format!("tomes/{tome_id}")
}

#[cfg(test)]
mod tests {
    use super::super::filesystem::FilesystemBackend;
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn prefixes_writes_and_reads() {
        let dir = tempdir().unwrap();
        let inner: Arc<dyn SyncBackend + Send + Sync> =
            Arc::new(FilesystemBackend::new(dir.path().to_path_buf()).await.unwrap());
        let pref = PrefixedBackend::new(inner.clone(), "tomes/abc");

        pref.put_object("snapshots/s1.snap", b"hello").await.unwrap();

        let (bytes, _) = pref.get_object("snapshots/s1.snap").await.unwrap();
        assert_eq!(bytes, b"hello");

        // Underlying file ended up at tomes/abc/snapshots/s1.snap.
        let raw = inner.get_object("tomes/abc/snapshots/s1.snap").await.unwrap();
        assert_eq!(raw.0, b"hello");
    }

    #[tokio::test]
    async fn list_strips_prefix_from_returned_keys() {
        let dir = tempdir().unwrap();
        let inner: Arc<dyn SyncBackend + Send + Sync> =
            Arc::new(FilesystemBackend::new(dir.path().to_path_buf()).await.unwrap());
        let pref = PrefixedBackend::new(inner, "tomes/abc");

        pref.put_object("journal/o1.op.enc", b"a").await.unwrap();
        pref.put_object("journal/o2.op.enc", b"b").await.unwrap();

        let infos = pref.list_prefix("journal").await.unwrap();
        let keys: Vec<&str> = infos.iter().map(|i| i.key.as_str()).collect();
        assert!(keys.contains(&"journal/o1.op.enc"));
        assert!(keys.contains(&"journal/o2.op.enc"));
    }

    #[test]
    fn tome_prefix_format() {
        assert_eq!(tome_prefix("01HABC"), "tomes/01HABC");
    }
}
