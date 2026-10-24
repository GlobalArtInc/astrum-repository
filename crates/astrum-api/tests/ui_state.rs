mod support;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const STATE: &str = "/service/rest/internal/ui/state";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_maven::Maven));
    fixture
}

async fn state(fixture: &Fixture, auth: Auth<'_>) -> Value {
    let reply = fixture
        .send(
            Method::GET,
            STATE,
            auth,
            None,
            &[("host", "repo.example.org:8081"), ("x-astrum-ui", "true")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

async fn admin_post(fixture: &Fixture, path: &str, body: Value) {
    let reply = fixture
        .call(Method::POST, path, Auth::Admin, Some(body))
        .await;
    assert!(reply.status.is_success(), "{path}: {}", reply.text());
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn a_stranger_sees_the_server_but_no_user_and_no_permissions() {
    let fixture = ready().await;
    let body = state(&fixture, Auth::None).await;
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(body["user"], Value::Null);
    assert_eq!(body["anonymousAccess"], false);
    assert_eq!(body["permissions"], json!([]));
    assert_eq!(body["baseUrl"], "http://repo.example.org:8081");
    assert_eq!(body["sessionTimeoutMinutes"], 0);
    assert_eq!(
        body["formats"],
        json!([
            {"id": "maven2", "apiName": "maven", "types": ["hosted", "proxy", "group"]},
            {"id": "raw", "apiName": "raw", "types": ["hosted", "proxy", "group"]},
        ])
    );
    let file = fixture.data.path().join("admin.password");
    assert_eq!(body["adminPasswordFile"], file.display().to_string());
}

#[tokio::test]
async fn the_admin_sees_everything_and_the_password_file() {
    let fixture = ready().await;
    let body = state(&fixture, Auth::Admin).await;
    assert_eq!(body["user"]["userId"], "admin");
    assert_eq!(body["user"]["name"], "Administrator User");
    assert_eq!(body["user"]["email"], "admin@example.org");
    assert_eq!(body["user"]["source"], "default");
    assert!(strings(&body["permissions"]).contains(&"astrum:*"));
    assert!(body["adminPasswordFile"].is_string());

    std::fs::remove_file(fixture.data.path().join("admin.password")).unwrap();
    let body = state(&fixture, Auth::Admin).await;
    assert_eq!(body["adminPasswordFile"], Value::Null);
}

#[tokio::test]
async fn a_user_gets_the_permissions_of_their_roles_and_selector_grants() {
    let fixture = ready().await;
    admin_post(
        &fixture,
        "/service/rest/v1/security/content-selectors",
        json!({"name": "public-only", "description": "", "expression": "path =^ \"/public/\""}),
    )
    .await;
    admin_post(
        &fixture,
        "/service/rest/v1/security/privileges/repository-content-selector",
        json!({"name": "public-read", "description": "", "actions": ["READ", "BROWSE"], "format": "raw", "repository": "*", "contentSelector": "public-only"}),
    )
    .await;
    admin_post(
        &fixture,
        "/service/rest/v1/security/roles",
        json!({"id": "readers", "name": "readers", "description": "", "privileges": ["public-read", "as-search-read", "as-repository-view-maven2-*-browse"], "roles": []}),
    )
    .await;
    fixture.add_user("jane", "jane-pass", &["readers"]).await;

    let body = state(&fixture, Auth::Basic("jane", "jane-pass")).await;
    assert_eq!(
        body["user"],
        json!({"userId": "jane", "name": "jane Tester", "email": "jane@example.org", "source": "default"})
    );
    assert_eq!(
        strings(&body["permissions"]),
        [
            "astrum:repository-content-selector:public-only:raw:*:browse,read",
            "astrum:repository-view:maven2:*:browse",
            "astrum:search:read",
        ]
    );
    assert_eq!(
        body["adminPasswordFile"],
        Value::Null,
        "only strangers and administrators learn where the first password is"
    );
}

#[tokio::test]
async fn anonymous_access_gives_the_anonymous_permissions_without_a_user() {
    let fixture = ready().await;
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert!(reply.status.is_success(), "{}", reply.text());
    let body = state(&fixture, Auth::None).await;
    assert_eq!(body["anonymousAccess"], true);
    assert_eq!(body["user"], Value::Null);
    let permissions = strings(&body["permissions"]);
    assert!(
        permissions.contains(&"astrum:search:read"),
        "{permissions:?}"
    );
    assert!(!permissions.contains(&"astrum:*"));
    assert!(body["adminPasswordFile"].is_string());
}

#[tokio::test]
async fn the_session_timeout_setting_is_reported() {
    let fixture = ready().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        "security.sessionTimeoutMinutes",
        &45u32,
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let body = state(&fixture, Auth::None).await;
    assert_eq!(body["sessionTimeoutMinutes"], 45);
}

#[tokio::test]
async fn a_wrong_password_is_refused_without_a_challenge_for_the_ui() {
    let fixture = ready().await;
    let reply = fixture
        .send(
            Method::GET,
            STATE,
            Auth::Basic("admin", "wrong"),
            None,
            &[("x-astrum-ui", "true")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(!reply.headers.contains_key("www-authenticate"));
}
