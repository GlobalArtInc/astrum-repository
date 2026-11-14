mod support;

use astrum_security::totp::{self, TotpSecret};
use axum::http::{Method, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use support::two_factor::*;
use support::{Auth, Fixture, Reply};

const BEGIN: &str = "/service/rest/internal/ui/user/two-factor/totp";
const CONFIRM: &str = "/service/rest/internal/ui/user/two-factor/totp/confirm";
const CODES: &str = "/service/rest/internal/ui/user/two-factor/recovery-codes";
const PASSWORD: &str = "/service/rest/internal/ui/user/password";

async fn require_enrollment(fixture: &Fixture, user: &str) {
    execute(
        fixture,
        &format!(
            "INSERT INTO user_second_factor (user_id, requirement, enroll_by) \
             VALUES ('{user}', 'required', now() - interval '1 second') \
             ON CONFLICT (user_id) DO UPDATE SET requirement = 'required', enroll_by = now() - interval '1 second'"
        ),
    )
    .await;
    fixture.state.security().changed().await.unwrap();
}

async fn act(fixture: &Fixture, method: Method, path: &str, login: &Reply, body: Value) -> Reply {
    with_session(fixture, method, path, login, Some(body)).await
}

async fn begin(fixture: &Fixture, login: &Reply, secret: &str) -> Reply {
    act(
        fixture,
        Method::POST,
        BEGIN,
        login,
        json!({"currentPassword": secret}),
    )
    .await
}

fn code_for(secret: &str, steps_ahead: i64) -> String {
    let secret = TotpSecret::from_base32(secret).unwrap();
    let step = totp::step_at(u64::try_from(Utc::now().timestamp()).unwrap());
    totp::code_at(&secret, step.checked_add_signed(steps_ahead).unwrap())
}

async fn confirm(fixture: &Fixture, login: &Reply, began: &Value, secret: &str) -> Reply {
    act(
        fixture,
        Method::POST,
        CONFIRM,
        login,
        json!({
            "ceremony": began["ceremony"],
            "code": code_for(began["secret"].as_str().unwrap(), 0),
            "currentPassword": secret,
        }),
    )
    .await
}

async fn enroll_through_api(fixture: &Fixture, login: &Reply, secret: &str) -> (Value, Value) {
    let began = begin(fixture, login, secret).await;
    assert_eq!(began.status, StatusCode::OK, "{}", began.text());
    let began = began.json();
    let done = confirm(fixture, login, &began, secret).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    (began, done.json())
}

async fn status(fixture: &Fixture, login: &Reply) -> Value {
    let reply = with_session(fixture, Method::GET, FACTORS, login, None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

fn field(reply: &Reply, id: &str) {
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert_eq!(reply.json()[0]["id"], id, "{}", reply.text());
}

async fn enrolled_in_database(fixture: &Fixture, user: &str) -> i64 {
    number(
        fixture,
        &format!("SELECT count(*) FROM user_second_factor WHERE user_id = '{user}' AND totp_secret IS NOT NULL"),
    )
    .await
}

#[tokio::test]
async fn enrolling_returns_the_secret_once_and_the_codes_with_the_first_factor() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let before = status(&fixture, &login).await;
    assert_eq!(before["enrolled"], false);
    assert_eq!(before["methods"], json!(["totp"]));
    assert_eq!(before["restricted"], false);

    let began = begin(&fixture, &login, JANE).await;
    assert_eq!(began.status, StatusCode::OK, "{}", began.text());
    assert_eq!(began.headers[header::CACHE_CONTROL], "no-store");
    let began = began.json();
    let secret = began["secret"].as_str().unwrap();
    let uri = began["otpauthUri"].as_str().unwrap();
    assert!(
        uri.starts_with("otpauth://totp/Astrum:jane?secret="),
        "{uri}"
    );
    assert!(uri.contains(&format!("secret={secret}")));
    assert!(uri.contains("&issuer=Astrum&"));
    assert!(began["qrSvg"].as_str().unwrap().contains("<svg"));

    let done = confirm(&fixture, &login, &began, JANE).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    assert_eq!(done.headers[header::CACHE_CONTROL], "no-store");
    let codes: Vec<String> = done.json()["recoveryCodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(codes.len(), 10);
    assert!(codes.iter().all(|code| code.len() == 11));

    let after = with_session(&fixture, Method::GET, FACTORS, &login, None).await;
    let text = after.text();
    let after = after.json();
    assert_eq!(after["enrolled"], true);
    assert_eq!(after["totp"], true);
    assert_eq!(after["recoveryCodesLeft"], 10);
    assert!(!text.contains(secret));
    assert!(codes.iter().all(|code| !text.contains(code)));

    let events = events(&fixture, "security.twofactor").await;
    let kinds: Vec<&str> = events
        .iter()
        .map(|event| event["type"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["totp.enrolled", "recovery.generated"]);
    let logged = serde_json::to_string(&events).unwrap();
    assert!(!logged.contains(secret));
    assert!(!logged.contains(began["ceremony"].as_str().unwrap()));
    assert!(codes.iter().all(|code| !logged.contains(code)));
    assert_eq!(events[0]["attributes"]["userId"], "jane");

    let again = begin(&fixture, &login, JANE).await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST, "{}", again.text());
}

#[tokio::test]
async fn every_change_needs_the_current_password_and_names_the_field() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE).await.json();
    let code = code_for(began["secret"].as_str().unwrap(), 0);
    let wrong = act(
        &fixture,
        Method::POST,
        CONFIRM,
        &login,
        json!({"ceremony": began["ceremony"], "code": code, "currentPassword": "not-the-password"}),
    )
    .await;
    field(&wrong, "currentPassword");
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);
    let missing = act(
        &fixture,
        Method::POST,
        CONFIRM,
        &login,
        json!({"ceremony": began["ceremony"], "code": code}),
    )
    .await;
    field(&missing, "currentPassword");
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);

    for (method, path) in [
        (Method::POST, BEGIN),
        (Method::DELETE, BEGIN),
        (Method::POST, CODES),
    ] {
        let reply = act(
            &fixture,
            method.clone(),
            path,
            &login,
            json!({"currentPassword": "not-the-password"}),
        )
        .await;
        field(&reply, "currentPassword");
    }
    let old_name = act(
        &fixture,
        Method::POST,
        BEGIN,
        &login,
        json!({"password": JANE}),
    )
    .await;
    assert_eq!(old_name.status, StatusCode::OK, "{}", old_name.text());
}

