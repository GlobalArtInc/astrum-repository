mod support;

use astrum_api::AppState;
use astrum_security::SecretBox;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::Auth;
use support::two_factor::*;

#[tokio::test]
async fn the_password_alone_gives_only_the_ceremony_cookie() {
    let fixture = world().await;
    enroll(&fixture, "jane").await;
    let reply = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(reply.status, StatusCode::OK);
    let body = reply.json();
    assert_eq!(body["authenticated"], false);
    assert_eq!(body["secondFactor"]["methods"], json!(["totp", "recovery"]));
    assert!(body["secondFactor"]["expiresAt"].is_string());
    let cookies = set_cookies(&reply);
    assert_eq!(cookies.len(), 1);
    let handle = cookie_value(&reply, "astrum_signin").unwrap();
    assert_eq!(
        cookies[0],
        format!(
            "astrum_signin={handle}; Path=/service/rest/internal/session; SameSite=Strict; Max-Age=300; HttpOnly"
        )
    );
    let secure = password(&fixture, "jane", JANE, &[("x-forwarded-proto", "https")]).await;
    assert!(set_cookies(&secure)[0].ends_with("; HttpOnly; Secure"));
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_session").await,
        0
    );
    assert_eq!(
        number(
            &fixture,
            &format!("SELECT count(*) FROM second_factor_ceremony WHERE handle_hash = '{handle}'")
        )
        .await,
        0
    );
    let anonymous = fixture
        .send(
            Method::GET,
            SESSION,
            Auth::None,
            None,
            &[("cookie", &format!("astrum_signin={handle}"))],
        )
        .await;
    assert_eq!(anonymous.json()["authenticated"], false);
}

#[tokio::test]
async fn a_right_code_opens_the_session_audits_the_login_and_the_cookie_works_once() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let handle = start(&fixture, "jane", JANE).await;
    let reply = step(&fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["authenticated"], true);
    assert_eq!(reply.json()["secondFactor"], json!({"restricted": false}));
    assert!(
        set_cookies(&reply)
            .iter()
            .any(|cookie| cookie.starts_with("astrum_signin=;") && cookie.contains("Max-Age=0"))
    );
    let account = with_session(&fixture, Method::GET, ACCOUNT, &reply, None).await;
    assert_eq!(account.status, StatusCode::OK);
    let logins = events(&fixture, REALM).await;
    let login = logins
        .iter()
        .find(|event| event["type"] == "login")
        .unwrap();
    assert_eq!(login["attributes"]["secondFactor"], "totp");
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        0
    );

    execute(&fixture, "UPDATE user_second_factor SET totp_last_step = 0").await;
    let replay = step(&fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&replay), "sign-in-expired");
    assert!(cookie_value(&replay, "astrum_session").is_none());
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_session").await,
        1
    );
}

#[tokio::test]
async fn five_wrong_codes_end_the_ceremony() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let handle = start(&fixture, "jane", JANE).await;
    for _ in 0..4 {
        let reply = step(&fixture, &handle, "totp", &factor.wrong()).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(error_code(&reply), "second-factor-invalid");
    }
    let last = step(&fixture, &handle, "totp", &factor.wrong()).await;
    assert_eq!(error_code(&last), "sign-in-expired");
    assert!(set_cookies(&last)[0].contains("Max-Age=0"));
    let late = step(&fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(error_code(&late), "sign-in-expired");
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_session").await,
        0
    );
}

#[tokio::test]
async fn another_users_ceremony_cookie_is_refused() {
    let fixture = world().await;
    let jane = enroll(&fixture, "jane").await;
    let pat = enroll(&fixture, "pat").await;
    let jane_handle = start(&fixture, "jane", JANE).await;
    let pat_handle = start(&fixture, "pat", PAT).await;

    let crossed = step(&fixture, &jane_handle, "totp", &pat.code()).await;
    assert_eq!(error_code(&crossed), "second-factor-invalid");
    assert_eq!(
        number(
            &fixture,
            "SELECT attempts FROM second_factor_ceremony WHERE user_id = 'pat'"
        )
        .await,
        0
    );
    let own = step(&fixture, &pat_handle, "totp", &pat.code()).await;
    assert_eq!(own.status, StatusCode::OK);
    let jane_own = step(&fixture, &jane_handle, "totp", &jane.code()).await;
    assert_eq!(jane_own.status, StatusCode::OK);
    let reused = step(&fixture, &pat_handle, "totp", &pat.code()).await;
    assert_eq!(error_code(&reused), "sign-in-expired");
}

