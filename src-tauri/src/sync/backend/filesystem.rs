//! Filesystem-backed [`SyncBackend`] implementation.
//!
//! Stores objects as files under a configured root directory. Etags are
//! SHA-256 hex of the object contents (deterministic, content-addressable).
//!
//! Atomic writes use the standard "write to temp file in same directory,
//! then rename" pattern — `rename(2)` is atomic on POSIX and on Windows
//! when source and destination are on the same volume. CAS via
//! [`atomic_swap`](SyncBackend::atomic_swap) reads the current etag, compares,
//! then writes; this is racy under concurrent writers to the same key, but
//! filesystem backends are intended for single-writer or Syncthing-style
//! eventually-consistent setups where that race is rare and tolerable.

use super::{BackendError, ObjectInfo, SyncBackend};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct FilesystemBackend {
    root: PathBuf,
}

impl FilesystemBackend {
    /// Create a new backend rooted at `root`. The directory is created if it
    /// doesn't exist.
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self, BackendError> {
        let root = root.into();
        fs::create_dir_all(&root).await?;
        Ok(Self { root })
    }

    fn path_for(&self, key: &str) -> PathBuf {
        // Object keys are forward-slash-separated logical paths. Map directly
        // into the OS filesystem; the OS handles separator translation.
        self.root.join(key)
    }

    fn etag_of(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    async fn read_etag(path: &Path) -> Result<String, BackendError> {
        let data = fs::read(path).await?;
        Ok(Self::etag_of(&data))
    }
}

#[async_trait]
impl SyncBackend for FilesystemBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<String, BackendError> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write to temp file in same directory, then atomic rename.
        let tmp = path.with_extension(format!(
            "tmp.{}",
            ulid::Ulid::new().to_string()
        ));
        {
            let mut f = fs::File::create(&tmp).await?;
            f.write_all(data).await?;
            f.sync_all().await?;
        }
        fs::rename(&tmp, &path).await?;

        Ok(Self::etag_of(data))
    }

    async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), BackendError> {
        let path = self.path_for(key);
        match fs::read(&path).await {
            Ok(data) => {
                let etag = Self::etag_of(&data);
                Ok((data, etag))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(BackendError::NotFound(key.to_string()))
            }
            Err(e) => Err(BackendError::Io(e)),
        }
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<ObjectInfo>, BackendError> {
        let prefix_path = self.path_for(prefix);
        // The "prefix" model maps to "list every regular file under
        // <root>/<prefix>/" plus any files whose name starts with the basename
        // of the prefix. For our usage prefix always names a directory, so we
        // walk it.
        let mut out = Vec::new();
        if !prefix_path.exists() {
            return Ok(out);
        }

        let prefix_meta = fs::metadata(&prefix_path).await?;
        if prefix_meta.is_file() {
            // Single-file prefix match.
            let info = file_to_object_info(&prefix_path, prefix.to_string()).await?;
            out.push(info);
            return Ok(out);
        }

        let mut stack = vec![prefix_path.clone()];
        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let ft = entry.file_type().await?;
                if ft.is_dir() {
                    stack.push(path);
                } else if ft.is_file() {
                    // Skip in-flight temp files from interrupted writes.
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.contains(".tmp.") {
                            continue;
                        }
                    }
                    let key = path
                        .strip_prefix(&self.root)
                        .map_err(|e| BackendError::Other(e.to_string()))?
                        .to_string_lossy()
                        .replace('\\', "/");
                    let info = file_to_object_info(&path, key).await?;
                    out.push(info);
                }
            }
        }

        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    async fn delete_object(&self, key: &str) -> Result<(), BackendError> {
        let path = self.path_for(key);
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(BackendError::Io(e)),
        }
    }

    async fn head_object(&self, key: &str) -> Result<ObjectInfo, BackendError> {
        let path = self.path_for(key);
        if !path.exists() {
            return Err(BackendError::NotFound(key.to_string()));
        }
        file_to_object_info(&path, key.to_string()).await
    }

    async fn atomic_swap(
        &self,
        key: &str,
        expected_etag: Option<&str>,
        data: &[u8],
    ) -> Result<String, BackendError> {
        let path = self.path_for(key);
        let current = if path.exists() {
            Some(Self::read_etag(&path).await?)
        } else {
            None
        };

        match (current.as_deref(), expected_etag) {
            (None, None) => { /* creating new — ok */ }
            (Some(c), Some(e)) if c == e => { /* matching expectation — ok */ }
            (Some(c), None) => {
                return Err(BackendError::EtagMismatch {
                    key: key.to_string(),
                    expected: "(absent)".to_string(),
                    found: c.to_string(),
                });
            }
            (Some(c), Some(e)) => {
                return Err(BackendError::EtagMismatch {
                    key: key.to_string(),
                    expected: e.to_string(),
                    found: c.to_string(),
                });
            }
            (None, Some(e)) => {
                return Err(BackendError::EtagMismatch {
                    key: key.to_string(),
                    expected: e.to_string(),
                    found: "(absent)".to_string(),
                });
            }
        }

        self.put_object(key, data).await
    }
}