#[tokio::test]
async fn wrong_passwords_count_in_the_password_change_throttle() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    for _ in 0..5 {
        let reply = act(
            &fixture,
            Method::PUT,
            PASSWORD,
            &login,
            json!({"currentPassword": "nope", "newPassword": "another-password-9"}),
        )
        .await;
        field(&reply, "currentPassword");
    }
    for _ in 0..5 {
        field(&begin(&fixture, &login, "nope").await, "currentPassword");
    }
    for (method, path) in [
        (Method::POST, BEGIN),
        (Method::DELETE, BEGIN),
        (Method::POST, CODES),
    ] {
        let blocked = act(
            &fixture,
            method,
            path,
            &login,
            json!({"currentPassword": JANE}),
        )
        .await;
        assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    }
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);
}

#[tokio::test]
async fn access_tokens_and_anonymous_callers_cannot_manage_factors() {
    let fixture = world().await;
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Basic("jane", JANE),
            Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let token = created.json()["token"].as_str().unwrap().to_owned();
    let bearer = format!("Bearer {token}");
    for (method, path) in [
        (Method::GET, FACTORS),
        (Method::POST, BEGIN),
        (Method::POST, CONFIRM),
        (Method::DELETE, BEGIN),
        (Method::POST, CODES),
    ] {
        let reply = fixture
            .send(
                method.clone(),
                path,
                Auth::None,
                Some(json!({"currentPassword": JANE, "ceremony": "x", "code": "123456"})),
                &[("authorization", &bearer)],
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        let anonymous = fixture
            .send(
                method.clone(),
                path,
                Auth::None,
                Some(json!({"currentPassword": JANE})),
                &[],
            )
            .await;
        assert_eq!(
            anonymous.status,
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
    }
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        0
    );
}

#[tokio::test]
async fn users_of_a_directory_are_refused() {
    let fixture = world().await;
    let realms = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!([
                "AstrumAuthenticatingRealm",
                "AccessTokenRealm",
                "OAuth2Realm"
            ])),
        )
        .await;
    assert_eq!(realms.status, StatusCode::NO_CONTENT, "{}", realms.text());
    execute(
        &fixture,
        "INSERT INTO security_oidc_user (user_id, subject) VALUES ('rita', 'u-1')",
    )
    .await;
    let security = fixture.state.security();
    security.changed().await.unwrap();
    let subject = security.model().oidc_subject("rita").unwrap();
    let session = security
        .open_session(&subject, "", "", false)
        .await
        .unwrap();
    let cookie = format!(
        "astrum_session={}; astrum_csrf={}",
        session.token, session.csrf
    );
    for (method, path) in [(Method::GET, FACTORS), (Method::POST, BEGIN)] {
        let reply = fixture
            .send(
                method,
                path,
                Auth::None,
                Some(json!({"currentPassword": "x"})),
                &[
                    ("cookie", &cookie),
                    ("x-astrum-csrf", &session.csrf),
                    ("sec-fetch-site", "same-origin"),
                ],
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert!(reply.text().contains("managed there"));
    }
}

