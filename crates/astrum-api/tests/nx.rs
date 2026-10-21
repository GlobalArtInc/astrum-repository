mod support;

use std::sync::Arc;

use astrum_security::Scope;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "nx-cache";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_nx::Nx));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/nx/hosted",
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

async fn token(fixture: &Fixture, scope: Scope) -> String {
    fixture
        .state
        .security()
        .create_token("admin", "nx", scope, None)
        .await
        .unwrap()
        .1
}

async fn nx(
    fixture: &Fixture,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Vec<u8>,
) -> Reply {
    let authorization = token.map(|token| format!("Bearer {token}"));
    let mut headers = vec![
        ("content-type", "application/octet-stream"),
        ("accept", "application/octet-stream"),
    ];
    if let Some(value) = authorization.as_deref() {
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
async fn a_task_output_goes_up_once_and_comes_back() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    let output = b"\x1f\x8b nx task output".to_vec();
    let stored = nx(
        &fixture,
        Method::PUT,
        "/v1/cache/9876543210123456789",
        Some(&writer),
        output.clone(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());

    let again = nx(
        &fixture,
        Method::PUT,
        "/v1/cache/9876543210123456789",
        Some(&writer),
        b"other".to_vec(),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);

    let fetched = nx(
        &fixture,
        Method::GET,
        "/v1/cache/9876543210123456789",
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, output);
    assert_eq!(
        fetched
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/octet-stream")
    );
    let checked = nx(
        &fixture,
        Method::HEAD,
        "/v1/cache/9876543210123456789",
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK);

    let missing = nx(
        &fixture,
        Method::GET,
        "/v1/cache/1111",
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let browsed = fixture
        .call(
            Method::GET,
            &format!("/repository/{REPOSITORY}/9876543210123456789"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(browsed.status, StatusCode::OK);
    assert_eq!(browsed.body, output);
}

#[tokio::test]
async fn reading_tokens_and_anonymous_callers_are_refused_as_nx_expects() {
    let fixture = ready().await;
    let reader = token(&fixture, Scope::Read).await;
    let refused = nx(
        &fixture,
        Method::PUT,
        "/v1/cache/42",
        Some(&reader),
        b"x".to_vec(),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    let anonymous = nx(&fixture, Method::GET, "/v1/cache/42", None, vec![]).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let read = nx(&fixture, Method::GET, "/v1/cache/42", Some(&reader), vec![]).await;
    assert_eq!(read.status, StatusCode::NOT_FOUND);
    for path in ["/v1/cache/a.b", "/v1/cache/a/b", "/elsewhere"] {
        let wrong = nx(&fixture, Method::GET, path, Some(&reader), vec![]).await;
        assert_eq!(wrong.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn an_output_can_be_deleted_and_uploaded_again() {
    let fixture = ready().await;
    let writer = token(&fixture, Scope::Write).await;
    let first = nx(
        &fixture,
        Method::PUT,
        "/v1/cache/77",
        Some(&writer),
        b"one".to_vec(),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let deleted = nx(
        &fixture,
        Method::DELETE,
        "/v1/cache/77",
        Some(&writer),
        vec![],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let gone = nx(&fixture, Method::GET, "/v1/cache/77", Some(&writer), vec![]).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let second = nx(
        &fixture,
        Method::PUT,
        "/v1/cache/77",
        Some(&writer),
        b"two".to_vec(),
    )
    .await;
    assert_eq!(second.status, StatusCode::OK);
    let fetched = nx(&fixture, Method::GET, "/v1/cache/77", Some(&writer), vec![]).await;
    assert_eq!(fetched.body, b"two");
}
