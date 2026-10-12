mod accounts;
mod support;

use accounts::{ROLES, USERS, audit, create_role, create_user, error_ids, ids};
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture};

fn role(id: &str, privileges: &[&str], roles: &[&str]) -> serde_json::Value {
    json!({"id": id, "name": format!("{id} name"), "privileges": privileges, "roles": roles})
}

#[tokio::test]
async fn built_in_roles_are_listed_and_read() {
    let fixture = Fixture::new().await;
    let reply = fixture.call(Method::GET, ROLES, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(ids(&reply, "id"), ["as-admin", "as-anonymous"]);
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/as-admin"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!({
            "id": "as-admin",
            "source": "default",
            "name": "as-admin",
            "description": "Administrator role",
            "readOnly": true,
            "privileges": ["as-all"],
            "roles": [],
        })
    );
    for (query, status, count) in [
        ("?source=default", StatusCode::OK, 2),
        ("?source=", StatusCode::OK, 2),
        ("?source=LDAP", StatusCode::OK, 0),
        ("?source=nope", StatusCode::BAD_REQUEST, 0),
        ("?source=OAuth2", StatusCode::OK, 0),
    ] {
        let reply = fixture
            .call(Method::GET, &format!("{ROLES}{query}"), Auth::Admin, None)
            .await;
        assert_eq!(reply.status, status, "{query}");
        if status == StatusCode::OK {
            assert_eq!(reply.json().as_array().unwrap().len(), count, "{query}");
        }
    }
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/nope"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["message"], "Role 'nope' not found.");
    let reply = fixture
        .call(
            Method::GET,
            &format!("{ROLES}/as-admin?source=LDAP"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn roles_are_created_with_checked_references() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(json!({"id": "", "name": ""})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["id", "name"]);

    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(role("devs", &["nope-privilege"], &["nope-role"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["roles", "privileges"]);

    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(role("devs", &[], &["devs"])),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "a new role cannot hold itself"
    );

    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(role(
                "devs",
                &["as-users-read", "as-users-read"],
                &["as-anonymous"],
            )),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!({
            "id": "devs",
            "source": "default",
            "name": "devs name",
            "description": "devs",
            "readOnly": false,
            "privileges": ["as-users-read"],
            "roles": ["as-anonymous"],
        })
    );

    for id in ["devs", "as-admin"] {
        let reply = fixture
            .call(Method::POST, ROLES, Auth::Admin, Some(role(id, &[], &[])))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{id}");
        assert_eq!(error_ids(&reply), ["id"]);
    }
    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(role("DEVS", &[], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "role ids are case-sensitive");

    let reply = fixture
        .call(
            Method::POST,
            &format!("{ROLES}?source=LDAP"),
            Auth::Admin,
            Some(role("ldap-group", &[], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "an external role mapping");
    assert_eq!(reply.json()["source"], "LDAP");

    let events = audit(&fixture, "security.role").await;
    assert_eq!(events.len(), 3);
    assert_eq!(
        (events[0].0.as_str(), events[0].1.as_str()),
        ("created", "devs")
    );
}

#[tokio::test]
async fn roles_are_updated_but_built_ins_stay() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "a", &[], &[]).await;
    create_role(&fixture, "b", &[], &["a"]).await;

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/as-admin"),
            Auth::Admin,
            Some(role("as-admin", &["as-all"], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/a"),
            Auth::Admin,
            Some(role("other", &[], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(reply.json()["id"], "*");
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/nope"),
            Auth::Admin,
            Some(role("nope", &[], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    for nested in ["b", "a"] {
        let reply = fixture
            .call(
                Method::PUT,
                &format!("{ROLES}/a"),
                Auth::Admin,
                Some(role("a", &[], &[nested])),
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{nested}");
        assert!(
            reply.text().contains("cannot contain itself"),
            "{}",
            reply.text()
        );
    }

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/a"),
            Auth::Admin,
            Some(json!({"id": "a", "name": "A", "description": "", "privileges": ["as-roles-read"], "readOnly": true})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/a"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json(),
        json!({"id": "a", "source": "default", "name": "A", "description": "", "readOnly": false, "privileges": ["as-roles-read"], "roles": []})
    );
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/a"),
            Auth::Admin,
            Some(role("a", &[], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/a"), Auth::Admin, None)
        .await;
    assert_eq!(
        reply.json()["description"],
        "a",
        "no description means the id"
    );
}

#[tokio::test]
async fn deleting_a_role_removes_it_from_roles_and_users() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "readers", &["as-users-read"], &[]).await;
    create_role(&fixture, "team", &[], &["readers", "as-anonymous"]).await;
    create_user(&fixture, "jane", "correct horse", &["readers", "team"]).await;

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{ROLES}/as-anonymous"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = fixture
        .call(Method::DELETE, &format!("{ROLES}/nope"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{ROLES}/readers"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture
        .call(Method::GET, &format!("{ROLES}/team"), Auth::Admin, None)
        .await;
    assert_eq!(reply.json()["roles"], json!(["as-anonymous"]));
    let reply = fixture
        .call(
            Method::GET,
            &format!("{USERS}?userId=jane"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.json()[0]["roles"], json!(["team"]));
    let reply = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("jane", "correct horse"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let events = audit(&fixture, "security.role").await;
    assert_eq!(events.last().unwrap().0, "deleted");
}

#[tokio::test]
async fn assignable_roles_hold_only_what_the_caller_holds() {
    let fixture = Fixture::new().await;
    create_role(
        &fixture,
        "role-readers",
        &["as-roles-read", "as-settings-read"],
        &[],
    )
    .await;
    create_role(&fixture, "empty", &[], &[]).await;
    create_role(&fixture, "wraps-anonymous", &[], &["as-anonymous"]).await;
    create_role(&fixture, "wraps-readers", &[], &["role-readers"]).await;
    create_user(&fixture, "jane", "correct horse", &["role-readers"]).await;

    let path = format!("{ROLES}/assignable");
    let reply = fixture
        .call(
            Method::GET,
            &path,
            Auth::Basic("jane", "correct horse"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        ids(&reply, "id"),
        ["empty", "role-readers", "wraps-readers"]
    );
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(
        ids(&reply, "id"),
        [
            "as-admin",
            "as-anonymous",
            "empty",
            "role-readers",
            "wraps-anonymous",
            "wraps-readers"
        ]
    );
    let reply = fixture
        .call(
            Method::GET,
            &format!("{path}?source=LDAP"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.json(), json!([]));
}

#[tokio::test]
async fn roles_cannot_grant_more_than_their_author_holds() {
    let fixture = Fixture::new().await;
    create_role(
        &fixture,
        "role-admins",
        &["as-roles-all", "as-users-read"],
        &[],
    )
    .await;
    create_user(&fixture, "jane", "correct horse", &["role-admins"]).await;
    let jane = || Auth::Basic("jane", "correct horse");

    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            jane(),
            Some(role("x", &["as-all"], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            jane(),
            Some(role("x", &[], &["as-admin"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            jane(),
            Some(role("x", &["as-users-read"], &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/role-admins"),
            jane(),
            Some(role(
                "role-admins",
                &["as-roles-all", "as-users-read", "as-users-update"],
                &[],
            )),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::FORBIDDEN,
        "no way to widen one's own role"
    );
}

#[tokio::test]
async fn every_role_endpoint_checks_permissions_first() {
    let fixture = Fixture::new().await;
    create_user(&fixture, "jane", "correct horse", &[]).await;
    let calls = [
        (Method::GET, ROLES.to_owned(), None),
        (Method::GET, format!("{ROLES}/assignable"), None),
        (Method::GET, format!("{ROLES}/as-admin"), None),
        (Method::POST, ROLES.to_owned(), Some(role("x", &[], &[]))),
        (
            Method::PUT,
            format!("{ROLES}/as-admin"),
            Some(role("as-admin", &[], &[])),
        ),
        (Method::DELETE, format!("{ROLES}/nope"), None),
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
                Auth::Basic("jane", "correct horse"),
                body,
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
    }
}
