mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::two_factor::*;
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const HOSTED: &str = "/repository/npm-hosted";
const WRONG_PASSWORD: &str = "the user name or password is wrong";
const DIRECTORY_ACCOUNT: &str =
    "this account signs in through a directory; use an access token instead";
const BOB: &str = "bob-local-password-1";

async fn ready() -> Fixture {
    let fixture = world().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_npm::Npm::default()));
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/npm/hosted",
            Auth::Admin,
            Some(json!({
                "name": "npm-hosted",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"}
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let realms = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!([
                "AstrumAuthenticatingRealm",
                "AccessTokenRealm",
                "NpmToken"
            ])),
        )
        .await;
    assert_eq!(realms.status, StatusCode::NO_CONTENT, "{}", realms.text());
    fixture
}

async fn send(
    fixture: &Fixture,
    method: Method,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
    headers: &[(&str, &str)],
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = bearer {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let body = match body {
        Some(value) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).unwrap())
        }
        None => Body::empty(),
    };
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_npm::routes());
    let response = app.oneshot(request.body(body).unwrap()).await.unwrap();
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

async fn login(fixture: &Fixture, user: &str, password: &str, otp: Option<&str>) -> Reply {
    let body =
        json!({"_id": format!("org.couchdb.user:{user}"), "name": user, "password": password});
    let headers: Vec<(&str, &str)> = otp.map(|code| ("npm-otp", code)).into_iter().collect();
    send(
        fixture,
        Method::PUT,
        &format!("{HOSTED}/-/user/org.couchdb.user:{user}"),
        None,
        Some(body),
        &headers,
    )
    .await
}

type Headers = Vec<(String, Vec<u8>)>;

fn answer(reply: &Reply) -> (StatusCode, Headers, Vec<u8>) {
    let mut headers: Headers = reply
        .headers
        .iter()
        .filter(|(name, _)| *name != header::DATE)
        .map(|(name, value)| (name.to_string(), value.as_bytes().to_vec()))
        .collect();
    headers.sort();
    (reply.status, headers, reply.body.clone())
}

fn challenged(reply: &Reply) -> bool {
    reply
        .headers
        .get(header::WWW_AUTHENTICATE)
        .is_some_and(|value| value == "OTP")
}

