mod cluster;
mod support;

use axum::http::{Method, StatusCode};
use cluster::{Node, connect, replicate};
use serde_json::json;
use support::Auth;

const ITEMS: &str = "SELECT count(*) AS n FROM replication_item";

async fn watched() -> Node {
    let node = Node::start(&["watched", "plain"]).await;
    let other = Node::start(&["mirror"]).await;
    let remote = connect(&node, &other).await;
    replicate(&node, &remote, "watched", "mirror", json!({})).await;
    replicate(&node, &remote, "plain", "mirror", json!({"enabled": false})).await;
    node
}

#[tokio::test]
async fn the_capture_skips_replicated_writes_and_repositories_without_replications() {
    let node = watched().await;
    node.put("plain", "/docs/a.txt", b"alpha").await;
    assert_eq!(node.state().await, None);
    node.insert("watched", "replicated", "/d/r").await;
    node.insert("watched", "tiered", "/d/t").await;
    assert_eq!(node.state().await, None);
    node.insert("watched", "", "/d/l").await;
    let queued = "SELECT count(*) AS n FROM replication_item WHERE name = '/d/l'";
    assert_eq!(node.number(queued).await, 1);
    assert_eq!(node.number(ITEMS).await, 1);
}

#[tokio::test]
async fn an_attribute_update_enqueues_unless_it_changes_nothing_or_was_replicated() {
    let node = watched().await;
    node.insert("watched", "", "/d/y").await;
    let update = "UPDATE component SET attributes = '{\"yanked\": true}' WHERE name = '/d/y'";
    let clear = "DELETE FROM replication_item";
    node.sql(clear).await;
    node.sql(update).await;
    assert_eq!(node.number(ITEMS).await, 1);
    node.sql(clear).await;
    node.sql(update).await;
    assert_eq!(node.number(ITEMS).await, 0);
    node.sql_as(
        "replicated",
        "UPDATE component SET attributes = '{\"yanked\": false}' WHERE name = '/d/y'",
    )
    .await;
    assert_eq!(node.number(ITEMS).await, 0);
    node.sql("UPDATE component SET attributes = '{\"yanked\": false}' WHERE name = '/d/y'")
        .await;
    assert_eq!(node.number(ITEMS).await, 0);
    node.sql("UPDATE component SET attributes = '{\"yanked\": true}' WHERE name = '/d/y'")
        .await;
    assert_eq!(node.number(ITEMS).await, 1);
}

#[tokio::test]
async fn deleting_one_of_two_assets_enqueues_the_component() {
    let node = Node::start(&[]).await;
    let other = Node::start(&["mirror"]).await;
    node.maven("releases").await;
    let remote = connect(&node, &other).await;
    replicate(&node, &remote, "releases", "mirror", json!({})).await;
    let pom = "<project><modelVersion>4.0.0</modelVersion><groupId>com.acme</groupId>\
               <artifactId>lib</artifactId><version>1.0</version></project>";
    node.put("releases", "/com/acme/lib/1.0/lib-1.0.jar", b"jar")
        .await;
    node.put("releases", "/com/acme/lib/1.0/lib-1.0.pom", pom.as_bytes())
        .await;
    node.sql("DELETE FROM replication_item").await;
    node.sql("DELETE FROM asset WHERE path = '/com/acme/lib/1.0/lib-1.0.pom'")
        .await;
    assert_eq!(node.number(ITEMS).await, 1);
    let named = "SELECT count(*) AS n FROM replication_item WHERE name = 'lib' AND version = '1.0'";
    assert_eq!(node.number(named).await, 1);
}

#[tokio::test]
async fn moving_an_asset_to_another_component_marks_both() {
    let node = watched().await;
    node.put("watched", "/a/x.txt", b"x").await;
    node.put("watched", "/b/y.txt", b"y").await;
    node.sql("DELETE FROM replication_item").await;
    node.sql(
        "UPDATE asset SET component_id = (SELECT component_id FROM asset WHERE path = '/b/y.txt') \
         WHERE path = '/a/x.txt'",
    )
    .await;
    assert_eq!(node.number(ITEMS).await, 2);
}

#[tokio::test]
async fn deleting_a_repository_with_a_replication_takes_the_queue_with_it() {
    let node = watched().await;
    node.put("watched", "/docs/a.txt", b"alpha").await;
    assert_eq!(node.number(ITEMS).await, 1);
    let deleted = node
        .fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/repositories/watched",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    assert_eq!(node.number(ITEMS).await, 0);
    let left = "SELECT count(*) AS n FROM replication WHERE source_repository = 'watched'";
    assert_eq!(node.number(left).await, 0);
}
