//! Vaelorium Cloud (M5) — signin / signout / status Tauri commands.
//!
//! Session state lives in the OS keychain under service
//! `"vaelorium-cloud"` with four named entries (see
//! `sync::keychain::CLOUD_KEY_*`). The raw password is discarded
//! immediately after deriving auth_hash + enc_key; only the
//! server-returned device token and unwrapped account_key persist.

use crate::sync::backend::hosted::crypto;
use crate::sync::backend::hosted::HostedClient;
use crate::sync::keychain;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
pub struct CloudSigninInput {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudAccountInfo {
    pub email: String,
    pub account_id: String,
    pub tier: Option<String>,
    pub signed_in_at: Option<String>,
    /// Usage snapshot from the last signin or /api/account call.
    /// Cloud embeds usage in every mutation response too; wiring those
    /// through to live-update this field is a future enhancement.
    pub usage: Option<UsagePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePayload {
    pub bytes_used: u64,
    pub tome_count: u32,
    pub quota_bytes: u64,
    pub tome_limit: Option<u32>,
    pub subscription_status: String,
}

/// POST salt → Argon2id(auth + crypt) → POST signin → unwrap account_key
/// → stash four entries in the OS keychain. Raw password is never
/// persisted.
#[tauri::command]
pub async fn cloud_signin(
    app: AppHandle,
    input: CloudSigninInput,
) -> Result<CloudAccountInfo, String> {
    use tauri::Manager;
    let email = input.email.trim();
    if email.is_empty() {
        return Err("email is required".to_string());
    }
    if input.password.is_empty() {
        return Err("password is required".to_string());
    }

    let client = HostedClient::new().map_err(|e| e.to_string())?;

    // 1. Fetch kdf_salt for this email. Server returns a decoy for
    //    unknown emails, so we don't branch on that.
    let kdf_salt_b64 = client.get_salt(email).await.map_err(|e| match e {
        crate::sync::backend::BackendError::Unauthorized(_) => {
            "email or password is incorrect".to_string()
        }
        other => other.to_string(),
    })?;
    let kdf_salt_bytes = B64
        .decode(&kdf_salt_b64)
        .map_err(|e| format!("salt decode: {e}"))?;

    // 2. Derive locally — CPU-bound, run on the blocking pool to avoid
    //    stalling the Tokio runtime.
    let password = input.password.clone();
    let salt_clone = kdf_salt_bytes.clone();
    let auth_hash = tokio::task::spawn_blocking(move || {
        crypto::derive_auth_hash(&password, &salt_clone)
    })
    .await
    .map_err(|e| format!("kdf join: {e}"))?
    .map_err(|e| e.to_string())?;

    let password = input.password.clone();
    let salt_clone = kdf_salt_bytes.clone();
    let enc_key = tokio::task::spawn_blocking(move || {
        crypto::derive_enc_key(&password, &salt_clone)
    })
    .await
    .map_err(|e| format!("kdf join: {e}"))?
    .map_err(|e| e.to_string())?;

    // 3. POST signin with the auth_hash.
    let device_name = input.device_name.unwrap_or_else(|| {
        std::env::var("HOSTNAME").unwrap_or_else(|_| "Vaelorium Device".into())
    });
    let auth_hash_b64 = B64.encode(auth_hash);
    let resp = client
        .signin(email, &auth_hash_b64, &device_name)
        .await
        .map_err(|e| match e {
            crate::sync::backend::BackendError::Unauthorized(_) => {
                "email or password is incorrect".to_string()
            }
            other => other.to_string(),
        })?;

    // 4. Unwrap account_key with enc_key.
    let wrapped = B64
        .decode(&resp.wrapped_by_password_b64)
        .map_err(|e| format!("wrapped decode: {e}"))?;
    let account_key = crypto::unwrap_account_key(&wrapped, &enc_key)
        .map_err(|e| format!("unwrap account_key: {e}"))?;

    // 5. Stash in keychain. Any keychain failure is fatal — without
    //    the device_token we can't make authenticated calls.
    keychain::store_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN, &resp.device_token)
        .map_err(|e| format!("keychain device-token: {e}"))?;
    keychain::store_cloud(
        keychain::CLOUD_KEY_ACCOUNT_KEY,
        &B64.encode(account_key),
    )
    .map_err(|e| format!("keychain account-key: {e}"))?;
    keychain::store_cloud(keychain::CLOUD_KEY_KDF_SALT, &resp.kdf_salt_b64)
        .map_err(|e| format!("keychain kdf-salt: {e}"))?;
    keychain::store_cloud(keychain::CLOUD_KEY_EMAIL, &resp.email)
        .map_err(|e| format!("keychain email: {e}"))?;
    if let Some(ref tier) = resp.tier {
        let _ = keychain::store_cloud(keychain::CLOUD_KEY_TIER, tier);
    }
    // Stash the full usage snapshot from the signin response (cloud
    // `efc9286` embeds tier + usage in the 200 body). Kept as a
    // JSON-encoded string so the shape can evolve without keychain
    // schema churn.
    let usage = build_usage_from_signin(&resp);
    if let Some(ref u) = usage {
        if let Ok(s) = serde_json::to_string(u) {
            let _ = keychain::store_cloud(keychain::CLOUD_KEY_USAGE, &s);
        }
    }

