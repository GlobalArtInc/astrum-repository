mod cluster;
mod support;

use astrum_replication::dispatch::Pass;
use astrum_replication::queue;
use axum::http::StatusCode;
use cluster::{LEASE, Node, connect, pair, replicate};
use serde_json::json;

#[tokio::test]
async fn a_pushed_component_arrives_with_its_bytes_origin_and_fingerprint() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender.put("source", "/docs/b.txt", b"beta").await;
    let pass = sender.pass().await;
    assert_eq!((pass.leased, pass.sent), (2, 2), "{pass:?}");
    assert_eq!(sender.state().await, None);
    for (path, bytes) in [("/docs/a.txt", &b"alpha"[..]), ("/docs/b.txt", b"beta")] {
        let arrived = receiver.get("mirror", path).await;
        assert_eq!(arrived.status, StatusCode::OK);
        assert_eq!(arrived.body, bytes);
        assert_eq!(
            receiver
                .bundle_fingerprint(&astrum_format_raw::Raw, "mirror", path)
                .await,
            sender
                .bundle_fingerprint(&astrum_format_raw::Raw, "source", path)
                .await
        );
    }
    let origin = "SELECT attributes -> 'replication' ->> 'origin' AS t FROM component \
                  WHERE repository = 'mirror' AND name = '/docs/a.txt'";
    assert_eq!(receiver.text(origin).await, Some(sender.instance().await));
    let stamped = "SELECT count(*) AS n FROM replication WHERE last_success_at IS NOT NULL";
    assert_eq!(sender.number(stamped).await, 1);
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(sender.number(recorded).await, 0);
    assert_eq!(sender.pass().await, Pass::default());
}

#[tokio::test]
async fn a_maven_component_arrives_whole_with_the_same_fingerprint() {
    let sender = Node::start(&[]).await;
    let receiver = Node::start(&[]).await;
    sender.maven("releases").await;
    receiver.maven("mirror").await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "releases", "mirror", json!({})).await;
    let pom = "<project><modelVersion>4.0.0</modelVersion><groupId>com.acme</groupId>\
               <artifactId>lib</artifactId><version>1.0</version></project>";
    let jar = "/com/acme/lib/1.0/lib-1.0.jar";
    sender.put("releases", jar, b"jar bytes").await;
    sender
        .put("releases", "/com/acme/lib/1.0/lib-1.0.pom", pom.as_bytes())
        .await;
    let pass = sender.pass().await;
    assert_eq!((pass.sent, pass.failed), (1, 0), "{pass:?}");
    for (path, bytes) in [(jar, "jar bytes"), ("/com/acme/lib/1.0/lib-1.0.pom", pom)] {
        let arrived = receiver.get("mirror", path).await;
        assert_eq!(arrived.status, StatusCode::OK, "{path}");
        assert_eq!(arrived.body, bytes.as_bytes());
    }
    assert_eq!(
        receiver
            .bundle_fingerprint(&astrum_format_maven::Maven, "mirror", jar)
            .await,
        sender
            .bundle_fingerprint(&astrum_format_maven::Maven, "releases", jar)
            .await
    );
}

#[tokio::test]
async fn a_burst_of_events_on_one_key_gives_one_transfer() {
    let (sender, receiver) = pair().await;
    for round in 0..50 {
        let bytes = format!("version {round}");
        sender.put("source", "/docs/a.txt", bytes.as_bytes()).await;
    }
    let (state, _, generation) = sender.state().await.unwrap();
    assert_eq!(state, "pending");
    assert!(generation >= 50, "{generation}");
    let pass = sender.pass().await;
    assert_eq!((pass.leased, pass.sent), (1, 1), "{pass:?}");
    assert_eq!(receiver.force.commits(), 1);
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.body,
        b"version 49"
    );
}

#[tokio::test]
async fn an_event_while_the_item_is_leased_is_sent_again_after_the_first_finishes() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"one").await;
    let db = sender.fixture.state.db().conn();
    let mut leased = queue::lease(db, sender.remote_id().await, LEASE, 5)
        .await
        .unwrap();
    let item = leased.pop().unwrap();
    assert!(leased.is_empty());
    sender.put("source", "/docs/a.txt", b"two").await;
    let (state, _, generation) = sender.state().await.unwrap();
    assert_eq!(state, "leased");
    assert!(generation > item.generation);
    assert_eq!(
        queue::finish(db, &item).await.unwrap(),
        queue::Finished::Again
    );
    assert_eq!(sender.state().await.unwrap().0, "pending");
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"two");
    assert_eq!(sender.state().await, None);
}

