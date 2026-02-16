mod accounts;
mod support;

use accounts::{PRIVILEGES, SELECTORS, audit, create_role, create_user, error_ids};
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture};

#[tokio::test]
async fn selectors_are_created_read_and_listed() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::POST, SELECTORS, Auth::Admin, Some(json!({})))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["name", "expression"]);

    let reply = fixture
        .call(
            Method::POST,
            SELECTORS,
            Auth::Admin,
            Some(json!({"name": "m1 bad", "expression": "format == \"raw\""})),
        )
        .await;
    assert_eq!(error_ids(&reply), ["name"]);

    let reply = fixture
        .call(
            Method::POST,
            SELECTORS,
            Auth::Admin,
            Some(json!({"name": "b", "expression": "format == \"raw\" AND path =^ \"/a\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reply.json(),
        json!([{
            "id": "expression",
            "message": "Invalid CSEL: unknown identifier 'AND', use format or path at line 1 column 17",
        }])
    );

    for (name, body) in [
        (
            "maven-org",
            json!({"name": "maven-org", "description": "Our groups", "expression": "format == \"maven2\" and path =^ \"/org/astrum/\""}),
        ),
        (
            "a-raw",
            json!({"name": "a-raw", "expression": "format == 'raw'"}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, SELECTORS, Auth::Admin, Some(body))
            .await;
        assert_eq!(
            reply.status,
            StatusCode::NO_CONTENT,
            "{name}: {}",
            reply.text()
        );
    }
    let reply = fixture
        .call(
            Method::POST,
            SELECTORS,
            Auth::Admin,
            Some(json!({"name": "a-raw", "expression": "format == 'npm'"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["name"]);

    let reply = fixture
        .call(Method::GET, SELECTORS, Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!([
            {"name": "a-raw", "type": "csel", "description": null, "expression": "format == 'raw'"},
            {"name": "maven-org", "type": "csel", "description": "Our groups", "expression": "format == \"maven2\" and path =^ \"/org/astrum/\""},
        ])
    );
    let reply = fixture
        .call(
            Method::GET,
            &format!("{SELECTORS}/maven-org"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.json()["description"], "Our groups");
    let reply = fixture
        .call(Method::GET, &format!("{SELECTORS}/nope"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let model = fixture.state.security().model();
    let selector = &model.selectors["maven-org"].selector;
    assert!(selector.matches("maven2", "/org/astrum/core/1.0/core-1.0.jar"));
    assert!(!selector.matches("npm", "/org/astrum/core"));
    let events = audit(&fixture, "security.contentselector").await;
    assert_eq!(events.len(), 2);
    assert_eq!(
        (events[0].0.as_str(), events[0].1.as_str()),
        ("created", "maven-org")
    );
}

#[tokio::test]
async fn selectors_are_updated_and_deleted_unless_in_use() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(
            Method::POST,
            SELECTORS,
            Auth::Admin,
            Some(json!({"name": "team", "description": "d", "expression": "path =^ \"/team/\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let path = format!("{SELECTORS}/team");

    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(json!({})))
        .await;
    assert_eq!(error_ids(&reply), ["expression"]);
    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"expression": "path =~ \"[a\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("Invalid CSEL"), "{}", reply.text());
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{SELECTORS}/nope"),
            Auth::Admin,
            Some(json!({"expression": "format == \"raw\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "renamed", "expression": "path =^ \"/team-b/\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(
        reply.json(),
        json!({"name": "team", "type": "csel", "description": null, "expression": "path =^ \"/team-b/\""})
    );
    assert!(
        fixture.state.security().model().selectors["team"]
            .selector
            .matches("raw", "/team-b/x")
    );

    let reply = fixture
        .call(
            Method::POST,
            &format!("{PRIVILEGES}/repository-content-selector"),
            Auth::Admin,
            Some(json!({"name": "team-read", "contentSelector": "team", "format": "*", "repository": "*", "actions": ["READ"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    let reply = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("team-read"), "{}", reply.text());

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{PRIVILEGES}/team-read"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(fixture.state.security().model().selectors.is_empty());
    let kinds: Vec<String> = audit(&fixture, "security.contentselector")
        .await
        .into_iter()
        .map(|event| event.0)
        .collect();
    assert_eq!(kinds, ["created", "updated", "deleted"]);
}

#[tokio::test]
async fn selector_endpoints_check_permissions_first() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "selector-readers", &["as-selectors-read"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["selector-readers"]).await;
    create_user(&fixture, "john", "john password", &[]).await;
    let body = json!({"name": "x", "expression": "format == \"raw\""});
    let calls = [
        (Method::GET, SELECTORS.to_owned(), None, true),
        (Method::GET, format!("{SELECTORS}/x"), None, true),
        (
            Method::POST,
            SELECTORS.to_owned(),
            Some(body.clone()),
            false,
        ),
        (Method::PUT, format!("{SELECTORS}/x"), Some(body), false),
        (Method::DELETE, format!("{SELECTORS}/x"), None, false),
    ];
    for (method, path, body, reads) in calls {
        let reply = fixture
            .call(method.clone(), &path, Auth::None, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        let reply = fixture
            .call(
                method.clone(),
                &path,
                Auth::Basic("john", "john password"),
                body.clone(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        let reply = fixture
            .call(
                method.clone(),
                &path,
                Auth::Basic("jane", "correct horse"),
                body,
            )
            .await;
        assert_eq!(
            reply.status == StatusCode::FORBIDDEN,
            !reads,
            "{method} {path}: {}",
            reply.status
        );
    }
}
