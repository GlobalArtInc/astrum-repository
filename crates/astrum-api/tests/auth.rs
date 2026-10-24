mod support;

use astrum_api::STATUS;
use astrum_security::model::{ANONYMOUS_KEY, AnonymousSettings};
use axum::http::{Method, StatusCode, header};
use support::{Auth, Fixture};

#[tokio::test]
async fn wrong_credentials_are_refused_with_a_challenge() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::GET, STATUS, Auth::Basic("admin", "wrong"), None)
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        reply.headers.get(header::WWW_AUTHENTICATE).unwrap(),
        "Basic realm=\"Astrum Repository\""
    );
    assert_eq!(reply.json()["id"], "*");

    let reply = fixture
        .send(
            Method::GET,
            STATUS,
            Auth::Basic("admin", "wrong"),
            None,
            &[("X-Astrum-UI", "true")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(
        reply.headers.get(header::WWW_AUTHENTICATE).is_none(),
        "the UI asks without a browser prompt"
    );
}

#[tokio::test]
async fn right_credentials_pass() {
    let fixture = Fixture::new().await;
    let reply = fixture.call(Method::GET, STATUS, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
async fn malformed_authorization_is_refused() {
    let fixture = Fixture::new().await;
    for value in ["Basic", "Basic !!!", "Basic YWRtaW4="] {
        let reply = fixture
            .send(
                Method::GET,
                STATUS,
                Auth::None,
                None,
                &[("Authorization", value)],
            )
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{value}");
    }
}

#[tokio::test]
async fn anonymous_access_can_be_switched_on() {
    let fixture = Fixture::new().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        ANONYMOUS_KEY,
        &AnonymousSettings {
            enabled: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let anonymous = fixture.state.security().anonymous().unwrap();
    assert!(anonymous.anonymous);
    assert_eq!(
        fixture
            .call(Method::GET, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn format_tokens_work_while_their_realm_is_active() {
    let fixture = Fixture::new().await;
    let security = fixture.state.security();
    let token = security
        .issue_format_token("NpmToken", "admin", None)
        .await
        .unwrap();
    assert!(token.starts_with("NpmToken."));
    let bearer = format!("Bearer {token}");
    let inactive = fixture
        .send(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::None,
            None,
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(inactive.status, StatusCode::UNAUTHORIZED);

    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "NpmToken"],
    )
    .await
    .unwrap();
    security.changed().await.unwrap();
    let active = fixture
        .send(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::None,
            None,
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(active.status, StatusCode::OK);

    assert!(security.revoke_format_token(&token).await.unwrap());
    let revoked = fixture
        .send(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::None,
            None,
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(revoked.status, StatusCode::UNAUTHORIZED);
}