#[tokio::test]
async fn a_due_queue_is_shared_between_the_workers_of_a_remote() {
    let (sender, _receiver) = pair().await;
    for index in 0..5 {
        sender
            .put("source", &format!("/docs/{index}.txt"), b"alpha")
            .await;
    }
    let db = sender.fixture.state.db().conn();
    let remote = sender.remote_id().await;
    let mut shares = Vec::new();
    for _ in 0..5 {
        let share = queue::lease_share(db, remote, LEASE, 20, 2).await.unwrap();
        shares.push(share.len());
    }
    assert_eq!(shares, [3, 1, 1, 0, 0]);
}

#[tokio::test]
async fn an_expired_lease_costs_an_attempt_and_the_last_one_kills_the_item() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    let db = sender.fixture.state.db().conn();
    let remote = sender.remote_id().await;
    assert_eq!(queue::lease(db, remote, LEASE, 5).await.unwrap().len(), 1);
    assert_eq!(sender.pass().await, Pass::default());
    let expire = "UPDATE replication_item SET leased_until = now() - interval '1 minute'";
    sender.sql(expire).await;
    assert_eq!(queue::requeue_expired(db).await.unwrap(), 1);
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 1));
    let delay =
        "SELECT (extract(epoch FROM (not_before - now())))::bigint AS n FROM replication_item";
    assert!(sender.number(delay).await >= 20);
    assert_eq!(sender.pass().await, Pass::default());
    sender.due().await;
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");

    sender.put("source", "/docs/b.txt", b"beta").await;
    assert_eq!(queue::lease(db, remote, LEASE, 5).await.unwrap().len(), 1);
    sender
        .sql(
            "UPDATE replication_item SET attempts = 11, leased_until = now() - interval '1 minute'",
        )
        .await;
    assert_eq!(queue::requeue_expired(db).await.unwrap(), 1);
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("dead", 12));
    let error = "SELECT last_error AS t FROM replication_item";
    assert_eq!(sender.text(error).await.as_deref(), Some("lease expired"));
}

#[tokio::test]
async fn a_bidirectional_pair_does_not_send_a_component_back_to_its_origin() {
    let first = Node::start(&["left"]).await;
    let second = Node::start(&["right"]).await;
    let forth = connect(&first, &second).await;
    let back = connect(&second, &first).await;
    replicate(&first, &forth, "left", "right", json!({})).await;
    replicate(&second, &back, "right", "left", json!({})).await;
    first.put("left", "/docs/a.txt", b"alpha").await;
    assert_eq!(first.pass().await.sent, 1);
    assert_eq!(second.get("right", "/docs/a.txt").await.body, b"alpha");
    assert_eq!(second.state().await, None);
    second
        .sql("SELECT replication_mark('right', '/docs', '/docs/a.txt', '')")
        .await;
    assert_eq!(second.state().await.unwrap().0, "pending");
    let pass = second.pass().await;
    assert_eq!(
        (pass.leased, pass.dropped, pass.sent),
        (1, 1, 0),
        "{pass:?}"
    );
    assert_eq!(second.state().await, None);
    assert_eq!(first.force.commits(), 0);
    let learned = "SELECT remote_instance_id::text AS t FROM replication_remote";
    assert_eq!(second.text(learned).await, Some(first.instance().await));
}