async fn file_to_object_info(path: &Path, key: String) -> Result<ObjectInfo, BackendError> {
    let meta = fs::metadata(path).await?;
    let data = fs::read(path).await?;
    let modified: DateTime<Utc> = meta
        .modified()
        .ok()
        .and_then(|t| Some(DateTime::<Utc>::from(t)))
        .unwrap_or_else(Utc::now);
    Ok(ObjectInfo {
        key,
        size: meta.len(),
        etag: FilesystemBackend::etag_of(&data),
        last_modified: modified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn make_backend() -> (TempDir, FilesystemBackend) {
        let dir = TempDir::new().unwrap();
        let backend = FilesystemBackend::new(dir.path()).await.unwrap();
        (dir, backend)
    }

    #[tokio::test]
    async fn put_and_get_roundtrip() {
        let (_dir, backend) = make_backend().await;
        let etag = backend.put_object("foo/bar.bin", b"hello").await.unwrap();
        let (data, got_etag) = backend.get_object("foo/bar.bin").await.unwrap();
        assert_eq!(data, b"hello");
        assert_eq!(etag, got_etag);
    }

    #[tokio::test]
    async fn etag_is_deterministic_for_same_content() {
        let (_dir, backend) = make_backend().await;
        let e1 = backend.put_object("a", b"same").await.unwrap();
        let e2 = backend.put_object("b", b"same").await.unwrap();
        assert_eq!(e1, e2);
    }

    #[tokio::test]
    async fn etag_changes_when_content_changes() {
        let (_dir, backend) = make_backend().await;
        let e1 = backend.put_object("a", b"first").await.unwrap();
        let e2 = backend.put_object("a", b"second").await.unwrap();
        assert_ne!(e1, e2);
    }

    #[tokio::test]
    async fn get_missing_returns_not_found() {
        let (_dir, backend) = make_backend().await;
        let result = backend.get_object("nope").await;
        assert!(matches!(result, Err(BackendError::NotFound(_))));
    }

    #[tokio::test]
    async fn list_prefix_walks_directory() {
        let (_dir, backend) = make_backend().await;
        backend.put_object("snapshots/01.snap", b"a").await.unwrap();
        backend.put_object("snapshots/02.snap", b"b").await.unwrap();
        backend.put_object("journal/op-1.enc", b"c").await.unwrap();

        let snaps = backend.list_prefix("snapshots").await.unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].key, "snapshots/01.snap");
        assert_eq!(snaps[1].key, "snapshots/02.snap");

        let journal = backend.list_prefix("journal").await.unwrap();
        assert_eq!(journal.len(), 1);
        assert_eq!(journal[0].key, "journal/op-1.enc");
    }

    #[tokio::test]
    async fn list_prefix_for_nonexistent_returns_empty() {
        let (_dir, backend) = make_backend().await;
        let out = backend.list_prefix("nope").await.unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn delete_is_idempotent() {
        let (_dir, backend) = make_backend().await;
        backend.put_object("k", b"v").await.unwrap();
        backend.delete_object("k").await.unwrap();
        backend.delete_object("k").await.unwrap(); // second time is fine
        let result = backend.get_object("k").await;
        assert!(matches!(result, Err(BackendError::NotFound(_))));
    }

    #[tokio::test]
    async fn head_object_returns_metadata() {
        let (_dir, backend) = make_backend().await;
        let etag = backend.put_object("k", b"hello").await.unwrap();
        let info = backend.head_object("k").await.unwrap();
        assert_eq!(info.key, "k");
        assert_eq!(info.size, 5);
        assert_eq!(info.etag, etag);
    }

    #[tokio::test]
    async fn atomic_swap_succeeds_when_etag_matches() {
        let (_dir, backend) = make_backend().await;
        let e1 = backend.put_object("k", b"v1").await.unwrap();
        let e2 = backend.atomic_swap("k", Some(&e1), b"v2").await.unwrap();
        assert_ne!(e1, e2);
        let (data, _) = backend.get_object("k").await.unwrap();
        assert_eq!(data, b"v2");
    }

    #[tokio::test]
    async fn atomic_swap_fails_on_etag_mismatch() {
        let (_dir, backend) = make_backend().await;
        backend.put_object("k", b"v1").await.unwrap();
        let result = backend
            .atomic_swap("k", Some("wrong-etag"), b"v2")
            .await;
        assert!(matches!(result, Err(BackendError::EtagMismatch { .. })));
        // Original content preserved.
        let (data, _) = backend.get_object("k").await.unwrap();
        assert_eq!(data, b"v1");
    }

    #[tokio::test]
    async fn atomic_swap_creates_when_no_existing_object_expected() {
        let (_dir, backend) = make_backend().await;
        let etag = backend.atomic_swap("new", None, b"hello").await.unwrap();
        let (data, got_etag) = backend.get_object("new").await.unwrap();
        assert_eq!(data, b"hello");
        assert_eq!(etag, got_etag);
    }

    #[tokio::test]
    async fn atomic_swap_fails_when_expected_absent_but_present() {
        let (_dir, backend) = make_backend().await;
        backend.put_object("k", b"existing").await.unwrap();
        let result = backend.atomic_swap("k", None, b"new").await;
        assert!(matches!(result, Err(BackendError::EtagMismatch { .. })));
    }

    #[tokio::test]
    async fn list_prefix_skips_inflight_temp_files() {
        let (dir, backend) = make_backend().await;
        // Manually drop a stray .tmp.* file as if a write was interrupted.
        let stray = dir.path().join("snapshots").join("real.snap.tmp.01HJK");
        fs::create_dir_all(stray.parent().unwrap()).await.unwrap();
        fs::write(&stray, b"interrupted").await.unwrap();
        backend.put_object("snapshots/real.snap", b"ok").await.unwrap();

        let out = backend.list_prefix("snapshots").await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].key, "snapshots/real.snap");
    }
}