#[tokio::test]
async fn enrolling_closes_the_other_sessions_and_keeps_the_tokens() {
    let fixture = world().await;
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Basic("jane", JANE),
            Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let mine = password(&fixture, "jane", JANE, &[]).await;
    let other = password(&fixture, "jane", JANE, &[]).await;
    let pats = password(&fixture, "pat", PAT, &[]).await;
    assert_eq!(status(&fixture, &mine).await["accessTokens"], 1);
    enroll_through_api(&fixture, &mine, JANE).await;
    assert_eq!(status(&fixture, &mine).await["accessTokens"], 1);
    let gone = with_session(&fixture, Method::GET, SESSION, &other, None).await;
    assert_eq!(gone.json()["authenticated"], false);
    let kept = with_session(&fixture, Method::GET, SESSION, &mine, None).await;
    assert_eq!(kept.json()["authenticated"], true);
    let untouched = with_session(&fixture, Method::GET, SESSION, &pats, None).await;
    assert_eq!(untouched.json()["authenticated"], true);
    execute(
        &fixture,
        "UPDATE access_token SET expires_at = now() - interval '1 minute'",
    )
    .await;
    assert_eq!(status(&fixture, &mine).await["accessTokens"], 0);
}

#[tokio::test]
async fn a_restricted_session_enrolls_becomes_normal_and_closes_the_others() {
    let fixture = world().await;
    let earlier = password(&fixture, "pat", PAT, &[]).await;
    require_enrollment(&fixture, "pat").await;
    let login = password(&fixture, "pat", PAT, &[]).await;
    assert_eq!(login.json()["secondFactor"], json!({"restricted": true}));

    let tokens = "/service/rest/internal/access-tokens";
    let shut = with_session(&fixture, Method::GET, tokens, &login, None).await;
    assert_eq!(shut.status, StatusCode::FORBIDDEN);
    let before = status(&fixture, &login).await;
    assert_eq!(before["restricted"], true);
    assert_eq!(before["required"], true);
    assert!(before["enrollBy"].is_string());
    for (method, path) in [
        (Method::DELETE, BEGIN),
        (Method::POST, CODES),
        (Method::POST, CONFIRM),
    ] {
        let reply = act(
            &fixture,
            method.clone(),
            path,
            &login,
            json!({"currentPassword": PAT, "ceremony": "x", "code": "123456"}),
        )
        .await;
        assert_ne!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        assert_ne!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }

    enroll_through_api(&fixture, &login, PAT).await;
    let open = with_session(&fixture, Method::GET, tokens, &login, None).await;
    assert_eq!(open.status, StatusCode::OK, "{}", open.text());
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE restricted"
        )
        .await,
        0
    );
    assert_eq!(status(&fixture, &login).await["restricted"], false);
    let closed = with_session(&fixture, Method::GET, SESSION, &earlier, None).await;
    assert_eq!(closed.json()["authenticated"], false);
}

