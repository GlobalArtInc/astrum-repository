mod support;

use axum::http::{Method, StatusCode};
use serde_json::json;
use support::Fixture;
use support::two_factor::*;

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

#[tokio::test]
async fn a_user_who_must_enroll_gets_a_restricted_session() {
    let fixture = world().await;
    let open = password(&fixture, "jane", JANE, &[]).await;
    assert_eq!(open.json()["secondFactor"], json!({"restricted": false}));
    require_enrollment(&fixture, "pat").await;
    let login = password(&fixture, "pat", PAT, &[]).await;
    assert_eq!(login.status, StatusCode::OK);
    assert_eq!(login.json()["authenticated"], true);
    assert_eq!(login.json()["secondFactor"], json!({"restricted": true}));
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) FROM user_session WHERE restricted"
        )
        .await,
        1
    );
    assert!(
        events(&fixture, "security.twofactor")
            .await
            .iter()
            .any(|event| event["type"] == "restricted.signin")
    );

    let described = with_session(&fixture, Method::GET, SESSION, &login, None).await;
    assert_eq!(
        described.json()["secondFactor"],
        json!({"restricted": true})
    );

    let allowed = [
        (Method::GET, SESSION),
        (Method::GET, ACCOUNT),
        (Method::GET, astrum_api::STATUS),
        (Method::GET, FACTORS),
        (
            Method::POST,
            "/service/rest/internal/ui/user/two-factor/totp",
        ),
    ];
    for (method, path) in allowed {
        let reply = with_session(&fixture, method.clone(), path, &login, None).await;
        assert_ne!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        assert_ne!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
    let refused = [
        (Method::POST, TOKENS),
        (Method::GET, TOKENS),
        (Method::PUT, "/service/rest/internal/ui/user/password"),
        (Method::PUT, ACCOUNT),
        (Method::POST, SESSION),
        (Method::POST, astrum_api::STATUS),
        (Method::GET, astrum_api::STATUS_WRITABLE),
        (Method::GET, "/service/rest/v1/repositories"),
        (Method::GET, "/service/rest/internal/ui/user/two-factors"),
        (
            Method::GET,
            "/service/rest/internal/ui/user/two-factor/%2e%2e/password",
        ),
        (
            Method::GET,
            "/service/rest/internal/ui/user/two-factor/../password",
        ),
        (
            Method::POST,
            "/service/rest/internal/ui/oauth/requests/abc/approve",
        ),
        (Method::GET, "/mcp"),
    ];
    for (method, path) in refused {
        let body = (method != Method::GET)
            .then(|| json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30}));
        let reply = with_session(&fixture, method.clone(), path, &login, body).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        assert_eq!(
            error_code(&reply),
            "two-factor-enrollment-required",
            "{method} {path}"
        );
    }
    let out = with_session(&fixture, Method::DELETE, SESSION, &login, None).await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn tokens_and_agent_consent_work_for_an_unrestricted_session_and_a_policy_change_is_not_retroactive()
 {
    let fixture = world().await;
    let login = password(&fixture, "pat", PAT, &[]).await;
    let body = json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30});
    let created = with_session(&fixture, Method::POST, TOKENS, &login, Some(body.clone())).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let consent = "/service/rest/internal/ui/oauth/requests/abc/approve";
    let unknown = with_session(&fixture, Method::POST, consent, &login, None).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    require_enrollment(&fixture, "pat").await;
    let still = with_session(&fixture, Method::POST, TOKENS, &login, Some(body)).await;
    assert_eq!(still.status, StatusCode::CREATED, "{}", still.text());
}
