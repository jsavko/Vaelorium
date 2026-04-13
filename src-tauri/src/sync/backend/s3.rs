//! S3-compatible [`SyncBackend`] implementation.
//!
//! Works against any service that speaks the S3 API:
//! - AWS S3 (default endpoint; set `region`, leave `endpoint` empty)
//! - Cloudflare R2, Backblaze B2, Wasabi (set `endpoint` to their URL)
//! - Minio, Garage, self-hosted (set `endpoint` to your URL; `region` often
//!   "auto" or "us-east-1" — check your service)
//!
//! Atomic `atomic_swap` uses the `If-Match` header (for existing objects)
//! or `If-None-Match: *` (for new objects). Most modern S3-compatible
//! services support these; notably Minio does, and real AWS S3 supports
//! conditional writes as of 2024.
//!
//! Credentials live in `sync_config.backend_config` plaintext (the local
//! `.tome` is trusted). OS keychain integration is a Phase 5+ enhancement.

use super::{BackendError, ObjectInfo, SyncBackend};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: Option<String>, // None → AWS default
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub prefix: Option<String>, // path prefix inside the bucket
}

pub struct S3Backend {
    client: Client,
    bucket: String,
    prefix: Option<String>,
}

impl S3Backend {
    pub async fn new(config: S3Config) -> Result<Self, BackendError> {
        let creds = Credentials::new(
            config.access_key,
            config.secret_key,
            None,
            None,
            "vaelorium-sync",
        );

        let mut loader = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region))
            .credentials_provider(creds);

        if let Some(endpoint) = config.endpoint.filter(|s| !s.is_empty()) {
            loader = loader.endpoint_url(endpoint);
        }

        let shared = loader.load().await;

        // force_path_style is useful for Minio and many S3-compatible services
        // that don't support virtual-host-style bucket addressing.
        let s3_cfg = aws_sdk_s3::config::Builder::from(&shared)
            .force_path_style(true)
            .build();

        Ok(S3Backend {
            client: Client::from_conf(s3_cfg),
            bucket: config.bucket,
            prefix: config.prefix.filter(|s| !s.is_empty()),
        })
    }

    fn full_key(&self, key: &str) -> String {
        match &self.prefix {
            Some(p) => format!("{}/{}", p.trim_end_matches('/'), key),
            None => key.to_string(),
        }
    }

    fn strip_prefix<'a>(&self, full_key: &'a str) -> &'a str {
        match &self.prefix {
            Some(p) => {
                let p = p.trim_end_matches('/');
                full_key
                    .strip_prefix(&format!("{p}/"))
                    .unwrap_or(full_key)
            }
            None => full_key,
        }
    }
}

/// Strip surrounding quotes from an S3 etag (they arrive as `"abc123"`).
fn clean_etag(etag: Option<String>) -> String {
    etag.unwrap_or_default().trim_matches('"').to_string()
}

/// Convert aws-sdk errors into our `BackendError`, with friendly messages
/// for the most common failure modes.
fn to_backend_error<E: std::fmt::Display>(e: E, context: &str) -> BackendError {
    let msg = e.to_string();
    // Pattern-match on common error strings. aws-sdk-s3's error types are
    // nested enums per operation; stringifying is simpler and robust across
    // provider variations.
    if msg.contains("NoSuchBucket") {
        BackendError::Other(format!("bucket not found (during {context})"))
    } else if msg.contains("InvalidAccessKeyId") || msg.contains("SignatureDoesNotMatch") {
        BackendError::Other(format!("authentication failed (during {context})"))
    } else if msg.contains("dispatch failure") || msg.contains("dns error") || msg.contains("Connection") {
        BackendError::Other(format!("endpoint unreachable (during {context})"))
    } else if msg.contains("PreconditionFailed") {
        // atomic_swap caller handles this specifically; this path is a safety net.
        BackendError::EtagMismatch {
            key: context.to_string(),
            expected: "(condition)".to_string(),
            found: "(actual differed)".to_string(),
        }
    } else {
        BackendError::Other(format!("{context}: {msg}"))
    }
}