#[tokio::test]
async fn a_later_factor_brings_no_codes_and_closes_nothing() {
    let fixture = world().await;
    let mine = password(&fixture, "jane", JANE, &[]).await;
    let other = password(&fixture, "jane", JANE, &[]).await;
    execute(
        &fixture,
        "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
         VALUES (gen_random_uuid(), 'jane', 'key', '\\x01', 'localhost', '{}', false, false)",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let (_, done) = enroll_through_api(&fixture, &mine, JANE).await;
    assert!(done.get("recoveryCodes").is_none());
    assert_eq!(status(&fixture, &mine).await["recoveryCodesLeft"], 0);
    let still = with_session(&fixture, Method::GET, SESSION, &other, None).await;
    assert_eq!(still.json()["authenticated"], true);
}

#[tokio::test]
async fn a_drifting_clock_gets_a_hint_and_is_not_enrolled() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE).await.json();
    let secret = began["secret"].as_str().unwrap();
    let drifting = act(
        &fixture,
        Method::POST,
        CONFIRM,
        &login,
        json!({"ceremony": began["ceremony"], "code": code_for(secret, 5), "currentPassword": JANE}),
    )
    .await;
    field(&drifting, "code");
    assert!(
        drifting.text().contains("minutes off"),
        "{}",
        drifting.text()
    );
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);
    let right = confirm(&fixture, &login, &began, JANE).await;
    assert_eq!(right.status, StatusCode::OK, "{}", right.text());
}

#[tokio::test]
async fn a_ceremony_of_another_user_or_an_old_one_is_refused() {
    let fixture = world().await;
    let jane = password(&fixture, "jane", JANE, &[]).await;
    let pat = password(&fixture, "pat", PAT, &[]).await;
    let began = begin(&fixture, &pat, PAT).await.json();
    let stolen = confirm(&fixture, &jane, &began, JANE).await;
    field(&stolen, "ceremony");
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);
    execute(
        &fixture,
        "UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'",
    )
    .await;
    let old = confirm(&fixture, &pat, &began, PAT).await;
    field(&old, "ceremony");
    assert_eq!(enrolled_in_database(&fixture, "pat").await, 0);
}

