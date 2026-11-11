mod support;

use std::sync::Arc;

use astrum_security::ldap::TlsClient;
use astrum_security::two_factor::POLICY_KEY;
use astrum_security::{AuthError, Gate};
use astrum_testkit::ldap::{BASE, LdapTestServer, SYSTEM_DN, SYSTEM_PASSWORD};
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const SESSION: &str = "/service/rest/internal/session";
const PASSWORD: &str = "/service/rest/internal/ui/user/password";
const TOKENS: &str = "/service/rest/internal/access-tokens";
const AUDIT: &str = "/service/rest/internal/audit";
const STATUS: &str = astrum_api::STATUS;
const SECRET: &str = "jane-password-1";

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(sql)
        .await
        .unwrap();
}

async fn enroll(fixture: &Fixture, user: &str) {
    execute(
        fixture,
        &format!(
            "INSERT INTO user_second_factor (user_id, totp_secret, totp_confirmed_at) VALUES ('{user}', 'sealed', now())"
        ),
    )
    .await;
    fixture.state.security().changed().await.unwrap();
}

async fn jane() -> Fixture {
    let fixture = Fixture::new().await;
    fixture.add_user("jane", SECRET, &["as-admin"]).await;
    fixture.add_user("pat", "pat-password-1", &[]).await;
    fixture
}

async fn basic(fixture: &Fixture, user: &str, password: &str) -> Reply {
    fixture
        .send(Method::GET, STATUS, Auth::Basic(user, password), None, &[])
        .await
}

fn same_answer(left: &Reply, right: &Reply) {
    assert_eq!(left.status, right.status);
    assert_eq!(left.headers, right.headers);
    assert_eq!(left.body, right.body);
}

async fn failure_reasons(fixture: &Fixture, user: &str) -> Vec<Value> {
    let mut reasons: Vec<Value> = fixture
        .call(Method::GET, AUDIT, Auth::Admin, None)
        .await
        .json()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["domain"] == "security.user"
                && event["type"] == "authentication"
                && event["attributes"]["userId"] == user
        })
        .map(|event| event["attributes"]["failureReasons"][0].clone())
        .collect();
    reasons.reverse();
    reasons
}

