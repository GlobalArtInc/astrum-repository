mod accounts;
mod cluster;
mod pull;
mod support;

use astrum_blob::BlobId;
use astrum_replication::reconcile::Outcome;
use axum::http::{Method, StatusCode};
use cluster::{Fail, Node, REPLICATIONS, connect, connect_with, replicate};
use pull::{column, drain, pull_body, pulling, queued, reconcile, status, user_token};
use serde_json::json;
use support::Auth;
use uuid::Uuid;

async fn origin_of(node: &Node, repository: &str, name: &str) -> Option<String> {
    let sql = format!(
        "SELECT attributes -> 'replication' ->> 'origin' AS t FROM component \
         WHERE repository = '{repository}' AND name = '{name}'"
    );
    node.text(&sql).await
}

fn pom(version: &str) -> String {
    format!(
        "<project><modelVersion>4.0.0</modelVersion><groupId>com.acme</groupId>\
         <artifactId>lib</artifactId><version>{version}</version></project>"
    )
}

#[tokio::test]
async fn a_pull_brings_whole_components_with_their_bytes_origin_and_fingerprint() {
    let remote = Node::start(&[]).await;
    let local = Node::start(&[]).await;
    remote.maven("libs").await;
    local.maven("libs-mirror").await;
    let privileges = ["as-replication-pull", "as-repository-view-*-*-read"];
    let token = user_token(&remote, "puller", &privileges).await;
    let connected = connect_with(&local, &remote, json!({"token": token})).await;
    let body = pull_body(json!({}));
    let replication = replicate(&local, &connected, "libs-mirror", "libs", body).await;
    let mut jars = Vec::new();
    for version in ["1.0", "2.0"] {
        let base = format!("/com/acme/lib/{version}/lib-{version}");
        remote
            .put(
                "libs",
                &format!("{base}.jar"),
                format!("jar {version}").as_bytes(),
            )
            .await;
        remote
            .put("libs", &format!("{base}-sources.jar"), b"sources")
            .await;
        remote
            .put("libs", &format!("{base}.pom"), pom(version).as_bytes())
            .await;
        jars.push(format!("{base}.jar"));
    }
    let first = reconcile(&local, &replication).await;
    assert_eq!(first.outcome, Outcome::Complete);
    assert_eq!((first.tally.remote_scanned, first.tally.enqueued), (2, 2));
    drain(&local).await;
    for version in ["1.0", "2.0"] {
        let base = format!("/com/acme/lib/{version}/lib-{version}");
        let jar = local.get("libs-mirror", &format!("{base}.jar")).await;
        assert_eq!(jar.body, format!("jar {version}").as_bytes());
        let pulled = local.get("libs-mirror", &format!("{base}.pom")).await;
        assert_eq!(pulled.body, pom(version).as_bytes());
    }
    let assets = "SELECT count(*) AS n FROM asset WHERE repository = 'libs-mirror'";
    assert_eq!(local.number(assets).await, 6);
    for jar in &jars {
        let maven = &astrum_format_maven::Maven;
        assert_eq!(
            local.bundle_fingerprint(maven, "libs-mirror", jar).await,
            remote.bundle_fingerprint(maven, "libs", jar).await
        );
    }
    let marks = "SELECT DISTINCT attributes -> 'replication' ->> 'origin' AS t FROM component \
                 WHERE repository = 'libs-mirror'";
    assert_eq!(column(&local, marks).await, [remote.instance().await]);
    let again = reconcile(&local, &replication).await;
    assert_eq!(
        (again.outcome, again.tally.enqueued),
        (Outcome::Complete, 0)
    );
    let stamped = "SELECT count(*) AS n FROM replication WHERE last_success_at IS NOT NULL";
    assert_eq!(local.number(stamped).await, 1);
}

async fn raw_repository(node: &Node, name: &str, policy: &str) {
    let body = json!({
        "name": name, "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": policy},
        "raw": {"contentDisposition": "INLINE"}
    });
    let path = "/service/rest/v1/repositories/raw/hosted";
    let made = node
        .fixture
        .call(Method::POST, path, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
}

