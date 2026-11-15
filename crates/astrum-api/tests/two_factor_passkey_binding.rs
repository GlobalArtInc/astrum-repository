mod passkey_support;
mod support;

use axum::http::{Method, StatusCode};
use passkey_support::*;
use serde_json::json;
use support::two_factor::*;

async fn sessions(fixture: &support::Fixture) -> i64 {
    number(fixture, "SELECT count(*) FROM user_session").await
}

#[tokio::test]
async fn a_sign_in_ceremony_of_one_user_cannot_finish_the_sign_in_of_another() {
    let fixture = passkey_world().await;
    let jane = password(&fixture, "jane", JANE, &[]).await;
    let pat = password(&fixture, "pat", PAT, &[]).await;
    add(&fixture, &jane, &key(1), "Laptop").await;
    add_as(&fixture, &pat, &key(2), "Phone", PAT).await;
    let (jane_handle, _) = challenge(&fixture, "jane", JANE).await;
    let (pat_handle, options) = challenge(&fixture, "pat", PAT).await;
    let assertion = key(2).assert(&options["publicKey"], 1);
    let before = sessions(&fixture).await;

    let crossed = submit(
        &fixture,
        &jane_handle,
        &options["ceremony"],
        assertion.clone(),
    )
    .await;
    assert_eq!(crossed.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&crossed), "second-factor-invalid");
    assert_eq!(sessions(&fixture).await, before);

    let own = submit(&fixture, &pat_handle, &options["ceremony"], assertion).await;
    assert_eq!(own.status, StatusCode::OK, "{}", own.text());
    assert_eq!(own.json()["user"]["userId"], "pat");
}

#[tokio::test]
async fn an_enrolment_ceremony_of_one_user_is_refused_to_another_and_stays_usable() {
    let fixture = passkey_world().await;
    let jane = password(&fixture, "jane", JANE, &[]).await;
    let pat = password(&fixture, "pat", PAT, &[]).await;
    let began = begin(&fixture, &pat, PAT, "Phone").await.json();
    let crossed = register_as(&fixture, &jane, &began, &key(2), JANE).await;
    field(&crossed, "ceremony");
    assert_eq!(passkey_rows(&fixture).await, 0);

    let own = register_as(&fixture, &pat, &began, &key(2), PAT).await;
    assert_eq!(own.status, StatusCode::OK, "{}", own.text());
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_passkey WHERE user_id = 'pat'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn of_two_concurrent_finishes_of_one_ceremony_exactly_one_succeeds() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let began = begin(&fixture, &login, JANE, "Laptop").await.json();
    let signing = key(1);
    let (first, second) = tokio::join!(
        register(&fixture, &login, &began, &signing),
        register(&fixture, &login, &began, &signing)
    );
    let mut statuses = [first.status, second.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::BAD_REQUEST]);
    assert_eq!(passkey_rows(&fixture).await, 1);

    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    let assertion = signing.assert(&options["publicKey"], 1);
    let before = sessions(&fixture).await;
    let (first, second) = tokio::join!(
        submit(&fixture, &handle, &options["ceremony"], assertion.clone()),
        submit(&fixture, &handle, &options["ceremony"], assertion)
    );
    let mut statuses = [first.status, second.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::UNAUTHORIZED]);
    assert_eq!(sessions(&fixture).await, before + 1);
}

#[tokio::test]
async fn a_credential_deleted_between_the_options_and_the_finish_is_refused() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let signing = key(1);
    add(&fixture, &login, &signing, "Laptop").await;
    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    execute(&fixture, "DELETE FROM user_passkey").await;
    fixture.state.security().changed().await.unwrap();
    let before = sessions(&fixture).await;
    let refused = submit(
        &fixture,
        &handle,
        &options["ceremony"],
        signing.assert(&options["publicKey"], 1),
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&refused), "second-factor-invalid");
    assert_eq!(sessions(&fixture).await, before);
}

#[tokio::test]
async fn an_authenticator_that_signs_for_another_origin_is_refused_through_the_api() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let mut evil = key(1);
    evil.origin = "https://evil.example.org".to_owned();
    let began = begin(&fixture, &login, JANE, "Laptop").await.json();
    field(
        &register(&fixture, &login, &began, &evil).await,
        "credential",
    );
    assert_eq!(passkey_rows(&fixture).await, 0);

    add(&fixture, &login, &key(1), "Laptop").await;
    let (handle, options) = challenge(&fixture, "jane", JANE).await;
    let before = sessions(&fixture).await;
    let refused = submit(
        &fixture,
        &handle,
        &options["ceremony"],
        evil.assert(&options["publicKey"], 1),
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
    assert_eq!(sessions(&fixture).await, before);
}

#[tokio::test]
async fn the_last_passkey_stays_while_the_policy_requires_a_second_factor() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let first = add(&fixture, &login, &key(1), "Laptop").await;
    let policy = json!({
        "totp": true,
        "passkeys": true,
        "required": {"everyone": true, "administrators": false, "roles": []},
        "graceDays": 7,
    });
    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::two_factor::POLICY_KEY,
        &policy,
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let path = format!("{PASSKEYS}/{}", first["passkey"]["id"].as_str().unwrap());
    let body = json!({"currentPassword": JANE});
    let kept = act(&fixture, Method::DELETE, &path, &login, body.clone()).await;
    assert_eq!(kept.status, StatusCode::BAD_REQUEST, "{}", kept.text());
    assert_eq!(passkey_rows(&fixture).await, 1);

    add(&fixture, &login, &key(2), "Phone").await;
    let removed = act(&fixture, Method::DELETE, &path, &login, body).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert_eq!(passkey_rows(&fixture).await, 1);
}
