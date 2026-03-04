mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const STORES: &str = "/service/rest/v1/blobstores";
const GROUP: &str = "/service/rest/v1/blobstores/group";
const LIST: &str = "/service/rest/internal/ui/blobstores";

async fn file_store(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("{STORES}/file"),
            Auth::Admin,
            Some(json!({"name": name, "path": name})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn send(fixture: &Fixture, method: Method, path: &str, body: Value) -> Reply {
    fixture.call(method, path, Auth::Admin, Some(body)).await
}

async fn stores() -> Fixture {
    let fixture = ready().await;
    for name in ["one", "two", "three"] {
        file_store(&fixture, name).await;
    }
    fixture
}

#[tokio::test]
async fn the_internal_list_shows_usage_and_who_uses_each_store() {
    let fixture = stores().await;
    send(
        &fixture,
        Method::POST,
        GROUP,
        json!({"name": "pool", "members": ["one", "two"], "softQuota": {"type": "spaceUsedQuota", "limit": 5}}),
    )
    .await;
    raw_hosted_in(&fixture, "pooled", "pool").await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "hidden").await;
    put(&fixture, "/repository/pooled/a.txt", "1234567890").await;

    let reply = get(&fixture, LIST, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let list = reply.json();
    let store = |name: &str| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|store| store["name"] == name)
            .unwrap()
            .clone()
    };
    let default = store("default");
    assert_eq!(default["type"], "File");
    assert_eq!(default["path"], "default");
    assert_eq!(default["repositories"], json!(["files", "hidden"]));
    assert_eq!(default["repositoryCount"], 2);
    assert_eq!(default["deletable"], false);
    assert_eq!(default["softQuota"], Value::Null);
    assert_eq!(default["quotaStatus"], Value::Null);
    assert!(default["availableSpace"].is_number());

    let pool = store("pool");
    assert_eq!(pool["type"], "Group");
    assert_eq!(pool["members"], json!(["one", "two"]));
    assert_eq!(pool["fillPolicy"], "writeToFirst");
    assert_eq!(pool["repositories"], json!(["pooled"]));
    assert_eq!(pool["blobCount"], 1);
    assert_eq!(pool["totalSize"], 10);
    assert_eq!(pool["quotaStatus"]["violation"], true);
    let one = store("one");
    assert_eq!(one["groups"], json!(["pool"]));
    assert_eq!(one["deletable"], false);
    assert_eq!(one["blobCount"], 1);
    let three = store("three");
    assert_eq!(three["deletable"], true);
    assert_eq!(three["groups"], json!([]));

    user_with(
        &fixture,
        "storekeeper",
        &["as-blobstores-read", "as-repository-view-raw-files-read"],
    )
    .await;
    let list = get(&fixture, LIST, user("storekeeper")).await.json();
    let default = list
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "default")
        .unwrap();
    assert_eq!(default["repositories"], json!(["files"]));
    assert_eq!(default["repositoryCount"], 2);

    user_with(&fixture, "outsider", &["as-repository-view-raw-files-read"]).await;
    assert_eq!(
        get(&fixture, LIST, user("outsider")).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&fixture, LIST, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
}