#[tokio::test]
async fn a_changed_component_is_pulled_again_and_the_policy_of_the_target_decides_conflicts() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    raw_repository(local, "once", "ALLOW_ONCE").await;
    let connected = local.remote_id().await.to_string();
    let once = replicate(local, &connected, "once", "source", pull_body(json!({}))).await;
    remote.put("source", "/docs/a.txt", b"first").await;
    for id in [&pulling.replication, &once] {
        assert_eq!(reconcile(local, id).await.tally.enqueued, 1);
    }
    drain(local).await;
    for repository in ["mirror", "once"] {
        assert_eq!(local.get(repository, "/docs/a.txt").await.body, b"first");
    }
    remote.put("source", "/docs/a.txt", b"second").await;
    for id in [&pulling.replication, &once] {
        assert_eq!(reconcile(local, id).await.tally.enqueued, 1);
    }
    drain(local).await;
    assert_eq!(local.get("mirror", "/docs/a.txt").await.body, b"second");
    assert_eq!(local.get("once", "/docs/a.txt").await.body, b"first");
    let states = "SELECT state AS t FROM replication_item";
    assert_eq!(column(local, states).await, ["dead"]);
    let said =
        format!("SELECT last_error AS t FROM replication_item WHERE replication_id = '{once}'");
    let error = local.text(&said).await.unwrap();
    assert!(error.contains("exist with different content"), "{error}");
}

#[tokio::test]
async fn components_from_this_server_and_outside_the_filters_are_never_pulled() {
    let pulling = pulling(json!({"nameFilter": "/keep/.*"})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    for path in ["/keep/a", "/skip/b", "/keep/back"] {
        remote.put("source", path, path.as_bytes()).await;
    }
    let here = local.instance().await;
    remote
        .sql(&format!(
            "UPDATE component SET attributes = jsonb_build_object('replication', \
             jsonb_build_object('origin', '{here}')) WHERE name = '/keep/back'"
        ))
        .await;
    let summary = reconcile(local, &pulling.replication).await;
    let tally = &summary.tally;
    assert_eq!((tally.filtered, tally.from_here, tally.enqueued), (1, 1, 1));
    assert_eq!(queued(local).await, ["/keep/a"]);
    local
        .sql(&format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, reason) \
             VALUES ('{}', '/keep', '/keep/back', '', 'reconcile'), \
             ('{}', '/skip', '/skip/b', '', 'reconcile')",
            pulling.replication, pulling.replication
        ))
        .await;
    drain(local).await;
    assert_eq!(status(local, "mirror", "/keep/a").await, StatusCode::OK);
    for path in ["/keep/back", "/skip/b"] {
        assert_eq!(status(local, "mirror", path).await, StatusCode::NOT_FOUND);
    }
    assert!(queued(local).await.is_empty());
    local
        .sql("UPDATE replication SET version_filter = 'v.*'")
        .await;
    let narrowed = reconcile(local, &pulling.replication).await;
    assert_eq!((narrowed.tally.filtered, narrowed.tally.enqueued), (3, 0));
}

#[tokio::test]
async fn a_component_removed_on_the_remote_stays_unless_deletes_are_propagated() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", "/a", b"a").await;
    remote.put("source", "/b", b"b").await;
    reconcile(local, &pulling.replication).await;
    drain(local).await;
    remote.delete("source", "/a").await;
    let summary = reconcile(local, &pulling.replication).await;
    assert_eq!(
        (summary.tally.remote_scanned, summary.tally.enqueued),
        (1, 0)
    );
    assert_eq!(status(local, "mirror", "/a").await, StatusCode::OK);
}

