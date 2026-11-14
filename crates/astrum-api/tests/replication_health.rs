mod cluster;
mod support;

use std::time::Duration;

use astrum_replication::dispatch::{Pass, Settings};
use axum::http::{Method, StatusCode};
use cluster::{Fail, REMOTES, pair};
use serde_json::json;
use support::Auth;

const FAILURES: &str = "SELECT failures::bigint AS n FROM replication_remote";

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

#[tokio::test]
async fn a_handshake_that_fails_in_any_way_pauses_the_remote_and_kills_no_item() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    for (status, said) in [(404_u16, "404"), (403, "refused the token (status 403)")] {
        receiver
            .force
            .fail(Fail::new(status, 1).on("/capabilities"));
        sender
            .sql("UPDATE replication_remote SET suspended_until = NULL")
            .await;
        sender.due().await;
        let pass = sender.pass().await;
        assert_eq!(
            (pass.suspended, pass.failed, pass.dead),
            (1, 0, 0),
            "{status}: {pass:?}"
        );
        let (state, attempts, _) = sender.state().await.unwrap();
        assert_eq!((state.as_str(), attempts), ("pending", 0));
        let said_by_remote = "SELECT last_error AS t FROM replication_remote";
        assert!(sender.text(said_by_remote).await.unwrap().contains(said));
    }
    assert_eq!(sender.number(FAILURES).await, 2);
}

#[tokio::test]
async fn a_client_that_cannot_be_built_pauses_the_remote_and_leaves_the_items_alone() {
    let (sender, _receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender
        .sql("UPDATE replication_remote SET token = 'not sealed at all'")
        .await;
    let pass = sender.pass().await;
    assert_eq!((pass.failed, pass.dead, pass.leased), (0, 0, 0), "{pass:?}");
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    assert_eq!(sender.number(FAILURES).await, 1);
    let said = "SELECT last_error AS t FROM replication_remote";
    assert!(
        sender
            .text(said)
            .await
            .unwrap()
            .contains("cannot be opened")
    );
    assert_eq!(sender.pass().await, Pass::default());
}

#[tokio::test]
async fn a_worker_on_an_old_remote_cannot_pause_the_one_that_was_just_fixed() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.hold("/blobs/missing");
    receiver.force.fail(Fail::new(401, 1).on("/blobs/missing"));
    let dispatcher = sender.dispatcher();
    let running = tokio::spawn(async move { dispatcher.run_once().await.unwrap() });
    receiver.force.reached().await;
    let id = sender
        .text("SELECT id::text AS t FROM replication_remote")
        .await
        .unwrap();
    let body = json!({
        "name": "fixed", "url": format!("http://{}", receiver.address),
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
    receiver.force.open();
    let pass = running.await.unwrap();
    assert_eq!((pass.suspended, pass.failed), (0, 0), "{pass:?}");
    assert_eq!(sender.number(FAILURES).await, 0);
    let paused = "SELECT count(*) AS n FROM replication_remote WHERE suspended_until IS NOT NULL";
    assert_eq!(sender.number(paused).await, 0);
    let waiting = "SELECT count(*) AS n FROM replication_item WHERE state = 'pending' AND not_before <= now()";
    assert_eq!(sender.number(waiting).await, 1);
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
}