#[tokio::test]
async fn an_enrolled_users_password_is_refused_like_a_wrong_one_and_audited() {
    let fixture = jane().await;
    let wrong = basic(&fixture, "jane", "wrong").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    enroll(&fixture, "jane").await;

    let refused = basic(&fixture, "jane", SECRET).await;
    same_answer(&refused, &wrong);
    assert_eq!(
        basic(&fixture, "pat", "pat-password-1").await.status,
        StatusCode::OK
    );

    execute(
        &fixture,
        "UPDATE user_second_factor SET totp_secret = NULL, totp_confirmed_at = NULL, requirement = 'required', \
         enroll_by = now() - interval '1 second' WHERE user_id = 'jane'",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    same_answer(&basic(&fixture, "jane", SECRET).await, &wrong);

    assert_eq!(
        failure_reasons(&fixture, "jane").await,
        [
            json!("INCORRECT_CREDENTIALS"),
            json!("SECOND_FACTOR_REQUIRED"),
            json!("SECOND_FACTOR_ENROLLMENT_REQUIRED"),
        ]
    );
}

#[tokio::test]
async fn a_refused_password_of_an_enrolled_user_counts_against_the_throttle() {
    let mut fixture = jane().await;
    fixture.trust(&["10.0.0.0/8"]);
    fixture.peer = Some("10.0.0.1".parse().unwrap());
    enroll(&fixture, "jane").await;
    let forwarded = [("x-forwarded-for", "198.51.100.1")];
    for _ in 0..10 {
        let reply = fixture
            .send(
                Method::GET,
                STATUS,
                Auth::Basic("jane", SECRET),
                None,
                &forwarded,
            )
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
    let blocked = fixture
        .send(
            Method::GET,
            STATUS,
            Auth::Basic("jane", "wrong"),
            None,
            &forwarded,
        )
        .await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn an_access_token_of_an_enrolled_user_still_works() {
    let fixture = jane().await;
    let created = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("jane", SECRET),
            Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let token = created.json()["token"].as_str().unwrap().to_owned();
    enroll(&fixture, "jane").await;

    let bearer = format!("Bearer {token}");
    let by_bearer = fixture
        .send(
            Method::GET,
            STATUS,
            Auth::None,
            None,
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(by_bearer.status, StatusCode::OK);
    assert_eq!(basic(&fixture, "jane", &token).await.status, StatusCode::OK);
}

struct Formats {
    fixture: Fixture,
    app: axum::Router,
}

async fn formats() -> Formats {
    let fixture = jane().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    content.register(Arc::new(astrum_format_openvsx::OpenVsx));
    content.register(Arc::new(astrum_format_raw::Raw));
    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "DockerToken"],
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/openvsx/hosted",
            Auth::Admin,
            Some(json!({
                "name": "ovsx-hosted", "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let routes = astrum_format_docker::routes().merge(astrum_format_openvsx::routes());
    let app = astrum_api::router_with(fixture.state.clone(), routes);
    Formats { fixture, app }
}

impl Formats {
    async fn send(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        body: Option<(&str, Vec<u8>)>,
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let body = match body {
            Some((content_type, bytes)) => {
                request = request.header(header::CONTENT_TYPE, content_type);
                Body::from(bytes)
            }
            None => Body::empty(),
        };
        let response = self
            .app
            .clone()
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

    async fn grant(&self, password: &str) -> Reply {
        let form = format!(
            "grant_type=password&username=jane&password={password}&scope=repository:a:pull"
        );
        self.send(
            Method::POST,
            "/v2/token",
            &[],
            Some(("application/x-www-form-urlencoded", form.into_bytes())),
        )
        .await
    }

    async fn token_by_basic(&self, password: &str) -> Reply {
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("jane:{password}"),
        );
        let authorization = format!("Basic {encoded}");
        self.send(
            Method::GET,
            "/v2/token?scope=repository:a:pull",
            &[("authorization", &authorization)],
            None,
        )
        .await
    }

    async fn verify_pat(&self, password: &str) -> Reply {
        let path = format!("/repository/ovsx-hosted/api/acme/verify-pat?token=jane:{password}");
        self.send(Method::GET, &path, &[], None).await
    }
}

#[tokio::test]
async fn the_docker_password_grant_refuses_an_enrolled_user_like_a_wrong_password() {
    let world = formats().await;
    assert_eq!(world.grant(SECRET).await.status, StatusCode::OK);
    let wrong = world.grant("wrong").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    enroll(&world.fixture, "jane").await;
    same_answer(&world.grant(SECRET).await, &wrong);
    assert_eq!(
        failure_reasons(&world.fixture, "jane").await,
        [
            json!("INCORRECT_CREDENTIALS"),
            json!("SECOND_FACTOR_REQUIRED")
        ]
    );
}

#[tokio::test]
async fn docker_token_requests_by_basic_refuse_an_enrolled_user() {
    let world = formats().await;
    assert_eq!(world.token_by_basic(SECRET).await.status, StatusCode::OK);
    let wrong = world.token_by_basic("wrong").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    enroll(&world.fixture, "jane").await;
    same_answer(&world.token_by_basic(SECRET).await, &wrong);
}

#[tokio::test]
async fn open_vsx_refuses_an_enrolled_user_like_a_wrong_password_and_audits_the_reason() {
    let world = formats().await;
    assert_eq!(world.verify_pat(SECRET).await.status, StatusCode::OK);
    let wrong = world.verify_pat("wrong").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    enroll(&world.fixture, "jane").await;
    same_answer(&world.verify_pat(SECRET).await, &wrong);
    assert_eq!(
        failure_reasons(&world.fixture, "jane").await,
        [
            json!("INCORRECT_CREDENTIALS"),
            json!("SECOND_FACTOR_REQUIRED")
        ]
    );
}

#[tokio::test]
async fn confirming_a_nuget_key_request_accepts_the_password_of_an_enrolled_user() {
    let fixture = jane().await;
    let (cookie, csrf) = session_cookie(&fixture, "jane", SECRET).await;
    enroll(&fixture, "jane").await;
    let wrong = reveal(&fixture, &cookie, &csrf, "wrong").await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST, "{}", wrong.text());
    let revealed = reveal(&fixture, &cookie, &csrf, SECRET).await;
    assert_eq!(revealed.status, StatusCode::OK, "{}", revealed.text());
    assert!(revealed.json()["apiKey"].is_string());
}

#[tokio::test]
async fn ui_sign_in_with_the_password_of_an_enrolled_user_waits_for_the_second_step() {
    let fixture = jane().await;
    let wrong = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "jane", "password": "wrong"})),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    enroll(&fixture, "jane").await;
    let pending = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "jane", "password": SECRET})),
        )
        .await;
    assert_eq!(pending.status, StatusCode::OK);
    assert_eq!(pending.json()["authenticated"], false);
    let cookies: Vec<&str> = pending
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert_eq!(cookies.len(), 1);
    assert!(cookies[0].starts_with("astrum_signin="));
    assert_eq!(
        failure_reasons(&fixture, "jane").await,
        [json!("INCORRECT_CREDENTIALS")]
    );
}

