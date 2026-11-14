mod passkey_support;
mod support;

use axum::http::{Method, StatusCode, header};
use passkey_support::*;
use serde_json::{Value, json};
use support::two_factor::*;
use support::{Auth, Fixture};

#[tokio::test]
async fn a_passkey_signs_in_after_the_password() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let signing = key(1);
    add(&fixture, &login, &signing, "Laptop").await;

    let first = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(
        first.json()["secondFactor"]["methods"],
        json!(["passkey", "recovery"])
    );
    let handle = cookie_value(&first, "astrum_signin").unwrap();
    let options = options_for(&fixture, &handle).await;
    assert_eq!(options.status, StatusCode::OK, "{}", options.text());
    assert_eq!(options.headers[header::CACHE_CONTROL], "no-store");
    let options = options.json();
    assert_eq!(options["publicKey"]["rpId"], RP);
    assert_eq!(options["publicKey"]["userVerification"], "required");
    assert_eq!(
        options["publicKey"]["allowCredentials"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(keys(&options), ["ceremony", "publicKey"]);

    let reply = submit(
        &fixture,
        &handle,
        &options["ceremony"],
        signing.assert(&options["publicKey"], 1),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["authenticated"], true);
    assert_eq!(reply.json()["secondFactor"], json!({"restricted": false}));
    let account = with_session(&fixture, Method::GET, ACCOUNT, &reply, None).await;
    assert_eq!(account.status, StatusCode::OK);
    let logins = events(&fixture, REALM).await;
    let login = logins
        .iter()
        .rfind(|event| event["type"] == "login")
        .unwrap();
    assert_eq!(login["attributes"]["secondFactor"], "passkey");
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_passkey WHERE last_used_at IS NOT NULL"
        )
        .await,
        1
    );
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        0
    );
}

#[tokio::test]
async fn a_passkey_works_while_the_authenticator_app_is_locked() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let factor = enroll(&fixture, "jane").await;
    let signing = key(1);
    add(&fixture, &login, &signing, "Laptop").await;
    execute(
        &fixture,
        "UPDATE user_second_factor SET locked_until = now() + interval '10 minutes', lock_level = 1",
    )
    .await;
    let handle = start(&fixture, "jane", JANE).await;
    let locked = step(&fixture, &handle, "totp", &factor.code()).await;
    assert_eq!(locked.status, StatusCode::TOO_MANY_REQUESTS);
    let reply = sign_in(&fixture, &signing, 1).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

#[tokio::test]
async fn a_replayed_assertion_is_refused() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let signing = key(1);
    add(&fixture, &login, &signing, "Laptop").await;

    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    let assertion = signing.assert(&options["publicKey"], 1);
    let ceremony = &options["ceremony"];
    let accepted = submit(&fixture, &handle, ceremony, assertion.clone()).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.text());
    let again = submit(&fixture, &handle, ceremony, assertion.clone()).await;
    assert_eq!(error_code(&again), "sign-in-expired");

    let (second, _) = challenge(&fixture, "jane", JANE).await;
    let replay = submit(&fixture, &second, ceremony, assertion).await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&replay), "second-factor-invalid");
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE user_id = 'jane'"
        )
        .await,
        2
    );
}