#[tokio::test]
async fn the_last_factor_stays_while_required_and_goes_otherwise() {
    let fixture = world().await;
    let jane_factor = enroll(&fixture, "jane").await;
    let pat_factor = enroll(&fixture, "pat").await;
    let jane = with_cookie_login(&fixture, "jane", JANE, &jane_factor).await;
    let pat = with_cookie_login(&fixture, "pat", PAT, &pat_factor).await;
    execute(
        &fixture,
        "UPDATE user_second_factor SET requirement = 'required' WHERE user_id = 'jane'",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let kept = act(
        &fixture,
        Method::DELETE,
        BEGIN,
        &jane,
        json!({"currentPassword": JANE}),
    )
    .await;
    assert_eq!(kept.status, StatusCode::BAD_REQUEST, "{}", kept.text());
    assert!(kept.text().contains("required"));
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 1);

    let removed = act(
        &fixture,
        Method::DELETE,
        BEGIN,
        &pat,
        json!({"currentPassword": PAT}),
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let after = status(&fixture, &pat).await;
    assert_eq!(after["enrolled"], false);
    assert_eq!(after["recoveryCodesLeft"], 0);
    assert!(
        events(&fixture, "security.twofactor")
            .await
            .iter()
            .any(|event| event["type"] == "totp.removed" && event["attributes"]["userId"] == "pat")
    );
}

async fn with_cookie_login(fixture: &Fixture, user: &str, secret: &str, factor: &Factor) -> Reply {
    let handle = start(fixture, user, secret).await;
    let reply = step(fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply
}

#[tokio::test]
async fn regenerating_kills_the_old_set() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let login = with_cookie_login(&fixture, "jane", JANE, &factor).await;
    let fresh = act(
        &fixture,
        Method::POST,
        CODES,
        &login,
        json!({"currentPassword": JANE}),
    )
    .await;
    assert_eq!(fresh.status, StatusCode::OK, "{}", fresh.text());
    assert_eq!(fresh.headers[header::CACHE_CONTROL], "no-store");
    let new: Vec<String> = fresh.json()["recoveryCodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(new.len(), 10);
    assert!(new.iter().all(|code| !factor.codes.contains(code)));
    assert_eq!(status(&fixture, &login).await["recoveryCodesLeft"], 10);

    let handle = start(&fixture, "jane", JANE).await;
    let old = step(&fixture, &handle, "recovery", &factor.codes[0]).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&old), "second-factor-invalid");
    let good = step(&fixture, &handle, "recovery", &new[0]).await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.text());

    let nobody = password(&fixture, "pat", PAT, &[]).await;
    let none = act(
        &fixture,
        Method::POST,
        CODES,
        &nobody,
        json!({"currentPassword": PAT}),
    )
    .await;
    assert_eq!(none.status, StatusCode::BAD_REQUEST);
    assert!(
        events(&fixture, "security.twofactor")
            .await
            .iter()
            .any(|event| event["type"] == "recovery.regenerated")
    );
}

#[tokio::test]
async fn the_status_reports_the_lock_and_the_codes_left() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let login = with_cookie_login(&fixture, "jane", JANE, &factor).await;
    let open = status(&fixture, &login).await;
    assert_eq!(open["lockedUntil"], Value::Null);
    execute(
        &fixture,
        "UPDATE user_second_factor SET locked_until = now() + interval '10 minutes' WHERE user_id = 'jane'",
    )
    .await;
    let report = status(&fixture, &login).await;
    assert!(report["lockedUntil"].is_string());
    assert_eq!(report["recoveryCodesLeft"], 10);
    assert_eq!(report["requirementSource"], "policy");
    assert_eq!(report["required"], false);
    assert_eq!(report["enrollBy"], Value::Null);
}

#[tokio::test]
async fn failing_to_close_the_other_sessions_still_hands_over_the_codes() {
    let fixture = world().await;
    let mine = password(&fixture, "jane", JANE, &[]).await;
    execute(
        &fixture,
        "CREATE FUNCTION refuse_session_change() RETURNS trigger LANGUAGE plpgsql AS \
         $$ BEGIN RAISE EXCEPTION 'sessions are frozen'; END $$",
    )
    .await;
    execute(
        &fixture,
        "CREATE TRIGGER freeze_sessions BEFORE UPDATE OR DELETE ON user_session \
         FOR EACH ROW EXECUTE FUNCTION refuse_session_change()",
    )
    .await;
    let began = begin(&fixture, &mine, JANE).await.json();
    let done = confirm(&fixture, &mine, &began, JANE).await;
    execute(&fixture, "DROP TRIGGER freeze_sessions ON user_session").await;
    execute(&fixture, "DROP FUNCTION refuse_session_change()").await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    let codes = done.json()["recoveryCodes"].clone();
    assert_eq!(codes.as_array().unwrap().len(), 10);
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 1);
    let handle = start(&fixture, "jane", JANE).await;
    let signed = step(&fixture, &handle, "recovery", codes[0].as_str().unwrap()).await;
    assert_eq!(signed.status, StatusCode::OK, "{}", signed.text());
}