#[tokio::test]
async fn the_ceremony_of_an_earlier_password_is_replaced_by_the_next_one() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    enroll(&fixture, "pat").await;
    let pats = start(&fixture, "pat", PAT).await;
    let first = start(&fixture, "jane", JANE).await;
    let mut last = first.clone();
    for _ in 0..9 {
        last = start(&fixture, "jane", JANE).await;
    }
    assert_ne!(first, last);
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM second_factor_ceremony WHERE user_id = 'jane' AND kind = 'sign-in'"
        )
        .await,
        1
    );
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        2
    );
    let old = step(&fixture, &first, "totp", &factor.code()).await;
    assert_eq!(error_code(&old), "sign-in-expired");
    let fresh = step(&fixture, &last, "totp", &factor.code()).await;
    assert_eq!(fresh.status, StatusCode::OK, "{}", fresh.text());
    let other = step(&fixture, &pats, "recovery", "aaaaa-bbbbb").await;
    assert_eq!(error_code(&other), "second-factor-invalid");
}

#[tokio::test]
async fn a_recovery_code_works_once() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let code = factor.codes[0].clone();
    let handle = start(&fixture, "jane", JANE).await;
    let reply = step(&fixture, &handle, "recovery", &code.to_uppercase()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let login = events(&fixture, REALM).await;
    assert_eq!(
        login.iter().find(|event| event["type"] == "login").unwrap()["attributes"]["secondFactor"],
        "recovery"
    );
    let handle = start(&fixture, "jane", JANE).await;
    let again = step(&fixture, &handle, "recovery", &code).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&again), "second-factor-invalid");
    let other = step(&fixture, &handle, "recovery", &factor.codes[1]).await;
    assert_eq!(other.status, StatusCode::OK);
}

#[tokio::test]
async fn a_second_node_finishes_a_ceremony_the_first_began() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let second = AppState::load(
        fixture.test.connect().await,
        fixture.data.path().to_owned(),
        SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap();
    let handle = start(&fixture, "jane", JANE).await;
    let reply = step_on(&second, Some(&handle), "totp", &factor.code()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["authenticated"], true);
    let replay = step_on(&fixture.state, Some(&handle), "totp", &factor.code()).await;
    assert_eq!(error_code(&replay), "sign-in-expired");
}

#[tokio::test]
async fn a_wrong_code_counts_in_the_sign_in_throttle_and_so_does_a_recovery_code() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let first = start(&fixture, "jane", JANE).await;
    for _ in 0..5 {
        step(&fixture, &first, "recovery", "aaaaa-bbbbb").await;
    }
    let second = start(&fixture, "jane", JANE).await;
    for _ in 0..4 {
        step(&fixture, &second, "recovery", "aaaaa-bbbbb").await;
    }
    let wrong = password(&fixture, "jane", "wrong", &[]).await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let held = step(&fixture, &second, "totp", &factor.code()).await;
    assert_eq!(held.status, StatusCode::TOO_MANY_REQUESTS);
    let blocked = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_session").await,
        0
    );
}

#[tokio::test]
async fn ten_wrong_totp_codes_count_in_the_throttle_and_lock_the_factor_in_the_audit() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    for _ in 0..2 {
        let handle = start(&fixture, "jane", JANE).await;
        for _ in 0..5 {
            step(&fixture, &handle, "totp", &factor.wrong()).await;
        }
    }
    assert_eq!(
        password(&fixture, "jane", JANE, &[]).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    let found = events(&fixture, "security.twofactor").await;
    assert_eq!(
        found
            .iter()
            .filter(|event| event["type"] == "verify.failed")
            .count(),
        10
    );
    let locked: Vec<&Value> = found
        .iter()
        .filter(|event| event["type"] == "locked")
        .collect();
    assert_eq!(locked.len(), 1);
    assert!(locked[0]["attributes"]["until"].is_string());
}

