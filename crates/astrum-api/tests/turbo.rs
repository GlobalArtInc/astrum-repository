mod support;

use std::sync::Arc;

use astrum_security::Scope;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "turbo-cache";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_turbo::Turbo));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/turbo/hosted",
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
        .create_token("admin", "turbo", scope, None)
        .await
        .unwrap()
        .1
}

async fn turbo(
    fixture: &Fixture,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    let authorization = token.map(|token| format!("Bearer {token}"));
    let mut all = headers.to_vec();
    if let Some(value) = authorization.as_deref() {
        all.push(("authorization", value));
    }
    fixture
        .send_body(
            method,
            &format!("/repository/{REPOSITORY}{path}"),
            Auth::None,
            Body::from(body),
            &all,
        )
        .await
}

fn header<'a>(reply: &'a Reply, name: &str) -> Option<&'a str> {
    reply
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn the_cache_reports_its_status_to_signed_in_clients() {
    let fixture = ready().await;
    let token = token(&fixture, Scope::Read).await;
    let status = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/status",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(status.status, StatusCode::OK, "{}", status.text());
    assert_eq!(status.json(), json!({ "status": "enabled" }));
    let unversioned = turbo(
        &fixture,
        Method::GET,
        "/artifacts/status",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(unversioned.status, StatusCode::OK);
    let anonymous = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/status",
        None,
        vec![],
        &[],
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_artifact_goes_up_and_comes_back_with_its_metadata() {
    let fixture = ready().await;
    let token = token(&fixture, Scope::Write).await;
    let artifact = b"\x1f\x8b turbo artifact bytes".to_vec();
    let uploaded = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/ABC123?slug=web",
        Some(&token),
        artifact.clone(),
        &[
            ("content-type", "application/octet-stream"),
            ("x-artifact-duration", "1500"),
            ("x-artifact-tag", "signature-1"),
            ("x-artifact-sha", "0123abcd"),
        ],
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::ACCEPTED, "{}", uploaded.text());
    assert_eq!(
        uploaded.json(),
        json!({ "urls": [format!("http://localhost/repository/{REPOSITORY}/v8/artifacts/abc123")] })
    );

    let checked = turbo(
        &fixture,
        Method::HEAD,
        "/v8/artifacts/abc123?slug=web",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK);
    assert_eq!(header(&checked, "x-artifact-duration"), Some("1500"));
    assert_eq!(header(&checked, "x-artifact-sha"), Some("0123abcd"));
    assert!(checked.body.is_empty());

    let fetched = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/abc123?slug=web",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, artifact);
    assert_eq!(header(&fetched, "x-artifact-tag"), Some("signature-1"));
    assert_eq!(header(&fetched, "x-artifact-duration"), Some("1500"));
    assert_eq!(
        header(&fetched, "content-type"),
        Some("application/octet-stream")
    );

    for path in [
        "/v8/artifacts/abc123?slug=other",
        "/v8/artifacts/abc123",
        "/v8/artifacts/ffff?slug=web",
    ] {
        let missing = turbo(&fixture, Method::GET, path, Some(&token), vec![], &[]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }

    let browsed = fixture
        .call(
            Method::GET,
            &format!("/repository/{REPOSITORY}/web/abc123"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(browsed.status, StatusCode::OK);
    assert_eq!(browsed.body, artifact);
}

#[tokio::test]
async fn teams_keep_their_artifacts_apart_and_the_query_answers_per_hash() {
    let fixture = ready().await;
    let token = token(&fixture, Scope::Write).await;
    for (team, body, duration) in [("web", "web build", "10"), ("docs", "docs build", "20")] {
        let reply = turbo(
            &fixture,
            Method::PUT,
            &format!("/v8/artifacts/beef?slug={team}"),
            Some(&token),
            body.as_bytes().to_vec(),
            &[("x-artifact-duration", duration)],
        )
        .await;
        assert_eq!(reply.status, StatusCode::ACCEPTED, "{}", reply.text());
    }
    let web = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/beef?slug=web",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(web.body, b"web build");
    let docs = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/beef?teamId=docs",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(docs.body, b"docs build");

    let query = serde_json::to_vec(&json!({ "hashes": ["beef", "cafe", "not-hex"] })).unwrap();
    let answered = turbo(
        &fixture,
        Method::POST,
        "/v8/artifacts?slug=docs",
        Some(&token),
        query,
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(answered.status, StatusCode::OK, "{}", answered.text());
    let answer = answered.json();
    assert_eq!(answer["beef"], json!({ "size": 10, "taskDurationMs": 20 }));
    assert_eq!(answer["cafe"], Value::Null);
    assert!(answer["not-hex"]["error"]["message"].is_string());
}

#[tokio::test]
async fn reading_tokens_cannot_upload_and_bad_input_is_refused() {
    let fixture = ready().await;
    let reader = token(&fixture, Scope::Read).await;
    let refused = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/abcd",
        Some(&reader),
        b"x".to_vec(),
        &[],
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    assert_eq!(refused.json()["error"]["code"], "forbidden");
    assert!(refused.json()["error"]["message"].is_string());
    let anonymous = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/abcd",
        None,
        b"x".to_vec(),
        &[],
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let writer = token(&fixture, Scope::Write).await;
    let team = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/abcd?slug=a%2Fb",
        Some(&writer),
        b"x".to_vec(),
        &[],
    )
    .await;
    assert_eq!(team.status, StatusCode::BAD_REQUEST);
    let duration = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/abcd",
        Some(&writer),
        b"x".to_vec(),
        &[("x-artifact-duration", "later")],
    )
    .await;
    assert_eq!(duration.status, StatusCode::BAD_REQUEST);
    let hash = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/zzzz",
        Some(&writer),
        vec![],
        &[],
    )
    .await;
    assert_eq!(hash.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn events_are_accepted_and_preflight_allows_the_token() {
    let fixture = ready().await;
    let token = token(&fixture, Scope::Write).await;
    let events = serde_json::to_vec(&json!([
        { "sessionId": "8b3f5e4e-1c1e-4b55-9b55-0f5e3d6c2a11", "source": "REMOTE", "event": "HIT", "hash": "beef", "duration": 5 }
    ]))
    .unwrap();
    let recorded = turbo(
        &fixture,
        Method::POST,
        "/v8/artifacts/events",
        Some(&token),
        events,
        &[],
    )
    .await;
    assert_eq!(recorded.status, StatusCode::OK, "{}", recorded.text());
    let broken = turbo(
        &fixture,
        Method::POST,
        "/v8/artifacts/events",
        Some(&token),
        b"{".to_vec(),
        &[],
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);

    let preflight = turbo(
        &fixture,
        Method::OPTIONS,
        "/v8/artifacts/beef?slug=web",
        None,
        vec![],
        &[
            ("access-control-request-method", "PUT"),
            (
                "access-control-request-headers",
                "Authorization, Content-Type, x-artifact-duration",
            ),
        ],
    )
    .await;
    assert_eq!(preflight.status, StatusCode::OK);
    assert_eq!(
        header(&preflight, "access-control-allow-headers"),
        Some("Authorization, Content-Type, x-artifact-duration")
    );
    assert!(
        header(&preflight, "access-control-allow-methods")
            .unwrap()
            .contains("PUT")
    );
}

#[tokio::test]
async fn an_artifact_can_be_deleted() {
    let fixture = ready().await;
    let token = token(&fixture, Scope::Write).await;
    let uploaded = turbo(
        &fixture,
        Method::PUT,
        "/v8/artifacts/0a0b",
        Some(&token),
        b"gone".to_vec(),
        &[],
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::ACCEPTED);
    let deleted = turbo(
        &fixture,
        Method::DELETE,
        "/v8/artifacts/0a0b",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let missing = turbo(
        &fixture,
        Method::GET,
        "/v8/artifacts/0a0b",
        Some(&token),
        vec![],
        &[],
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}
