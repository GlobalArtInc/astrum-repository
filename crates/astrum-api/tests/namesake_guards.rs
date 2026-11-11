mod support;
#[path = "support/namesake.rs"]
mod world;

use astrum_security::Scope;
use astrum_testkit::ldap::Entry;
use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use support::Auth;
use world::*;

const AUTHENTICATE: &str = "/service/rest/wonderland/authenticate";
const NUGET_TICKET: &str = "/service/rest/internal/nuget-api-key";

fn zed() -> Auth<'static> {
    Auth::Basic("zed", "zed-secret")
}

#[tokio::test]
async fn a_directory_entry_whose_canonical_id_is_local_is_refused() {
    let world = setup_bare(local_first()).await;
    world.directory.add(Entry::new(
        "uid=zed,ou=people,dc=example,dc=com",
        &[
            ("objectClass", &["top", "person", "inetOrgPerson"]),
            ("uid", &["bob", "zed"]),
            ("cn", &["Zed Twin"]),
            ("sn", &["Twin"]),
            ("mail", &["zed@example.com"]),
            ("userPassword", &["zed-secret"]),
        ],
    ));
    world
        .answer(StatusCode::OK, Method::GET, GRANTS, zed(), None)
        .await;
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    world
        .answer(REFUSED, Method::GET, GRANTS, zed(), None)
        .await;
}

fn credentials(user: &str, password: &str) -> serde_json::Value {
    json!({"u": STANDARD.encode(user), "p": STANDARD.encode(password)})
}

#[tokio::test]
async fn the_nuget_authenticate_route_refuses_a_directory_user() {
    let world = setup().await;
    let body = credentials("alice", ALICE_PASSWORD);
    let reply = world
        .answer(
            StatusCode::FORBIDDEN,
            Method::POST,
            AUTHENTICATE,
            alice(),
            Some(body),
        )
        .await;
    assert!(
        reply.text().contains("managed outside Astrum"),
        "{}",
        reply.text()
    );
    let body = credentials("bob", LOCAL_PASSWORD);
    world
        .answer(
            StatusCode::OK,
            Method::POST,
            AUTHENTICATE,
            local(),
            Some(body),
        )
        .await;
}

fn ticket_path(world: &World, user: &str) -> String {
    let expires = chrono::Utc::now().timestamp() + 120;
    let sealed = world
        .fixture
        .state
        .secrets()
        .seal(&json!({"user": user, "expires": expires}).to_string());
    let token: String =
        url::form_urlencoded::byte_serialize(STANDARD.encode(sealed).as_bytes()).collect();
    format!("{NUGET_TICKET}?authToken={token}")
}

#[tokio::test]
async fn the_nuget_ticket_routes_refuse_a_directory_user() {
    let world = setup().await;
    world
        .answer(
            StatusCode::FORBIDDEN,
            Method::GET,
            &ticket_path(&world, "alice"),
            alice(),
            None,
        )
        .await;
    world
        .answer(
            StatusCode::OK,
            Method::GET,
            &ticket_path(&world, "bob"),
            local(),
            None,
        )
        .await;
}

#[tokio::test]
async fn a_directory_session_cannot_delete_a_token_left_under_its_id() {
    let world = setup().await;
    let security = world.fixture.state.security();
    let (left, _) = security
        .create_token("alice", "left", Scope::Write, None)
        .await
        .unwrap();
    let session = world.session("alice", ALICE_PASSWORD).await;
    let path = format!("{TOKENS}/{}", left.id);
    world
        .within(StatusCode::FORBIDDEN, Method::DELETE, &path, &session)
        .await;
    assert!(security.token(left.id).await.unwrap().is_some());
}
