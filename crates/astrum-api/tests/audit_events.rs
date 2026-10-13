mod support;

use std::sync::Arc;

use astrum_db::entity::audit_event;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::{EntityTrait, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const SESSION: &str = "/service/rest/internal/session";

async fn events(fixture: &Fixture) -> Vec<audit_event::Model> {
    audit_event::Entity::find()
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn of_kind(fixture: &Fixture, kind: &str) -> Vec<audit_event::Model> {
    events(fixture)
        .await
        .into_iter()
        .filter(|event| event.kind == kind)
        .collect()
}

async fn sign_in(fixture: &Fixture, password: &str) -> StatusCode {
    fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": password})),
        )
        .await
        .status
}

#[tokio::test]
async fn sign_ins_and_their_failures_are_recorded_like_the_old_server() {
    let fixture = Fixture::new().await;
    assert_eq!(sign_in(&fixture, "wrong").await, StatusCode::UNAUTHORIZED);
    let failed = of_kind(&fixture, "authentication").await;
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].domain, "security.user");
    assert_eq!(failed[0].initiator, "*UNKNOWN");
    assert_eq!(failed[0].attributes["path"], SESSION);
    assert_eq!(failed[0].attributes["userId"], "admin");
    assert_eq!(failed[0].attributes["wasSuccessful"], false);
    assert_eq!(
        failed[0].attributes["failureReasons"],
        json!(["INCORRECT_CREDENTIALS"])
    );

    let password = fixture.admin_password.clone();
    assert_eq!(sign_in(&fixture, &password).await, StatusCode::OK);
    let logins = of_kind(&fixture, "login").await;
    assert_eq!(logins.len(), 1);
    assert_eq!(logins[0].domain, "AstrumAuthenticatingRealm");
    assert_eq!(logins[0].initiator, "admin");
    assert_eq!(logins[0].attributes, json!({"principal": "admin"}));

    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::Basic("admin", "wrong"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::Basic("nobody", "wrong"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    let failed = of_kind(&fixture, "authentication").await;
    assert_eq!(failed.len(), 2, "unknown users are not recorded");
    assert_eq!(failed[1].attributes["path"], "/service/rest");

    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        of_kind(&fixture, "login").await.len(),
        1,
        "requests with a password are not sign-ins"
    );
}

#[tokio::test]
async fn repeated_failures_record_the_block() {
    let fixture = Fixture::new().await;
    for _ in 0..10 {
        assert_eq!(sign_in(&fixture, "wrong").await, StatusCode::UNAUTHORIZED);
    }
    let blocked = of_kind(&fixture, "blocked").await;
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].domain, "security.auth.ratelimit");
    assert_eq!(blocked[0].context, "admin");
    assert_eq!(blocked[0].attributes["authMethod"], "UI");
    let password = fixture.admin_password.clone();
    assert_eq!(
        sign_in(&fixture, &password).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn the_first_download_of_an_asset_is_recorded() {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": "files",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let reply = fixture
        .send_body(
            Method::PUT,
            "/repository/files/docs/readme.txt",
            Auth::Admin,
            Body::from("read me"),
            &[],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    for _ in 0..3 {
        let reply = fixture
            .call(
                Method::GET,
                "/repository/files/docs/readme.txt",
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK);
    }
    let downloads = of_kind(&fixture, "updated-downloaded").await;
    assert_eq!(downloads.len(), 1, "one event per asset and interval");
    assert_eq!(downloads[0].domain, "repository.asset");
    assert_eq!(downloads[0].context, "/docs/readme.txt");
    assert_eq!(downloads[0].initiator, "admin");
    assert_eq!(
        downloads[0].attributes["repository.name"],
        Value::from("files")
    );
    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?repository=files&type=updated-downloaded",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json().as_array().map(Vec::len), Some(1));
}
