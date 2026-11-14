mod cluster;
mod support;

use std::time::Duration;

use astrum_replication::dispatch::{Dispatcher, Lock, Settings};
use axum::http::StatusCode;
use cluster::{Node, connect, pair, replicate, wait_for};
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn quick() -> Settings {
    Settings {
        tick: Duration::from_millis(50),
        idle: Duration::from_millis(50),
        discover: Duration::from_millis(100),
        ..Settings::default()
    }
}

#[tokio::test]
async fn only_the_node_that_holds_the_lock_dispatches() {
    let (sender, receiver) = pair().await;
    let content = sender.fixture.state.content().clone();
    let holder = Lock::try_take(&content).await.unwrap().unwrap();
    assert!(Lock::try_take(&content).await.unwrap().is_none());
    let stop = CancellationToken::new();
    let second =
        tokio::spawn(Dispatcher::with_settings(content.clone(), quick()).serve(stop.clone()));
    sender.put("source", "/docs/a.txt", b"alpha").await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(sender.state().await.unwrap().0, "pending");
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    holder.release().await.unwrap();
    wait_for(async || sender.state().await.is_none()).await;
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
    stop.cancel();
    second.await.unwrap();
    assert!(Lock::try_take(&content).await.unwrap().is_some());
}

#[tokio::test]
async fn a_dispatcher_that_loses_its_lock_session_stops_and_another_takes_over() {
    let (sender, receiver) = pair().await;
    let content = sender.fixture.state.content().clone();
    let first = Dispatcher::with_settings(
        content.clone(),
        Settings {
            heartbeat: Duration::from_millis(100),
            idle: Duration::from_secs(30),
            ..quick()
        },
    );
    let second = Dispatcher::with_settings(content.clone(), quick());
    let stop = CancellationToken::new();
    let running = tokio::spawn(first.clone().serve(stop.clone()));
    wait_for(async || first.leading()).await;
    let waiting = tokio::spawn(second.clone().serve(stop.clone()));
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!second.leading());
    let holder = "SELECT pid::bigint AS n FROM pg_locks WHERE locktype = 'advisory' AND granted \
                  AND database = (SELECT oid FROM pg_database WHERE datname = current_database())";
    let pid = sender.number(holder).await;
    sender
        .sql(&format!("SELECT pg_terminate_backend({pid})"))
        .await;
    wait_for(async || !first.leading()).await;
    wait_for(async || second.leading()).await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    wait_for(async || sender.state().await.is_none()).await;
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
    stop.cancel();
    running.await.unwrap();
    waiting.await.unwrap();
}

async fn fan_out() -> (Node, Node, Node) {
    let sender = Node::start(&["source"]).await;
    let slow = Node::start(&["mirror"]).await;
    let fast = Node::start(&["mirror"]).await;
    for receiver in [&slow, &fast] {
        let remote = connect(&sender, receiver).await;
        replicate(&sender, &remote, "source", "mirror", json!({})).await;
    }
    (sender, slow, fast)
}

#[tokio::test]
async fn a_slow_remote_delays_neither_another_remote_nor_new_events() {
    let (sender, slow, fast) = fan_out().await;
    slow.force.hold("/blobs/missing");
    let dispatcher = sender.dispatcher_with(quick());
    let stop = CancellationToken::new();
    let running = tokio::spawn(dispatcher.clone().serve(stop.clone()));
    wait_for(async || dispatcher.leading()).await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    slow.force.reached().await;
    wait_for(async || fast.get("mirror", "/docs/a.txt").await.status == StatusCode::OK).await;
    sender.put("source", "/docs/b.txt", b"beta").await;
    wait_for(async || fast.get("mirror", "/docs/b.txt").await.status == StatusCode::OK).await;
    assert_eq!(
        slow.get("mirror", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    slow.force.open();
    wait_for(async || slow.get("mirror", "/docs/a.txt").await.status == StatusCode::OK).await;
    wait_for(async || slow.get("mirror", "/docs/b.txt").await.status == StatusCode::OK).await;
    stop.cancel();
    running.await.unwrap();
}

#[tokio::test]
async fn a_stop_gives_the_leased_items_back_before_the_tasks_end() {
    let (sender, receiver) = pair().await;
    receiver.force.hold("/blobs/missing");
    let dispatcher = sender.dispatcher_with(quick());
    let stop = CancellationToken::new();
    let running = tokio::spawn(dispatcher.clone().serve(stop.clone()));
    wait_for(async || dispatcher.leading()).await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.reached().await;
    assert_eq!(sender.state().await.unwrap().0, "leased");
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(10), running)
        .await
        .unwrap()
        .unwrap();
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 0));
    let waiting = "SELECT count(*) AS n FROM replication_item WHERE leased_until IS NULL AND not_before <= now()";
    assert_eq!(sender.number(waiting).await, 1);
    receiver.force.open();
    assert_eq!((receiver.force.uploads(), receiver.force.commits()), (0, 0));
}
