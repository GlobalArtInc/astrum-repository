mod accounts;
mod support;

use accounts::{SECURITY, USERS, audit, create_role, create_user, error_ids, ids, send_raw};
use astrum_db::entity::security_user_role;
use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use astrum_security::model::{ANONYMOUS_KEY, AnonymousSettings};
use axum::http::{Method, StatusCode, header};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::json;
use support::{Auth, Fixture};

fn user(user_id: &str) -> serde_json::Value {
    json!({
        "userId": user_id,
        "firstName": "Jane",
        "lastName": "Doe",
        "emailAddress": "jane@example.org",
        "password": "correct horse",
        "status": "active",
        "roles": [],
    })
}

fn update(user_id: &str, roles: &[&str]) -> serde_json::Value {
    json!({
        "userId": user_id,
        "firstName": "Jane",
        "lastName": "Doe",
        "emailAddress": "jane@example.org",
        "source": "default",
        "status": "active",
        "roles": roles,
    })
}

#[tokio::test]
async fn users_are_listed_by_prefix_and_source() {
    let fixture = Fixture::new().await;
    let reply = fixture.call(Method::GET, USERS, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(ids(&reply, "userId"), ["admin", "anonymous"]);
    assert_eq!(
        reply.json()[0],
        json!({
            "userId": "admin",
            "firstName": "Administrator",
            "lastName": "User",
            "emailAddress": "admin@example.org",
            "source": "default",
            "status": "active",
            "readOnly": false,
            "roles": ["as-admin"],
            "externalRoles": [],
        })
    );
    for (query, expected) in [
        ("?userId=AD", vec!["admin"]),
        ("?userId=nobody", vec![]),
        ("?userId=%20", vec!["admin", "anonymous"]),
        ("?source=default&userId=an", vec!["anonymous"]),
        ("?source=LDAP", vec![]),
        ("?source=nope", vec![]),
    ] {
        let reply = fixture
            .call(Method::GET, &format!("{USERS}{query}"), Auth::Admin, None)
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{query}");
        assert_eq!(ids(&reply, "userId"), expected, "{query}");
    }
}

#[tokio::test]
async fn new_users_are_checked_like_the_reference_server() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(json!({})))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        error_ids(&reply),
        [
            "userId",
            "firstName",
            "lastName",
            "emailAddress",
            "password",
            "status"
        ]
    );

    let mut bad = user("jane");
    bad["emailAddress"] = json!("bad");
    bad["password"] = json!("short");
    bad["status"] = json!("bogus");
    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(bad))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["status", "emailAddress", "password"]);

    for user_id in ["jane:doe", " jane", "ja\tne"] {
        let reply = fixture
            .call(Method::POST, USERS, Auth::Admin, Some(user(user_id)))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{user_id}");
        assert_eq!(error_ids(&reply), ["userId"]);
    }

    let mut unknown_role = user("jane");
    unknown_role["roles"] = json!(["nope"]);
    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(unknown_role))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()[0]["id"], "roles");

    let mut mixed_case = user("jane");
    mixed_case["status"] = json!("Locked");
    mixed_case.as_object_mut().unwrap().remove("roles");
    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(mixed_case))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let created = reply.json();
    assert_eq!(created["status"], "locked");
    assert_eq!(created["roles"], json!([]));
    assert_eq!(created["source"], "default");
    assert!(created.get("password").is_none());

    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(user("jane")))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["userId"]);

    let reply = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(user("JANE")))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "user ids are case-sensitive");

    let events = audit(&fixture, "security.user").await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "created");
    assert_eq!(events[0].1, "jane");
    assert_eq!(events[0].2, "admin");
    assert!(!events[0].3.to_string().contains("correct horse"));
}

