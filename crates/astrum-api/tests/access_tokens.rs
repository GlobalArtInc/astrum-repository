mod support;

use astrum_db::entity::access_token;
use astrum_security::model::REALMS_KEY;
use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const TOKENS: &str = "/service/rest/internal/access-tokens";
const AUDIT: &str = "/service/rest/internal/audit";

async fn create(fixture: &Fixture, auth: Auth<'_>, scope: &str) -> Value {
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            auth,
            Some(json!({"name": format!("ci-{scope}"), "scope": scope, "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()
}

fn basic(user: &str, password: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{user}:{password}")))
}

#[tokio::test]
async fn a_token_works_as_a_password_and_as_a_bearer() {
    let fixture = Fixture::new().await;
    let created = create(&fixture, Auth::Admin, "WRITE").await;
    let token = created["token"].as_str().unwrap();
    assert!(token.starts_with("astrum_") && token.len() == 47, "{token}");
    assert_eq!(created["hint"], token[..11]);
    assert_eq!(created["userId"], "admin");

    let as_password = fixture
        .call(Method::GET, AUDIT, Auth::Basic("admin", token), None)
        .await;
    assert_eq!(as_password.status, StatusCode::OK);
    let as_bearer = fixture
        .send(
            Method::GET,
            AUDIT,
            Auth::None,
            None,
            &[("authorization", &format!("Bearer {token}"))],
        )
        .await;
    assert_eq!(as_bearer.status, StatusCode::OK);
    let other_user = fixture
        .call(Method::GET, AUDIT, Auth::Basic("jane", token), None)
        .await;
    assert_eq!(other_user.status, StatusCode::UNAUTHORIZED);

    let listed = fixture
        .call(Method::GET, TOKENS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert!(
        listed[0].get("token").is_none(),
        "the token is shown once only"
    );
    assert!(listed[0]["lastUsedAt"].is_string());
}

#[tokio::test]
async fn a_token_cannot_make_tokens() {
    let fixture = Fixture::new().await;
    let created = create(&fixture, Auth::Admin, "WRITE").await;
    let token = created["token"].as_str().unwrap();
    let reply = fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(json!({"name": "again", "expiresInDays": 1})),
            &[("authorization", &basic("admin", token))],
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn expired_tokens_and_inactive_realms_refuse() {
    let fixture = Fixture::new().await;
    let created = create(&fixture, Auth::Admin, "READ").await;
    let token = created["token"].as_str().unwrap().to_owned();
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::Basic("admin", &token), None)
            .await
            .status,
        StatusCode::OK
    );

    astrum_db::setting::put(
        fixture.state.db().conn(),
        REALMS_KEY,
        &["AstrumAuthenticatingRealm"],
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::Basic("admin", &token), None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    astrum_db::setting::put(
        fixture.state.db().conn(),
        REALMS_KEY,
        &["AstrumAuthenticatingRealm", "AccessTokenRealm"],
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();

    let id = uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
    access_token::Entity::update(access_token::ActiveModel {
        id: ActiveValue::Unchanged(id),
        expires_at: ActiveValue::Set(Some(
            chrono::Utc::now().fixed_offset() - chrono::Duration::days(1),
        )),
        ..Default::default()
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::Basic("admin", &token), None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn tokens_belong_to_their_owner() {
    let fixture = Fixture::new().await;
    fixture.add_user("jane", "secret-1", &[]).await;
    let janes = create(&fixture, Auth::Basic("jane", "secret-1"), "READ").await;
    let admins = create(&fixture, Auth::Admin, "WRITE").await;

    let mine = fixture
        .call(Method::GET, TOKENS, Auth::Basic("jane", "secret-1"), None)
        .await
        .json();
    assert_eq!(mine.as_array().unwrap().len(), 1);
    assert_eq!(
        fixture
            .call(
                Method::GET,
                &format!("{TOKENS}?userId=admin"),
                Auth::Basic("jane", "secret-1"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let everyone = fixture
        .call(
            Method::GET,
            &format!("{TOKENS}?userId=*"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(everyone.as_array().unwrap().len(), 2);

    let admins_path = format!("{TOKENS}/{}", admins["id"].as_str().unwrap());
    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                &admins_path,
                Auth::Basic("jane", "secret-1"),
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let janes_path = format!("{TOKENS}/{}", janes["id"].as_str().unwrap());
    assert_eq!(
        fixture
            .call(Method::DELETE, &janes_path, Auth::Admin, None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        fixture
            .call(Method::DELETE, &janes_path, Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(Method::GET, TOKENS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_token_may_never_expire() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Admin,
            Some(json!({"name": "forever", "scope": "READ", "expiresInDays": null})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let created = reply.json();
    assert_eq!(created["expiresAt"], Value::Null);
    let token = created["token"].as_str().unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::Basic("admin", token), None)
            .await
            .status,
        StatusCode::OK
    );
    let listed = fixture
        .call(Method::GET, TOKENS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(listed[0]["name"], "forever");
    assert_eq!(listed[0]["expiresAt"], Value::Null);
}

#[tokio::test]
async fn bad_token_requests_are_explained() {
    let fixture = Fixture::new().await;
    for (body, field) in [
        (json!({"name": " ", "expiresInDays": 5}), "name"),
        (json!({"name": "x", "expiresInDays": 0}), "expiresInDays"),
        (json!({"name": "x", "expiresInDays": 3651}), "expiresInDays"),
        (json!({"name": "x"}), "expiresInDays"),
    ] {
        let reply = fixture
            .call(Method::POST, TOKENS, Auth::Admin, Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
        assert_eq!(reply.json()[0]["id"], field);
    }
}

#[tokio::test]
async fn the_audit_log_records_and_needs_its_permission() {
    let fixture = Fixture::new().await;
    fixture.add_user("jane", "secret-1", &[]).await;
    create(&fixture, Auth::Admin, "READ").await;
    let events = fixture
        .call(Method::GET, AUDIT, Auth::Admin, None)
        .await
        .json();
    let first = &events[0];
    assert_eq!(first["domain"], "security.accesstoken");
    assert_eq!(first["type"], "created");
    assert_eq!(first["initiator"], "admin");
    assert!(first["attributes"].get("token").is_none());
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::Basic("jane", "secret-1"), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .call(Method::GET, AUDIT, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let filtered = fixture
        .call(
            Method::GET,
            &format!("{AUDIT}?domain=nothing"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(filtered, json!([]));
}
