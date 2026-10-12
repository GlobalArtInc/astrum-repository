mod accounts;
mod support;

use accounts::{
    PRIVILEGES, ROLES, SELECTORS, audit, create_repository, create_role, create_user, error_ids,
};
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture};

async fn create(fixture: &Fixture, kind: &str, body: Value) -> support::Reply {
    fixture
        .call(
            Method::POST,
            &format!("{PRIVILEGES}/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn read(fixture: &Fixture, name: &str) -> support::Reply {
    fixture
        .call(
            Method::GET,
            &format!("{PRIVILEGES}/{name}"),
            Auth::Admin,
            None,
        )
        .await
}

#[tokio::test]
async fn built_in_and_repository_privileges_are_listed() {
    let fixture = Fixture::new().await;
    create_repository(&fixture, "raw-files", "raw").await;
    let reply = fixture
        .call(Method::GET, PRIVILEGES, Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let names = accounts::ids(&reply, "name");
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert!(names.contains(&"as-repository-view-raw-raw-files-read".to_owned()));

    let reply = read(&fixture, "as-repository-view-*-*-read").await;
    assert_eq!(
        reply.json(),
        json!({
            "type": "repository-view",
            "name": "as-repository-view-*-*-read",
            "description": "Read the content of every repository",
            "readOnly": true,
            "actions": ["READ"],
            "format": "*",
            "repository": "*",
        })
    );
    let reply = read(&fixture, "as-all").await;
    assert_eq!(reply.json()["pattern"], "astrum:*");
    assert!(reply.json().get("actions").is_none());
    let reply = read(&fixture, "as-users-create").await;
    assert_eq!(reply.json()["domain"], "users");
    assert_eq!(reply.json()["actions"], json!(["ADD", "READ"]));
    let reply = read(&fixture, "nope").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["message"], "Privilege 'nope' not found.");
}

#[tokio::test]
async fn application_and_wildcard_privileges_are_checked() {
    let fixture = Fixture::new().await;
    let reply = create(&fixture, "application", json!({})).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["name", "domain", "actions"]);

    let reply = create(
        &fixture,
        "application",
        json!({"name": "m1 bad", "domain": "x", "actions": ["READ", "CREATE"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["name", "actions"]);

    let reply = create(
        &fixture,
        "application",
        json!({"name": "x", "domain": "x", "actions": ["READ", "BROWSE", "RUN"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("BROWSE, RUN"), "{}", reply.text());

    let reply = create(
        &fixture,
        "application",
        json!({"name": "as-users-read", "domain": "users", "actions": ["READ"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["name"]);

    let reply = create(
        &fixture,
        "application",
        json!({"name": "user-reader", "domain": "users", "actions": ["read", "READ", "start"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert!(reply.body.is_empty());
    assert_eq!(
        read(&fixture, "user-reader").await.json(),
        json!({
            "type": "application",
            "name": "user-reader",
            "description": "",
            "readOnly": false,
            "actions": ["READ", "START"],
            "domain": "users",
        })
    );

    let reply = create(
        &fixture,
        "wildcard",
        json!({"name": "w", "pattern": "astrum::read"}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["pattern"]);
    for pattern in ["*", "nexus:*", "other:users:read"] {
        let reply = create(
            &fixture,
            "wildcard",
            json!({"name": "w", "pattern": pattern}),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::FORBIDDEN,
            "{pattern}: even the admin holds only astrum:*"
        );
    }
    let reply = create(
        &fixture,
        "wildcard",
        json!({"name": "w", "description": "d", "pattern": "astrum:users:read"}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    assert_eq!(
        read(&fixture, "w").await.json()["pattern"],
        "astrum:users:read"
    );

    let events = audit(&fixture, "security.privilege").await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].3["type"], "application");
}

#[tokio::test]
async fn repository_privileges_name_real_formats_and_repositories() {
    let fixture = Fixture::new().await;
    create_repository(&fixture, "raw-files", "raw").await;
    for kind in ["repository-view", "repository-admin"] {
        for (body, message) in [
            (json!({"format": "nope", "repository": "*"}), "format nope"),
            (json!({"format": "RAW", "repository": "*"}), "format RAW"),
            (
                json!({"format": "raw", "repository": "nope"}),
                "repository nope",
            ),
            (
                json!({"format": "*", "repository": "nope"}),
                "repository nope",
            ),
            (
                json!({"format": "npm", "repository": "raw-files"}),
                "format raw, not npm",
            ),
        ] {
            let mut body = body;
            body["name"] = json!("p");
            body["actions"] = json!(["READ"]);
            let reply = create(&fixture, kind, body).await;
            assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{kind} {message}");
            assert!(reply.text().contains(message), "{}", reply.text());
        }
        let reply = create(
            &fixture,
            kind,
            json!({"name": "p", "format": "raw", "repository": "raw-files", "actions": ["START"]}),
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
        let reply = create(
            &fixture,
            kind,
            json!({"name": "p", "format": "raw", "repository": "*"}),
        )
        .await;
        assert_eq!(error_ids(&reply), ["actions"]);
    }

    let reply = create(
        &fixture,
        "repository-view",
        json!({"name": "raw-read", "format": "raw", "repository": "RAW-FILES", "actions": ["BROWSE", "READ"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(
        read(&fixture, "raw-read").await.json()["repository"],
        "raw-files"
    );
    let reply = create(
        &fixture,
        "repository-admin",
        json!({"name": "any-admin", "format": "*", "repository": "*", "actions": ["ALL"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);

    create_role(&fixture, "raw-readers", &["raw-read"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["raw-readers"]).await;
    let jane = fixture
        .state
        .security()
        .authenticate("jane", "correct horse")
        .await
        .unwrap();
    assert!(jane.is_permitted("astrum:repository-view:raw:raw-files:read"));
    assert!(!jane.is_permitted("astrum:repository-view:raw:raw-files:add"));
}

#[tokio::test]
async fn content_selector_privileges_name_an_existing_selector() {
    let fixture = Fixture::new().await;
    let body = json!({
        "name": "team-read",
        "description": "team files",
        "contentSelector": "team",
        "format": "raw",
        "repository": "*",
        "actions": ["READ", "BROWSE"],
    });
    let reply = create(&fixture, "repository-content-selector", body.clone()).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(
        reply.text().contains("content selector team"),
        "{}",
        reply.text()
    );

    let reply = fixture
        .call(
            Method::POST,
            SELECTORS,
            Auth::Admin,
            Some(json!({"name": "team", "expression": "path =^ \"/team/\""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = create(&fixture, "repository-content-selector", body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(
        read(&fixture, "team-read").await.json(),
        json!({
            "type": "repository-content-selector",
            "name": "team-read",
            "description": "team files",
            "readOnly": false,
            "actions": ["READ", "BROWSE"],
            "format": "raw",
            "repository": "*",
            "contentSelector": "team",
        })
    );

    create_role(&fixture, "team", &["team-read"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["team"]).await;
    let jane = fixture
        .state
        .security()
        .authenticate("jane", "correct horse")
        .await
        .unwrap();
    let grant = &jane.selectors()[0];
    assert_eq!(
        (
            grant.selector.as_str(),
            grant.format.as_str(),
            grant.repository.as_str()
        ),
        ("team", "raw", "*")
    );
    let model = fixture.state.security().model();
    assert!(
        model.selectors["team"]
            .selector
            .matches("raw", "/team/a.txt")
    );
}

#[tokio::test]
async fn privileges_are_updated_through_their_own_type() {
    let fixture = Fixture::new().await;
    let reply = create(
        &fixture,
        "application",
        json!({"name": "p", "domain": "users", "actions": ["READ"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    let put = |kind: &str, name: &str| format!("{PRIVILEGES}/{kind}/{name}");

    let reply = fixture
        .call(
            Method::PUT,
            &put("application", "p"),
            Auth::Admin,
            Some(json!({"name": "other", "domain": "users", "actions": ["READ"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    let reply = fixture
        .call(
            Method::PUT,
            &put("wildcard", "p"),
            Auth::Admin,
            Some(json!({"name": "p", "pattern": "astrum:users:read"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert!(reply.text().contains("of type application"));
    let reply = fixture
        .call(
            Method::PUT,
            &put("application", "as-users-read"),
            Auth::Admin,
            Some(json!({"name": "as-users-read", "domain": "users", "actions": ["READ"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = fixture
        .call(
            Method::PUT,
            &put("application", "nope"),
            Auth::Admin,
            Some(json!({"name": "nope", "domain": "users", "actions": ["READ"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::PUT,
            &put("nope", "p"),
            Auth::Admin,
            Some(json!({"name": "p"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    create_role(&fixture, "holders", &["p"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["holders"]).await;
    let reply = fixture
        .call(
            Method::PUT,
            &put("application", "p"),
            Auth::Admin,
            Some(json!({"name": "p", "description": "now roles", "domain": "roles", "actions": ["READ"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = read(&fixture, "p").await;
    assert_eq!(reply.json()["domain"], "roles");
    assert_eq!(reply.json()["description"], "now roles");
    let jane = Auth::Basic("jane", "correct horse");
    assert_eq!(
        fixture.call(Method::GET, ROLES, jane, None).await.status,
        StatusCode::OK
    );
    let events = audit(&fixture, "security.privilege").await;
    assert_eq!(events.last().unwrap().0, "updated");
}

#[tokio::test]
async fn deleting_a_privilege_removes_it_from_roles() {
    let fixture = Fixture::new().await;
    let reply = create(
        &fixture,
        "application",
        json!({"name": "p", "domain": "users", "actions": ["READ"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    create_role(&fixture, "holders", &["p", "as-roles-read"], &[]).await;

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{PRIVILEGES}/as-all"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{PRIVILEGES}/nope"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{PRIVILEGES}/p"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/holders"), Auth::Admin, None)
        .await;
    assert_eq!(reply.json()["privileges"], json!(["as-roles-read"]));
    assert_eq!(read(&fixture, "p").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn script_privileges_are_refused_with_a_reason() {
    let fixture = Fixture::new().await;
    let body = json!({"name": "s", "scriptName": "x", "actions": ["READ"]});
    let reply = create(&fixture, "script", body.clone()).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("no script API"), "{}", reply.text());
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{PRIVILEGES}/script/s"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = create(&fixture, "nope", json!({"name": "s"})).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn privileges_need_permissions_and_cannot_exceed_them() {
    let fixture = Fixture::new().await;
    create_role(
        &fixture,
        "privilege-admins",
        &["as-privileges-all", "as-users-read"],
        &[],
    )
    .await;
    create_user(&fixture, "jane", "correct horse", &["privilege-admins"]).await;
    create_user(&fixture, "john", "john password", &[]).await;

    let body = json!({"name": "w", "pattern": "astrum:users:delete"});
    let reply = fixture
        .call(
            Method::POST,
            &format!("{PRIVILEGES}/wildcard"),
            Auth::Basic("jane", "correct horse"),
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::POST,
            &format!("{PRIVILEGES}/wildcard"),
            Auth::Basic("jane", "correct horse"),
            Some(json!({"name": "w", "pattern": "astrum:users:read"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED);

    let calls = [
        (Method::GET, PRIVILEGES.to_owned(), None),
        (Method::GET, format!("{PRIVILEGES}/as-all"), None),
        (
            Method::POST,
            format!("{PRIVILEGES}/application"),
            Some(json!({"name": "x", "domain": "x", "actions": ["READ"]})),
        ),
        (
            Method::PUT,
            format!("{PRIVILEGES}/wildcard/w"),
            Some(json!({"name": "w", "pattern": "astrum:users:read"})),
        ),
        (Method::DELETE, format!("{PRIVILEGES}/w"), None),
        (
            Method::POST,
            format!("{PRIVILEGES}/script"),
            Some(json!({"name": "s"})),
        ),
    ];
    for (method, path, body) in calls {
        let reply = fixture
            .call(method.clone(), &path, Auth::None, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        let reply = fixture
            .call(
                method.clone(),
                &path,
                Auth::Basic("john", "john password"),
                body,
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
    }
}
