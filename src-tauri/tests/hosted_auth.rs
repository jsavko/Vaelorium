//! M5.2 — wiremock-based tests for the signin flow protocol layer.
//!
//! These exercise `HostedClient::get_salt` + `HostedClient::signin`
//! directly (the Tauri command wrapper adds keychain writes which can't
//! be mocked in pure cargo tests). The same endpoints + shapes are what
//! `cloud_signin` hits in production.

use vaelorium_lib::sync::backend::hosted::HostedClient;
use vaelorium_lib::sync::backend::BackendError;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn get_salt_returns_b64() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/auth/salt"))
        .and(query_param("email", "a@b.co"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "kdf_salt_b64": "AAAAAAAAAAAAAAAAAAAAAA=="
        })))
        .mount(&server)
        .await;
    let client = HostedClient::with_base_url(server.uri()).unwrap();
    let salt = client.get_salt("a@b.co").await.expect("salt fetch");
    assert_eq!(salt, "AAAAAAAAAAAAAAAAAAAAAA==");
}

#[tokio::test]
async fn signin_happy_path_returns_token_and_wrapped_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/signin"))
        .and(header("X-Client", "app"))
        .and(header("X-Device-Name", "Test Laptop"))
        .and(body_json(serde_json::json!({
            "email": "a@b.co",
            "auth_hash_b64": "dGVzdA=="
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account_id": "acc-123",
            "email": "a@b.co",
            "device_token": "vld_abcdef",
            "kdf_salt_b64": "AAAAAAAAAAAAAAAAAAAAAA==",
            "wrapped_by_password_b64": "d3JhcHBlZA==",
            "tier": "hobbyist"
        })))
        .mount(&server)
        .await;
    let client = HostedClient::with_base_url(server.uri()).unwrap();
    let resp = client
        .signin("a@b.co", "dGVzdA==", "Test Laptop")
        .await
        .expect("signin");
    assert_eq!(resp.account_id, "acc-123");
    assert_eq!(resp.device_token, "vld_abcdef");
    assert_eq!(resp.tier.as_deref(), Some("hobbyist"));
}

#[tokio::test]
async fn signin_401_maps_to_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/signin"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "email or password is incorrect",
            "code": "invalid_credentials"
        })))
        .mount(&server)
        .await;
    let client = HostedClient::with_base_url(server.uri()).unwrap();
    let err = client
        .signin("wrong@b.co", "badhash", "dev")
        .await
        .unwrap_err();
    assert!(matches!(err, BackendError::Unauthorized(_)), "got: {err:?}");
}

#[tokio::test]
async fn signout_swallows_non_2xx_gracefully() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/signout"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let client = HostedClient::with_base_url(server.uri()).unwrap();
    // Even when the server rejects the token, signout returns Ok —
    // callers must clear local state regardless of server outcome.
    client.signout("vld_stale").await.expect("signout never errors");
}