    // Also persist the device token into sync-backend.json if the user
    // has configured hosted backup already. This gives us a second
    // durable source of truth — `require_device_token` checks the
    // keychain first, then falls back to AppBackendConfig. Belt + braces
    // for platforms where the keychain fallback file is also missing.
    if let Ok(dir) = app.path().app_data_dir() {
        if let Ok(Some(mut cfg)) = crate::sync::app_backend::load(&dir).await {
            cfg.device_token = Some(resp.device_token.clone());
            let _ = crate::sync::app_backend::save(&dir, &cfg).await;
        }
    }

    Ok(CloudAccountInfo {
        email: resp.email,
        account_id: resp.account_id,
        tier: resp.tier,
        signed_in_at: Some(chrono::Utc::now().to_rfc3339()),
        usage,
    })
}

fn build_usage_from_signin(
    resp: &crate::sync::backend::hosted::protocol::SigninResponse,
) -> Option<UsagePayload> {
    let subscription_status = resp.subscription_status.clone()?;
    Some(UsagePayload {
        bytes_used: resp.bytes_used.unwrap_or(0),
        tome_count: resp.tome_count.unwrap_or(0),
        quota_bytes: resp.quota_bytes.unwrap_or(0),
        tome_limit: resp.tome_limit,
        subscription_status,
    })
}

fn build_usage_from_account(
    resp: &crate::sync::backend::hosted::protocol::AccountInfoResponse,
) -> Option<UsagePayload> {
    let subscription_status = resp.subscription_status.clone()?;
    Some(UsagePayload {
        bytes_used: resp.bytes_used.unwrap_or(0),
        tome_count: resp.tome_count.unwrap_or(0),
        quota_bytes: resp.quota_bytes.unwrap_or(0),
        tome_limit: resp.tome_limit,
        subscription_status,
    })
}

/// POST signout (swallow failures) + clear all four keychain entries.
#[tauri::command]
pub async fn cloud_signout(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let token = keychain::retrieve_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN)
        .ok()
        .flatten()
        .or_else(|| {
            // Fall back to AppBackendConfig if the keychain has nothing
            // (e.g. mismatched / stale state). Best-effort — a signout
            // that can't reach the server should still clear local state.
            let dir = app.path().app_data_dir().ok()?;
            // Can't .await inside an Option chain — use blocking read.
            let bytes = std::fs::read(dir.join("sync-backend.json")).ok()?;
            let cfg: crate::sync::app_backend::AppBackendConfig =
                serde_json::from_slice(&bytes).ok()?;
            cfg.device_token
        });
    if let Some(ref t) = token {
        if let Ok(client) = HostedClient::new() {
            let _ = client.signout(t).await; // best-effort
        }
    }
    keychain::clear_all_cloud();
    // Also strip the token from sync-backend.json.
    if let Ok(dir) = app.path().app_data_dir() {
        if let Ok(Some(mut cfg)) = crate::sync::app_backend::load(&dir).await {
            if cfg.device_token.is_some() {
                cfg.device_token = None;
                let _ = crate::sync::app_backend::save(&dir, &cfg).await;
            }
        }
    }
    Ok(())
}

