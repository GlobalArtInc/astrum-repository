mod accounts;
mod factor_admin;
mod support;

use accounts::{audit, create_role, error_ids};
use axum::http::{Method, StatusCode};
use factor_admin::{
    POLICY, UPDATER, people, policy, put_policy, put_requirement, set_base, status_path,
};
use serde_json::{Value, json};
use support::Auth;
use support::two_factor::{JANE, enroll, execute, number, password};

const BOSS: &str = "boss-password-1";

fn message(reply: &support::Reply) -> String {
    reply.json()[0]["message"].as_str().unwrap().to_owned()
}

fn everyone(patch: Value) -> Value {
    let mut body =
        policy(json!({"required": {"everyone": true, "administrators": false, "roles": []}}));
    for (key, value) in patch.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

#[tokio::test]
async fn the_policy_and_the_requirement_refuse_each_others_state() {
    let fixture = people().await;
    let off = policy(json!({"totp": false, "passkeys": false}));
    assert_eq!(
        put_policy(&fixture, off.clone()).await.status,
        StatusCode::OK
    );
    let refused = put_requirement(&fixture, Auth::Admin, "jane", "required").await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&refused), ["requirement"]);
    assert_eq!(
        message(&refused),
        "turn on at least one method before requiring a second factor"
    );
    for allowed in ["exempt", "policy"] {
        let reply = put_requirement(&fixture, Auth::Admin, "jane", allowed).await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{allowed}");
    }

    assert_eq!(
        put_policy(&fixture, policy(json!({}))).await.status,
        StatusCode::OK
    );
    let reply = put_requirement(&fixture, Auth::Admin, "jane", "required").await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let blocked = put_policy(&fixture, off).await;
    assert_eq!(blocked.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&blocked), ["totp"]);
    let stored = fixture
        .call(Method::GET, POLICY, Auth::Admin, None)
        .await
        .json();
    assert_eq!(stored["totp"], true);
}

#[tokio::test]
async fn a_method_counts_only_when_it_can_be_enrolled() {
    let fixture = people().await;
    let passkeys_only = everyone(json!({"totp": false}));
    for base in [None, Some("http://127.0.0.1:8081")] {
        if let Some(url) = base {
            set_base(&fixture, url).await;
        }
        let reply = put_policy(&fixture, passkeys_only.clone()).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{base:?}");
        assert_eq!(error_ids(&reply), ["passkeys"], "{base:?}");
        assert_eq!(
            message(&reply),
            "passkeys need a relying party id: set the base URL or relyingParty.id"
        );
    }
    set_base(&fixture, "https://astrum.example.org").await;
    let accepted = put_policy(&fixture, passkeys_only.clone()).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.text());
    let required = put_requirement(&fixture, Auth::Admin, "jane", "required").await;
    assert_eq!(required.status, StatusCode::NO_CONTENT);
    set_base(&fixture, "").await;
    let refused = put_requirement(&fixture, Auth::Admin, "pat", "required").await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&refused), ["requirement"]);
}

#[tokio::test]
async fn a_reset_that_cannot_close_the_sessions_changes_nothing() {
    let fixture = people().await;
    assert_eq!(
        password(&fixture, "jane", JANE, &[]).await.status,
        StatusCode::OK
    );
    enroll(&fixture, "jane").await;
    execute(
        &fixture,
        "CREATE FUNCTION keep_sessions() RETURNS trigger LANGUAGE plpgsql \
         AS $$ BEGIN RAISE EXCEPTION 'sessions are kept'; END $$",
    )
    .await;
    execute(
        &fixture,
        "CREATE TRIGGER keep_sessions BEFORE DELETE ON user_session \
         FOR EACH STATEMENT EXECUTE FUNCTION keep_sessions()",
    )
    .await;
    let failed = fixture
        .call(Method::DELETE, &status_path("jane"), Auth::Admin, None)
        .await;
    assert_eq!(failed.status, StatusCode::INTERNAL_SERVER_ERROR);
    let factor = "SELECT count(*) FROM user_second_factor WHERE user_id = 'jane' AND totp_secret IS NOT NULL";
    let sessions = "SELECT count(*) FROM user_session WHERE user_id = 'jane'";
    assert_eq!(number(&fixture, factor).await, 1);
    assert_eq!(number(&fixture, sessions).await, 1);
    assert!(audit(&fixture, "security.twofactor").await.is_empty());

    execute(&fixture, "DROP TRIGGER keep_sessions ON user_session").await;
    execute(&fixture, "DROP FUNCTION keep_sessions()").await;
    let done = fixture
        .call(Method::DELETE, &status_path("jane"), Auth::Admin, None)
        .await;
    assert_eq!(done.status, StatusCode::NO_CONTENT);
    assert_eq!(number(&fixture, factor).await, 0);
    assert_eq!(number(&fixture, sessions).await, 0);
    let events = audit(&fixture, "security.twofactor").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "reset");
}

