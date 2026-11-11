mod support;
#[path = "support/namesake.rs"]
mod world;

use astrum_db::entity::oauth_grant;
use axum::http::{Method, StatusCode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use support::Auth;
use world::*;

fn carol() -> Auth<'static> {
    Auth::Basic("carol", "carol-secret")
}

fn ids(listed: &Value) -> Vec<String> {
    let mut ids: Vec<String> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect();
    ids.sort();
    ids
}

#[tokio::test]
async fn grants_are_seen_and_revoked_only_in_their_own_realm() {
    let world = setup_bare(local_first()).await;
    let directory = world.session("bob", DIRECTORY_PASSWORD).await;
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    let mine = world.local_grant().await;
    let theirs = world.grant("bob", "LdapRealm").await;
    let ok = StatusCode::OK;
    let own = world.answer(ok, Method::GET, GRANTS, local(), None).await;
    assert_eq!(ids(&own.json()), [mine.to_string()]);
    let other = world.within(ok, Method::GET, GRANTS, &directory).await;
    assert_eq!(ids(&other.json()), [theirs.to_string()]);

    let missing = StatusCode::NOT_FOUND;
    let path = format!("{GRANTS}/{theirs}");
    world
        .answer(missing, Method::DELETE, &path, local(), None)
        .await;
    let path = format!("{GRANTS}/{mine}");
    world
        .within(missing, Method::DELETE, &path, &directory)
        .await;
    let gone = StatusCode::NO_CONTENT;
    world
        .answer(gone, Method::DELETE, &path, local(), None)
        .await;
    let path = format!("{GRANTS}/{theirs}");
    world.within(gone, Method::DELETE, &path, &directory).await;
}

#[tokio::test]
async fn deleting_a_local_user_keeps_the_grants_of_the_directory_namesake() {
    let world = setup_bare(local_first()).await;
    world.session("bob", DIRECTORY_PASSWORD).await;
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    world.local_grant().await;
    world.grant("bob", "LdapRealm").await;
    let user = "/service/rest/v1/security/users/bob";
    world
        .answer(
            StatusCode::NO_CONTENT,
            Method::DELETE,
            user,
            Auth::Admin,
            None,
        )
        .await;
    let kept = oauth_grant::Entity::find()
        .filter(oauth_grant::Column::UserId.eq("bob"))
        .all(world.fixture.state.db().conn())
        .await
        .unwrap();
    let realms: Vec<&str> = kept.iter().map(|grant| grant.realm.as_str()).collect();
    assert_eq!(realms, ["LdapRealm"]);
}

#[tokio::test]
async fn a_directory_administrator_manages_the_tokens_of_other_users() {
    let world = setup().await;
    let id = world.local_token().await["id"].as_str().unwrap().to_owned();
    let ok = StatusCode::OK;
    let one = format!("{TOKENS}?userId=bob");
    let listed = world.answer(ok, Method::GET, &one, alice(), None).await;
    assert_eq!(ids(&listed.json()), std::slice::from_ref(&id));
    let every = format!("{TOKENS}?userId=*");
    let listed = world.answer(ok, Method::GET, &every, alice(), None).await;
    assert_eq!(ids(&listed.json()), std::slice::from_ref(&id));
    let path = format!("{TOKENS}/{id}");
    let gone = StatusCode::NO_CONTENT;
    world
        .answer(gone, Method::DELETE, &path, alice(), None)
        .await;
    let left = world.answer(ok, Method::GET, TOKENS, local(), None).await;
    assert!(ids(&left.json()).is_empty());
}

#[tokio::test]
async fn a_directory_user_without_the_privilege_manages_no_tokens() {
    let world = setup().await;
    let id = world.local_token().await["id"].as_str().unwrap().to_owned();
    let denied = StatusCode::FORBIDDEN;
    for path in [format!("{TOKENS}?userId=bob"), format!("{TOKENS}?userId=*")] {
        world
            .answer(denied, Method::GET, &path, carol(), None)
            .await;
    }
    world
        .answer(denied, Method::GET, TOKENS, carol(), None)
        .await;
    let body = json!({"name": "mine", "scope": "WRITE", "expiresInDays": null});
    world
        .answer(denied, Method::POST, TOKENS, carol(), Some(body))
        .await;
    let path = format!("{TOKENS}/{id}");
    world
        .answer(StatusCode::NOT_FOUND, Method::DELETE, &path, carol(), None)
        .await;
    let left = world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
    assert_eq!(ids(&left.json()), [id]);
}

#[tokio::test]
async fn directory_only_users_get_no_credentials_to_keep() {
    let world = setup().await;
    let denied = StatusCode::FORBIDDEN;
    let token = json!({"name": "dead", "scope": "WRITE", "expiresInDays": null});
    let reply = world
        .answer(denied, Method::POST, TOKENS, alice(), Some(token))
        .await;
    assert!(
        reply.text().contains("managed outside Astrum"),
        "{}",
        reply.text()
    );
    let key = json!({"password": ALICE_PASSWORD});
    world
        .answer(denied, Method::POST, NUGET_KEY, alice(), Some(key))
        .await;
    world
        .answer(denied, Method::GET, CARGO_ME, alice(), None)
        .await;
    world
        .answer(denied, Method::GET, GEMS_KEY, alice(), None)
        .await;
}