#[tokio::test]
async fn a_locked_factor_answers_uniformly_and_a_recovery_code_lifts_it() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    execute(
        &fixture,
        "UPDATE user_second_factor SET locked_until = now() + interval '10 minutes', lock_level = 1",
    )
    .await;
    let handle = start(&fixture, "jane", JANE).await;
    let right = step(&fixture, &handle, "totp", &factor.code()).await;
    let wrong = step(&fixture, &handle, "totp", &factor.wrong()).await;
    assert_eq!(right.status, StatusCode::TOO_MANY_REQUESTS);
    let wait: u64 = right.headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(wait > 0);
    assert_eq!(right.status, wrong.status);
    assert_eq!(right.body, wrong.body);
    assert!(!String::from_utf8_lossy(&right.body).contains("locked"));
    let lifted = step(&fixture, &handle, "recovery", &factor.codes[0]).await;
    assert_eq!(lifted.status, StatusCode::OK, "{}", lifted.text());
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_second_factor WHERE locked_until IS NULL"
        )
        .await,
        1
    );
    let again = start(&fixture, "jane", JANE).await;
    let next = step(&fixture, &again, "totp", &factor.code()).await;
    assert_eq!(next.status, StatusCode::OK, "{}", next.text());
}

#[tokio::test]
async fn refusals_are_uniform() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let handle = start(&fixture, "jane", JANE).await;
    let wrong_totp = step(&fixture, &handle, "totp", &factor.wrong()).await;
    let wrong_recovery = step(&fixture, &handle, "recovery", "aaaaa-bbbbb").await;
    let malformed = step(&fixture, &handle, "recovery", "not a code").await;
    let empty = step(&fixture, &handle, "totp", "").await;
    same_answer(&wrong_totp, &wrong_recovery);
    same_answer(&wrong_totp, &malformed);
    same_answer(&wrong_totp, &empty);
    assert_eq!(wrong_totp.status, StatusCode::UNAUTHORIZED);

    let unknown = step(&fixture, "f".repeat(64).as_str(), "totp", &factor.code()).await;
    let missing = step_on(&fixture.state, None, "totp", &factor.code()).await;
    let long = step(&fixture, &"f".repeat(500), "totp", &factor.code()).await;
    execute(
        &fixture,
        "UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'",
    )
    .await;
    let expired = step(&fixture, &handle, "totp", &factor.code()).await;
    same_answer(&unknown, &missing);
    same_answer(&unknown, &long);
    same_answer(&unknown, &expired);
    assert_eq!(error_code(&unknown), "sign-in-expired");
    assert!(!String::from_utf8_lossy(&unknown.body).contains("jane"));
}

#[tokio::test]
async fn codes_are_trimmed_and_oversized_input_is_refused_before_any_work() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let spaced = |code: &str| format!("{} {}", &code[..3], &code[3..]);
    let handle = start(&fixture, "jane", JANE).await;
    let code = factor.code();
    let reply = step(&fixture, &handle, "totp", &spaced(&code)).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());

    execute(&fixture, "UPDATE user_second_factor SET totp_last_step = 0").await;
    let handle = start(&fixture, "jane", JANE).await;
    let padded = step(&fixture, &handle, "totp", &format!("  {code}\t")).await;
    assert_eq!(padded.status, StatusCode::OK, "{}", padded.text());

    execute(&fixture, "UPDATE user_second_factor SET totp_last_step = 0").await;
    let handle = start(&fixture, "jane", JANE).await;
    let every = code.chars().map(String::from).collect::<Vec<_>>().join(" ");
    let all = step(&fixture, &handle, "totp", &every).await;
    assert_eq!(all.status, StatusCode::OK, "{}", all.text());

    let handle = start(&fixture, "jane", JANE).await;
    let long = step(&fixture, &handle, "totp", &"1".repeat(65)).await;
    assert_eq!(long.status, StatusCode::BAD_REQUEST);
    let method = step(&fixture, &handle, "sms", "x").await;
    assert_eq!(method.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        number(&fixture, "SELECT sum(attempts) FROM second_factor_ceremony").await,
        0
    );
}

