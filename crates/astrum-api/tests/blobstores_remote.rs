mod support;
mod ui_support;

use std::net::SocketAddr;
use std::path::Path;

use astrum_blob_agent::{Agent, serve};
use astrum_db::entity::{audit_event, blob_store};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::http::{Method, StatusCode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tokio::net::TcpListener;
use ui_support::*;

const STORES: &str = "/service/rest/v1/blobstores";
const REMOTE: &str = "/service/rest/v1/blobstores/remote";
const GROUP: &str = "/service/rest/v1/blobstores/group";
const TOKEN: &str = "an-agent-token-for-the-api-tests-0123456789";

async fn agent(dir: &Path) -> SocketAddr {
    let agent = Agent::open(dir, TOKEN).await.unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(serve(
        listener,
        agent.router(),
        None,
        std::future::pending(),
    ));
    address
}

async fn allow_loopback(fixture: &Fixture) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

async fn send(fixture: &Fixture, method: Method, path: &str, body: Value) -> Reply {
    fixture.call(method, path, Auth::Admin, Some(body)).await
}

async fn post(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::POST, path, Auth::Admin, None).await
}

fn ids(reply: &Reply) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap_or_else(|| panic!("an array of errors: {}", reply.text()))
        .iter()
        .map(|error| error["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn stored_attributes(fixture: &Fixture, name: &str) -> Value {
    blob_store::Entity::find_by_id(name.to_owned())
        .one(fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap()
        .attributes
}

async fn audit_kinds(fixture: &Fixture, name: &str) -> Vec<String> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("blobstore"))
        .filter(audit_event::Column::Context.eq(name))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
        .into_iter()
        .map(|event| event.kind)
        .collect()
}

#[tokio::test]
async fn a_remote_store_is_created_read_and_changed() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let dir = tempfile::tempdir().unwrap();
    let address = agent(dir.path()).await;
    let url = format!("http://{address}");

    let created = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": format!("{url}/"), "token": TOKEN, "softQuota": {"type": "spaceRemainingQuota", "limit": 1000}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let read = get(&fixture, &format!("{REMOTE}/far"), Auth::Admin).await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(
        read.json(),
        json!({"name": "far", "type": "Remote", "softQuota": {"type": "spaceRemainingQuota", "limit": 1000}, "url": url, "certificate": null})
    );
    assert!(!read.text().contains(TOKEN));
    let attributes = stored_attributes(&fixture, "far").await;
    let sealed = attributes["remote"]["token"].as_str().unwrap();
    assert_ne!(sealed, TOKEN);
    assert_eq!(fixture.state.secrets().open(sealed).unwrap(), TOKEN);

    let listed = get(&fixture, STORES, Auth::Admin).await.json();
    let far = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "far")
        .unwrap()
        .clone();
    assert_eq!(far["type"], "Remote");
    assert_eq!(far["unavailable"], false);
    assert!(far["availableSpaceInBytes"].as_u64().unwrap() > 0, "{far}");
    let ui = get(
        &fixture,
        "/service/rest/internal/ui/blobstores",
        Auth::Admin,
    )
    .await
    .json();
    let far_ui = ui
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "far")
        .unwrap()
        .clone();
    assert_eq!(far_ui["url"], url);

    let unchanged = send(
        &fixture,
        Method::PUT,
        &format!("{REMOTE}/far"),
        json!({"url": url}),
    )
    .await;
    assert_eq!(
        unchanged.status,
        StatusCode::NO_CONTENT,
        "{}",
        unchanged.text()
    );
    assert_eq!(
        stored_attributes(&fixture, "far").await["remote"]["token"],
        attributes["remote"]["token"]
    );
    assert_eq!(
        get(&fixture, &format!("{REMOTE}/far"), Auth::Admin)
            .await
            .json()["softQuota"],
        Value::Null
    );

    let other = tempfile::tempdir().unwrap();
    let moved_to = format!("http://{}", agent(other.path()).await);
    let without_token = send(
        &fixture,
        Method::PUT,
        &format!("{REMOTE}/far"),
        json!({"url": moved_to}),
    )
    .await;
    assert_eq!(without_token.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&without_token), vec!["token"]);
    let moved = send(
        &fixture,
        Method::PUT,
        &format!("{REMOTE}/far"),
        json!({"url": moved_to, "token": TOKEN}),
    )
    .await;
    assert_eq!(moved.status, StatusCode::NO_CONTENT, "{}", moved.text());

    let file = send(
        &fixture,
        Method::PUT,
        &format!("{STORES}/file/far"),
        json!({"path": "far"}),
    )
    .await;
    assert_eq!(file.status, StatusCode::BAD_REQUEST, "{}", file.text());
    assert_eq!(
        get(&fixture, &format!("{REMOTE}/default"), Auth::Admin)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        audit_kinds(&fixture, "far").await,
        ["created", "updated", "updated"]
    );
}

