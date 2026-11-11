mod support;

use std::sync::Arc;

use astrum_security::Scope;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use serde_json::json;
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "bazel-cache";
const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BLOB: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
const OCTET_STREAM: &str = "application/octet-stream";

async fn booted() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_bazelcache::BazelCache));
    fixture
}

async fn repository(fixture: &Fixture, name: &str, write_policy: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/bazelcache/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": write_policy},
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn ready() -> Fixture {
    let fixture = booted().await;
    repository(&fixture, REPOSITORY, "allow").await;
    fixture
}

async fn credentials(fixture: &Fixture, scope: Scope) -> String {
    let token = fixture
        .state
        .security()
        .create_token("admin", "bazel", scope, None)
        .await
        .unwrap()
        .1;
    format!("Bearer {token}")
}

async fn bazel(
    fixture: &Fixture,
    method: Method,
    path: &str,
    authorization: Option<&str>,
    body: Vec<u8>,
) -> Reply {
    bazel_in(fixture, REPOSITORY, method, path, authorization, body).await
}

async fn bazel_in(
    fixture: &Fixture,
    repository: &str,
    method: Method,
    path: &str,
    authorization: Option<&str>,
    body: Vec<u8>,
) -> Reply {
    let mut headers = vec![("content-type", OCTET_STREAM)];
    if let Some(value) = authorization {
        headers.push(("authorization", value));
    }
    fixture
        .send_body(
            method,
            &format!("/repository/{repository}{path}"),
            Auth::None,
            Body::from(body),
            &headers,
        )
        .await
}

#[tokio::test]
async fn a_miss_answers_not_found_in_both_namespaces() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    for namespace in ["ac", "cas"] {
        let missing = bazel(
            &fixture,
            Method::GET,
            &format!("/{namespace}/{KEY}"),
            Some(&writer),
            vec![],
        )
        .await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{namespace}");
    }
}

#[tokio::test]
async fn an_action_result_goes_up_and_comes_back() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let entry = b"opaque action result".to_vec();
    let stored = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        entry.clone(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, entry);
}

#[tokio::test]
async fn a_blob_goes_up_and_comes_back_when_its_sha256_is_its_key() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = bazel(
        &fixture,
        Method::PUT,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        b"test".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, b"test");
}

#[tokio::test]
async fn a_blob_whose_sha256_is_not_its_key_is_refused_and_not_kept() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let refused = bazel(
        &fixture,
        Method::PUT,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        b"other".to_vec(),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    let missing = bazel(
        &fixture,
        Method::GET,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_second_cas_upload_answers_ok_and_keeps_the_first_bytes() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = bazel(
        &fixture,
        Method::PUT,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        b"test".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let again = bazel(
        &fixture,
        Method::PUT,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        b"other".to_vec(),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/cas/{BLOB}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"test");
}

#[tokio::test]
async fn a_second_action_result_replaces_the_first_when_writes_are_allowed() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"first".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let again = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"second".to_vec(),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"second");
}

#[tokio::test]
async fn a_second_action_result_keeps_the_first_when_writes_are_allowed_once() {
    let fixture = booted().await;
    repository(&fixture, "bazel-cache-once", "allow_once").await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = bazel_in(
        &fixture,
        "bazel-cache-once",
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"first".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let again = bazel_in(
        &fixture,
        "bazel-cache-once",
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"second".to_vec(),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let fetched = bazel_in(
        &fixture,
        "bazel-cache-once",
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"first");
}

#[tokio::test]
async fn an_action_result_is_served_as_octet_stream_whatever_its_bytes() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"<html><script>alert(1)</script></html>".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.headers[header::CONTENT_TYPE], OCTET_STREAM);
}

#[tokio::test]
async fn keys_are_stored_in_lower_case() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{}", KEY.to_uppercase()),
        Some(&writer),
        b"upper".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"upper");
}

#[tokio::test]
async fn a_read_token_restores_but_cannot_upload() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let reader = credentials(&fixture, Scope::Read).await;
    let stored = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"entry".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK);
    let restored = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&reader),
        vec![],
    )
    .await;
    assert_eq!(restored.status, StatusCode::OK);
    let refused = bazel(
        &fixture,
        Method::PUT,
        &format!("/cas/{BLOB}"),
        Some(&reader),
        b"test".to_vec(),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
}

#[tokio::test]
async fn anonymous_callers_are_challenged() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"entry".to_vec(),
    )
    .await;
    let anonymous = bazel(&fixture, Method::GET, &format!("/ac/{KEY}"), None, vec![]).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn keys_that_are_not_sha256_digests_are_refused() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let short = &KEY[1..];
    let long = format!("{KEY}0");
    let not_hex = format!("{}g", &KEY[1..]);
    for key in [short, long.as_str(), not_hex.as_str()] {
        let path = format!("/ac/{key}");
        let refused = bazel(&fixture, Method::PUT, &path, Some(&writer), b"x".to_vec()).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{path}");
        let missing = bazel(&fixture, Method::GET, &path, Some(&writer), vec![]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn an_entry_is_listed_as_a_component() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"entry".to_vec(),
    )
    .await;
    let listed = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/components?repository={REPOSITORY}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(listed.json()["items"][0]["name"], format!("ac/{KEY}"));
}

#[tokio::test]
async fn an_entry_can_be_deleted_and_uploaded_again() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"one".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let deleted = bazel(
        &fixture,
        Method::DELETE,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let gone = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let second = bazel(
        &fixture,
        Method::PUT,
        &format!("/ac/{KEY}"),
        Some(&writer),
        b"two".to_vec(),
    )
    .await;
    assert_eq!(second.status, StatusCode::OK);
    let fetched = bazel(
        &fixture,
        Method::GET,
        &format!("/ac/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"two");
}

#[tokio::test]
async fn an_unknown_namespace_is_not_found() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let path = format!("/other/{KEY}");
    let fetched = bazel(&fixture, Method::GET, &path, Some(&writer), vec![]).await;
    assert_eq!(fetched.status, StatusCode::NOT_FOUND);
    let stored = bazel(&fixture, Method::PUT, &path, Some(&writer), b"x".to_vec()).await;
    assert_eq!(stored.status, StatusCode::NOT_FOUND, "{}", stored.text());
}