#[tokio::test]
async fn deletes_are_kept_by_default_and_travel_with_the_flag() {
    let sender = Node::start(&["keep", "mirror-on"]).await;
    let receiver = Node::start(&["kept", "mirrored"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "keep", "kept", json!({})).await;
    replicate(
        &sender,
        &remote,
        "mirror-on",
        "mirrored",
        json!({"propagateDeletes": true}),
    )
    .await;
    for (source, target) in [("keep", "kept"), ("mirror-on", "mirrored")] {
        sender.put(source, "/docs/a.txt", b"alpha").await;
        assert_eq!(sender.pass().await.sent, 1);
        assert_eq!(
            receiver.get(target, "/docs/a.txt").await.status,
            StatusCode::OK
        );
    }
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(sender.number(recorded).await, 1);
    for source in ["keep", "mirror-on"] {
        sender.delete(source, "/docs/a.txt").await;
        let pass = sender.pass().await;
        assert_eq!((pass.leased, pass.failed), (1, 0), "{source}: {pass:?}");
    }
    assert_eq!(
        receiver.get("kept", "/docs/a.txt").await.status,
        StatusCode::OK
    );
    assert_eq!(
        receiver.get("mirrored", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(sender.number(recorded).await, 0);
    assert_eq!(sender.state().await, None);
}

#[tokio::test]
async fn a_tombstone_for_a_republished_copy_is_refused_and_the_item_dies() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "mirror",
        json!({"propagateDeletes": true}),
    )
    .await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    assert_eq!(sender.pass().await.sent, 1);
    receiver.put("mirror", "/docs/a.txt", b"local").await;
    sender.delete("source", "/docs/a.txt").await;
    let pass = sender.pass().await;
    assert_eq!((pass.failed, pass.dead), (1, 1), "{pass:?}");
    let error = "SELECT last_error AS t FROM replication_item";
    assert!(sender.text(error).await.unwrap().contains("409"));
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"local");
}

#[tokio::test]
async fn a_tombstone_for_a_copy_the_remote_already_lost_counts_as_done() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "mirror",
        json!({"propagateDeletes": true}),
    )
    .await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    assert_eq!(sender.pass().await.sent, 1);
    receiver.delete("mirror", "/docs/a.txt").await;
    sender.delete("source", "/docs/a.txt").await;
    let pass = sender.pass().await;
    assert_eq!((pass.sent, pass.failed), (1, 0), "{pass:?}");
    assert_eq!(receiver.force.tombstones(), 1);
    assert_eq!(sender.state().await, None);
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(sender.number(recorded).await, 0);
}

#[tokio::test]
async fn no_tombstone_is_sent_for_a_component_that_came_from_the_remote() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "mirror",
        json!({"propagateDeletes": true}),
    )
    .await;
    let record = format!(
        "INSERT INTO replication_sent (replication_id, namespace, name, version, fingerprint, origin) \
         SELECT id, '/docs', '/docs/a.txt', '', repeat('a', 64), '{}' FROM replication",
        receiver.instance().await
    );
    sender.sql(&record).await;
    sender
        .sql("SELECT replication_mark('source', '/docs', '/docs/a.txt', '')")
        .await;
    let pass = sender.pass().await;
    assert_eq!(
        (pass.leased, pass.dropped, pass.failed),
        (1, 1, 0),
        "{pass:?}"
    );
    assert_eq!(receiver.force.tombstones(), 0);
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(sender.number(recorded).await, 0);
}

#[tokio::test]
async fn components_outside_the_filters_and_disabled_replications_are_not_sent() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["by-name", "by-version", "off"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "by-name",
        json!({"nameFilter": "/docs/a\\.txt"}),
    )
    .await;
    replicate(
        &sender,
        &remote,
        "source",
        "by-version",
        json!({"versionFilter": "1\\..*"}),
    )
    .await;
    let off = replicate(&sender, &remote, "source", "off", json!({})).await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender.put("source", "/docs/b.txt", b"beta").await;
    let disabled = json!({
        "name": "off", "sourceRepository": "source", "remoteId": remote,
        "remoteRepository": "off", "enabled": false, "settleSeconds": 0
    });
    let path = format!("{}/{off}", cluster::REPLICATIONS);
    let updated = sender
        .fixture
        .call(
            axum::http::Method::PUT,
            &path,
            support::Auth::Admin,
            Some(disabled),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    let pass = sender.pass().await;
    assert_eq!(
        (pass.leased, pass.sent, pass.dropped),
        (4, 1, 3),
        "{pass:?}"
    );
    assert_eq!(
        receiver.get("by-name", "/docs/a.txt").await.status,
        StatusCode::OK
    );
    for (repository, path) in [
        ("by-name", "/docs/b.txt"),
        ("by-version", "/docs/a.txt"),
        ("off", "/docs/a.txt"),
    ] {
        let missing = receiver.get(repository, path).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{repository}");
    }
    let waiting = "SELECT count(*) AS n FROM replication_item i JOIN replication r \
                   ON r.id = i.replication_id WHERE r.enabled = false AND i.state = 'pending'";
    assert_eq!(sender.number(waiting).await, 2);
    sender.put("source", "/docs/c.txt", b"gamma").await;
    assert_eq!(sender.number(waiting).await, 2);
}