#[tokio::test]
async fn a_ceremony_is_single_use_for_enrolment_and_for_sign_in() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE, "Laptop").await.json();
    let done = register(&fixture, &login, &began, &key(1)).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    let reused = register(&fixture, &login, &began, &key(2)).await;
    field(&reused, "ceremony");
    assert_eq!(passkey_rows(&fixture).await, 1);

    let failed = begin(&fixture, &login, JANE, "Phone").await.json();
    let wrong = act(
        &fixture,
        Method::POST,
        PASSKEYS,
        &login,
        json!({"ceremony": failed["ceremony"], "credential": {"id": 1}, "currentPassword": JANE}),
    )
    .await;
    field(&wrong, "credential");
    let right = register(&fixture, &login, &failed, &key(2)).await;
    field(&right, "ceremony");
    assert_eq!(passkey_rows(&fixture).await, 1);

    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    let stranger = key(9).assert(&options["publicKey"], 1);
    let refused = submit(&fixture, &handle, &options["ceremony"], stranger).await;
    assert_eq!(error_code(&refused), "second-factor-invalid");
    let retried = submit(
        &fixture,
        &handle,
        &options["ceremony"],
        key(1).assert(&options["publicKey"], 1),
    )
    .await;
    assert_eq!(error_code(&retried), "second-factor-invalid");
    let no_credential = submit(&fixture, &handle, &options["ceremony"], json!("x")).await;
    field(&no_credential, "credential");
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE user_id = 'jane'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn an_expired_ceremony_is_refused() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let signing = key(1);
    let began = begin(&fixture, &login, JANE, "Laptop").await.json();
    execute(
        &fixture,
        "UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'",
    )
    .await;
    field(
        &register(&fixture, &login, &began, &signing).await,
        "ceremony",
    );
    assert_eq!(passkey_rows(&fixture).await, 0);

    add(&fixture, &login, &signing, "Laptop").await;
    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    execute(
        &fixture,
        "UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second' \
         WHERE state ->> 'session' IS NOT NULL",
    )
    .await;
    let late = submit(
        &fixture,
        &handle,
        &options["ceremony"],
        signing.assert(&options["publicKey"], 1),
    )
    .await;
    assert_eq!(error_code(&late), "second-factor-invalid");
    execute(
        &fixture,
        "UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'",
    )
    .await;
    let gone = options_for(&fixture, &handle).await;
    assert_eq!(error_code(&gone), "sign-in-expired");
}

#[tokio::test]
async fn a_passkey_of_another_relying_party_is_listed_but_not_offered() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    add(&fixture, &login, &key(1), "Laptop").await;
    set_base(&fixture, "https://other.example.org").await;
    let listed = status(&fixture, &login).await;
    assert_eq!(listed["passkeys"][0]["usable"], false);
    let reply = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(reply.json()["secondFactor"]["methods"], json!(["recovery"]));
    let handle = cookie_value(&reply, "astrum_signin").unwrap();
    let options = options_for(&fixture, &handle).await;
    assert_eq!(
        options.status,
        StatusCode::BAD_REQUEST,
        "{}",
        options.text()
    );
}

#[tokio::test]
async fn without_a_relying_party_the_options_answer_conflict() {
    let fixture = world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let refused = begin(&fixture, &login, JANE, "Laptop").await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.text());
    assert!(
        refused.json()["message"]
            .as_str()
            .unwrap()
            .contains("base URL")
    );
    let state = status(&fixture, &login).await;
    assert_eq!(state["passkeysAvailable"], false);
    assert_eq!(state["methods"], json!(["totp"]));
    enroll(&fixture, "pat").await;
    let handle = start(&fixture, "pat", PAT).await;
    let sign = options_for(&fixture, &handle).await;
    assert_eq!(sign.status, StatusCode::CONFLICT, "{}", sign.text());
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM second_factor_ceremony WHERE kind = 'passkey-enroll'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_counter_that_does_not_increase_is_refused_and_audited_but_zero_is_accepted() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let counting = key(1);
    let added = add(&fixture, &login, &counting, "Laptop").await;
    assert_eq!(sign_in(&fixture, &counting, 5).await.status, StatusCode::OK);
    for counter in [5, 3, 0] {
        let refused = sign_in(&fixture, &counting, counter).await;
        assert_eq!(refused.status, StatusCode::UNAUTHORIZED, "{counter}");
        assert_eq!(error_code(&refused), "second-factor-invalid");
    }
    assert_eq!(sign_in(&fixture, &counting, 6).await.status, StatusCode::OK);
    let found = events(&fixture, "security.twofactor").await;
    let failed: Vec<&Value> = found
        .iter()
        .filter(|event| event["type"] == "verify.failed")
        .collect();
    assert_eq!(failed.len(), 3);
    assert!(
        failed
            .iter()
            .all(|event| event["attributes"]["method"] == "passkey")
    );
    let regressions: Vec<&Value> = found
        .iter()
        .filter(|event| event["type"] == "passkey.counter")
        .collect();
    assert_eq!(regressions.len(), 3);
    assert_eq!(
        regressions[0]["attributes"]["passkeyId"],
        added["passkey"]["id"]
    );

    execute(&fixture, "DELETE FROM user_passkey").await;
    fixture.state.security().changed().await.unwrap();
    let silent = key(2);
    add(&fixture, &login, &silent, "Synced").await;
    for _ in 0..2 {
        assert_eq!(sign_in(&fixture, &silent, 0).await.status, StatusCode::OK);
    }
}