#[tokio::test]
async fn a_failed_confirmation_leaves_the_restricted_session_and_the_others() {
    let fixture = world().await;
    let earlier = password(&fixture, "pat", PAT, &[]).await;
    require_enrollment(&fixture, "pat").await;
    let login = password(&fixture, "pat", PAT, &[]).await;
    let jane = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, PAT).await.json();
    let others = begin(&fixture, &jane, JANE).await.json();
    let body = |ceremony: &Value, code: String, secret: &str| json!({"ceremony": ceremony, "code": code, "currentPassword": secret});
    let right = code_for(began["secret"].as_str().unwrap(), 0);
    let wrong_code = body(
        &began["ceremony"],
        code_for(began["secret"].as_str().unwrap(), 5),
        PAT,
    );
    let wrong_password = body(&began["ceremony"], right.clone(), "not-the-password");
    let foreign = body(&others["ceremony"], right, PAT);
    for attempt in [wrong_code, wrong_password, foreign] {
        let reply = act(&fixture, Method::POST, CONFIRM, &login, attempt).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    }
    assert_eq!(enrolled_in_database(&fixture, "pat").await, 0);
    assert_eq!(status(&fixture, &login).await["restricted"], true);
    let alive = with_session(&fixture, Method::GET, SESSION, &earlier, None).await;
    assert_eq!(alive.json()["authenticated"], true);
    let open = number(
        &fixture,
        "SELECT count(*) FROM user_session WHERE user_id = 'pat'",
    )
    .await;
    assert_eq!(open, 2);
}

#[tokio::test]
async fn a_wrong_password_does_not_use_up_a_ceremony_attempt() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE).await.json();
    for _ in 0..6 {
        let reply = act(
            &fixture,
            Method::POST,
            CONFIRM,
            &login,
            json!({"ceremony": began["ceremony"], "code": "000000", "currentPassword": "nope"}),
        )
        .await;
        field(&reply, "currentPassword");
    }
    assert_eq!(
        number(&fixture, "SELECT attempts FROM second_factor_ceremony").await,
        0
    );
}

#[tokio::test]
async fn confirming_is_refused_once_authenticator_apps_are_turned_off() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE).await.json();
    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::two_factor::POLICY_KEY,
        &json!({"totp": false}),
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let reply = confirm(&fixture, &login, &began, JANE).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert!(reply.text().contains("turned off"), "{}", reply.text());
    assert_eq!(enrolled_in_database(&fixture, "jane").await, 0);
}

#[tokio::test]
async fn wrong_passwords_and_the_throttle_leave_audit_rows_without_the_password() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    field(
        &begin(&fixture, &login, "wrong-secret-1").await,
        "currentPassword",
    );
    let failed = events(&fixture, "security.user").await;
    let failed: Vec<&Value> = failed
        .iter()
        .filter(|event| event["type"] == "authentication")
        .collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["attributes"]["wasSuccessful"], false);
    assert_eq!(failed[0]["attributes"]["userId"], "jane");
    assert_eq!(failed[0]["attributes"]["path"], BEGIN);
    assert!(
        !serde_json::to_string(failed[0])
            .unwrap()
            .contains("wrong-secret-1")
    );
    assert!(events(&fixture, "security.auth.ratelimit").await.is_empty());
    for _ in 0..9 {
        field(
            &begin(&fixture, &login, "wrong-secret-1").await,
            "currentPassword",
        );
    }
    let limited = events(&fixture, "security.auth.ratelimit").await;
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0]["attributes"]["username"], "jane");
}

#[tokio::test]
async fn passkeys_are_not_offered_while_they_cannot_be_enrolled() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let both = status(&fixture, &login).await;
    assert_eq!(both["methods"], json!(["totp"]));

    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::two_factor::POLICY_KEY,
        &json!({"totp": false, "passkeys": true}),
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let passkeys_only = status(&fixture, &login).await;
    assert_eq!(passkeys_only["methods"], json!([]));
}