#[tokio::test]
async fn remote_requests_are_validated() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let dir = tempfile::tempdir().unwrap();
    let address = agent(dir.path()).await;

    let empty = send(&fixture, Method::POST, REMOTE, json!({"name": "far"})).await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&empty), vec!["url", "token"]);

    let bad = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": "ftp://x", "token": "short", "certificate": "nonsense"}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&bad), vec!["url", "token", "certificate"]);

    let path = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": format!("http://{address}/blobs"), "token": TOKEN}),
    )
    .await;
    assert_eq!(ids(&path), vec!["url"]);

    let wrong_token = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": format!("http://{address}"), "token": "the-wrong-token-for-this-agent-0123456789"}),
    )
    .await;
    assert_eq!(wrong_token.status, StatusCode::BAD_REQUEST);
    assert!(wrong_token.text().contains("401"), "{}", wrong_token.text());

    let closed = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gone = closed.local_addr().unwrap();
    drop(closed);
    let unreachable = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": format!("http://{gone}"), "token": TOKEN}),
    )
    .await;
    assert_eq!(unreachable.status, StatusCode::BAD_REQUEST);
    assert!(
        unreachable.text().contains("The agent cannot be used"),
        "{}",
        unreachable.text()
    );
    assert!(
        blob_store::Entity::find_by_id("far".to_owned())
            .one(fixture.state.db().conn())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn an_agent_on_an_internal_address_needs_to_be_allowed() {
    let fixture = ready().await;
    let dir = tempfile::tempdir().unwrap();
    let address = agent(dir.path()).await;
    let body = json!({"name": "far", "url": format!("http://{address}"), "token": TOKEN});
    let refused = send(&fixture, Method::POST, REMOTE, body.clone()).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&refused), vec!["url"]);
    assert!(
        refused.text().contains("private/local"),
        "{}",
        refused.text()
    );
    allow_loopback(&fixture).await;
    let created = send(&fixture, Method::POST, REMOTE, body).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
}

