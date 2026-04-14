//! M5.1 — Wiremock-based integration test for `HostedBackend`.
//!
//! Exercises the 6-method SyncBackend trait against a local HTTP server
//! returning canned responses matching the cloud protocol in
//! `~/Projects/vaelorium-cloud/docs/m5-app-integration-brief.md`.

use vaelorium_lib::sync::backend::hosted::{HostedBackend, HostedClient};
use vaelorium_lib::sync::backend::{BackendError, SyncBackend};
use wiremock::matchers::{bearer_token, header, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "vld_test_token";
const UUID: &str = "00000000-0000-0000-0000-000000000000";

async fn server_and_backend() -> (MockServer, HostedBackend) {
    let server = MockServer::start().await;
    let client = HostedClient::with_base_url(server.uri()).unwrap();
    let backend = HostedBackend::new(client, UUID.to_string(), TOKEN.to_string());
    (server, backend)
}

#[tokio::test]
async fn put_object_returns_etag_from_response_body() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("PUT"))
        .and(path(format!("/v1/tomes/{UUID}/object/journal/abc.op.enc")))
        .and(bearer_token(TOKEN))
        .and(header("Content-Type", "application/octet-stream"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "size": 42,
            "etag": "\"abcdef0123\"",
            "last_modified": 1_700_000_000_000_i64
        })))
        .mount(&server)
        .await;

    let etag = backend
        .put_object("journal/abc.op.enc", b"hello world")
        .await
        .expect("put succeeds");
    assert_eq!(etag, "\"abcdef0123\"");
}

#[tokio::test]
async fn get_object_returns_body_and_etag() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/tomes/{UUID}/object/snapshots/s1.snap.enc")))
        .and(bearer_token(TOKEN))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"ciphertext".as_slice())
                .insert_header("ETag", "\"xyz\"")
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .mount(&server)
        .await;

    let (bytes, etag) = backend
        .get_object("snapshots/s1.snap.enc")
        .await
        .expect("get succeeds");
    assert_eq!(bytes, b"ciphertext");
    assert_eq!(etag, "\"xyz\"");
}

#[tokio::test]
async fn get_object_404_maps_to_not_found() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("GET"))
        .and(path_regex(format!("^/v1/tomes/{UUID}/object/.*$")))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": "object does not exist",
            "code": "not_found"
        })))
        .mount(&server)
        .await;

    let err = backend.get_object("does/not/exist.enc").await.unwrap_err();
    assert!(matches!(err, BackendError::NotFound(_)), "got: {err:?}");
}

#[tokio::test]
async fn delete_object_accepts_404_as_success() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("DELETE"))
        .and(path_regex(format!("^/v1/tomes/{UUID}/object/.*$")))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    backend.delete_object("gone.enc").await.expect("404 is idempotent success");
}

#[tokio::test]
async fn list_prefix_follows_pagination_cursor() {
    let (server, backend) = server_and_backend().await;

    // Page 1: truncated with cursor.
    Mock::given(method("GET"))
        .and(path(format!("/v1/tomes/{UUID}/list")))
        .and(query_param("prefix", "journal"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "objects": [{"key": "journal/b", "size": 2, "etag": "\"b\"", "last_modified": 2000}],
            "truncated": false,
            "cursor": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/tomes/{UUID}/list")))
        .and(query_param("prefix", "journal"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "objects": [{"key": "journal/a", "size": 1, "etag": "\"a\"", "last_modified": 1000}],
            "truncated": true,
            "cursor": "page2"
        })))
        .mount(&server)
        .await;

    let infos = backend.list_prefix("journal").await.expect("list succeeds");
    assert_eq!(infos.len(), 2);
    let keys: Vec<&str> = infos.iter().map(|i| i.key.as_str()).collect();
    assert!(keys.contains(&"journal/a"));
    assert!(keys.contains(&"journal/b"));
}

#[tokio::test]
async fn atomic_swap_precondition_failed_maps_to_etag_mismatch() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("PUT"))
        .and(path(format!("/v1/tomes/{UUID}/sync-meta")))
        .and(header("If-Match", "\"old\""))
        .respond_with(
            ResponseTemplate::new(412)
                .set_body_json(serde_json::json!({
                    "error": "etag mismatch",
                    "code": "precondition_failed"
                }))
                .insert_header("ETag", "\"current\""),
        )
        .mount(&server)
        .await;

    let err = backend
        .atomic_swap("sync-meta.json", Some("\"old\""), b"new-body")
        .await
        .expect_err("CAS should fail");
    match err {
        BackendError::EtagMismatch { key, expected, found } => {
            assert_eq!(key, "sync-meta.json");
            assert_eq!(expected, "\"old\"");
            assert_eq!(found, "\"current\"");
        }
        other => panic!("expected EtagMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn atomic_swap_first_write_sends_if_none_match_star() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("PUT"))
        .and(path(format!("/v1/tomes/{UUID}/sync-meta")))
        .and(header("If-None-Match", "*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "size": 10,
            "etag": "\"fresh\"",
            "last_modified": 3000
        })))
        .mount(&server)
        .await;

    let etag = backend
        .atomic_swap("sync-meta.json", None, b"body")
        .await
        .expect("first-write CAS succeeds");
    assert_eq!(etag, "\"fresh\"");
}

#[tokio::test]
async fn put_object_401_maps_to_unauthorized() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("PUT"))
        .and(path_regex(format!("^/v1/tomes/{UUID}/object/.*$")))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "token revoked",
            "code": "unauthenticated"
        })))
        .mount(&server)
        .await;

    let err = backend.put_object("journal/x.op.enc", b"data").await.unwrap_err();
    assert!(matches!(err, BackendError::Unauthorized(_)), "got: {err:?}");
}

#[tokio::test]
async fn put_object_413_maps_to_quota_exceeded() {
    let (server, backend) = server_and_backend().await;

    Mock::given(method("PUT"))
        .and(path_regex(format!("^/v1/tomes/{UUID}/object/.*$")))
        .respond_with(ResponseTemplate::new(413).set_body_json(serde_json::json!({
            "error": "Account over 1 GB quota",
            "code": "quota_exceeded"
        })))
        .mount(&server)
        .await;

    let err = backend.put_object("snapshots/huge.snap.enc", b"...").await.unwrap_err();
    assert!(matches!(err, BackendError::QuotaExceeded(_)), "got: {err:?}");
}

#[tokio::test]
async fn atomic_swap_rejects_non_meta_key() {
    let (_server, backend) = server_and_backend().await;
    let err = backend
        .atomic_swap("snapshots/bad.snap.enc", None, b"x")
        .await
        .expect_err("hosted only supports CAS on sync-meta");
    assert!(matches!(err, BackendError::Other(_)));
}