#[tokio::test]
async fn propagated_deletes_remove_only_what_came_from_the_remote() {
    let pulling = pulling(json!({"propagateDeletes": true})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    local.put("mirror", "/local", b"mine").await;
    local.put("mirror", "/third", b"theirs").await;
    local
        .sql(&format!(
            "UPDATE component SET attributes = jsonb_build_object('replication', \
             jsonb_build_object('origin', '{}')) WHERE name = '/third'",
            Uuid::new_v4()
        ))
        .await;
    local.sql("DELETE FROM replication_item").await;
    remote.put("source", "/a", b"a").await;
    remote.put("source", "/b", b"b").await;
    assert_eq!(
        reconcile(local, &pulling.replication).await.tally.enqueued,
        2
    );
    drain(local).await;
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(local.number(recorded).await, 2);
    remote.delete("source", "/a").await;
    let summary = reconcile(local, &pulling.replication).await;
    assert_eq!(summary.tally.enqueued, 1);
    assert_eq!(queued(local).await, ["/a"]);
    drain(local).await;
    assert_eq!(status(local, "mirror", "/a").await, StatusCode::NOT_FOUND);
    for path in ["/b", "/local", "/third"] {
        assert_eq!(
            status(local, "mirror", path).await,
            StatusCode::OK,
            "{path}"
        );
    }
    assert_eq!(local.number(recorded).await, 1);
    local.put("mirror", "/b", b"edited here").await;
    remote.delete("source", "/b").await;
    assert_eq!(
        reconcile(local, &pulling.replication).await.tally.enqueued,
        0
    );
    assert_eq!(origin_of(local, "mirror", "/b").await, None);
}

async fn dead(node: &Node) -> Vec<String> {
    column(
        node,
        "SELECT last_error AS t FROM replication_item WHERE state = 'dead'",
    )
    .await
}

fn blob_file(node: &Node, id: &str) -> std::path::PathBuf {
    let id = BlobId::from(Uuid::parse_str(id).unwrap());
    node.fixture
        .data
        .path()
        .join(astrum_blob::BLOBS_DIR)
        .join(astrum_blob::DEFAULT_STORE)
        .join("content")
        .join(id.location())
}

#[tokio::test]
async fn a_blob_that_does_not_match_its_digest_is_refused_and_leaves_nothing_staged() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", "/docs/a.txt", b"alpha").await;
    let id = remote
        .text("SELECT blob_id::text AS t FROM asset WHERE path = '/docs/a.txt'")
        .await
        .unwrap();
    std::fs::write(blob_file(remote, &id), b"omega").unwrap();
    assert_eq!(
        reconcile(local, &pulling.replication).await.tally.enqueued,
        1
    );
    drain(local).await;
    let errors = dead(local).await;
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("SHA-256"), "{errors:?}");
    let staged = "SELECT count(*) AS n FROM replication_staged_blob";
    assert_eq!(local.number(staged).await, 0);
    let components = "SELECT count(*) AS n FROM component WHERE repository = 'mirror'";
    assert_eq!(local.number(components).await, 0);
    let held = "SELECT count(*) AS n FROM blob_hold";
    assert_eq!(local.number(held).await, 0);
}

#[tokio::test]
async fn a_blob_over_the_cap_is_refused_before_it_is_fetched() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", "/docs/a.txt", b"alpha").await;
    remote
        .sql("UPDATE asset SET size = 75161927680 WHERE path = '/docs/a.txt'")
        .await;
    reconcile(local, &pulling.replication).await;
    drain(local).await;
    let errors = dead(local).await;
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("at most"), "{errors:?}");
    let staged = "SELECT count(*) AS n FROM replication_staged_blob";
    assert_eq!(local.number(staged).await, 0);
}

#[tokio::test]
async fn a_refused_token_suspends_the_remote_and_keeps_the_items() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", "/docs/a.txt", b"alpha").await;
    reconcile(local, &pulling.replication).await;
    remote.force.fail(Fail::new(401, 5).on("/bundle"));
    let pass = local.pass().await;
    assert_eq!(
        (pass.suspended, pass.failed, pass.sent),
        (1, 0, 0),
        "{pass:?}"
    );
    let (state, attempts, _) = local.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    let failures = "SELECT failures::bigint AS n FROM replication_remote";
    assert_eq!(local.number(failures).await, 1);
    let said = "SELECT last_error AS t FROM replication_remote";
    assert!(
        local
            .text(said)
            .await
            .unwrap()
            .contains("refused the token")
    );
    assert_eq!(
        status(local, "mirror", "/docs/a.txt").await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn local_writes_to_a_pull_target_are_not_captured_but_a_push_source_still_is() {
    let remote = Node::start(&["source"]).await;
    let local = Node::start(&["mirror", "watched"]).await;
    let connected = connect(&local, &remote).await;
    replicate(&local, &connected, "mirror", "source", pull_body(json!({}))).await;
    replicate(&local, &connected, "watched", "source", json!({})).await;
    local.put("mirror", "/docs/a.txt", b"alpha").await;
    let items = "SELECT count(*) AS n FROM replication_item";
    assert_eq!(local.number(items).await, 0);
    local.put("watched", "/docs/b.txt", b"beta").await;
    assert_eq!(local.number(items).await, 1);
    let pushed = "SELECT count(*) AS n FROM replication_item i \
                  JOIN replication r ON r.id = i.replication_id WHERE r.direction = 'push'";
    assert_eq!(local.number(pushed).await, 1);
    let listed = local
        .fixture
        .call(Method::GET, REPLICATIONS, Auth::Admin, None)
        .await
        .json();
    let directions: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|found| found["direction"].as_str().unwrap())
        .collect();
    assert_eq!(directions.len(), 2);
    assert!(directions.contains(&"pull") && directions.contains(&"push"));
}
