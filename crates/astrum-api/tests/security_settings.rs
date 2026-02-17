mod accounts;
mod support;

use accounts::{SECURITY, USERS, audit, create_role, create_user, error_ids};
use astrum_api::STATUS;
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture};

fn path(tail: &str) -> String {
    format!("{SECURITY}/{tail}")
}

#[tokio::test]
async fn realms_are_listed_and_activated_in_order() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::GET, &path("realms/available"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let ids = accounts::ids(&reply, "id");
    assert_eq!(
        ids,
        [
            "AccessTokenRealm",
            "CargoToken",
            "ConanToken",
            "DefaultRole",
            "DockerToken",
            "LdapRealm",
            "AstrumAuthenticatingRealm",
            "NpmToken",
            "NuGetApiKey",
            "OAuth2Realm",
            "RubyGemsApiKey",
            "TerraformToken",
        ]
    );
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../.claude/rewrite/reference/realms-available.json"
    ))
    .unwrap();
    assert_eq!(reply.json(), reference, "the names of the old server");
    let reply = fixture
        .call(Method::GET, &path("realms/active"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!(["AstrumAuthenticatingRealm", "AccessTokenRealm"])
    );

    let reply = fixture
        .call(
            Method::PUT,
            &path("realms/active"),
            Auth::Admin,
            Some(json!(["AstrumAuthenticatingRealm", "nope", "other"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("nope, other"), "{}", reply.text());

    let reply = fixture
        .call(
            Method::PUT,
            &path("realms/active"),
            Auth::Admin,
            Some(json!([
                "DockerToken",
                "AstrumAuthenticatingRealm",
                "DockerToken"
            ])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(Method::GET, &path("realms/active"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!(["DockerToken", "AstrumAuthenticatingRealm"])
    );
    let events = audit(&fixture, "security.realm").await;
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].3["active"],
        json!(["DockerToken", "AstrumAuthenticatingRealm"])
    );

    let reply = fixture
        .call(
            Method::PUT,
            &path("realms/active"),
            Auth::Admin,
            Some(json!(["DockerToken"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::GET, STATUS, Auth::Admin, None).await;
    assert_eq!(
        reply.status,
        StatusCode::UNAUTHORIZED,
        "without the local realm local users cannot sign in"
    );
}

#[tokio::test]
async fn anonymous_access_is_switched_on_and_off() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::GET, &path("anonymous"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!({"enabled": false, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})
    );
    let reply = fixture
        .call(
            Method::PUT,
            &path("anonymous"),
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(error_ids(&reply), ["userId", "realmName"]);
    let reply = fixture
        .call(
            Method::PUT,
            &path("anonymous"),
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "nope"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["realmName"]);

    assert_eq!(
        fixture
            .call(Method::GET, USERS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let reply = fixture
        .call(
            Method::PUT,
            &path("anonymous"),
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["enabled"], true);
    assert!(fixture.state.security().anonymous().is_some());
    assert_eq!(
        fixture
            .call(Method::GET, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .call(Method::GET, USERS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED,
        "anonymous callers are asked to sign in, not refused"
    );
    let events = audit(&fixture, "security.anonymous").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].3["enabled"], true);

    let reply = fixture
        .call(
            Method::PUT,
            &path("anonymous"),
            Auth::Admin,
            Some(json!({"userId": "anonymous", "realmName": "AstrumAuthenticatingRealm"})),
        )
        .await;
    assert_eq!(reply.json()["enabled"], false);
    assert!(fixture.state.security().anonymous().is_none());
}

#[tokio::test]
async fn ssrf_protection_is_stored_for_the_http_client() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::GET, &path("ssrf-protection"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!({"enabled": true, "allowedIPs": [], "allowedDomains": []})
    );
    let reply = fixture
        .call(
            Method::PUT,
            &path("ssrf-protection"),
            Auth::Admin,
            Some(json!({"allowedIPs": ["not-an-ip"], "allowedDomains": ["bad domain!"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        error_ids(&reply),
        ["enabled", "allowedIPs", "allowedDomains"]
    );

    let reply = fixture
        .call(
            Method::PUT,
            &path("ssrf-protection"),
            Auth::Admin,
            Some(json!({
                "enabled": true,
                "allowedIPs": ["10.0.0.50", " 192.168.0.0/16 ", "10.0.0.50"],
                "allowedDomains": ["Registry.Local", "*.corp.example"],
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let expected = json!({
        "enabled": true,
        "allowedIPs": ["10.0.0.50", "192.168.0.0/16"],
        "allowedDomains": ["registry.local", "*.corp.example"],
    });
    assert_eq!(reply.json(), expected);
    let reply = fixture
        .call(Method::GET, &path("ssrf-protection"), Auth::Admin, None)
        .await;
    assert_eq!(reply.json(), expected);
    let events = audit(&fixture, "security.ssrf").await;
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn settings_need_the_settings_permissions_and_ssrf_an_administrator() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "settings-readers", &["as-settings-read"], &[]).await;
    create_role(&fixture, "settings-admins", &["as-settings-all"], &[]).await;
    create_user(&fixture, "reader", "reader password", &["settings-readers"]).await;
    create_user(&fixture, "keeper", "keeper password", &["settings-admins"]).await;
    let reader = || Auth::Basic("reader", "reader password");
    let keeper = || Auth::Basic("keeper", "keeper password");

    for tail in ["realms/available", "realms/active", "anonymous"] {
        let reply = fixture
            .call(Method::GET, &path(tail), Auth::None, None)
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{tail}");
        let reply = fixture.call(Method::GET, &path(tail), reader(), None).await;
        assert_eq!(reply.status, StatusCode::OK, "{tail}");
    }
    let realms = json!(["AstrumAuthenticatingRealm", "AccessTokenRealm"]);
    let anonymous =
        json!({"enabled": false, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"});
    for (tail, body) in [("realms/active", &realms), ("anonymous", &anonymous)] {
        let reply = fixture
            .call(Method::PUT, &path(tail), reader(), Some(body.clone()))
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{tail}");
        let reply = fixture
            .call(Method::PUT, &path(tail), keeper(), Some(body.clone()))
            .await;
        assert!(reply.status.is_success(), "{tail}: {}", reply.status);
    }

    let ssrf = json!({"enabled": true, "allowedIPs": [], "allowedDomains": []});
    for auth in [keeper(), reader()] {
        let reply = fixture
            .call(Method::GET, &path("ssrf-protection"), auth, None)
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
    }
    let reply = fixture
        .call(Method::PUT, &path("ssrf-protection"), keeper(), Some(ssrf))
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(Method::GET, &path("ssrf-protection"), Auth::None, None)
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}
