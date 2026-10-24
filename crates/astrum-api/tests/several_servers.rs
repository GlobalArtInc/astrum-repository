mod accounts;
mod support;

use std::sync::Arc;
use std::time::Duration;

use accounts::{ROLES, USERS, create_role, create_user};
use astrum_api::{AppState, revisions, router};
use astrum_security::SecretBox;
use astrum_security::catalogue::ADMIN_USER;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const BLOBSTORES: &str = "/service/rest/v1/blobstores";
const SESSION: &str = "/service/rest/internal/session";
const ACCOUNT: &str = "/service/rest/internal/ui/user";
const PASSWORD: &str = "/service/rest/internal/ui/user/password";
const POLL: Duration = Duration::from_millis(50);
const PATIENCE: Duration = Duration::from_secs(10);

async fn second_server(fixture: &Fixture) -> AppState {
    AppState::load(
        fixture.test.connect().await,
        fixture.data.path().to_owned(),
        SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap()
}

fn basic(user: &str, password: &str) -> (&'static str, String) {
    let encoded = STANDARD.encode(format!("{user}:{password}"));
    ("authorization", format!("Basic {encoded}"))
}

async fn on(
    state: &AppState,
    method: Method,
    path: &str,
    headers: &[(&str, String)],
    body: Option<Value>,
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let body = match body {
        Some(value) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).unwrap())
        }
        None => Body::empty(),
    };
    let response = router(state.clone())
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Reply {
        status,
        headers,
        body,
    }
}

async fn eventually(what: &str, mut check: impl AsyncFnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + PATIENCE;
    while !check().await {
        assert!(
            tokio::time::Instant::now() < deadline,
            "{what} did not reach the other server"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn cookie_header(reply: &Reply) -> (String, String) {
    let mut pairs = Vec::new();
    let mut csrf = String::new();
    for value in reply.headers.get_all(header::SET_COOKIE) {
        let pair = value
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        if let Some(token) = pair.strip_prefix("astrum_csrf=") {
            csrf = token.to_owned();
        }
        pairs.push(pair);
    }
    assert_eq!(pairs.len(), 2, "{pairs:?}");
    (pairs.join("; "), csrf)
}

fn password_hash(state: &AppState, user_id: &str) -> Option<String> {
    state
        .security()
        .model()
        .users
        .get(user_id)
        .and_then(|user| user.password_hash.clone())
}

#[tokio::test]
async fn security_changes_reach_the_other_server_within_the_poll_interval() {
    let fixture = Fixture::new().await;
    let other = second_server(&fixture).await;
    let pollers = [
        revisions::spawn(fixture.state.clone(), POLL),
        revisions::spawn(other.clone(), POLL),
    ];
    let admin = [basic(ADMIN_USER, &fixture.admin_password)];

    create_role(&fixture, "readers", &["as-search-read"], &[]).await;
    create_user(&fixture, "erin", "first-secret-1", &["readers"]).await;
    eventually("a new user", async || {
        let reply = on(
            &other,
            Method::GET,
            &format!("{USERS}?userId=erin"),
            &admin,
            None,
        )
        .await;
        reply
            .json()
            .as_array()
            .is_some_and(|users| users.len() == 1)
    })
    .await;
    let erin = [basic("erin", "first-secret-1")];
    let refused = on(&other, Method::GET, USERS, &erin, None).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/readers"),
            Auth::Admin,
            Some(json!({
                "id": "readers",
                "name": "readers",
                "privileges": ["as-search-read", "as-users-read"],
                "roles": [],
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    eventually("an edited role", async || {
        on(&other, Method::GET, USERS, &erin, None).await.status == StatusCode::OK
    })
    .await;
    let role = on(
        &other,
        Method::GET,
        &format!("{ROLES}/readers"),
        &admin,
        None,
    )
    .await;
    assert_eq!(
        role.json()["privileges"],
        json!(["as-search-read", "as-users-read"])
    );

    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("{USERS}/erin/change-password"),
            Auth::Admin,
            Body::from("second-secret-2"),
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let changed = password_hash(&fixture.state, "erin");
    eventually("a changed password", async || {
        password_hash(&other, "erin") == changed
    })
    .await;
    let new = [basic("erin", "second-secret-2")];
    assert_eq!(
        on(&other, Method::GET, ACCOUNT, &new, None).await.status,
        StatusCode::OK
    );
    assert_eq!(
        on(&other, Method::GET, ACCOUNT, &erin, None).await.status,
        StatusCode::UNAUTHORIZED
    );

    for poller in pollers {
        poller.abort();
    }
}

#[tokio::test]
async fn blob_store_changes_reach_the_other_server_within_the_poll_interval() {
    let fixture = Fixture::new().await;
    let other = second_server(&fixture).await;
    let poller = revisions::spawn(other.clone(), POLL);

    let reply = fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/file"),
            Auth::Admin,
            Some(json!({"name": "files", "path": "files"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let opened = other.blob_stores().get("files").await.unwrap();
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/file/files"),
            Auth::Admin,
            Some(json!({"path": "files", "softQuota": {"type": "spaceUsedQuota", "limit": 1000}})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    eventually("a changed blob store", async || {
        let current = other.blob_stores().get("files").await.unwrap();
        !Arc::ptr_eq(&opened, &current)
    })
    .await;

    poller.abort();
}

#[tokio::test]
async fn a_session_survives_a_server_that_has_not_caught_up_yet() {
    let fixture = Fixture::new().await;
    let other = second_server(&fixture).await;

    create_user(&fixture, "frank", "first-secret-1", &[]).await;
    let signed_in = on(
        &other,
        Method::POST,
        SESSION,
        &[],
        Some(json!({"username": "frank", "password": "first-secret-1"})),
    )
    .await;
    assert_eq!(
        signed_in.status,
        StatusCode::OK,
        "a user created through one server signs in through another at once: {}",
        signed_in.text()
    );

    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "frank", "password": "first-secret-1"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let (cookies, csrf) = cookie_header(&reply);
    let session = [("cookie", cookies.clone())];

    let reply = on(
        &fixture.state,
        Method::PUT,
        PASSWORD,
        &[("cookie", cookies.clone()), ("x-astrum-csrf", csrf.clone())],
        Some(json!({"currentPassword": "first-secret-1", "newPassword": "second-secret-2"})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_ne!(
        password_hash(&other, "frank"),
        password_hash(&fixture.state, "frank"),
        "the other server has not caught up yet"
    );

    let there = on(&other, Method::GET, SESSION, &session, None).await;
    assert_eq!(there.json()["authenticated"], true, "{}", there.text());
    assert_eq!(there.json()["user"]["userId"], "frank");
    let here = on(&fixture.state, Method::GET, SESSION, &session, None).await;
    assert_eq!(here.json()["authenticated"], true, "{}", here.text());
}
