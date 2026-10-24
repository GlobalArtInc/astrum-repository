mod support;

use astrum_db::entity::security_user;
use axum::http::{Method, StatusCode, header};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::json;
use support::{Auth, Fixture, Reply};

const SESSION: &str = "/service/rest/internal/session";
const TOKENS: &str = "/service/rest/internal/access-tokens";

struct Cookies {
    session: String,
    csrf: String,
}

impl Cookies {
    fn header(&self) -> String {
        format!("astrum_session={}; astrum_csrf={}", self.session, self.csrf)
    }
}

fn cookies(reply: &Reply) -> Cookies {
    let mut session = None;
    let mut csrf = None;
    for value in reply.headers.get_all(header::SET_COOKIE) {
        let value = value.to_str().unwrap();
        let pair = value.split(';').next().unwrap();
        let (name, token) = pair.split_once('=').unwrap();
        match name {
            "astrum_session" => {
                assert!(value.contains("HttpOnly"), "{value}");
                session = Some(token.to_owned());
            }
            "astrum_csrf" => {
                assert!(!value.contains("HttpOnly"), "{value}");
                csrf = Some(token.to_owned());
            }
            _ => {}
        }
    }
    Cookies {
        session: session.unwrap(),
        csrf: csrf.unwrap(),
    }
}

async fn sign_in(fixture: &Fixture) -> Cookies {
    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": fixture.admin_password})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["user"]["userId"], "admin");
    assert_eq!(reply.json()["authenticated"], true);
    cookies(&reply)
}

#[tokio::test]
async fn signing_in_sets_a_session_and_an_anti_forgery_cookie() {
    let fixture = Fixture::new().await;
    let wrong = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": "nope"})),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    assert!(
        wrong.headers.get(header::WWW_AUTHENTICATE).is_none(),
        "no browser prompt"
    );

    let nobody = fixture.call(Method::GET, SESSION, Auth::None, None).await;
    assert_eq!(nobody.json()["authenticated"], false);
    assert_eq!(nobody.json()["user"], json!(null));

    let cookies = sign_in(&fixture).await;
    let current = fixture
        .send(
            Method::GET,
            SESSION,
            Auth::None,
            None,
            &[("cookie", &cookies.header())],
        )
        .await;
    let body = current.json();
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["user"]["userId"], "admin");
    assert_eq!(body["user"]["realm"], "AstrumAuthenticatingRealm");
    assert_eq!(body["permissions"], json!(["astrum:*"]));
}

#[tokio::test]
async fn cookies_are_secure_behind_https() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .send(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": fixture.admin_password})),
            &[("x-forwarded-proto", "https")],
        )
        .await;
    for value in reply.headers.get_all(header::SET_COOKIE) {
        assert!(value.to_str().unwrap().contains("; Secure"), "{value:?}");
    }
}

#[tokio::test]
async fn changes_through_a_session_need_the_anti_forgery_token() {
    let fixture = Fixture::new().await;
    let cookies = sign_in(&fixture).await;
    let body = json!({"name": "ci", "scope": "READ", "expiresInDays": 30});

    let without = fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(body.clone()),
            &[("cookie", &cookies.header())],
        )
        .await;
    assert_eq!(without.status, StatusCode::FORBIDDEN);

    let wrong = fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(body.clone()),
            &[("cookie", &cookies.header()), ("x-astrum-csrf", "forged")],
        )
        .await;
    assert_eq!(wrong.status, StatusCode::FORBIDDEN);

    let cross_site = fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(body.clone()),
            &[
                ("cookie", &cookies.header()),
                ("x-astrum-csrf", &cookies.csrf),
                ("sec-fetch-site", "cross-site"),
            ],
        )
        .await;
    assert_eq!(cross_site.status, StatusCode::FORBIDDEN);

    let right = fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(body),
            &[
                ("cookie", &cookies.header()),
                ("x-astrum-csrf", &cookies.csrf),
                ("sec-fetch-site", "same-origin"),
            ],
        )
        .await;
    assert_eq!(right.status, StatusCode::CREATED, "{}", right.text());
}

