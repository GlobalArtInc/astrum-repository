mod support;
#[path = "support/namesake.rs"]
mod world;

use axum::http::{Method, StatusCode};
use serde_json::json;
use support::Auth;
use world::*;

#[tokio::test]
async fn a_namesake_cannot_create_access_tokens_of_the_local_user() {
    let world = setup().await;
    let body = json!({"name": "stolen", "scope": "WRITE", "expiresInDays": null});
    world
        .answer(REFUSED, Method::POST, TOKENS, namesake(), Some(body))
        .await;
    let listed = world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
    assert_eq!(listed.json().as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_namesake_cannot_list_the_access_tokens_of_the_local_user() {
    let world = setup().await;
    world.local_token().await;
    world
        .answer(REFUSED, Method::GET, TOKENS, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_delete_the_access_tokens_of_the_local_user() {
    let world = setup().await;
    let id = world.local_token().await["id"].as_str().unwrap().to_owned();
    let path = format!("{TOKENS}/{id}");
    world
        .answer(REFUSED, Method::DELETE, &path, namesake(), None)
        .await;
    let listed = world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_namesake_cannot_read_the_nuget_key_of_the_local_user() {
    let world = setup().await;
    world.local_nuget_key().await;
    let body = json!({"password": DIRECTORY_PASSWORD});
    world
        .answer(REFUSED, Method::POST, NUGET_KEY, namesake(), Some(body))
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_reset_the_nuget_key_of_the_local_user() {
    let world = setup().await;
    let key = world.local_nuget_key().await;
    let body = json!({"password": DIRECTORY_PASSWORD});
    let path = format!("{NUGET_KEY}/reset");
    world
        .answer(REFUSED, Method::POST, &path, namesake(), Some(body))
        .await;
    assert_eq!(world.local_nuget_key().await, key);
}

#[tokio::test]
async fn a_namesake_gets_no_cargo_token_of_the_local_user() {
    let world = setup().await;
    world
        .answer(REFUSED, Method::GET, CARGO_ME, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_gets_no_rubygems_key_of_the_local_user() {
    let world = setup().await;
    world
        .answer(StatusCode::OK, Method::GET, GEMS_KEY, local(), None)
        .await;
    world
        .answer(REFUSED, Method::GET, GEMS_KEY, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_revoke_the_rubygems_key_of_the_local_user() {
    let world = setup().await;
    world
        .answer(StatusCode::OK, Method::GET, GEMS_KEY, local(), None)
        .await;
    world
        .answer(REFUSED, Method::DELETE, GEMS_KEY, namesake(), None)
        .await;
    world
        .answer(StatusCode::OK, Method::DELETE, GEMS_KEY, local(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_list_the_oauth_grants_of_the_local_user() {
    let world = setup().await;
    world.local_grant().await;
    world
        .answer(REFUSED, Method::GET, GRANTS, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_revoke_the_oauth_grants_of_the_local_user() {
    let world = setup().await;
    let id = world.local_grant().await;
    let path = format!("{GRANTS}/{id}");
    world
        .answer(REFUSED, Method::DELETE, &path, namesake(), None)
        .await;
    world
        .answer(StatusCode::NO_CONTENT, Method::DELETE, &path, local(), None)
        .await;
}

#[tokio::test]
async fn the_rule_holds_when_the_directory_realm_comes_first() {
    let realms = json!(["LdapRealm", "AstrumAuthenticatingRealm", "AccessTokenRealm"]);
    let world = setup_with(realms).await;
    world
        .answer(REFUSED, Method::GET, TOKENS, namesake(), None)
        .await;
    world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
}

#[tokio::test]
async fn the_rule_ignores_the_case_of_ids() {
    let world = setup().await;
    let password = "carol-local-password";
    world
        .fixture
        .add_user("Carol", password, &["as-admin"])
        .await;
    for typed in ["carol", "CAROL"] {
        let who = Auth::Basic(typed, "carol-secret");
        world.answer(REFUSED, Method::GET, TOKENS, who, None).await;
    }
    let own = Auth::Basic("Carol", password);
    world
        .answer(StatusCode::OK, Method::GET, TOKENS, own, None)
        .await;
}

#[tokio::test]
async fn a_deleted_local_user_frees_the_id_for_the_directory() {
    let world = setup().await;
    let path = "/service/rest/v1/repositories";
    world
        .answer(REFUSED, Method::GET, path, namesake(), None)
        .await;
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
    world
        .answer(StatusCode::OK, Method::GET, path, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_directory_user_without_a_local_namesake_still_reads_repositories() {
    let world = setup().await;
    let path = "/service/rest/v1/repositories";
    let reply = world
        .answer(StatusCode::OK, Method::GET, path, alice(), None)
        .await;
    assert!(reply.text().contains("crates"), "{}", reply.text());
}
