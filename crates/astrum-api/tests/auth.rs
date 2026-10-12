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
    fixture.state.security().reload().await.unwrap();
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