fn error_of(reply: &Reply) -> String {
    reply.json()["error"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn a_missing_code_is_asked_for_and_does_not_count_as_a_failure() {
    let fixture = ready().await;
    enroll(&fixture, "jane").await;
    for _ in 0..12 {
        let reply = login(&fixture, "jane", JANE, None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{}", reply.text());
        assert!(challenged(&reply));
        assert_eq!(reply.json()["success"], false);
        assert_eq!(error_of(&reply), "a one-time password is required");
    }
    let blank = login(&fixture, "jane", JANE, Some("  ")).await;
    assert!(challenged(&blank));
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_second_factor WHERE failed_codes > 0"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_right_code_gives_a_token_once_and_the_audit_names_the_factor() {
    let fixture = ready().await;
    let factor = enroll(&fixture, "jane").await;
    let code = factor.code();
    let reply = login(&fixture, "jane", JANE, Some(&code)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let token = reply.json()["token"].as_str().unwrap().to_owned();
    let whoami = send(
        &fixture,
        Method::GET,
        &format!("{HOSTED}/-/whoami"),
        Some(&token),
        None,
        &[],
    )
    .await;
    assert_eq!(whoami.json(), json!({"username": "jane"}));
    let created = events(&fixture, "security.npmtoken").await;
    assert_eq!(created.len(), 1);
    assert_eq!(created[0]["attributes"]["secondFactor"], "totp");

    let again = login(&fixture, "jane", JANE, Some(&code)).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert!(challenged(&again));
    assert_eq!(
        error_of(&again),
        "the one-time password is wrong or was already used"
    );
}

#[tokio::test]
async fn wrong_codes_fail_then_throttle_and_a_lock_holds_for_the_right_code() {
    let fixture = ready().await;
    let factor = enroll(&fixture, "jane").await;
    let wrong = factor.wrong();
    let first = login(&fixture, "jane", JANE, Some(&wrong)).await;
    assert_eq!(first.status, StatusCode::UNAUTHORIZED);
    assert!(challenged(&first));
    let failed = events(&fixture, "security.twofactor").await;
    assert_eq!(failed[0]["type"], "verify.failed");
    assert_eq!(failed[0]["attributes"]["method"], "totp");

    execute(
        &fixture,
        "UPDATE user_second_factor SET locked_until = now() + interval '10 minutes', lock_level = 1",
    )
    .await;
    let locked = login(&fixture, "jane", JANE, Some(&wrong)).await;
    assert_eq!(locked.status, StatusCode::TOO_MANY_REQUESTS);
    let wait: u64 = locked.headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(wait > 0);
    let right = login(&fixture, "jane", JANE, Some(&factor.code())).await;
    assert_eq!(right.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(error_of(&right), "too many wrong codes; wait and try again");

    for _ in 0..10 {
        login(&fixture, "jane", JANE, Some(&wrong)).await;
    }
    let blocked = login(&fixture, "jane", JANE, Some(&wrong)).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn a_recovery_code_works_once() {
    let fixture = ready().await;
    let factor = enroll(&fixture, "jane").await;
    let code = &factor.codes[0];
    let reply = login(&fixture, "jane", JANE, Some(code)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let created = events(&fixture, "security.npmtoken").await;
    assert_eq!(created[0]["attributes"]["secondFactor"], "recovery");
    let used = events(&fixture, "security.twofactor").await;
    assert!(used.iter().any(|event| event["type"] == "recovery.used"));
    let again = login(&fixture, "jane", JANE, Some(code)).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert!(challenged(&again));
}

#[tokio::test]
async fn a_wrong_password_gets_the_old_answer_whatever_the_code() {
    let fixture = ready().await;
    let factor = enroll(&fixture, "jane").await;
    let reply = login(&fixture, "jane", "nope", Some(&factor.code())).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(!challenged(&reply));
    assert_eq!(error_of(&reply), WRONG_PASSWORD);
}

#[tokio::test]
async fn enrolment_and_passkey_only_accounts_are_told_what_to_do() {
    let fixture = ready().await;
    execute(
        &fixture,
        "INSERT INTO user_second_factor (user_id, requirement, enroll_by) \
         VALUES ('pat', 'required', now() - interval '1 second')",
    )
    .await;
    execute(
        &fixture,
        "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
         VALUES (gen_random_uuid(), 'jane', 'key', '\\x01', 'localhost', '{}', false, false)",
    )
    .await;
    fixture.state.security().changed().await.unwrap();

    let enrol = login(&fixture, "pat", PAT, Some("123456")).await;
    assert_eq!(enrol.status, StatusCode::UNAUTHORIZED);
    assert!(!challenged(&enrol));
    assert!(error_of(&enrol).contains("must set up a second factor"));

    let passkeys = login(&fixture, "jane", JANE, Some("123456")).await;
    assert_eq!(passkeys.status, StatusCode::UNAUTHORIZED);
    assert!(!challenged(&passkeys));
    assert!(error_of(&passkeys).contains("passkeys only"));
}

#[tokio::test]
async fn a_user_without_a_second_factor_signs_in_in_one_request() {
    let fixture = ready().await;
    let reply = login(&fixture, "pat", PAT, None).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let created = events(&fixture, "security.npmtoken").await;
    assert!(created[0]["attributes"].get("secondFactor").is_none());
}

#[tokio::test]
async fn a_code_of_another_user_does_not_open_the_account_of_the_body() {
    let fixture = ready().await;
    let jane = enroll(&fixture, "jane").await;
    enroll(&fixture, "pat").await;
    let body = json!({"_id": "org.couchdb.user:pat", "name": "pat", "password": PAT});
    let reply = send(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/-/user/org.couchdb.user:jane"),
        None,
        Some(body),
        &[("npm-otp", &jane.code())],
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{}", reply.text());
    assert!(challenged(&reply));
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM format_token").await,
        0
    );
}

#[tokio::test]
async fn a_wrong_password_looks_the_same_for_every_kind_of_user() {
    let fixture = ready().await;
    let factor = enroll(&fixture, "jane").await;
    let with_factor = login(&fixture, "jane", "nope", None).await;
    let with_code = login(&fixture, "jane", "nope", Some(&factor.code())).await;
    let without_factor = login(&fixture, "pat", "nope", None).await;
    let unknown = login(&fixture, "ghost", "nope", None).await;
    assert_eq!(with_factor.status, StatusCode::UNAUTHORIZED);
    assert_eq!(answer(&with_factor), answer(&with_code));
    assert_eq!(answer(&with_factor), answer(&without_factor));
    assert_eq!(answer(&with_factor), answer(&unknown));
}

#[tokio::test]
async fn a_directory_account_gets_no_token_even_with_the_right_directory_password() {
    let fixture = ready().await;
    let _directory = fixture.directory().await;
    fixture.add_user("bob", BOB, &[]).await;
    enroll(&fixture, "bob").await;
    let shadowed = login(&fixture, "bob", "bob-secret", None).await;
    assert_eq!(
        shadowed.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        shadowed.text()
    );
    assert!(!challenged(&shadowed));
    assert_eq!(error_of(&shadowed), WRONG_PASSWORD);
    for _ in 0..12 {
        let reply = login(&fixture, "alice", "alice-secret", None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{}", reply.text());
        assert!(!challenged(&reply));
        assert_eq!(error_of(&reply), DIRECTORY_ACCOUNT);
    }
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM format_token").await,
        0
    );
    assert!(events(&fixture, "security.npmtoken").await.is_empty());
    let whoami = send(
        &fixture,
        Method::GET,
        &format!("{HOSTED}/-/whoami"),
        None,
        None,
        &[],
    )
    .await;
    assert_eq!(whoami.status, StatusCode::UNAUTHORIZED);
}
