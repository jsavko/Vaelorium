//! HTTPS protocol client for Vaelorium Cloud.
//!
//! Wraps `reqwest::Client` with typed request/response shapes matching
//! `~/Projects/vaelorium-cloud/docs/m5-app-integration-brief.md`. Knows
//! nothing about crypto or keychain — takes bytes/tokens in, returns
//! typed results.

use crate::sync::backend::BackendError;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Default cloud base URL. Override via `VAELORIUM_CLOUD_URL` env var
/// for staging / local dev (respected at client construction).
pub const DEFAULT_BASE_URL: &str = "https://cloud.vaelorium.com";

/// Conservative request timeout. Large PUTs may need longer; callers can
/// override via `Client::with_timeout`.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAccountInfo {
    pub email: String,
    pub account_id: String,
    pub tier: Option<String>,
    /// RFC3339 timestamp of the signin that produced the currently-held
    /// device token. `None` pre-signin.
    pub signed_in_at: Option<String>,
    /// Account usage snapshot. Populated from the signin response and
    /// updated on every mutation response (cloud embeds `usage` in PUT
    /// and DELETE replies so the app never has to poll). `None` on
    /// non-hosted backends or when the server omits usage fields.
    #[serde(default)]
    pub usage: Option<CloudUsage>,
}

/// Usage snapshot shape emitted by the cloud on signin + every mutation
/// response. Contract frozen with cloud `efc9286` / `6b234f5` /
/// `50bd459` (see `~/Projects/vaelorium-cloud/docs/m5-status.md`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudUsage {
    pub bytes_used: u64,
    pub tome_count: u32,
    pub quota_bytes: u64,
    /// `None` on Author tier (unlimited).
    #[serde(default)]
    pub tome_limit: Option<u32>,
    pub subscription_status: String,
}

#[derive(Debug, Deserialize)]
struct SaltResponse {
    kdf_salt_b64: String,
}

#[derive(Debug, Serialize)]
struct SigninRequest<'a> {
    email: &'a str,
    auth_hash_b64: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct SigninResponse {
    pub account_id: String,
    pub email: String,
    pub device_token: String,
    pub kdf_salt_b64: String,
    pub wrapped_by_password_b64: String,
    #[serde(default)]
    pub tier: Option<String>,
    // Usage fields embedded in the signin 200 body (cloud `efc9286`).
    // All optional for backward compat with pre-change cloud builds.
    #[serde(default)]
    pub subscription_status: Option<String>,
    #[serde(default)]
    pub tome_count: Option<u32>,
    #[serde(default)]
    pub tome_limit: Option<u32>,
    #[serde(default)]
    pub quota_bytes: Option<u64>,
    #[serde(default)]
    pub bytes_used: Option<u64>,
}

/// Per-Tome summary returned by `GET /v1/tomes`. Cloud ships one of
/// these per Tome the account owns, newest-first.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTomeSummary {
    pub tome_uuid: String,
    pub snapshot_count: u32,
    /// ULID stem of `snapshots/<ulid>.snap.enc`, or `None` on
    /// journal-only Tomes (no snapshot taken yet).
    #[serde(default)]
    pub latest_snapshot_id: Option<String>,
    pub size_bytes: u64,
    pub last_modified_ms: i64,
}

/// `GET /api/account` response. Same shape as the signin response's
/// usage fields plus account_id + email + tier. Used for startup
/// refresh when signed in via stored device token.
#[derive(Debug, Deserialize)]
pub struct AccountInfoResponse {
    pub account_id: String,
    pub email: String,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub subscription_status: Option<String>,
    #[serde(default)]
    pub tome_count: Option<u32>,
    #[serde(default)]
    pub tome_limit: Option<u32>,
    #[serde(default)]
    pub quota_bytes: Option<u64>,
    #[serde(default)]
    pub bytes_used: Option<u64>,
}

/// Object metadata returned by PUT / HEAD / list.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjectMeta {
    pub size: u64,
    pub etag: String,
    pub last_modified: i64,
    #[serde(default)]
    pub key: Option<String>,
    /// Cloud embeds `usage` in PUT / sync-meta / DELETE responses so
    /// the app never has to poll account state. Parsed here so the
    /// HostedBackend can surface it to the runner / UI. `None` on
    /// HEAD / list (no mutation, usage unchanged).
    #[serde(default)]
    pub usage: Option<CloudUsage>,
}

