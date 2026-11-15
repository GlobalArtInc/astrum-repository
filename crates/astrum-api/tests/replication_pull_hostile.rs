mod accounts;
mod cluster;
mod pull;
mod support;

use std::time::Duration;

use astrum_replication::dispatch::Settings;
use astrum_replication::queue;
use astrum_replication::reconcile::{Limits, Outcome, run_with};
use axum::http::{Method, StatusCode};
use cluster::{REMOTES, connect_with, replicate};
use pull::{
    ALPHA, Answer, Blob, NAME, bundle, column, drain, hostile, pull_body, pulling, queued,
    reconcile, status, user_token,
};
use serde_json::json;
use support::Auth;
use uuid::Uuid;

const ALPHA_SHA: &str = "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8";
const SIZE: u64 = ALPHA.len() as u64;

#[tokio::test]
async fn a_bundle_of_the_wrong_format_or_key_is_refused_and_one_of_this_server_is_skipped() {
    let hostile = hostile().await;
    hostile
        .stub
        .says(Answer::Json(bundle("maven2", NAME, None, SIZE, ALPHA)));
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert_eq!(dead.len(), 1);
    assert!(dead[0].contains("format"), "{dead:?}");
    assert!(hostile.wrote_nothing().await);

    let other = bundle("raw", "/docs/other.txt", None, SIZE, ALPHA);
    hostile.stub.says(Answer::Json(other));
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert!(dead[0].contains("another component"), "{dead:?}");
    assert!(hostile.wrote_nothing().await);

    let own = hostile.local.instance().await;
    let mine = bundle("raw", NAME, Some(&own), SIZE, ALPHA);
    hostile.stub.says(Answer::Json(mine));
    hostile.attempt().await;
    assert!(queued(&hostile.local).await.is_empty());
    assert!(hostile.wrote_nothing().await);
    assert_eq!(hostile.stub.blob_requests(), 0);
}

#[tokio::test]
async fn an_endless_blob_and_a_lying_length_are_refused_and_leave_nothing_staged() {
    let hostile = hostile().await;
    hostile.stub.serves(Blob::Endless);
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert_eq!(dead.len(), 1);
    assert!(dead[0].contains("more than"), "{dead:?}");
    assert!(hostile.wrote_nothing().await);
    hostile.stub.serves(Blob::Announce(1000));
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert!(dead[0].contains("announced another size"), "{dead:?}");
    assert!(hostile.wrote_nothing().await);
    assert_eq!(hostile.stub.blob_requests(), 2);
}

#[tokio::test]
async fn a_reply_that_is_too_large_or_not_a_bundle_kills_its_item_and_not_the_remote() {
    let hostile = hostile().await;
    hostile.stub.says(Answer::Raw(vec![b' '; 17 << 20]));
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert_eq!(dead.len(), 1);
    assert!(dead[0].contains("too large"), "{dead:?}");
    hostile.stub.says(Answer::Json(json!({"nope": 1})));
    hostile.attempt().await;
    let dead = hostile.dead().await;
    assert!(dead[0].contains("does not follow"), "{dead:?}");
    let node = &hostile.local;
    let failures = "SELECT failures::bigint AS n FROM replication_remote";
    assert_eq!(node.number(failures).await, 0);
    let suspended =
        "SELECT count(*) AS n FROM replication_remote WHERE suspended_until IS NOT NULL";
    assert_eq!(node.number(suspended).await, 0);
}

#[tokio::test]
async fn a_download_that_outlasts_its_cap_fails_retryable_and_leaves_nothing() {
    let hostile = hostile().await;
    hostile
        .stub
        .serves(Blob::Slow(ALPHA.to_vec(), Duration::from_millis(400)));
    reconcile(&hostile.local, &hostile.replication).await;
    let settings = Settings {
        download_min: Duration::from_millis(500),
        ..Settings::default()
    };
    let pass = hostile
        .local
        .dispatcher_with(settings)
        .run_once()
        .await
        .unwrap();
    assert_eq!(
        (pass.failed, pass.dead, pass.suspended),
        (1, 0, 0),
        "{pass:?}"
    );
    let (state, attempts, _) = hostile.local.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 1));
    let said = "SELECT last_error AS t FROM replication_item";
    assert!(
        hostile
            .local
            .text(said)
            .await
            .unwrap()
            .contains("took longer")
    );
    assert!(hostile.wrote_nothing().await);
    hostile.local.due().await;
    drain(&hostile.local).await;
    assert_eq!(hostile.local.get("mirror", NAME).await.body, ALPHA);
}

