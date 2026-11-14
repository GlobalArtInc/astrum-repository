mod passkey_support;
mod support;

use axum::http::{Method, StatusCode, header};
use passkey_support::*;
use serde_json::{Value, json};
use support::Auth;
use support::two_factor::*;

#[tokio::test]
async fn the_first_factor_brings_recovery_codes_and_a_later_one_does_not() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let earlier = password(&fixture, "jane", JANE, &[]).await;
    let before = status(&fixture, &login).await;
    assert_eq!(before["methods"], json!(["totp", "passkey"]));
    assert_eq!(before["passkeysAvailable"], true);

    let began = begin(&fixture, &login, JANE, "Laptop").await;
    assert_eq!(began.status, StatusCode::OK, "{}", began.text());
    assert_eq!(began.headers[header::CACHE_CONTROL], "no-store");
    let began = began.json();
    assert_eq!(keys(&began), ["ceremony", "expiresAt", "publicKey"]);
    assert_eq!(began["publicKey"]["rp"]["id"], RP);
    assert_eq!(
        began["publicKey"]["authenticatorSelection"]["userVerification"],
        "required"
    );
    let first = key(1);
    let done = register(&fixture, &login, &began, &first).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    assert_eq!(done.headers[header::CACHE_CONTROL], "no-store");
    let done = done.json();
    assert_eq!(done["recoveryCodes"].as_array().unwrap().len(), 10);
    assert_eq!(done["passkey"]["name"], "Laptop");

    let after = status(&fixture, &login).await;
    assert_eq!(after["enrolled"], true);
    assert_eq!(after["recoveryCodesLeft"], 10);
    assert_eq!(after["passkeys"][0]["id"], done["passkey"]["id"]);
    assert_eq!(after["passkeys"][0]["usable"], true);
    assert_eq!(after["passkeys"][0]["backedUp"], false);
    assert!(after["passkeys"][0]["lastUsedAt"].is_null());
    let closed = with_session(&fixture, Method::GET, SESSION, &earlier, None).await;
    assert_eq!(closed.json()["authenticated"], false);

    let second = add(&fixture, &login, &key(2), "Phone").await;
    assert!(second.get("recoveryCodes").is_none());
    assert_eq!(
        status(&fixture, &login).await["passkeys"][1]["name"],
        "Phone"
    );

    let events = events(&fixture, "security.twofactor").await;
    assert_eq!(
        kinds(&events),
        ["passkey.enrolled", "recovery.generated", "passkey.enrolled"]
    );
    let logged = serde_json::to_string(&events).unwrap();
    assert!(!logged.contains(began["ceremony"].as_str().unwrap()));
    assert!(!logged.contains(&base64_url(first.credential_id())));
}

#[tokio::test]
async fn a_passkey_after_an_authenticator_app_brings_no_codes() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    enroll(&fixture, "jane").await;
    let done = add(&fixture, &login, &key(1), "Laptop").await;
    assert!(done.get("recoveryCodes").is_none());
    assert_eq!(status(&fixture, &login).await["recoveryCodesLeft"], 10);
}

#[tokio::test]
async fn a_passkey_can_be_renamed_by_its_owner_only() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let added = add(&fixture, &login, &key(1), "Laptop").await;
    let path = format!("{PASSKEYS}/{}", added["passkey"]["id"].as_str().unwrap());
    let renamed = act(
        &fixture,
        Method::PATCH,
        &path,
        &login,
        json!({"name": " Work laptop "}),
    )
    .await;
    assert_eq!(renamed.status, StatusCode::NO_CONTENT, "{}", renamed.text());
    assert_eq!(
        status(&fixture, &login).await["passkeys"][0]["name"],
        "Work laptop"
    );
    for name in [
        json!(""),
        json!("x".repeat(65)),
        json!("a\u{7}b"),
        Value::Null,
    ] {
        let refused = act(
            &fixture,
            Method::PATCH,
            &path,
            &login,
            json!({"name": name}),
        )
        .await;
        field(&refused, "name");
    }
    let missing = format!("{PASSKEYS}/not-a-uuid");
    let unknown = act(
        &fixture,
        Method::PATCH,
        &missing,
        &login,
        json!({"name": "x"}),
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    let pat = password(&fixture, "pat", PAT, &[]).await;
    let taken = act(
        &fixture,
        Method::PATCH,
        &path,
        &pat,
        json!({"name": "mine"}),
    )
    .await;
    assert_eq!(taken.status, StatusCode::NOT_FOUND);
    let removed = act(
        &fixture,
        Method::DELETE,
        &path,
        &pat,
        json!({"currentPassword": PAT}),
    )
    .await;
    assert_eq!(removed.status, StatusCode::NOT_FOUND);
    assert_eq!(passkey_rows(&fixture).await, 1);
    let found = events(&fixture, "security.twofactor").await;
    assert_eq!(
        kinds(&found),
        ["passkey.enrolled", "recovery.generated", "passkey.renamed"]
    );
}