#[tokio::test]
async fn remote_stores_need_the_blob_store_permissions() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    user_with(&fixture, "reader", &["as-blobstores-read"]).await;
    let dir = tempfile::tempdir().unwrap();
    let address = agent(dir.path()).await;
    let body = json!({"name": "far", "url": format!("http://{address}"), "token": TOKEN});
    let denied = fixture
        .call(
            Method::POST,
            REMOTE,
            Auth::Basic("reader", "password-1"),
            Some(body.clone()),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let converted = fixture
        .call(
            Method::POST,
            &format!("{GROUP}/convert/default/default-disk"),
            Auth::Basic("reader", "password-1"),
            None,
        )
        .await;
    assert_eq!(converted.status, StatusCode::FORBIDDEN);
    assert_eq!(
        send(&fixture, Method::POST, REMOTE, body).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(
            &fixture,
            &format!("{REMOTE}/far"),
            Auth::Basic("reader", "password-1")
        )
        .await
        .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_store_with_repositories_grows_through_a_group_with_an_agent() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    raw_hosted(&fixture, "files").await;
    put(
        &fixture,
        "/repository/files/old.txt",
        "kept on the local disk",
    )
    .await;

    let converted = post(&fixture, &format!("{GROUP}/convert/default/default-disk")).await;
    assert_eq!(
        converted.status,
        StatusCode::NO_CONTENT,
        "{}",
        converted.text()
    );
    assert_eq!(
        get(&fixture, &format!("{GROUP}/default"), Auth::Admin)
            .await
            .json(),
        json!({"name": "default", "softQuota": null, "members": ["default-disk"], "fillPolicy": "writeToFirst"})
    );
    assert_eq!(
        get(
            &fixture,
            &format!("{STORES}/file/default-disk"),
            Auth::Admin
        )
        .await
        .json()["path"],
        "default"
    );
    let old = get(&fixture, "/repository/files/old.txt", Auth::Admin).await;
    assert_eq!(old.status, StatusCode::OK);
    assert_eq!(old.text(), "kept on the local disk");

    let dir = tempfile::tempdir().unwrap();
    let address = agent(dir.path()).await;
    let created = send(
        &fixture,
        Method::POST,
        REMOTE,
        json!({"name": "far", "url": format!("http://{address}"), "token": TOKEN}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let grown = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/default"),
        json!({"members": ["far", "default-disk"], "fillPolicy": "writeToFirst"}),
    )
    .await;
    assert_eq!(grown.status, StatusCode::NO_CONTENT, "{}", grown.text());

    put(&fixture, "/repository/files/new.txt", "kept by the agent").await;
    let fresh = get(&fixture, "/repository/files/new.txt", Auth::Admin).await;
    assert_eq!(fresh.status, StatusCode::OK);
    assert_eq!(fresh.text(), "kept by the agent");
    let on_agent = get(&fixture, STORES, Auth::Admin)
        .await
        .json()
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "far")
        .unwrap()["blobCount"]
        .as_u64()
        .unwrap();
    assert!(on_agent >= 1, "{on_agent}");
    assert_eq!(
        get(&fixture, "/repository/files/old.txt", Auth::Admin)
            .await
            .text(),
        "kept on the local disk"
    );

    let roomiest = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/default"),
        json!({"members": ["far", "default-disk"], "fillPolicy": "mostFreeSpace"}),
    )
    .await;
    assert_eq!(
        roomiest.status,
        StatusCode::NO_CONTENT,
        "{}",
        roomiest.text()
    );
    assert_eq!(
        get(&fixture, &format!("{GROUP}/default"), Auth::Admin)
            .await
            .json()["fillPolicy"],
        "mostFreeSpace"
    );
    put(&fixture, "/repository/files/third.txt", "anywhere").await;
    assert_eq!(
        get(&fixture, "/repository/files/third.txt", Auth::Admin)
            .await
            .text(),
        "anywhere"
    );

    let deleted = fixture
        .call(Method::DELETE, &format!("{STORES}/far"), Auth::Admin, None)
        .await;
    assert_eq!(
        deleted.status,
        StatusCode::BAD_REQUEST,
        "{}",
        deleted.text()
    );
    assert_eq!(
        audit_kinds(&fixture, "default").await,
        ["converted", "updated", "updated"]
    );
}

#[tokio::test]
async fn only_a_plain_store_outside_groups_converts() {
    let fixture = ready().await;
    for name in ["one", "two"] {
        let reply = send(
            &fixture,
            Method::POST,
            &format!("{STORES}/file"),
            json!({"name": name, "path": name}),
        )
        .await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    }
    let pool = send(
        &fixture,
        Method::POST,
        GROUP,
        json!({"name": "pool", "members": ["one"]}),
    )
    .await;
    assert_eq!(pool.status, StatusCode::CREATED, "{}", pool.text());

    let member = post(&fixture, &format!("{GROUP}/convert/one/one-disk")).await;
    assert_eq!(member.status, StatusCode::BAD_REQUEST);
    assert!(
        member.text().contains("member of the group pool"),
        "{}",
        member.text()
    );
    let group = post(&fixture, &format!("{GROUP}/convert/pool/pool-disk")).await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST);
    assert!(group.text().contains("already"), "{}", group.text());
    let taken = post(&fixture, &format!("{GROUP}/convert/two/ONE")).await;
    assert_eq!(taken.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&taken), vec!["name"]);
    let invalid = post(&fixture, &format!("{GROUP}/convert/two/_x")).await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&invalid), vec!["PARAMETER newNameForOriginal"]);
    let missing = post(&fixture, &format!("{GROUP}/convert/nothing/x")).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let wrong_policy = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/pool"),
        json!({"members": ["one"], "fillPolicy": "biggestFirst"}),
    )
    .await;
    assert_eq!(wrong_policy.status, StatusCode::BAD_REQUEST);
    assert!(
        wrong_policy.text().contains("mostFreeSpace"),
        "{}",
        wrong_policy.text()
    );
}