#[tokio::test]
async fn permissions_of_a_new_user_take_effect_at_once() {
    let fixture = Fixture::new().await;
    create_user(&fixture, "jane", "correct horse", &[]).await;
    let jane = || Auth::Basic("jane", "correct horse");
    let reply = fixture.call(Method::GET, USERS, jane(), None).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);

    create_role(&fixture, "readers", &["as-users-read"], &[]).await;
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/jane"),
            Auth::Admin,
            Some(update("jane", &["readers"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture.call(Method::GET, USERS, jane(), None).await;
    assert_eq!(reply.status, StatusCode::OK);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{ROLES}/readers"),
            Auth::Admin,
            Some(json!({"id": "readers", "name": "readers", "privileges": []})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::GET, USERS, jane(), None).await;
    assert_eq!(
        reply.status,
        StatusCode::FORBIDDEN,
        "taking a grant away is immediate too"
    );
}

const ROLES: &str = accounts::ROLES;

#[tokio::test]
async fn users_are_updated_in_place() {
    let fixture = Fixture::new().await;
    create_user(&fixture, "jane", "correct horse", &["as-anonymous"]).await;
    let path = format!("{USERS}/jane");

    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(update("other", &[])))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/nobody"),
            Auth::Admin,
            Some(update("nobody", &[])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["message"], "User 'nobody' not found.");

    let mut unknown_source = update("jane", &[]);
    unknown_source["source"] = json!("nope");
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(unknown_source))
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(json!({})))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["userId", "source", "status"]);

    let mut bad_email = update("jane", &[]);
    bad_email["emailAddress"] = json!("a@b..c");
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(bad_email))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_ids(&reply), ["emailAddress"]);

    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({
                "userId": "jane",
                "source": "default",
                "status": "disabled",
                "readOnly": true,
                "externalRoles": ["ignored"],
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(
            Method::GET,
            &format!("{USERS}?userId=jane"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json()[0],
        json!({
            "userId": "jane",
            "firstName": "",
            "lastName": "",
            "emailAddress": "",
            "source": "default",
            "status": "disabled",
            "readOnly": false,
            "roles": [],
            "externalRoles": [],
        })
    );
    let reply = fixture
        .call(
            Method::GET,
            astrum_api::STATUS,
            Auth::Basic("jane", "correct horse"),
            None,
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::UNAUTHORIZED,
        "a disabled user cannot sign in"
    );
}

#[tokio::test]
async fn external_users_only_get_role_mappings() {
    let fixture = Fixture::new().await;
    security_user_role::Entity::insert(security_user_role::ActiveModel {
        user_id: ActiveValue::Set("jdoe".to_owned()),
        source: ActiveValue::Set("LDAP".to_owned()),
        roles: ActiveValue::Set(json!(["as-anonymous"])),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().reload().await.unwrap();

    let reply = fixture
        .call(
            Method::GET,
            &format!("{USERS}?source=LDAP"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json(),
        json!([]),
        "LDAP users are listed from the directory, and none is configured"
    );

    let mut mapping = update("jdoe", &["as-admin"]);
    mapping["source"] = json!("LDAP");
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/jdoe"),
            Auth::Admin,
            Some(mapping),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let model = fixture.state.security().model();
    assert_eq!(model.roles_of("jdoe", "LDAP"), ["as-admin"]);
    assert!(!model.users.contains_key("jdoe"), "no local user appears");

    let mut unknown = update("ghost", &[]);
    unknown["source"] = json!("OAuth2");
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/ghost"),
            Auth::Admin,
            Some(unknown),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    for realm in ["NpmToken", "", "nope"] {
        let reply = fixture
            .call(
                Method::DELETE,
                &format!("{USERS}/jdoe?realm={realm}"),
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{realm}");
    }
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{USERS}/jdoe?realm=AstrumAuthenticatingRealm"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::NOT_FOUND,
        "jdoe is not a local user"
    );
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{USERS}/jdoe?realm=LdapRealm"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(
        fixture
            .state
            .security()
            .model()
            .roles_of("jdoe", "LDAP")
            .is_empty()
    );
}

#[tokio::test]
async fn deleting_users_follows_the_reference_server() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "deleters", &["as-users-all"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["deleters"]).await;
    let jane = || Auth::Basic("jane", "correct horse");

    let reply = fixture
        .call(Method::DELETE, &format!("{USERS}/jane"), jane(), None)
        .await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "nobody deletes themselves"
    );
    let reply = fixture
        .call(Method::DELETE, &format!("{USERS}/nobody"), jane(), None)
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["message"], "User 'nobody' not found.");

    astrum_db::setting::put(
        fixture.state.db().conn(),
        ANONYMOUS_KEY,
        &AnonymousSettings {
            enabled: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    fixture.state.security().reload().await.unwrap();
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{USERS}/anonymous"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "the anonymous user stays while anonymous access is on"
    );

    let reply = fixture
        .call(Method::DELETE, &format!("{USERS}/jane"), Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture.call(Method::GET, USERS, jane(), None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    create_user(&fixture, "jane", "correct horse", &[]).await;
    let reply = fixture
        .call(
            Method::GET,
            &format!("{USERS}?userId=jane"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json()[0]["roles"],
        json!([]),
        "role mappings go with the user"
    );
    let events = audit(&fixture, "security.user").await;
    assert!(
        events
            .iter()
            .any(|(kind, context, ..)| kind == "deleted" && context == "jane")
    );
}

#[tokio::test]
async fn changing_a_password_needs_its_own_permission() {
    let fixture = Fixture::new().await;
    create_role(&fixture, "readers", &["as-users-all"], &[]).await;
    create_role(&fixture, "helpdesk", &["as-userschangepw"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["readers"]).await;
    create_user(&fixture, "desk", "desk password", &["helpdesk"]).await;
    let change = |user: &str| format!("{USERS}/{user}/change-password");

    let reply = send_raw(
        &fixture,
        Method::PUT,
        &change("jane"),
        Auth::Basic("jane", "correct horse"),
        Some("text/plain"),
        "battery staple",
    )
    .await;
    assert_eq!(
        reply.status,
        StatusCode::FORBIDDEN,
        "not even one's own password without the permission"
    );

    let reply = send_raw(
        &fixture,
        Method::PUT,
        &change("jane"),
        Auth::Basic("desk", "desk password"),
        Some("application/json"),
        "\"battery staple\"",
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let reply = send_raw(
        &fixture,
        Method::PUT,
        &change("jane"),
        Auth::Basic("desk", "desk password"),
        None,
        "battery staple",
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    for (password, expected) in [
        ("short", StatusCode::BAD_REQUEST),
        ("", StatusCode::BAD_REQUEST),
    ] {
        let reply = send_raw(
            &fixture,
            Method::PUT,
            &change("jane"),
            Auth::Basic("desk", "desk password"),
            Some("text/plain"),
            password,
        )
        .await;
        assert_eq!(reply.status, expected, "{password:?}");
        assert_eq!(error_ids(&reply), ["password"]);
    }
    let reply = send_raw(
        &fixture,
        Method::PUT,
        &change("nobody"),
        Auth::Basic("desk", "desk password"),
        Some("text/plain"),
        "battery staple",
    )
    .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/jane"),
            Auth::Admin,
            Some(json!({"userId": "jane", "source": "default", "status": "changepassword", "roles": ["readers"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = send_raw(
        &fixture,
        Method::PUT,
        &change("jane"),
        Auth::Basic("desk", "desk password"),
        Some("text/plain; charset=utf-8"),
        "  battery staple  ",
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let old = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("jane", "correct horse"),
            None,
        )
        .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    let new = fixture
        .call(
            Method::GET,
            &format!("{USERS}?userId=jane"),
            Auth::Basic("jane", "  battery staple  "),
            None,
        )
        .await;
    assert_eq!(new.status, StatusCode::OK, "passwords are not trimmed");
    assert_eq!(new.json()[0]["status"], "active");

    let events = audit(&fixture, "security.user").await;
    let last = events.last().unwrap();
    assert_eq!(
        (last.0.as_str(), last.1.as_str(), last.2.as_str()),
        ("updated", "jane", "desk")
    );
    assert!(!last.3.to_string().contains("battery"));
}

#[tokio::test]
async fn changing_the_admin_password_removes_the_first_password_file() {
    let fixture = Fixture::new().await;
    let file = fixture.data.path().join(ADMIN_PASSWORD_FILE);
    assert!(file.exists());
    let reply = send_raw(
        &fixture,
        Method::PUT,
        &format!("{USERS}/admin/change-password"),
        Auth::Admin,
        Some("text/plain"),
        "a new admin password",
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert!(!file.exists());
    let reply = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("admin", "a new admin password"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
async fn every_user_endpoint_checks_permissions_first() {
    let fixture = Fixture::new().await;
    create_user(&fixture, "jane", "correct horse", &[]).await;
    let calls = [
        (Method::GET, USERS.to_owned(), None),
        (Method::POST, USERS.to_owned(), Some(user("john"))),
        (
            Method::PUT,
            format!("{USERS}/admin"),
            Some(update("admin", &[])),
        ),
        (Method::DELETE, format!("{USERS}/admin"), None),
        (Method::GET, format!("{SECURITY}/user-sources"), None),
    ];
    for (method, path, body) in calls {
        let reply = fixture
            .call(method.clone(), &path, Auth::None, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        assert!(reply.headers.get(header::WWW_AUTHENTICATE).is_some());
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
    let reply = send_raw(
        &fixture,
        Method::PUT,
        &format!("{USERS}/admin/change-password"),
        Auth::None,
        Some("text/plain"),
        "whatever password",
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn roles_are_granted_only_by_those_who_hold_them() {
    let fixture = Fixture::new().await;
    create_role(
        &fixture,
        "user-admins",
        &["as-users-all", "as-roles-read"],
        &[],
    )
    .await;
    create_role(&fixture, "readers", &["as-users-read"], &[]).await;
    create_user(&fixture, "jane", "correct horse", &["user-admins"]).await;
    let jane = || Auth::Basic("jane", "correct horse");

    let mut admin = user("john");
    admin["roles"] = json!(["as-admin"]);
    let reply = fixture.call(Method::POST, USERS, jane(), Some(admin)).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/jane"),
            jane(),
            Some(update("jane", &["user-admins", "as-admin"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "no way up to admin");

    let mut reader = user("john");
    reader["roles"] = json!(["readers"]);
    let reply = fixture
        .call(Method::POST, USERS, jane(), Some(reader))
        .await;
    assert_eq!(reply.status, StatusCode::OK);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/admin"),
            jane(),
            Some(json!({
                "userId": "admin",
                "firstName": "Administrator",
                "lastName": "Renamed",
                "emailAddress": "admin@example.org",
                "source": "default",
                "status": "active",
                "roles": ["as-admin"],
            })),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::NO_CONTENT,
        "roles a user already has are not grants"
    );
}

#[tokio::test]
async fn user_sources_name_their_realms() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(
            Method::GET,
            &format!("{SECURITY}/user-sources"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        json!([
            {"id": "default", "name": "AstrumAuthenticatingRealm"},
            {"id": "LDAP", "name": "LdapRealm"},
            {"id": "OAuth2", "name": "OAuth2Realm"},
        ])
    );
}

#[tokio::test]
async fn deleting_a_user_ends_their_sessions_and_tokens() {
    let fixture = Fixture::new().await;
    fixture.add_user("leaver", "secret-123", &[]).await;
    let token = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Basic("leaver", "secret-123"),
            Some(json!({"name": "ci", "expiresInDays": 5})),
        )
        .await
        .json()["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let signed_in = fixture
        .call(
            Method::POST,
            "/service/rest/internal/session",
            Auth::None,
            Some(json!({"username": "leaver", "password": "secret-123"})),
        )
        .await;
    assert_eq!(signed_in.status, StatusCode::OK);

    let deleted = fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/security/users/leaver",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let tokens = fixture
        .call(
            Method::GET,
            "/service/rest/internal/access-tokens?userId=*",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(tokens, json!([]));
    let sessions = astrum_db::entity::user_session::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert!(sessions.iter().all(|session| session.user_id != "leaver"));
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/service/rest/v1/security/users",
                Auth::Basic("leaver", &token),
                None
            )
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