#[tokio::test]
async fn a_new_password_ends_a_ceremony_that_was_already_begun() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let handle = start(&fixture, "jane", JANE).await;
    let hash = astrum_security::password::hash("jane-password-2");
    execute(
        &fixture,
        &format!("UPDATE security_user SET password_hash = '{hash}' WHERE id = 'jane'"),
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let reply = step(&fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(error_code(&reply), "sign-in-expired");
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_session").await,
        0
    );
}

#[tokio::test]
async fn audit_events_name_the_user_and_hold_no_secrets() {
    let fixture = world().await;
    let factor = enroll(&fixture, "jane").await;
    let recovery = factor.codes[0].clone();
    let wrong_code = factor.wrong();
    let handle = start(&fixture, "jane", JANE).await;
    step(&fixture, &handle, "totp", &wrong_code).await;
    step(&fixture, &handle, "recovery", "aaaaa-bbbbb").await;
    let done = step(&fixture, &handle, "recovery", &recovery).await;
    assert_eq!(done.status, StatusCode::OK);
    execute(
        &fixture,
        "UPDATE user_second_factor SET totp_secret = 'garbage'",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let handle = start(&fixture, "jane", JANE).await;
    step(&fixture, &handle, "totp", &wrong_code).await;

    let found = events(&fixture, "security.twofactor").await;
    let kinds: Vec<&str> = found
        .iter()
        .map(|event| event["type"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "verify.failed",
            "verify.failed",
            "recovery.used",
            "totp.unreadable",
            "verify.failed"
        ]
    );
    assert_eq!(found[0]["attributes"]["method"], "totp");
    assert_eq!(found[1]["attributes"]["method"], "recovery");
    assert_eq!(found[2]["attributes"]["left"], 9);
    assert!(found.iter().all(|event| event["context"] == "jane"));
    let everything = serde_json::to_string(&json!([
        found,
        events(&fixture, REALM).await,
        events(&fixture, "security.auth.ratelimit").await
    ]))
    .unwrap();
    for secret in [
        recovery.as_str(),
        "aaaaa-bbbbb",
        handle.as_str(),
        JANE,
        &factor.secret.to_base32(),
    ] {
        assert!(!everything.contains(secret), "{secret}");
    }
}

#[tokio::test]
async fn the_session_description_names_the_second_factor_state() {
    let fixture = world().await;
    let anonymous = fixture
        .send(Method::GET, SESSION, Auth::None, None, &[])
        .await;
    assert_eq!(anonymous.status, StatusCode::OK);
    assert_eq!(anonymous.json()["secondFactor"], Value::Null);
    let login = password(&fixture, "pat", PAT, &[]).await;
    let signed_in = with_session(&fixture, Method::GET, SESSION, &login, None).await;
    assert_eq!(signed_in.json()["authenticated"], true);
    assert_eq!(
        signed_in.json()["secondFactor"],
        json!({"restricted": false})
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn first_steps_of_different_users_with_expired_ceremonies_never_fail() {
    let fixture = world().await;
    enroll(&fixture, "jane").await;
    enroll(&fixture, "pat").await;
    for _ in 0..40 {
        execute(
            &fixture,
            "INSERT INTO second_factor_ceremony (handle_hash, kind, user_id, realm, expires_at) \
             SELECT md5(random()::text) || u, 'sign-in', u, 'r', now() - interval '1 minute' \
             FROM (VALUES ('jane'), ('pat')) AS users (u)",
        )
        .await;
        let (jane, pat) = tokio::join!(
            password(&fixture, "jane", JANE, &[]),
            password(&fixture, "pat", PAT, &[]),
        );
        assert_eq!(jane.status, StatusCode::OK, "{}", jane.text());
        assert_eq!(pat.status, StatusCode::OK, "{}", pat.text());
    }
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        2
    );
}
