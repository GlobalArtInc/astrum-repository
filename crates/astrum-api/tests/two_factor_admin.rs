mod accounts;
mod factor_admin;
mod support;

use accounts::{USERS, audit, error_ids};
use axum::http::{Method, StatusCode};
use factor_admin::{
    PLAIN, POLICY, READER, UPDATER, people, policy, put_policy, requirement_path, set_base,
    status_path,
};
use serde_json::{Value, json};
use support::Auth;
use support::two_factor::{enroll, execute, number, password};

#[tokio::test]
async fn every_route_checks_the_privilege_it_needs() {
    let fixture = people().await;
    let requirement = Some(json!({"requirement": "exempt"}));
    let calls = [
        (Method::GET, status_path("jane"), None),
        (Method::PUT, requirement_path("jane"), requirement.clone()),
        (Method::DELETE, status_path("jane"), None),
        (Method::GET, POLICY.to_owned(), None),
        (Method::PUT, POLICY.to_owned(), Some(policy(json!({})))),
    ];
    for (method, path, body) in &calls {
        let reply = fixture
            .call(method.clone(), path, Auth::None, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        let reply = fixture
            .call(
                method.clone(),
                path,
                Auth::Basic("plain", PLAIN),
                body.clone(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
    }
    let allowed = fixture
        .call(
            Method::GET,
            &status_path("jane"),
            Auth::Basic("reader", READER),
            None,
        )
        .await;
    assert_eq!(allowed.status, StatusCode::OK);
    for (method, path, body) in &calls[1..] {
        let reply = fixture
            .call(
                method.clone(),
                path,
                Auth::Basic("reader", READER),
                body.clone(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
    }
    for (method, body) in [(Method::GET, None), (Method::PUT, Some(policy(json!({}))))] {
        let reply = fixture
            .call(
                method.clone(),
                POLICY,
                Auth::Basic("updater", UPDATER),
                body,
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method}");
    }
    for (method, path, body) in [
        (Method::GET, POLICY.to_owned(), None),
        (Method::PUT, POLICY.to_owned(), Some(policy(json!({})))),
        (Method::PUT, requirement_path("jane"), requirement),
    ] {
        let reply = fixture.call(method.clone(), &path, Auth::Admin, body).await;
        assert!(reply.status.is_success(), "{method} {path}");
    }
}

#[tokio::test]
async fn the_status_of_a_user_shows_the_state_and_no_secret() {
    let fixture = people().await;
    let missing = fixture
        .call(Method::GET, &status_path("nobody"), Auth::Admin, None)
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let before = fixture
        .call(Method::GET, &status_path("jane"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(before["enrolled"], false);
    assert_eq!(before["requirement"], "policy");
    assert_eq!(before["required"], false);
    enroll(&fixture, "jane").await;
    let reply = fixture
        .call(Method::GET, &status_path("jane"), Auth::Admin, None)
        .await;
    let body = reply.json();
    let mut keys: Vec<&String> = body.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "enrollBy",
            "enrolled",
            "lockedUntil",
            "recoveryCodesLeft",
            "required",
            "requirement",
            "requirementSource",
            "totp",
            "userId"
        ]
    );
    assert_eq!(body["userId"], "jane");
    assert_eq!(body["enrolled"], true);
    assert_eq!(body["totp"], true);
    assert_eq!(body["recoveryCodesLeft"], 10);
}

#[tokio::test]
async fn a_requirement_is_stored_audited_and_checked() {
    let fixture = people().await;
    for value in [
        json!({"requirement": "always"}),
        json!({}),
        json!({"requirement": "unreadable"}),
    ] {
        let reply = fixture
            .call(
                Method::PUT,
                &requirement_path("jane"),
                Auth::Admin,
                Some(value),
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
        assert_eq!(error_ids(&reply), ["requirement"]);
    }
    let missing = fixture
        .call(
            Method::PUT,
            &requirement_path("nobody"),
            Auth::Admin,
            Some(json!({"requirement": "required"})),
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::PUT,
            &requirement_path("jane"),
            Auth::Admin,
            Some(json!({"requirement": "required"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let status = fixture
        .call(Method::GET, &status_path("jane"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(status["requirement"], "required");
    assert_eq!(status["required"], true);
    let events = audit(&fixture, "security.twofactor").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "requirement.updated");
    assert_eq!(events[0].1, "jane");
    assert_eq!(
        events[0].3,
        json!({"userId": "jane", "before": "policy", "after": "required"})
    );
}

#[tokio::test]
async fn a_reset_removes_the_factor_and_closes_the_sessions_of_the_target_only() {
    let fixture = people().await;
    let login = password(&fixture, "jane", support::two_factor::JANE, &[]).await;
    assert_eq!(login.status, StatusCode::OK);
    let admin = password(&fixture, "admin", &fixture.admin_password, &[]).await;
    assert_eq!(admin.status, StatusCode::OK);
    enroll(&fixture, "jane").await;
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE user_id = 'jane'"
        )
        .await,
        1
    );
    let reply = fixture
        .call(Method::DELETE, &status_path("jane"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE user_id = 'jane'"
        )
        .await,
        0
    );
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE user_id = 'admin'"
        )
        .await,
        1
    );
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_second_factor WHERE user_id = 'jane' AND totp_secret IS NOT NULL"
        )
        .await,
        0
    );
    let status = fixture
        .call(Method::GET, &status_path("jane"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(status["enrolled"], false);
    let events = audit(&fixture, "security.twofactor").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "reset");
    assert_eq!(events[0].1, "jane");
    assert_eq!(events[0].3, json!({"target": "jane"}));
    let missing = fixture
        .call(Method::DELETE, &status_path("nobody"), Auth::Admin, None)
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn nobody_resets_their_own_second_factor() {
    let fixture = people().await;
    let own = fixture
        .call(Method::DELETE, &status_path("admin"), Auth::Admin, None)
        .await;
    assert_eq!(own.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&own), ["*"]);
    let deleting = fixture
        .call(Method::DELETE, &format!("{USERS}/admin"), Auth::Admin, None)
        .await;
    assert_eq!(deleting.status, own.status);
    assert!(audit(&fixture, "security.twofactor").await.is_empty());
}

#[tokio::test]
async fn the_policy_is_read_replaced_and_audited() {
    let fixture = people().await;
    let reply = fixture.call(Method::GET, POLICY, Auth::Admin, None).await;
    assert_eq!(reply.json(), policy(json!({})));
    let replaced = put_policy(
        &fixture,
        policy(json!({"graceDays": 30, "required": {"everyone": false, "administrators": true, "roles": ["readers"]}})),
    )
    .await;
    assert_eq!(replaced.status, StatusCode::OK, "{}", replaced.text());
    let stored = fixture
        .call(Method::GET, POLICY, Auth::Admin, None)
        .await
        .json();
    assert_eq!(stored["graceDays"], 30);
    assert_eq!(stored["required"]["roles"], json!(["readers"]));
    let events = audit(&fixture, "security.twofactor").await;
    assert_eq!(events[0].0, "policy.updated");
    assert_eq!(events[0].3["before"]["graceDays"], 7);
    assert_eq!(events[0].3["after"]["graceDays"], 30);
}

#[tokio::test]
async fn a_policy_that_cannot_work_is_refused_with_the_field() {
    let fixture = people().await;
    set_base(&fixture, "https://astrum.example.org").await;
    let off = json!({"totp": false, "passkeys": false});
    let cases = [
        (policy(json!({"graceDays": 91})), "graceDays"),
        (
            policy(
                json!({"required": {"everyone": false, "administrators": false, "roles": ["nope"]}}),
            ),
            "required.roles",
        ),
        (
            policy(
                json!({"required": {"everyone": false, "administrators": false, "roles": ["readers", "readers"]}}),
            ),
            "required.roles",
        ),
        (
            policy(json!({"relyingParty": {"id": "other.org", "origins": []}})),
            "relyingParty.id",
        ),
        (
            policy(json!({"relyingParty": {"id": "https://example.org", "origins": []}})),
            "relyingParty.id",
        ),
        (
            policy(
                json!({"relyingParty": {"id": "example.org", "origins": ["https://evil.test"]}}),
            ),
            "relyingParty.origins",
        ),
        (
            policy(
                json!({"relyingParty": {"id": "example.org", "origins": ["http://sso.example.org"]}}),
            ),
            "relyingParty.origins",
        ),
        (
            policy(
                json!({"relyingParty": {"id": "example.org", "origins": ["https://sso.example.org/"]}}),
            ),
            "relyingParty.origins",
        ),
        (
            policy(
                json!({"totp": false, "passkeys": false, "required": {"everyone": true, "administrators": false, "roles": []}}),
            ),
            "totp",
        ),
    ];
    for (body, field) in cases {
        let reply = put_policy(&fixture, body.clone()).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(error_ids(&reply), [field], "{body}");
    }
    let good = put_policy(
        &fixture,
        policy(
            json!({"relyingParty": {"id": "example.org", "origins": ["https://sso.example.org"]}}),
        ),
    )
    .await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.text());
    let reply = fixture
        .call(
            Method::PUT,
            &requirement_path("jane"),
            Auth::Admin,
            Some(json!({"requirement": "required"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let blocked = put_policy(&fixture, policy(off.clone())).await;
    assert_eq!(blocked.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&blocked), ["totp"]);
    execute(
        &fixture,
        "UPDATE user_second_factor SET requirement = 'policy'",
    )
    .await;
    let allowed = put_policy(&fixture, policy(off)).await;
    assert_eq!(allowed.status, StatusCode::OK, "{}", allowed.text());
    set_base(&fixture, "http://localhost:8081").await;
    let local = put_policy(
        &fixture,
        policy(json!({"relyingParty": {"id": null, "origins": ["http://localhost:3000"]}})),
    )
    .await;
    assert_eq!(local.status, StatusCode::OK, "{}", local.text());
}

#[tokio::test]
async fn required_since_is_stamped_only_when_the_policy_widens() {
    let fixture = people().await;
    let since = |reply: &support::Reply| reply.json()["requiredSince"].clone();
    let everyone = |on: bool| json!({"everyone": on, "administrators": false, "roles": []});
    let first = put_policy(
        &fixture,
        policy(json!({"graceDays": 10, "requiredSince": "2000-01-01T00:00:00Z"})),
    )
    .await;
    assert_eq!(since(&first), Value::Null);
    let widened = put_policy(
        &fixture,
        policy(json!({"required": everyone(true), "requiredSince": "2000-01-01T00:00:00Z"})),
    )
    .await;
    let stamp = since(&widened);
    assert!(stamp.is_string());
    assert!(!stamp.as_str().unwrap().starts_with("2000"));
    let unrelated = put_policy(
        &fixture,
        policy(json!({"required": everyone(true), "graceDays": 20})),
    )
    .await;
    assert_eq!(since(&unrelated), stamp);
    let narrowed = put_policy(&fixture, policy(json!({"required": everyone(false)}))).await;
    assert_eq!(since(&narrowed), stamp);
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let role = put_policy(
        &fixture,
        policy(
            json!({"required": {"everyone": false, "administrators": false, "roles": ["readers"]}}),
        ),
    )
    .await;
    assert_ne!(since(&role), stamp);
    assert!(since(&role).is_string());
    let roles = |names: &[&str]| {
        policy(json!({"required": {"everyone": false, "administrators": false, "roles": names}}))
    };
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let replaced = put_policy(&fixture, roles(&["updaters"])).await;
    assert_ne!(since(&replaced), since(&role));
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let both = put_policy(&fixture, roles(&["updaters", "readers"])).await;
    assert_ne!(since(&both), since(&replaced));
    let reordered = put_policy(&fixture, roles(&["readers", "updaters"])).await;
    assert_eq!(since(&reordered), since(&both));
}

#[tokio::test]
async fn the_user_list_carries_the_second_factor_standing() {
    let fixture = people().await;
    enroll(&fixture, "jane").await;
    fixture
        .call(
            Method::PUT,
            &requirement_path("pat"),
            Auth::Admin,
            Some(json!({"requirement": "required"})),
        )
        .await;
    let reply = fixture.call(Method::GET, USERS, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    let users = reply.json();
    let standing = |id: &str| {
        users
            .as_array()
            .unwrap()
            .iter()
            .find(|user| user["userId"] == id)
            .unwrap()["twoFactor"]
            .clone()
    };
    assert_eq!(
        standing("jane"),
        json!({"enrolled": true, "required": false, "requirement": "policy"})
    );
    assert_eq!(
        standing("pat"),
        json!({"enrolled": false, "required": true, "requirement": "required"})
    );
    for user in users.as_array().unwrap() {
        assert!(user["twoFactor"].is_object(), "{user}");
    }
}
