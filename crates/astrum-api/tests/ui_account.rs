mod support;
mod ui_support;

use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const ACCOUNT: &str = "/service/rest/internal/ui/user";
const PASSWORD: &str = "/service/rest/internal/ui/user/password";
const SESSION: &str = "/service/rest/internal/session";

struct Cookies {
    session: String,
    csrf: String,
}

impl Cookies {
    fn header(&self) -> String {
        format!("astrum_session={}; astrum_csrf={}", self.session, self.csrf)
    }
}

async fn sign_in(fixture: &Fixture, user: &str, password: &str) -> Cookies {
    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": user, "password": password})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let mut session = String::new();
    let mut csrf = String::new();
    for value in reply.headers.get_all(header::SET_COOKIE) {
        let pair = value.to_str().unwrap().split(';').next().unwrap();
        let (name, token) = pair.split_once('=').unwrap();
        match name {
            "astrum_session" => session = token.to_owned(),
            "astrum_csrf" => csrf = token.to_owned(),
            _ => {}
        }
    }
    Cookies { session, csrf }
}

async fn through(
    fixture: &Fixture,
    cookies: &Cookies,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Reply {
    fixture
        .send(
            method,
            path,
            Auth::None,
            body,
            &[
                ("cookie", &cookies.header()),
                ("x-astrum-csrf", &cookies.csrf),
                ("sec-fetch-site", "same-origin"),
            ],
        )
        .await
}

async fn jane() -> Fixture {
    let fixture = ready().await;
    role(&fixture, "readers", &["as-search-read"]).await;
    fixture.add_user("jane", "password-1", &["readers"]).await;
    fixture
}

async fn change(fixture: &Fixture, auth: Auth<'_>, current: &str, new: &str) -> Reply {
    fixture
        .call(
            Method::PUT,
            PASSWORD,
            auth,
            Some(json!({"currentPassword": current, "newPassword": new})),
        )
        .await
}

#[tokio::test]
async fn signed_in_users_read_and_change_their_own_account() {
    let fixture = jane().await;
    let reply = get(&fixture, ACCOUNT, user("jane")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!({
            "userId": "jane",
            "firstName": "jane",
            "lastName": "Tester",
            "email": "jane@example.org",
            "source": "default",
            "realm": "AstrumAuthenticatingRealm",
            "status": "active",
            "roles": ["readers"],
            "readOnly": false
        })
    );

    let updated = fixture
        .call(
            Method::PUT,
            ACCOUNT,
            user("jane"),
            Some(json!({"firstName": "Jane", "lastName": "Doe", "email": "jane.doe@example.org"})),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    assert_eq!(updated.json()["firstName"], "Jane");
    assert_eq!(updated.json()["email"], "jane.doe@example.org");
    let users = get(
        &fixture,
        "/service/rest/v1/security/users?userId=jane",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(users[0]["lastName"], "Doe");
    assert_eq!(users[0]["roles"], json!(["readers"]));
    assert_eq!(users[0]["status"], "active");

    let alias = fixture
        .call(
            Method::PUT,
            ACCOUNT,
            user("jane"),
            Some(json!({"firstName": "Jane", "lastName": "Doe", "emailAddress": "jd@example.org"})),
        )
        .await;
    assert_eq!(alias.json()["email"], "jd@example.org");

    let invalid = fixture
        .call(
            Method::PUT,
            ACCOUNT,
            user("jane"),
            Some(json!({"firstName": "", "email": "not-an-address"})),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        names(&invalid.json(), "id"),
        ["firstName", "lastName", "email"]
    );

    let events = get(
        &fixture,
        "/service/rest/internal/audit?domain=security.user&initiator=jane",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(names(&events, "context"), ["jane", "jane"]);
}

#[tokio::test]
async fn anonymous_callers_have_no_account() {
    let fixture = jane().await;
    assert_eq!(
        get(&fixture, ACCOUNT, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    anonymous(&fixture, true).await;
    assert_eq!(
        get(&fixture, ACCOUNT, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        change(&fixture, Auth::None, "x", "password-2").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn the_current_password_proves_the_change() {
    let fixture = jane().await;
    let wrong = change(&fixture, user("jane"), "wrong-password", "password-2").await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    assert_eq!(names(&wrong.json(), "id"), ["currentPassword"]);
    let short = change(&fixture, user("jane"), "password-1", "short").await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST);
    assert_eq!(names(&short.json(), "id"), ["newPassword"]);
    let empty = fixture
        .call(Method::PUT, PASSWORD, user("jane"), Some(json!({})))
        .await;
    assert_eq!(
        names(&empty.json(), "id"),
        ["currentPassword", "newPassword"]
    );

    let done = change(&fixture, user("jane"), "password-1", "password-2").await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.text());
    assert_eq!(
        get(&fixture, ACCOUNT, user("jane")).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get(&fixture, ACCOUNT, Auth::Basic("jane", "password-2"))
            .await
            .status,
        StatusCode::OK
    );
    let events = get(
        &fixture,
        "/service/rest/internal/audit?domain=security.user&context=jane",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(events[0]["attributes"]["password"], "changed");
    assert_eq!(events[0]["initiator"], "jane");
}

#[tokio::test]
async fn wrong_current_passwords_are_rate_limited() {
    let fixture = jane().await;
    for _ in 0..10 {
        let reply = change(&fixture, user("jane"), "wrong-password", "password-2").await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    }
    let paused = change(&fixture, user("jane"), "password-1", "password-2").await;
    assert_eq!(paused.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn a_new_password_ends_the_other_sessions_only() {
    let fixture = jane().await;
    let here = sign_in(&fixture, "jane", "password-1").await;
    let elsewhere = sign_in(&fixture, "jane", "password-1").await;
    let done = through(
        &fixture,
        &here,
        Method::PUT,
        PASSWORD,
        Some(json!({"currentPassword": "password-1", "newPassword": "password-2"})),
    )
    .await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.text());
    let still = through(&fixture, &here, Method::GET, ACCOUNT, None).await;
    assert_eq!(still.status, StatusCode::OK, "{}", still.text());
    let ended = through(&fixture, &elsewhere, Method::GET, ACCOUNT, None).await;
    assert_eq!(ended.status, StatusCode::UNAUTHORIZED);

    let basic = sign_in(&fixture, "jane", "password-2").await;
    let done = change(
        &fixture,
        Auth::Basic("jane", "password-2"),
        "password-2",
        "password-3",
    )
    .await;
    assert_eq!(done.status, StatusCode::NO_CONTENT);
    assert_eq!(
        through(&fixture, &basic, Method::GET, ACCOUNT, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        through(&fixture, &here, Method::GET, ACCOUNT, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn the_first_admin_password_file_goes_with_the_first_change() {
    let fixture = ready().await;
    let file = fixture.data.path().join(ADMIN_PASSWORD_FILE);
    assert!(file.exists());
    let password = fixture.admin_password.clone();
    let done = change(&fixture, Auth::Admin, &password, "a-new-admin-password").await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.text());
    assert!(!file.exists());
}

#[tokio::test]
async fn read_only_tokens_cannot_change_the_account() {
    let fixture = jane().await;
    let cookies = sign_in(&fixture, "jane", "password-1").await;
    let created = through(
        &fixture,
        &cookies,
        Method::POST,
        "/service/rest/internal/access-tokens",
        Some(json!({"name": "ci", "scope": "READ", "expiresInDays": 1})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let token = created.json()["token"].as_str().unwrap().to_owned();
    assert_eq!(
        get(&fixture, ACCOUNT, Auth::Basic("jane", &token))
            .await
            .status,
        StatusCode::OK
    );
    let refused = fixture
        .call(
            Method::PUT,
            ACCOUNT,
            Auth::Basic("jane", &token),
            Some(json!({"firstName": "J", "lastName": "D", "email": "j@example.org"})),
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(
        change(
            &fixture,
            Auth::Basic("jane", &token),
            "password-1",
            "password-2"
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
}