#[tokio::test]
async fn deleting_a_passkey_needs_the_password_and_keeps_the_last_factor_while_required() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let added = add(&fixture, &login, &key(1), "Laptop").await;
    let path = format!("{PASSKEYS}/{}", added["passkey"]["id"].as_str().unwrap());
    let wrong = act(
        &fixture,
        Method::DELETE,
        &path,
        &login,
        json!({"currentPassword": "nope"}),
    )
    .await;
    field(&wrong, "currentPassword");
    assert_eq!(passkey_rows(&fixture).await, 1);

    execute(
        &fixture,
        "UPDATE user_second_factor SET requirement = 'required' WHERE user_id = 'jane'",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let kept = act(
        &fixture,
        Method::DELETE,
        &path,
        &login,
        json!({"currentPassword": JANE}),
    )
    .await;
    assert_eq!(kept.status, StatusCode::BAD_REQUEST, "{}", kept.text());
    assert_eq!(passkey_rows(&fixture).await, 1);

    execute(
        &fixture,
        "UPDATE user_second_factor SET requirement = 'policy' WHERE user_id = 'jane'",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let removed = act(
        &fixture,
        Method::DELETE,
        &path,
        &login,
        json!({"currentPassword": JANE}),
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert_eq!(passkey_rows(&fixture).await, 0);
    assert_eq!(
        number(&fixture, "SELECT count(*) FROM user_recovery_code").await,
        0
    );
    assert_eq!(status(&fixture, &login).await["enrolled"], false);
    let found = events(&fixture, "security.twofactor").await;
    assert_eq!(found.last().unwrap()["type"], "passkey.removed");
}

#[tokio::test]
async fn wrong_passwords_are_throttled_like_those_of_the_authenticator_app() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let added = add(&fixture, &login, &key(1), "Laptop").await;
    for _ in 0..10 {
        field(
            &begin(&fixture, &login, "nope", "Phone").await,
            "currentPassword",
        );
    }
    let path = format!("{PASSKEYS}/{}", added["passkey"]["id"].as_str().unwrap());
    let blocked = begin(&fixture, &login, JANE, "Phone").await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    let delete = act(
        &fixture,
        Method::DELETE,
        &path,
        &login,
        json!({"currentPassword": JANE}),
    )
    .await;
    assert_eq!(delete.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(passkey_rows(&fixture).await, 1);
}

#[tokio::test]
async fn one_credential_is_registered_once_and_at_most_ten_passkeys_exist() {
    let fixture = passkey_world().await;
    let login = password(&fixture, "jane", JANE, &[]).await;
    let pat = password(&fixture, "pat", PAT, &[]).await;
    add(&fixture, &login, &key(1), "Laptop").await;
    let again = begin(&fixture, &login, JANE, "Same").await.json();
    field(
        &register(&fixture, &login, &again, &key(1)).await,
        "credential",
    );
    let other = begin(&fixture, &pat, PAT, "Stolen").await.json();
    let body = json!({
        "ceremony": other["ceremony"],
        "credential": key(1).register(&other["publicKey"], 0),
        "currentPassword": PAT,
    });
    let stolen = act(&fixture, Method::POST, PASSKEYS, &pat, body).await;
    field(&stolen, "credential");
    assert_eq!(passkey_rows(&fixture).await, 1);

    for seed in 2..=10 {
        add(&fixture, &login, &key(seed), "More").await;
    }
    assert_eq!(passkey_rows(&fixture).await, 10);
    let eleventh = begin(&fixture, &login, JANE, "Eleventh").await;
    assert_eq!(
        eleventh.status,
        StatusCode::BAD_REQUEST,
        "{}",
        eleventh.text()
    );
    assert!(eleventh.text().contains("at most 10"));
    assert_eq!(passkey_rows(&fixture).await, 10);
}

#[tokio::test]
async fn tokens_and_anonymous_callers_cannot_manage_passkeys() {
    let fixture = passkey_world().await;
    let created = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("jane", JANE),
            Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let bearer = format!("Bearer {}", created.json()["token"].as_str().unwrap());
    let id = "00000000-0000-4000-8000-000000000001";
    let item = format!("{PASSKEYS}/{id}");
    for (method, path) in [
        (Method::POST, OPTIONS),
        (Method::POST, PASSKEYS),
        (Method::PATCH, item.as_str()),
        (Method::DELETE, item.as_str()),
    ] {
        let body =
            Some(json!({"currentPassword": JANE, "name": "x", "ceremony": "x", "credential": {}}));
        let token = fixture
            .send(
                method.clone(),
                path,
                Auth::None,
                body.clone(),
                &[("authorization", &bearer)],
            )
            .await;
        assert_eq!(token.status, StatusCode::FORBIDDEN, "{method} {path}");
        let anonymous = fixture
            .send(method.clone(), path, Auth::None, body, &[])
            .await;
        assert_eq!(
            anonymous.status,
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
    }
}

#[tokio::test]
async fn passkeys_alone_can_be_required_and_a_restricted_session_enrols_one() {
    let fixture = passkey_world().await;
    let policy = json!({
        "totp": false,
        "passkeys": true,
        "required": {"everyone": false, "administrators": false, "roles": []},
        "graceDays": 7,
        "requiredSince": null,
        "relyingParty": {"id": null, "origins": []},
    });
    let put = fixture
        .call(Method::PUT, POLICY, Auth::Admin, Some(policy))
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    execute(
        &fixture,
        "INSERT INTO user_second_factor (user_id, requirement, enroll_by) \
         VALUES ('jane', 'required', now() - interval '1 second')",
    )
    .await;
    fixture.state.security().changed().await.unwrap();
    let login = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(login.json()["secondFactor"], json!({"restricted": true}));
    let before = status(&fixture, &login).await;
    assert_eq!(before["restricted"], true);
    assert_eq!(before["methods"], json!(["passkey"]));

    let done = add(&fixture, &login, &key(1), "Laptop").await;
    assert_eq!(done["recoveryCodes"].as_array().unwrap().len(), 10);
    let after = status(&fixture, &login).await;
    assert_eq!(after["restricted"], false);
    let tokens = with_session(&fixture, Method::GET, TOKENS, &login, None).await;
    assert_eq!(tokens.status, StatusCode::OK, "{}", tokens.text());

    let again = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(
        again.json()["secondFactor"]["methods"],
        json!(["passkey", "recovery"])
    );
}