#[async_trait]
impl SyncBackend for S3Backend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<String, BackendError> {
        let full = self.full_key(key);
        let out = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .map_err(|e| to_backend_error(e, &format!("put_object {full}")))?;
        Ok(clean_etag(out.e_tag))
    }

    async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), BackendError> {
        let full = self.full_key(key);
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full)
            .send()
            .await
            .map_err(|e| {
                // Prefer typed error inspection — `SdkError::ServiceError`
                // stringifies to "service error" on some providers (e.g.
                // Backblaze B2), so substring-matching on `e.to_string()`
                // misses NoSuchKey and the caller sees a spurious error.
                // `into_service_error()` takes ownership, so check first
                // via `as_service_error()`, and keep the substring fallback
                // for any formatting we haven't seen yet.
                use aws_sdk_s3::operation::get_object::GetObjectError;
                let nsk = e
                    .as_service_error()
                    .map(|se| matches!(se, GetObjectError::NoSuchKey(_)))
                    .unwrap_or(false);
                let http_404 = e
                    .raw_response()
                    .map(|r| r.status().as_u16() == 404)
                    .unwrap_or(false);
                if nsk || http_404 || e.to_string().contains("NoSuchKey") {
                    BackendError::NotFound(full.clone())
                } else {
                    to_backend_error(e, &format!("get_object {full}"))
                }
            })?;
        let etag = clean_etag(out.e_tag);
        let body = out
            .body
            .collect()
            .await
            .map_err(|e| BackendError::Other(format!("read body: {e}")))?
            .into_bytes()
            .to_vec();
        Ok((body, etag))
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<ObjectInfo>, BackendError> {
        let full_prefix = self.full_key(prefix);
        let mut out = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&full_prefix);
            if let Some(ref tok) = continuation_token {
                req = req.continuation_token(tok);
            }
            let resp = req
                .send()
                .await
                .map_err(|e| to_backend_error(e, &format!("list_prefix {full_prefix}")))?;

            for obj in resp.contents() {
                let Some(full_key) = obj.key() else { continue };
                let stripped = self.strip_prefix(full_key).to_string();
                let modified: DateTime<Utc> = obj
                    .last_modified()
                    .and_then(|t| DateTime::from_timestamp(t.secs(), 0))
                    .unwrap_or_else(Utc::now);
                out.push(ObjectInfo {
                    key: stripped,
                    size: obj.size().unwrap_or(0).max(0) as u64,
                    etag: clean_etag(obj.e_tag().map(|s| s.to_string())),
                    last_modified: modified,
                });
            }

            if resp.is_truncated() == Some(true) {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
                if continuation_token.is_none() {
                    break;
                }
            } else {
                break;
            }
        }

        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    async fn delete_object(&self, key: &str) -> Result<(), BackendError> {
        let full = self.full_key(key);
        match self
            .client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("NoSuchKey") {
                    Ok(()) // idempotent
                } else {
                    Err(to_backend_error(e, &format!("delete_object {full}")))
                }
            }
        }
    }

    async fn head_object(&self, key: &str) -> Result<ObjectInfo, BackendError> {
        let full = self.full_key(key);
        let resp = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full)
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("NotFound") || msg.contains("NoSuchKey") {
                    BackendError::NotFound(full.clone())
                } else {
                    to_backend_error(e, &format!("head_object {full}"))
                }
            })?;

        let modified: DateTime<Utc> = resp
            .last_modified()
            .and_then(|t| DateTime::from_timestamp(t.secs(), 0))
            .unwrap_or_else(Utc::now);

        Ok(ObjectInfo {
            key: key.to_string(),
            size: resp.content_length().unwrap_or(0).max(0) as u64,
            etag: clean_etag(resp.e_tag),
            last_modified: modified,
        })
    }

    async fn atomic_swap(
        &self,
        key: &str,
        expected_etag: Option<&str>,
        data: &[u8],
    ) -> Result<String, BackendError> {
        let full = self.full_key(key);
        let mut req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full)
            .body(ByteStream::from(data.to_vec()));

        match expected_etag {
            Some(etag) => {
                // Quote the etag per S3 convention.
                req = req.if_match(format!("\"{}\"", etag.trim_matches('"')));
            }
            None => {
                // Expect object absent.
                req = req.if_none_match("*");
            }
        }

        let out = req.send().await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("PreconditionFailed") || msg.contains("412") {
                BackendError::EtagMismatch {
                    key: full.clone(),
                    expected: expected_etag.unwrap_or("(absent)").to_string(),
                    found: "(precondition failed)".to_string(),
                }
            } else {
                to_backend_error(e, &format!("atomic_swap {full}"))
            }
        })?;

        Ok(clean_etag(out.e_tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> S3Config {
        S3Config {
            endpoint: Some("http://localhost:9000".to_string()),
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
            prefix: Some("vaelorium".to_string()),
        }
    }

    #[tokio::test]
    async fn s3_backend_constructs_with_custom_endpoint() {
        // Just verifies the client builds without panicking.
        let backend = S3Backend::new(sample_config()).await.unwrap();
        assert_eq!(backend.bucket, "test-bucket");
        assert_eq!(backend.prefix.as_deref(), Some("vaelorium"));
    }

    #[tokio::test]
    async fn s3_backend_full_key_applies_prefix() {
        let backend = S3Backend::new(sample_config()).await.unwrap();
        assert_eq!(backend.full_key("snapshots/a.snap.enc"), "vaelorium/snapshots/a.snap.enc");
    }

    #[tokio::test]
    async fn s3_backend_full_key_no_prefix() {
        let mut cfg = sample_config();
        cfg.prefix = None;
        let backend = S3Backend::new(cfg).await.unwrap();
        assert_eq!(backend.full_key("snapshots/a"), "snapshots/a");
    }

    #[tokio::test]
    async fn s3_backend_strip_prefix() {
        let backend = S3Backend::new(sample_config()).await.unwrap();
        assert_eq!(backend.strip_prefix("vaelorium/snapshots/a"), "snapshots/a");
        // Unchanged if prefix isn't present.
        assert_eq!(backend.strip_prefix("other/path"), "other/path");
    }

    #[test]
    fn clean_etag_strips_quotes() {
        assert_eq!(clean_etag(Some(r#""abc123""#.to_string())), "abc123");
        assert_eq!(clean_etag(Some("abc123".to_string())), "abc123");
        assert_eq!(clean_etag(None), "");
    }

    #[test]
    fn to_backend_error_classifies_common_failures() {
        // No easy way to build a real AWS error; test the string-based classifier
        // by passing fake error strings directly.
        let err = to_backend_error("service error: NoSuchBucket", "test");
        assert!(matches!(err, BackendError::Other(ref m) if m.contains("bucket not found")));
        let err = to_backend_error("service error: InvalidAccessKeyId", "test");
        assert!(matches!(err, BackendError::Other(ref m) if m.contains("authentication failed")));
        let err = to_backend_error("dispatch failure: dns error", "test");
        assert!(matches!(err, BackendError::Other(ref m) if m.contains("endpoint unreachable")));
    }

    // Full end-to-end integration tests against a real S3-compatible service
    // (Minio, Localstack) are gated behind a manual recipe — see
    // docs/sync-s3-testing.md. The engine logic is exercised thoroughly by
    // the filesystem-backed scenarios in `tests/sync_integration.rs`; the S3
    // backend is a thin translation layer whose primitives work-or-don't at
    // the aws-sdk level.
}