#[tokio::test]
async fn signing_out_ends_the_session() {
    let fixture = Fixture::new().await;
    let cookies = sign_in(&fixture).await;
    let out = fixture
        .send(
            Method::DELETE,
            SESSION,
            Auth::None,
            None,
            &[
                ("cookie", &cookies.header()),
                ("x-astrum-csrf", &cookies.csrf),
            ],
        )
        .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    let cleared: Vec<_> = out.headers.get_all(header::SET_COOKIE).iter().collect();
    assert!(
        cleared
            .iter()
            .all(|value| value.to_str().unwrap().contains("Max-Age=0"))
    );
    let after = fixture
        .send(
            Method::GET,
            SESSION,
            Auth::None,
            None,
            &[("cookie", &cookies.header())],
        )
        .await;
    assert_eq!(after.json()["authenticated"], false);
}

#[tokio::test]
async fn a_new_password_ends_older_sessions() {
    let fixture = Fixture::new().await;
    let cookies = sign_in(&fixture).await;
    security_user::Entity::update(security_user::ActiveModel {
        id: ActiveValue::Unchanged("admin".to_owned()),
        password_hash: ActiveValue::Set(Some(astrum_security::password::hash("another"))),
        ..Default::default()
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let after = fixture
        .send(
            Method::GET,
            SESSION,
            Auth::None,
            None,
            &[("cookie", &cookies.header())],
        )
        .await;
    assert_eq!(after.json()["authenticated"], false);
}

#[tokio::test]
async fn a_disabled_user_loses_the_session() {
    let fixture = Fixture::new().await;
    fixture.add_user("jane", "secret-1", &[]).await;
    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "jane", "password": "secret-1"})),
        )
        .await;
    let cookies = cookies(&reply);
    security_user::Entity::update(security_user::ActiveModel {
        id: ActiveValue::Unchanged("jane".to_owned()),
        status: ActiveValue::Set("disabled".to_owned()),
        ..Default::default()
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let after = fixture
        .send(
            Method::GET,
            SESSION,
            Auth::None,
            None,
            &[("cookie", &cookies.header())],
        )
        .await;
    assert_eq!(after.json()["authenticated"], false);
}

#[tokio::test]
async fn repeated_failures_pause_sign_in() {
    let fixture = Fixture::new().await;
    for _ in 0..10 {
        let reply = fixture
            .call(
                Method::POST,
                SESSION,
                Auth::None,
                Some(json!({"username": "admin", "password": "x"})),
            )
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
    let blocked = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": fixture.admin_password})),
        )
        .await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn sign_ins_and_failures_are_audited_with_the_client_address() {
    let fixture = Fixture::new().await;
    let forwarded = [("x-forwarded-for", "203.0.113.9, 10.0.0.1")];
    let wrong = fixture
        .send(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": "nope"})),
            &forwarded,
        )
        .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let right = fixture
        .send(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "admin", "password": fixture.admin_password})),
            &forwarded,
        )
        .await;
    assert_eq!(right.status, StatusCode::OK);
    let events = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let events = events.as_array().unwrap();
    let login = events
        .iter()
        .find(|event| event["type"] == "login")
        .expect("a sign-in event");
    assert_eq!(login["initiator"], "admin/203.0.113.9");
    assert_eq!(login["attributes"]["principal"], "admin");
    let failure = events
        .iter()
        .find(|event| event["domain"] == "security.user" && event["type"] == "authentication")
        .expect("a failed sign-in event");
    assert_eq!(failure["attributes"]["userId"], "admin");
    assert_eq!(failure["attributes"]["remoteIp"], "203.0.113.9");
    assert_eq!(failure["attributes"]["wasSuccessful"], false);
}
