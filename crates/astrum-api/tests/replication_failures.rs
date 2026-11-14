mod cluster;
mod support;

use std::time::Duration;

use astrum_replication::dispatch::{Pass, Settings};
use astrum_replication::queue;
use axum::http::{Method, StatusCode};
use cluster::{Fail, LEASE, REMOTES, pair};
use serde_json::json;
use support::Auth;

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
    for (index, status) in [409_u16, 422].into_iter().enumerate() {
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
async fn a_refused_token_pauses_the_remote_and_a_put_on_it_resumes_the_work() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.fail(Fail::new(401, 1).on("/capabilities"));
    let pass = sender.pass().await;
    assert_eq!(
        (pass.suspended, pass.failed, pass.sent),
        (1, 0, 0),
        "{pass:?}"
    );
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    assert_eq!(sender.number(FAILURES).await, 1);
    let pause = "SELECT (extract(epoch FROM (suspended_until - now())))::bigint AS n FROM replication_remote";
    assert!(sender.number(pause).await > 800);
    let said = "SELECT last_error AS t FROM replication_remote";
    assert!(
        sender
            .text(said)
            .await
            .unwrap()
            .contains("refused the token")
    );
    sender.due().await;
    assert_eq!(sender.pass().await, Pass::default());

    let id = sender
        .text("SELECT id::text AS t FROM replication_remote")
        .await
        .unwrap();
    let shown = sender
        .fixture
        .call(Method::GET, &format!("{REMOTES}/{id}"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(shown["failures"], 1);
    assert!(shown["suspendedUntil"].is_string());
    assert!(
        shown["lastError"]
            .as_str()
            .unwrap()
            .contains("refused the token")
    );
    let body = json!({
        "name": "renewed", "url": format!("http://{}", receiver.address),
        "token": receiver.token, "allowPrivateNetwork": true
    });
    let put = sender
        .fixture
        .call(
            Method::PUT,
            &format!("{REMOTES}/{id}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    assert_eq!(put.json()["failures"], 0);
    assert!(put.json()["suspendedUntil"].is_null());
    assert!(put.json()["lastError"].is_null());
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
}

#[tokio::test]
async fn an_outage_longer_than_the_attempts_allow_does_not_kill_the_items() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver
        .force
        .fail(Fail::new(503, usize::MAX).on("/capabilities"));
    for round in 1..=15 {
        sender
            .sql("UPDATE replication_remote SET suspended_until = NULL")
            .await;
        sender.due().await;
        let pass = sender.pass().await;
        assert_eq!((pass.suspended, pass.failed), (1, 0), "{round}: {pass:?}");
    }
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    assert_eq!(sender.number(FAILURES).await, 15);
    receiver.force.fail(Fail::new(503, 0));
    sender
        .sql("UPDATE replication_remote SET suspended_until = NULL")
        .await;
    sender.due().await;
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(sender.number(FAILURES).await, 0);
    let clear = "SELECT count(*) AS n FROM replication_remote \
                 WHERE suspended_until IS NULL AND last_error IS NULL";
    assert_eq!(sender.number(clear).await, 1);
}

#[tokio::test]
async fn a_remote_that_answers_with_another_instance_is_stored_again() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender
        .sql("UPDATE replication_remote SET remote_instance_id = gen_random_uuid()")
        .await;
    assert_eq!(sender.pass().await.sent, 1);
    let stored = "SELECT remote_instance_id::text AS t FROM replication_remote";
    assert_eq!(sender.text(stored).await, Some(receiver.instance().await));
}

#[tokio::test]
async fn a_remote_changed_in_the_middle_of_a_transfer_gets_nothing_more() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.hold("/blobs/missing");
    let settings = Settings {
        fresh: Duration::ZERO,
        ..Settings::default()
    };
    let dispatcher = sender.dispatcher_with(settings);
    let running = tokio::spawn(async move { dispatcher.run_once().await.unwrap() });
    receiver.force.reached().await;
    let id = sender
        .text("SELECT id::text AS t FROM replication_remote")
        .await
        .unwrap();
    let body = json!({
        "name": "moved", "url": "http://127.0.0.1:9", "token": receiver.token,
        "allowPrivateNetwork": true
    });
    let put = sender
        .fixture
        .call(
            Method::PUT,
            &format!("{REMOTES}/{id}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    receiver.force.open();
    let pass = running.await.unwrap();
    assert_eq!(
        (pass.released, pass.sent, pass.failed),
        (1, 0, 0),
        "{pass:?}"
    );
    assert_eq!((receiver.force.uploads(), receiver.force.commits()), (0, 0));
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
}
