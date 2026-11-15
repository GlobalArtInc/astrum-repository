mod support;

use std::sync::Arc;

use astrum_security::{NewToken, Scope};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "gradle-cache";
const KEY: &str = "9876543210abcdef";
const ARTIFACT: &str = "application/vnd.gradle.build-cache-artifact.v2";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_gradlecache::GradleCache));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/gradlecache/hosted",
            Auth::Admin,
            Some(json!({
                "name": REPOSITORY,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "allow"},
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    fixture
}

async fn credentials(fixture: &Fixture, scope: Scope) -> String {
    let token = fixture
        .state
        .security()
        .create_token("admin", NewToken::personal("gradle", scope, None))
        .await
        .unwrap()
        .1;
    format!("Basic {}", STANDARD.encode(format!("admin:{token}")))
}

async fn gradle(
    fixture: &Fixture,
    method: Method,
    path: &str,
    authorization: Option<&str>,
    body: Vec<u8>,
) -> Reply {
    let mut headers = vec![("content-type", ARTIFACT)];
    if let Some(value) = authorization {
        headers.push(("authorization", value));
    }
    fixture
        .send_body(
            method,
            &format!("/repository/{REPOSITORY}{path}"),
            Auth::None,
            Body::from(body),
            &headers,
        )
        .await
}

#[tokio::test]
async fn an_entry_goes_up_and_comes_back() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let entry = b"gradle build cache entry".to_vec();
    let stored = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        entry.clone(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());

    let fetched = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, entry);

    let checked = gradle(
        &fixture,
        Method::HEAD,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK);

    let missing = gradle(
        &fixture,
        Method::GET,
        "/0123456789abcdef",
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_second_upload_answers_ok_and_keeps_the_first_entry() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"first".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let again = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"second".to_vec(),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let fetched = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"first");
}

#[tokio::test]
async fn an_entry_is_served_as_octet_stream_whatever_its_bytes() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"<html><script>alert(1)</script></html>".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(
        fetched.headers[header::CONTENT_TYPE],
        "application/octet-stream"
    );
}

#[tokio::test]
async fn keys_are_stored_in_lower_case() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = gradle(
        &fixture,
        Method::PUT,
        &format!("/{}", KEY.to_uppercase()),
        Some(&writer),
        b"upper".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let fetched = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
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
    let stored = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"entry".to_vec(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK);
    let restored = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&reader),
        vec![],
    )
    .await;
    assert_eq!(restored.status, StatusCode::OK);
    let refused = gradle(
        &fixture,
        Method::PUT,
        "/1111222233334444",
        Some(&reader),
        b"x".to_vec(),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
}

#[tokio::test]
async fn anonymous_callers_are_challenged() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"entry".to_vec(),
    )
    .await;
    let anonymous = gradle(&fixture, Method::GET, &format!("/{KEY}"), None, vec![]).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn keys_that_are_not_hexadecimal_or_too_short_are_refused() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    for path in ["/abc", "/0123456789abcdeg"] {
        let refused = gradle(&fixture, Method::PUT, path, Some(&writer), b"x".to_vec()).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{path}");
        let missing = gradle(&fixture, Method::GET, path, Some(&writer), vec![]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }
    let traversal = gradle(&fixture, Method::PUT, "/../x", Some(&writer), b"x".to_vec()).await;
    assert_eq!(traversal.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_entry_is_listed_as_a_component() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
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
    assert_eq!(listed.json()["items"][0]["name"], KEY);
}

#[tokio::test]
async fn an_entry_can_be_deleted_and_uploaded_again() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"one".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let deleted = gradle(
        &fixture,
        Method::DELETE,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let gone = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let second = gradle(
        &fixture,
        Method::PUT,
        &format!("/{KEY}"),
        Some(&writer),
        b"two".to_vec(),
    )
    .await;
    assert_eq!(second.status, StatusCode::OK);
    let fetched = gradle(
        &fixture,
        Method::GET,
        &format!("/{KEY}"),
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.body, b"two");
}