#[tokio::test]
async fn a_user_whose_passkeys_all_stopped_working_is_told_to_ask_an_administrator() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    add(&fixture, &login, &key(1), "Laptop").await;
    set_base(&fixture, "https://other.example.org").await;
    execute(&fixture, "DELETE FROM user_recovery_code").await;

    let reply = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    assert_eq!(error_code(&reply), "passkeys-unusable");
    assert!(
        reply.json()["message"]
            .as_str()
            .unwrap()
            .contains("administrator")
    );
    assert!(cookie_value(&reply, "astrum_signin").is_none());
    assert!(cookie_value(&reply, "astrum_session").is_none());
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM second_factor_ceremony").await,
        0
    );

    fixture.state.security().reset("jane").await.unwrap();
    let after = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(after.status, StatusCode::OK, "{}", after.text());
    assert_eq!(after.json()["authenticated"], true);
}

#[tokio::test]
async fn options_calls_are_not_failures_but_refused_assertions_are() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    add(&fixture, &login, &key(1), "Laptop").await;
    let handle = start(&fixture, "jane", JANE).await;
    for _ in 0..11 {
        let options = options_for(&fixture, &handle).await;
        assert_eq!(options.status, StatusCode::OK, "{}", options.text());
    }
    for _ in 0..2 {
        let (handle, options) = challenge(&fixture, "jane", JANE).await;
        let stranger = key(9).assert(&options["publicKey"], 1);
        for _ in 0..5 {
            submit(&fixture, &handle, &options["ceremony"], stranger.clone()).await;
        }
    }
    let blocked = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
}

async fn stranded(fixture: &Fixture) -> Vec<Value> {
    events(fixture, "security.twofactor")
        .await
        .into_iter()
        .filter(|event| event["type"] == "passkey.stranded")
        .collect()
}

#[tokio::test]
async fn changing_the_relying_party_audits_the_passkeys_it_strands_once() {
    let fixture = passkey_world().await;
    let jane = password(&fixture, "jane", JANE, &[]).await;
    let pat = password(&fixture, "pat", PAT, &[]).await;
    add(&fixture, &jane, &key(1), "Laptop").await;
    add_as(&fixture, &pat, &key(2), "Phone", PAT).await;
    let capability = "/service/rest/v1/capabilities/baseurl";
    let save = |url: &'static str| {
        let body = json!({"type": "baseurl", "enabled": true, "properties": {"url": url}});
        fixture.call(Method::PUT, capability, Auth::Admin, Some(body))
    };

    assert_eq!(
        save("https://other.example.org").await.status,
        StatusCode::OK
    );
    let found = stranded(&fixture).await;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["attributes"], json!({"passkeys": 2, "users": 2}));
    assert!(!serde_json::to_string(&found).unwrap().contains("jane"));

    assert_eq!(
        save("https://other.example.org").await.status,
        StatusCode::OK
    );
    assert_eq!(stranded(&fixture).await.len(), 1);
    assert_eq!(save(BASE).await.status, StatusCode::OK);
    assert_eq!(stranded(&fixture).await.len(), 1);

    let policy = json!({
        "totp": true,
        "passkeys": true,
        "required": {"everyone": false, "administrators": false, "roles": []},
        "graceDays": 7,
        "requiredSince": null,
        "relyingParty": {"id": "example.org", "origins": []},
    });
    let put = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/two-factor",
            Auth::Admin,
            Some(policy),
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    assert_eq!(stranded(&fixture).await.len(), 2);

    let removed = fixture
        .call(Method::DELETE, capability, Auth::Admin, None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let found = stranded(&fixture).await;
    assert_eq!(found.len(), 3);
    assert_eq!(found[2]["attributes"], json!({"passkeys": 2, "users": 2}));
}
