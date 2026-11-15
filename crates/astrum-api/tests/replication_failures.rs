mod cluster;
mod support;

use std::time::Duration;

use astrum_replication::dispatch::Pass;
use astrum_replication::queue;
use axum::http::StatusCode;
use cluster::{Fail, LEASE, pair};
use serde_json::json;

const LAST_ERROR: &str = "SELECT last_error AS t FROM replication_item";
const FAILURES: &str = "SELECT failures::bigint AS n FROM replication_remote";

#[tokio::test]
async fn a_temporary_failure_on_the_commit_is_retried_with_backoff_and_the_hint_of_the_remote() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver
        .force
        .fail(Fail::new(503, 1).on("/components").retry_after("120"));
    let pass = sender.pass().await;
    assert_eq!((pass.failed, pass.dead, pass.sent), (1, 0, 0), "{pass:?}");
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 1));
    let delay =
        "SELECT (extract(epoch FROM (not_before - now())))::bigint AS n FROM replication_item";
    assert!(sender.number(delay).await >= 100);
    assert!(sender.text(LAST_ERROR).await.unwrap().contains("503"));
    assert_eq!(sender.number(FAILURES).await, 0);
    assert_eq!(sender.pass().await, Pass::default());
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    sender.due().await;
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
    assert_eq!(sender.state().await, None);
    let healthy = "SELECT count(*) AS n FROM replication \
                   WHERE last_failure_at IS NOT NULL AND last_success_at IS NOT NULL AND last_error IS NULL";
    assert_eq!(sender.number(healthy).await, 1);
}

#[tokio::test]
async fn a_refusal_about_the_item_kills_it_at_once_and_a_new_event_revives_it() {
    let (sender, receiver) = pair().await;
    for (index, status) in [409_u16, 422, 403].into_iter().enumerate() {
        let path = format!("/docs/{index}.txt");
        sender.put("source", &path, b"alpha").await;
        receiver.force.fail(Fail::new(status, 1).on("/components"));
        let pass = sender.pass().await;
        assert_eq!((pass.failed, pass.dead), (1, 1), "{status}: {pass:?}");
        let (state, attempts, _) = sender.state().await.unwrap();
        assert_eq!((state.as_str(), attempts), ("dead", 1));
        let text = sender.text(LAST_ERROR).await.unwrap();
        assert!(
            text.contains(&status.to_string()) && text.contains("forced"),
            "{text}"
        );
        assert_eq!(sender.number(FAILURES).await, 0);
        assert_eq!(sender.pass().await, Pass::default());
        sender.put("source", &path, b"beta").await;
        assert_eq!(sender.state().await.unwrap().0, "pending");
        assert_eq!(sender.pass().await.sent, 1);
        assert_eq!(receiver.get("mirror", &path).await.body, b"beta");
    }
}

#[tokio::test]
async fn twelve_temporary_failures_on_the_commit_make_an_item_dead() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver
        .force
        .fail(Fail::new(503, usize::MAX).on("/components"));
    for attempt in 1..=12 {
        sender.due().await;
        let pass = sender.pass().await;
        assert_eq!(pass.failed, 1, "{attempt}: {pass:?}");
        let (state, attempts, _) = sender.state().await.unwrap();
        assert_eq!(attempts, attempt);
        assert_eq!(state, if attempt == 12 { "dead" } else { "pending" });
    }
    sender.due().await;
    assert_eq!(sender.pass().await, Pass::default());
    assert_eq!(sender.number(FAILURES).await, 0);
}

#[tokio::test]
async fn a_failure_on_a_moved_generation_keeps_the_new_state() {
    let (sender, _receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"one").await;
    let db = sender.fixture.state.db().conn();
    let item = queue::lease(db, sender.remote_id().await, LEASE, 5)
        .await
        .unwrap()
        .pop()
        .unwrap();
    sender.put("source", "/docs/a.txt", b"two").await;
    let failed = queue::fail(db, &item, true, Duration::from_secs(30), "boom")
        .await
        .unwrap()
        .unwrap();
    assert_eq!((failed.state.as_str(), failed.attempts), ("pending", 0));
    assert_eq!(sender.text(LAST_ERROR).await, None);
}

#[tokio::test]
async fn a_lease_taken_away_during_the_transfer_ends_it_without_touching_the_item() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.hold("/blobs/missing");
    let dispatcher = sender.dispatcher();
    let running = tokio::spawn(async move { dispatcher.run_once().await.unwrap() });
    receiver.force.reached().await;
    sender
        .sql("UPDATE replication_item SET state = 'dead', leased_until = NULL")
        .await;
    receiver.force.open();
    let pass = running.await.unwrap();
    assert_eq!(
        (pass.abandoned, pass.sent, pass.failed),
        (1, 0, 0),
        "{pass:?}"
    );
    assert_eq!((receiver.force.uploads(), receiver.force.commits()), (0, 0));
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("dead", 0));
}

#[tokio::test]
async fn a_second_blobs_missing_belongs_to_the_item_not_to_the_remote() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    let lacking =
        json!({"code": "blobs_missing", "message": "lacking", "missing": ["a".repeat(64)]});
    receiver
        .force
        .fail(Fail::new(409, 2).on("/components").body(lacking));
    let pass = sender.pass().await;
    assert_eq!(
        (pass.failed, pass.dead, pass.suspended),
        (1, 0, 0),
        "{pass:?}"
    );
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 1));
    assert!(sender.text(LAST_ERROR).await.unwrap().contains("lacks"));
    assert_eq!(sender.number(FAILURES).await, 0);
    sender.due().await;
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
}

#[tokio::test]
async fn a_forbidden_commit_is_the_items_and_never_pauses_the_remote() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.fail(Fail::new(403, 1).on("/components"));
    let pass = sender.pass().await;
    assert_eq!(
        (pass.failed, pass.dead, pass.suspended),
        (1, 1, 0),
        "{pass:?}"
    );
    assert_eq!(sender.number(FAILURES).await, 0);
    let paused = "SELECT count(*) AS n FROM replication_remote WHERE suspended_until IS NOT NULL";
    assert_eq!(sender.number(paused).await, 0);
}

#[tokio::test]
async fn an_item_whose_commit_never_answers_dies_while_the_others_keep_flowing() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/p.txt", b"poison").await;
    sender.put("source", "/docs/g.txt", b"good").await;
    receiver.force.fail(
        Fail::reset(usize::MAX)
            .on("/components")
            .marked("/docs/p.txt"),
    );
    let poison =
        "SELECT state || ' ' || attempts AS t FROM replication_item WHERE name = '/docs/p.txt'";
    for _ in 0..20 {
        sender
            .sql("UPDATE replication_remote SET suspended_until = NULL")
            .await;
        sender.due().await;
        sender.pass().await;
        if sender.text(poison).await.as_deref() == Some("dead 12") {
            break;
        }
    }
    assert_eq!(sender.text(poison).await.as_deref(), Some("dead 12"));
    assert_eq!(receiver.get("mirror", "/docs/g.txt").await.body, b"good");
    assert_eq!(
        receiver.get("mirror", "/docs/p.txt").await.status,
        StatusCode::NOT_FOUND
    );
    let left = "SELECT count(*) AS n FROM replication_item";
    assert_eq!(sender.number(left).await, 1);
}