#[tokio::test]
async fn a_lease_that_expired_under_a_slow_worker_commits_only_once() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", NAME, ALPHA).await;
    assert_eq!(
        reconcile(local, &pulling.replication).await.tally.enqueued,
        1
    );
    remote.force.hold(ALPHA_SHA);
    let first = local.dispatcher();
    let slow = tokio::spawn(async move { first.run_once().await.unwrap() });
    remote.force.reached().await;
    local
        .sql("UPDATE replication_item SET leased_until = now() - interval '1 minute'")
        .await;
    let db = local.fixture.state.db().conn();
    assert_eq!(queue::requeue_expired(db).await.unwrap(), 1);
    local.due().await;
    drain(local).await;
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while local.get("mirror", NAME).await.body != ALPHA {
        assert!(
            std::time::Instant::now() < deadline,
            "the mirror did not answer with the replicated object within 20 s"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(local.get("mirror", NAME).await.body, ALPHA);
    remote.force.open();
    let pass = slow.await.unwrap();
    assert_eq!(pass.failed, 0, "{pass:?}");
    let replicated = accounts::audit(&local.fixture, "repository.content").await;
    let commits = replicated.iter().filter(|event| event.0 == "replicated");
    assert_eq!(commits.count(), 1);
    let rows = "SELECT (SELECT count(*) FROM asset WHERE repository = 'mirror') \
                + (SELECT count(*) FROM replication_item) \
                + (SELECT count(*) FROM replication_staged_blob) AS n";
    assert_eq!(local.number(rows).await, 1);
}

#[tokio::test]
async fn a_token_that_may_receive_but_not_pull_suspends_the_remote_until_it_is_replaced() {
    let remote = cluster::Node::start(&["source"]).await;
    let local = cluster::Node::start(&["mirror"]).await;
    let read = "as-repository-view-*-*-read";
    let weak = user_token(&remote, "receiver", &["as-replication-receive", read]).await;
    let strong = user_token(&remote, "puller", &["as-replication-pull", read]).await;
    let connected = connect_with(&local, &remote, json!({"token": weak})).await;
    let replication = replicate(&local, &connected, "mirror", "source", pull_body(json!({}))).await;
    remote.put("source", NAME, ALPHA).await;
    assert_eq!(reconcile(&local, &replication).await.tally.enqueued, 1);
    let pass = local.pass().await;
    assert_eq!(
        (pass.suspended, pass.failed, pass.sent),
        (1, 0, 0),
        "{pass:?}"
    );
    let (state, attempts, _) = local.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    let said = local
        .text("SELECT last_error AS t FROM replication_remote")
        .await
        .unwrap();
    assert!(
        said.contains("refused the token") && said.contains("receive or pull"),
        "{said}"
    );
    let renewed = json!({
        "name": "renewed", "url": format!("http://{}", remote.address),
        "token": strong, "allowPrivateNetwork": true
    });
    let path = format!("{REMOTES}/{connected}");
    let put = local
        .fixture
        .call(Method::PUT, &path, Auth::Admin, Some(renewed))
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    local.due().await;
    drain(&local).await;
    assert_eq!(local.get("mirror", NAME).await.body, ALPHA);
    assert!(queued(&local).await.is_empty());
}

#[tokio::test]
async fn an_empty_remote_never_wipes_a_mirror_and_one_run_queues_a_limited_number_of_deletions() {
    let pulling = pulling(json!({"propagateDeletes": true})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    for path in ["/a", "/b"] {
        remote.put("source", path, path.as_bytes()).await;
    }
    reconcile(local, &pulling.replication).await;
    drain(local).await;
    for path in ["/a", "/b"] {
        remote.delete("source", path).await;
    }
    let emptied = reconcile(local, &pulling.replication).await;
    let tally = &emptied.tally;
    assert_eq!(
        (tally.remote_scanned, tally.withheld, tally.enqueued),
        (0, 2, 0)
    );
    assert!(
        emptied.message().contains("lists no components"),
        "{}",
        emptied.message()
    );
    for path in ["/a", "/b"] {
        assert_eq!(status(local, "mirror", path).await, StatusCode::OK);
    }

    remote.put("source", "/c", b"c").await;
    let id = Uuid::parse_str(&pulling.replication).unwrap();
    let limits = Limits {
        deletions: 1,
        ..Limits::default()
    };
    let content = local.fixture.state.content();
    let first = run_with(content, id, &limits, &|| false).await.unwrap();
    assert_eq!(
        (first.outcome.clone(), first.tally.enqueued),
        (Outcome::Capped, 2)
    );
    assert!(first.message().contains("deletions"), "{}", first.message());
    drain(local).await;
    let second = reconcile(local, &pulling.replication).await;
    assert_eq!(
        (second.outcome, second.tally.enqueued),
        (Outcome::Complete, 1)
    );
    drain(local).await;
    for path in ["/a", "/b"] {
        assert_eq!(status(local, "mirror", path).await, StatusCode::NOT_FOUND);
    }
    assert_eq!(status(local, "mirror", "/c").await, StatusCode::OK);
}

#[tokio::test]
async fn a_dead_pull_item_is_revived_by_the_next_reconcile_and_then_pulled() {
    let pulling = pulling(json!({})).await;
    let (remote, local) = (&pulling.remote, &pulling.local);
    remote.put("source", NAME, ALPHA).await;
    local
        .sql(&format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, state, \
             reason, attempts, last_error) \
             VALUES ('{}', '/docs', '{NAME}', '', 'dead', 'event', 12, 'refused')",
            pulling.replication
        ))
        .await;
    let summary = reconcile(local, &pulling.replication).await;
    assert_eq!(summary.tally.enqueued, 1);
    let shown = "SELECT state || ' ' || reason || ' ' || attempts AS t FROM replication_item";
    assert_eq!(column(local, shown).await, ["pending reconcile 0"]);
    drain(local).await;
    assert_eq!(local.get("mirror", NAME).await.body, ALPHA);
    assert!(queued(local).await.is_empty());
}