async fn session_cookie(fixture: &Fixture, user: &str, password: &str) -> (String, String) {
    let signed_in = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": user, "password": password})),
        )
        .await;
    assert_eq!(signed_in.status, StatusCode::OK);
    let mut session = String::new();
    let mut csrf = String::new();
    for value in signed_in.headers.get_all(header::SET_COOKIE) {
        let pair = value.to_str().unwrap().split(';').next().unwrap();
        let (name, token) = pair.split_once('=').unwrap();
        match name {
            "astrum_session" => session = token.to_owned(),
            "astrum_csrf" => csrf = token.to_owned(),
            _ => {}
        }
    }
    (
        format!("astrum_session={session}; astrum_csrf={csrf}"),
        csrf,
    )
}

async fn reveal(fixture: &Fixture, cookie: &str, csrf: &str, password: &str) -> Reply {
    fixture
        .send(
            Method::POST,
            "/service/rest/internal/ui/user/nuget-api-key",
            Auth::None,
            Some(json!({"password": password})),
            &[
                ("cookie", cookie),
                ("x-astrum-csrf", csrf),
                ("sec-fetch-site", "same-origin"),
            ],
        )
        .await
}

async fn change(fixture: &Fixture, cookie: &str, csrf: &str, current: &str) -> Reply {
    fixture
        .send(
            Method::PUT,
            PASSWORD,
            Auth::None,
            Some(json!({"currentPassword": current, "newPassword": "jane-password-2"})),
            &[
                ("cookie", cookie),
                ("x-astrum-csrf", csrf),
                ("sec-fetch-site", "same-origin"),
            ],
        )
        .await
}

#[tokio::test]
async fn changing_the_password_with_the_correct_one_works_for_an_enrolled_user() {
    let fixture = jane().await;
    let (cookie, csrf) = session_cookie(&fixture, "jane", SECRET).await;
    enroll(&fixture, "jane").await;
    let wrong = change(&fixture, &cookie, &csrf, "wrong-password").await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    let done = change(&fixture, &cookie, &csrf, SECRET).await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.text());
    assert!(matches!(
        fixture
            .state
            .security()
            .authenticate("jane", "jane-password-2")
            .await,
        Err(AuthError::SecondFactor(pending)) if pending.gate == Gate::Verify
    ));
}

#[tokio::test]
async fn directory_users_are_never_gated() {
    let fixture = Fixture::new().await;
    let directory = LdapTestServer::start().await;
    fixture
        .state
        .security()
        .ldap()
        .set_tls(TlsClient::with_roots(vec![directory.ca()]));
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/ldap",
            Auth::Admin,
            Some(json!({
                "name": "corp", "protocol": "ldap", "host": "127.0.0.1", "port": directory.port,
                "searchBase": BASE, "authScheme": "SIMPLE", "authUsername": SYSTEM_DN,
                "authPassword": SYSTEM_PASSWORD, "connectionTimeoutSeconds": 5,
                "userBaseDn": "ou=people", "userSubtree": true, "userObjectClass": "inetOrgPerson",
                "userIdAttribute": "uid", "userRealNameAttribute": "cn",
                "userEmailAddressAttribute": "mail", "ldapGroupsAsRoles": true,
                "groupType": "dynamic", "userMemberOfAttribute": "memberOf"
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    astrum_db::setting::put(
        fixture.state.db().conn(),
        POLICY_KEY,
        &json!({"required": {"everyone": true}, "graceDays": 0}),
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    assert_eq!(
        basic(&fixture, "bob", "bob-secret").await.status,
        StatusCode::OK
    );
    assert_eq!(
        basic(&fixture, "admin", &fixture.admin_password)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