/// Body shape of the DELETE 200 response (changed from 204 in cloud
/// `50bd459`). Carries `ok` + optional `already_absent` + `usage`.
#[derive(Debug, Deserialize)]
struct DeleteResponse {
    #[serde(default)]
    #[allow(dead_code)]
    ok: bool,
    #[serde(default)]
    #[allow(dead_code)]
    already_absent: bool,
    #[serde(default)]
    usage: Option<CloudUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ListResponse {
    pub objects: Vec<ObjectMeta>,
    pub truncated: bool,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

/// Hosted-cloud HTTPS client. Holds a single `reqwest::Client` for
/// connection pooling + the base URL. Bearer token is held by callers
/// (passed per-request) so the same Client can be reused across
/// signin/signout cycles without rebuilding.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new() -> Result<Self, BackendError> {
        let base_url = std::env::var("VAELORIUM_CLOUD_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent(concat!("vaelorium/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| BackendError::Other(format!("reqwest build: {e}")))?;
        Ok(Self { http, base_url })
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, BackendError> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent(concat!("vaelorium/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| BackendError::Other(format!("reqwest build: {e}")))?;
        Ok(Self {
            http,
            base_url: base_url.into(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// GET /api/auth/salt?email=...
    pub async fn get_salt(&self, email: &str) -> Result<String, BackendError> {
        let url = format!(
            "{}/api/auth/salt?email={}",
            self.base_url,
            urlencoding::encode(email)
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("salt request: {e}")))?;
        map_auth_status(resp.status())?;
        let body: SaltResponse = resp
            .json()
            .await
            .map_err(|e| BackendError::Other(format!("salt decode: {e}")))?;
        Ok(body.kdf_salt_b64)
    }

    /// POST /api/auth/signin
    pub async fn signin(
        &self,
        email: &str,
        auth_hash_b64: &str,
        device_name: &str,
    ) -> Result<SigninResponse, BackendError> {
        let url = format!("{}/api/auth/signin", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("X-Client", "app")
            .header("X-Device-Name", device_name)
            .json(&SigninRequest {
                email,
                auth_hash_b64,
            })
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("signin request: {e}")))?;
        map_auth_status(resp.status())?;
        resp.json()
            .await
            .map_err(|e| BackendError::Other(format!("signin decode: {e}")))
    }

    /// POST /api/auth/signout. Failure-tolerant — callers should clear
    /// local state regardless of whether this succeeds.
    pub async fn signout(&self, device_token: &str) -> Result<(), BackendError> {
        let url = format!("{}/api/auth/signout", self.base_url);
        let _ = self
            .http
            .post(&url)
            .bearer_auth(device_token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("signout request: {e}")))?;
        Ok(())
    }

    // ----- Storage endpoints -----

    fn object_url(&self, tome_uuid: &str, key: &str) -> String {
        // Keys may contain forward slashes (snapshots/<ulid>.snap.enc);
        // encode each path segment separately so slashes stay literal.
        let encoded: Vec<String> = key
            .split('/')
            .map(|seg| urlencoding::encode(seg).into_owned())
            .collect();
        format!(
            "{}/v1/tomes/{}/object/{}",
            self.base_url,
            tome_uuid,
            encoded.join("/")
        )
    }

    pub async fn put_object(
        &self,
        token: &str,
        tome_uuid: &str,
        key: &str,
        body: Vec<u8>,
    ) -> Result<ObjectMeta, BackendError> {
        let url = self.object_url(tome_uuid, key);
        let len = body.len();
        let resp = self
            .http
            .put(&url)
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .header(reqwest::header::CONTENT_LENGTH, len)
            .body(body)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("put_object: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            return resp
                .json::<ObjectMeta>()
                .await
                .map_err(|e| BackendError::Other(format!("put_object decode: {e}")));
        }
        Err(map_storage_error(status, resp, key).await)
    }

    pub async fn get_object(
        &self,
        token: &str,
        tome_uuid: &str,
        key: &str,
    ) -> Result<(Vec<u8>, String), BackendError> {
        let url = self.object_url(tome_uuid, key);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("get_object: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            let etag = resp
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| BackendError::Other(format!("get_object read: {e}")))?
                .to_vec();
            return Ok((bytes, etag));
        }
        Err(map_storage_error(status, resp, key).await)
    }

    pub async fn head_object(
        &self,
        token: &str,
        tome_uuid: &str,
        key: &str,
    ) -> Result<ObjectMeta, BackendError> {
        let url = self.object_url(tome_uuid, key);
        let resp = self
            .http
            .head(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("head_object: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(map_storage_error(status, resp, key).await);
        }
        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let size: u64 = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let last_modified = resp
            .headers()
            .get("X-Last-Modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        Ok(ObjectMeta {
            size,
            etag,
            last_modified,
            key: Some(key.to_string()),
            usage: None,
        })
    }

    /// DELETE /v1/tomes/<uuid>/object/<key>. Returns the embedded
    /// usage snapshot when the cloud ships it (cloud `50bd459` changed
    /// the response from 204 to 200 with a body). 404 → still treated
    /// as success (idempotent delete).
    pub async fn delete_object(
        &self,
        token: &str,
        tome_uuid: &str,
        key: &str,
    ) -> Result<Option<CloudUsage>, BackendError> {
        let url = self.object_url(tome_uuid, key);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("delete_object: {e}")))?;
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            return Err(map_storage_error(status, resp, key).await);
        }
        // 2xx success. Try to parse the body for `usage`; tolerate
        // empty / missing body (older cloud builds that still send 204
        // or newer builds on unusual paths).
        let body: Option<DeleteResponse> = resp.json().await.ok();
        Ok(body.and_then(|b| b.usage))
    }

    /// GET /v1/tomes — enumerate every Tome under the signed-in
    /// account. Ordered by `last_modified_ms` descending (newest
    /// first). `latest_snapshot_id` is `None` for journal-only Tomes
    /// (no snapshots taken yet).
    pub async fn list_account_tomes(
        &self,
        token: &str,
    ) -> Result<Vec<AccountTomeSummary>, BackendError> {
        let url = format!("{}/v1/tomes", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("list_account_tomes: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            map_auth_status(status)?;
            return Err(BackendError::Other(format!("list_account_tomes HTTP {status}")));
        }
        #[derive(Deserialize)]
        struct Body {
            tomes: Vec<AccountTomeSummary>,
        }
        let body: Body = resp
            .json()
            .await
            .map_err(|e| BackendError::Other(format!("list_account_tomes decode: {e}")))?;
        Ok(body.tomes)
    }

    /// DELETE /v1/tomes/<uuid> — wipe a whole Tome from cloud in one
    /// call. Returns `(deleted_objects, deleted_bytes, usage)`. A 404
    /// "tome not found" is treated as idempotent success: `(0, 0, None)`.
    pub async fn delete_tome(
        &self,
        token: &str,
        tome_uuid: &str,
    ) -> Result<(u64, u64, Option<CloudUsage>), BackendError> {
        let url = format!("{}/v1/tomes/{}", self.base_url, tome_uuid);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("delete_tome: {e}")))?;
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Ok((0, 0, None));
        }
        if !status.is_success() {
            return Err(map_storage_error(status, resp, tome_uuid).await);
        }
        #[derive(Deserialize)]
        struct DeleteTomeBody {
            #[serde(default)]
            deleted_objects: u64,
            #[serde(default)]
            deleted_bytes: u64,
            #[serde(default)]
            usage: Option<CloudUsage>,
        }
        let body: DeleteTomeBody = resp
            .json()
            .await
            .unwrap_or(DeleteTomeBody { deleted_objects: 0, deleted_bytes: 0, usage: None });
        Ok((body.deleted_objects, body.deleted_bytes, body.usage))
    }

    /// GET /api/account — authoritative state refresh when already
    /// signed in (read keychain/config device token, call this at app
    /// startup to pick up out-of-band changes like plan upgrades).
    pub async fn get_account(
        &self,
        token: &str,
    ) -> Result<AccountInfoResponse, BackendError> {
        let url = format!("{}/api/account", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("account request: {e}")))?;
        map_auth_status(resp.status())?;
        resp.json::<AccountInfoResponse>()
            .await
            .map_err(|e| BackendError::Other(format!("account decode: {e}")))
    }

    /// List objects under a prefix. Handles pagination transparently by
    /// following `cursor` until `truncated=false`.
    pub async fn list_prefix(
        &self,
        token: &str,
        tome_uuid: &str,
        prefix: &str,
    ) -> Result<Vec<ObjectMeta>, BackendError> {
        let mut out: Vec<ObjectMeta> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut url = format!(
                "{}/v1/tomes/{}/list?prefix={}",
                self.base_url,
                tome_uuid,
                urlencoding::encode(prefix)
            );
            if let Some(ref c) = cursor {
                url.push_str("&cursor=");
                url.push_str(&urlencoding::encode(c));
            }
            let resp = self
                .http
                .get(&url)
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| BackendError::Other(format!("list_prefix: {e}")))?;
            let status = resp.status();
            if !status.is_success() {
                return Err(map_storage_error(status, resp, prefix).await);
            }
            let page: ListResponse = resp
                .json()
                .await
                .map_err(|e| BackendError::Other(format!("list decode: {e}")))?;
            out.extend(page.objects);
            if !page.truncated {
                break;
            }
            cursor = page.cursor;
            if cursor.is_none() {
                break;
            }
        }
        Ok(out)
    }

    /// CAS update of `sync-meta.json`. `expected_etag = None` sends
    /// `If-None-Match: *` (first-write gate); `Some(etag)` sends
    /// `If-Match: <etag>` including the HTTP-standard quotes verbatim.
    pub async fn atomic_swap_meta(
        &self,
        token: &str,
        tome_uuid: &str,
        expected_etag: Option<&str>,
        body: Vec<u8>,
    ) -> Result<ObjectMeta, BackendError> {
        let url = format!("{}/v1/tomes/{}/sync-meta", self.base_url, tome_uuid);
        let len = body.len();
        let mut req = self
            .http
            .put(&url)
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::CONTENT_LENGTH, len);
        match expected_etag {
            Some(etag) => req = req.header(reqwest::header::IF_MATCH, etag),
            None => req = req.header(reqwest::header::IF_NONE_MATCH, "*"),
        }
        let resp = req
            .body(body)
            .send()
            .await
            .map_err(|e| BackendError::Other(format!("sync-meta: {e}")))?;
        let status = resp.status();
        if status.is_success() {
            return resp
                .json::<ObjectMeta>()
                .await
                .map_err(|e| BackendError::Other(format!("sync-meta decode: {e}")));
        }
        if status == StatusCode::PRECONDITION_FAILED {
            return Err(BackendError::EtagMismatch {
                key: "sync-meta.json".to_string(),
                expected: expected_etag.unwrap_or("").to_string(),
                found: resp
                    .headers()
                    .get(reqwest::header::ETAG)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        Err(map_storage_error(status, resp, "sync-meta.json").await)
    }
}

/// Map an HTTP status from an auth endpoint. Success returns Ok; 401
/// becomes `Unauthorized`, 402 becomes `PaymentRequired`, others become
/// `Other(..)`.
fn map_auth_status(status: StatusCode) -> Result<(), BackendError> {
    if status.is_success() {
        return Ok(());
    }
    match status {
        StatusCode::UNAUTHORIZED => Err(BackendError::Unauthorized("invalid_credentials".into())),
        StatusCode::PAYMENT_REQUIRED => {
            Err(BackendError::PaymentRequired("subscription not active".into()))
        }
        s => Err(BackendError::Other(format!("auth HTTP {s}"))),
    }
}

/// Map a storage-endpoint error response to a `BackendError` by reading
/// the JSON `{ error, code }` body. Falls back to status-only mapping
/// if the body can't be parsed.
async fn map_storage_error(
    status: StatusCode,
    resp: reqwest::Response,
    key_hint: &str,
) -> BackendError {
    let body: Option<ErrorBody> = resp.json::<ErrorBody>().await.ok();
    let (msg, code) = body
        .map(|b| (b.error.unwrap_or_default(), b.code.unwrap_or_default()))
        .unwrap_or_default();
    match status {
        StatusCode::NOT_FOUND => BackendError::NotFound(key_hint.to_string()),
        StatusCode::UNAUTHORIZED => BackendError::Unauthorized(msg),
        StatusCode::PAYMENT_REQUIRED => BackendError::PaymentRequired(msg),
        StatusCode::PRECONDITION_FAILED => BackendError::EtagMismatch {
            key: key_hint.to_string(),
            expected: String::new(),
            found: String::new(),
        },
        StatusCode::PAYLOAD_TOO_LARGE => BackendError::QuotaExceeded(match code.as_str() {
            "quota_exceeded" | "tome_limit_exceeded" => msg,
            _ => format!("payload too large: {msg}"),
        }),
        StatusCode::INTERNAL_SERVER_ERROR => {
            BackendError::Other(format!("cloud 500: {msg}"))
        }
        s => BackendError::Other(format!("cloud HTTP {s}: {msg} ({code})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_url_preserves_slashes_in_key() {
        let c = Client::with_base_url("https://example.com").unwrap();
        let url = c.object_url("00000000-0000-0000-0000-000000000000", "snapshots/abc.snap.enc");
        assert_eq!(
            url,
            "https://example.com/v1/tomes/00000000-0000-0000-0000-000000000000/object/snapshots/abc.snap.enc"
        );
    }

    #[test]
    fn object_url_percent_encodes_within_segments() {
        let c = Client::with_base_url("https://example.com").unwrap();
        let url = c.object_url("uuid", "snap shots/a b.enc");
        // Spaces within a segment encode to %20; the slash between segments stays.
        assert!(url.contains("/snap%20shots/a%20b.enc"), "got: {url}");
    }

    #[test]
    fn auth_401_maps_to_unauthorized() {
        let r = map_auth_status(StatusCode::UNAUTHORIZED);
        assert!(matches!(r, Err(BackendError::Unauthorized(_))));
    }

    #[test]
    fn auth_402_maps_to_payment_required() {
        let r = map_auth_status(StatusCode::PAYMENT_REQUIRED);
        assert!(matches!(r, Err(BackendError::PaymentRequired(_))));
    }

    #[test]
    fn auth_500_maps_to_other() {
        let r = map_auth_status(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(r, Err(BackendError::Other(_))));
    }

    #[test]
    fn auth_success_returns_ok() {
        assert!(map_auth_status(StatusCode::OK).is_ok());
    }
}