#[tokio::test]
async fn a_delegated_administrator_cannot_touch_administrators_or_exempt_anyone() {
    let fixture = people().await;
    create_role(&fixture, "root", &["as-all"], &[]).await;
    fixture.add_user("boss", BOSS, &["root"]).await;
    let updater = || Auth::Basic("updater", UPDATER);
    for target in ["admin", "boss"] {
        let reset = fixture
            .call(Method::DELETE, &status_path(target), updater(), None)
            .await;
        assert_eq!(reset.status, StatusCode::FORBIDDEN, "reset {target}");
        for requirement in ["policy", "required", "exempt"] {
            let reply = put_requirement(&fixture, updater(), target, requirement).await;
            assert_eq!(
                reply.status,
                StatusCode::FORBIDDEN,
                "{requirement} {target}"
            );
        }
    }
    let exempt = put_requirement(&fixture, updater(), "jane", "exempt").await;
    assert_eq!(exempt.status, StatusCode::FORBIDDEN);
    let status = fixture
        .call(Method::GET, &status_path("jane"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(status["requirement"], "policy");
    assert!(audit(&fixture, "security.twofactor").await.is_empty());

    let required = put_requirement(&fixture, updater(), "jane", "required").await;
    assert_eq!(required.status, StatusCode::NO_CONTENT);
    let reset = fixture
        .call(Method::DELETE, &status_path("jane"), updater(), None)
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);

    let reset = fixture
        .call(Method::DELETE, &status_path("boss"), Auth::Admin, None)
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    for (target, requirement) in [
        ("jane", "exempt"),
        ("boss", "required"),
        ("admin", "policy"),
    ] {
        let reply = put_requirement(&fixture, Auth::Admin, target, requirement).await;
        assert_eq!(
            reply.status,
            StatusCode::NO_CONTENT,
            "{requirement} {target}"
        );
    }
}

#[tokio::test]
async fn a_relying_party_id_must_be_a_registrable_host_name() {
    let fixture = people().await;
    let cases = [
        ("https://astrum.example.org", "example.org", None),
        ("https://astrum.example.org", "astrum.example.org", None),
        (
            "https://astrum.example.org",
            "org",
            Some("at least two labels"),
        ),
        ("https://astrum.example.org", "1.2.3.4", Some("IP address")),
        ("https://astrum.example.org", "127.1", Some("IP address")),
        (
            "https://astrum.example.org",
            "Example.org",
            Some("lower-case"),
        ),
        (
            "https://astrum.example.org",
            "example.org:8443",
            Some("lower-case"),
        ),
        (
            "https://astrum.example.org",
            "other.org",
            Some("parent domain"),
        ),
        ("http://10.0.0.1:8081", "10.0.0.1", Some("IP address")),
        ("http://localhost:8081", "localhost", None),
    ];
    for (base, id, problem) in cases {
        set_base(&fixture, base).await;
        let reply = put_policy(
            &fixture,
            policy(json!({"relyingParty": {"id": id, "origins": []}})),
        )
        .await;
        match problem {
            None => assert_eq!(
                reply.status,
                StatusCode::OK,
                "{base} {id}: {}",
                reply.text()
            ),
            Some(text) => {
                assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{base} {id}");
                assert_eq!(error_ids(&reply), ["relyingParty.id"], "{base} {id}");
                assert!(
                    message(&reply).contains(text),
                    "{base} {id}: {}",
                    reply.text()
                );
            }
        }
    }
}

#[tokio::test]
async fn the_policy_body_must_be_an_object_with_known_fields() {
    let fixture = people().await;
    let bodies = [
        (json!([]), "*"),
        (json!(null), "*"),
        (json!("policy"), "*"),
        (json!([true, true, {}, 7, null, {}]), "*"),
        (policy(json!({"extra": 1})), "extra"),
        (
            policy(json!({"required": {"everyone": false, "admins": true, "roles": []}})),
            "required.admins",
        ),
        (
            policy(json!({"relyingParty": {"id": null, "host": "example.org"}})),
            "relyingParty.host",
        ),
        (policy(json!({"required": [false, false, []]})), "required"),
        (policy(json!({"relyingParty": null})), "relyingParty"),
    ];
    for (body, field) in bodies {
        let reply = put_policy(&fixture, body.clone()).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(error_ids(&reply), [field], "{body}");
    }
    let stored = fixture.call(Method::GET, POLICY, Auth::Admin, None).await;
    assert_eq!(stored.json(), policy(json!({})));
    let changed = put_policy(&fixture, policy(json!({"graceDays": 30}))).await;
    assert_eq!(changed.status, StatusCode::OK);
    let defaults = put_policy(&fixture, json!({})).await;
    assert_eq!(defaults.status, StatusCode::OK, "{}", defaults.text());
    assert_eq!(defaults.json(), policy(json!({})));
}