/// Read keychain, return account metadata if signed in.
#[tauri::command]
pub async fn cloud_status(_app: AppHandle) -> Result<Option<CloudAccountInfo>, String> {
    let token = match keychain::retrieve_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN) {
        Ok(Some(t)) => t,
        _ => return Ok(None),
    };
    if token.is_empty() {
        return Ok(None);
    }
    let email = keychain::retrieve_cloud(keychain::CLOUD_KEY_EMAIL)
        .ok()
        .flatten()
        .unwrap_or_default();
    let tier = keychain::retrieve_cloud(keychain::CLOUD_KEY_TIER)
        .ok()
        .flatten();
    let usage = keychain::retrieve_cloud(keychain::CLOUD_KEY_USAGE)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<UsagePayload>(&s).ok());
    Ok(Some(CloudAccountInfo {
        email,
        account_id: String::new(),
        tier,
        signed_in_at: None,
        usage,
    }))
}

/// Refresh account state from `/api/account`. Safe to call when
/// already signed in (uses the keychain device token). Updates the
/// keychain usage snapshot so subsequent `cloud_status` calls return
/// the fresh data. Per cloud `efc9286` no polling is needed outside
/// login / sync actions — this is the one-shot "something changed
/// out of band" refresh (plan upgrade, another device's usage hits).
#[tauri::command]
pub async fn cloud_account_refresh(
    _app: AppHandle,
) -> Result<Option<CloudAccountInfo>, String> {
    let token = match keychain::retrieve_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN) {
        Ok(Some(t)) if !t.is_empty() => t,
        _ => return Ok(None),
    };
    let client = HostedClient::new().map_err(|e| e.to_string())?;
    let resp = client
        .get_account(&token)
        .await
        .map_err(|e| e.to_string())?;

    // Update cached tier + usage from the fresh server response.
    if let Some(ref tier) = resp.tier {
        let _ = keychain::store_cloud(keychain::CLOUD_KEY_TIER, tier);
    }
    let usage = build_usage_from_account(&resp);
    if let Some(ref u) = usage {
        if let Ok(s) = serde_json::to_string(u) {
            let _ = keychain::store_cloud(keychain::CLOUD_KEY_USAGE, &s);
        }
    }
    Ok(Some(CloudAccountInfo {
        email: resp.email,
        account_id: resp.account_id,
        tier: resp.tier,
        signed_in_at: None,
        usage,
    }))
}

/// Internal helper: return a device token for authenticated requests.
/// Checks in order: OS keychain (primary), file-backed keychain
/// fallback (for platforms without a working secure store, handled
/// transparently by `sync::keychain`), then AppBackendConfig as a
/// last-resort source. Missing from all three = user must sign in.
pub fn require_device_token_with_app(app: &AppHandle) -> Result<String, String> {
    use tauri::Manager;
    if let Ok(Some(t)) = keychain::retrieve_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN) {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    // Fall back to AppBackendConfig (sync-backend.json) — durable across
    // restarts on every platform regardless of keychain availability.
    if let Ok(dir) = app.path().app_data_dir() {
        let path = dir.join("sync-backend.json");
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(cfg) =
                serde_json::from_slice::<crate::sync::app_backend::AppBackendConfig>(&bytes)
            {
                if let Some(t) = cfg.device_token.filter(|s| !s.is_empty()) {
                    return Ok(t);
                }
            }
        }
    }
    Err("not signed in to Vaelorium Cloud — sign in from Settings → Backup".to_string())
}

/// Thin shim that doesn't need an AppHandle — kept for call sites that
/// don't plumb it through yet. Consults keychain only. Prefer
/// `require_device_token_with_app` where the handle is available.
#[allow(dead_code)]
pub fn require_device_token() -> Result<String, String> {
    match keychain::retrieve_cloud(keychain::CLOUD_KEY_DEVICE_TOKEN) {
        Ok(Some(t)) if !t.is_empty() => Ok(t),
        _ => Err(
            "not signed in to Vaelorium Cloud — sign in from Settings → Backup".to_string(),
        ),
    }
}
