mod support;
#[path = "support/namesake.rs"]
mod world;

use astrum_db::entity::{audit_event, oauth_grant, user_session};
use astrum_security::Scope;
use axum::http::{Method, StatusCode};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use support::Auth;
use world::*;

const USERS: &str = "/service/rest/v1/security/users";
const LOCAL_REALM: &str = "AstrumAuthenticatingRealm";
const DIRECTORY_REALM: &str = "LdapRealm";

#[tokio::test]
async fn a_directory_session_ends_when_a_local_namesake_appears() {
    let world = setup_bare(local_first()).await;
    let session = world.session("bob", DIRECTORY_PASSWORD).await;
    world
        .within(StatusCode::OK, Method::GET, GRANTS, &session)
        .await;
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    world.within(REFUSED, Method::GET, GRANTS, &session).await;
}

#[tokio::test]
async fn grants_and_tickets_of_a_directory_user_stop_resolving_under_a_local_namesake() {
    let world = setup_bare(local_first()).await;
    let security = world.fixture.state.security();
    let subject = security.subject_of("bob", DIRECTORY_REALM).await;
    assert_eq!(
        subject.map(|subject| subject.source),
        Some("LDAP".to_owned())
    );
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    assert!(security.subject_of("bob", DIRECTORY_REALM).await.is_none());
    assert!(security.subject_of("bob", LOCAL_REALM).await.is_some());
}

#[tokio::test]
async fn a_directory_session_without_a_namesake_lives_on() {
    let world = setup_bare(local_first()).await;
    let session = world.session("alice", ALICE_PASSWORD).await;
    let ok = StatusCode::OK;
    world.within(ok, Method::GET, GRANTS, &session).await;
    world
        .fixture
        .add_user("dora", "dora-local-password", &["as-admin"])
        .await;
    world.within(ok, Method::GET, GRANTS, &session).await;
    world.within(ok, Method::GET, GRANTS, &session).await;
}

async fn session_row(world: &World, user: &str, realm: &str) {
    let now = chrono::Utc::now().fixed_offset();
    user_session::Entity::insert(user_session::ActiveModel {
        token_hash: ActiveValue::Set(format!("{user}-{realm}")),
        user_id: ActiveValue::Set(user.to_owned()),
        realm: ActiveValue::Set(realm.to_owned()),
        credential_stamp: ActiveValue::Set(String::new()),
        csrf_token: ActiveValue::Set("csrf".to_owned()),
        created_at: ActiveValue::Set(now),
        last_seen_at: ActiveValue::Set(now),
        client_ip: ActiveValue::Set(String::new()),
        user_agent: ActiveValue::Set(String::new()),
    })
    .exec(world.fixture.state.db().conn())
    .await
    .unwrap();
}

async fn create_local_user(world: &World, id: &str) {
    let body = json!({"userId": id, "firstName": "New", "lastName": "User",
        "emailAddress": format!("{id}@example.org"), "password": "fresh-password-1",
        "status": "active", "roles": ["as-admin"]});
    world
        .answer(StatusCode::OK, Method::POST, USERS, Auth::Admin, Some(body))
        .await;
}

async fn created_event(world: &World, id: &str) -> Value {
    let events = audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("security.user"))
        .filter(audit_event::Column::Kind.eq("created"))
        .filter(audit_event::Column::Context.eq(id))
        .all(world.fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    events[0].attributes.clone()
}

#[tokio::test]
async fn a_new_local_user_does_not_inherit_the_credentials_of_its_id() {
    let world = setup().await;
    let security = world.fixture.state.security();
    let (_, old) = security
        .create_token("zoe", "old", Scope::Write, None)
        .await
        .unwrap();
    let dead = || Auth::Basic("zoe", old.as_str());
    world
        .answer(REFUSED, Method::GET, USERS, dead(), None)
        .await;
    security
        .api_key(
            world.fixture.state.secrets(),
            astrum_security::api_keys::NUGET,
            "zoe",
        )
        .await
        .unwrap();
    security
        .issue_format_token("NpmToken", "zoe", None)
        .await
        .unwrap();
    world.grant("zoe", LOCAL_REALM).await;
    world.grant("zoe", DIRECTORY_REALM).await;
    session_row(&world, "zoe", LOCAL_REALM).await;
    session_row(&world, "zoe", DIRECTORY_REALM).await;

    create_local_user(&world, "zoe").await;

    world
        .answer(REFUSED, Method::GET, USERS, dead(), None)
        .await;
    let created = json!({"name": "fresh", "scope": "WRITE", "expiresInDays": null});
    let own = Auth::Basic("zoe", "fresh-password-1");
    let reply = world
        .answer(
            StatusCode::CREATED,
            Method::POST,
            TOKENS,
            own,
            Some(created),
        )
        .await;
    let fresh = reply.json()["token"].as_str().unwrap().to_owned();
    let who = Auth::Basic("zoe", fresh.as_str());
    world
        .answer(StatusCode::OK, Method::GET, USERS, who, None)
        .await;

    let event = created_event(&world, "zoe").await;
    let counts = json!({"accessTokens": 1, "apiKeys": 1, "formatTokens": 1,
        "oauthGrants": 1, "sessions": 1});
    assert_eq!(event["credentialsCleared"], counts);
    let db = world.fixture.state.db().conn();
    let grants = oauth_grant::Entity::find().all(db).await.unwrap();
    let realms: Vec<&str> = grants.iter().map(|grant| grant.realm.as_str()).collect();
    assert_eq!(realms, [DIRECTORY_REALM]);
    let sessions = user_session::Entity::find().all(db).await.unwrap();
    let realms: Vec<&str> = sessions.iter().map(|row| row.realm.as_str()).collect();
    assert_eq!(realms, [DIRECTORY_REALM]);
}

#[tokio::test]
async fn creating_a_user_without_leftovers_clears_nothing() {
    let world = setup().await;
    let held = world.local_token().await["token"]
        .as_str()
        .unwrap()
        .to_owned();
    create_local_user(&world, "yara").await;
    let event = created_event(&world, "yara").await;
    assert!(event.get("credentialsCleared").is_none(), "{event}");
    let who = Auth::Basic("bob", held.as_str());
    world
        .answer(StatusCode::OK, Method::GET, USERS, who, None)
        .await;
}
