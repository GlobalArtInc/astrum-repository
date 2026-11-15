mod cluster;
mod support;

use axum::http::{Method, StatusCode};
use cluster::{Fail, Node, connect, replicate};
use serde_json::json;
use support::Auth;

const CAPABILITIES: &str = "/service/rest/v1/capabilities";
const METRICS: &str = "/service/metrics/prometheus";

async fn scrape(node: &Node) -> String {
    let reply = node
        .fixture
        .call(Method::GET, METRICS, Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.text()
}

#[tokio::test]
async fn the_metrics_show_the_backlog_the_counters_and_escape_the_names() {
    let name = "mir\"ror\\one";
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "source", "mirror", json!({"name": name})).await;
    let enabled = sender
        .fixture
        .call(
            Method::POST,
            CAPABILITIES,
            Auth::Admin,
            Some(json!({"type": "metrics.prometheus", "enabled": true, "properties": {}})),
        )
        .await;
    assert_eq!(enabled.status, StatusCode::CREATED, "{}", enabled.text());
    let label = "replication=\"mir\\\"ror\\\\one\"";

    sender.put("source", "/docs/a.txt", b"alpha").await;
    assert_eq!(sender.pass().await.sent, 1);
    sender.put("source", "/docs/b.txt", b"beta").await;
    receiver.force.fail(Fail::new(503, 1).on("/components"));
    assert_eq!(sender.pass().await.failed, 1);
    sender.due().await;
    receiver.force.fail(Fail::new(422, 1).on("/components"));
    assert_eq!(sender.pass().await.dead, 1);
    sender
        .sql(
            "INSERT INTO replication_item (replication_id, namespace, name, version, reason, \
             first_queued_at) SELECT id, 'ns', 'w', '1', 'event', now() - interval '90 seconds' \
             FROM replication",
        )
        .await;
    sender
        .sql("UPDATE replication_remote SET suspended_until = now() + interval '5 minutes'")
        .await;

    let text = scrape(&sender).await;
    let remote = sender
        .text("SELECT name AS t FROM replication_remote")
        .await
        .unwrap();
    for expected in [
        format!("astrum_replication_pending{{{label}}} 1"),
        format!("astrum_replication_dead{{{label}}} 1"),
        format!("astrum_replication_transfers_total{{{label},result=\"ok\"}} 1"),
        format!("astrum_replication_transfers_total{{{label},result=\"failed\"}} 1"),
        format!("astrum_replication_transfers_total{{{label},result=\"dead\"}} 1"),
        format!("astrum_replication_bytes_total{{{label}}} 9"),
        format!("astrum_replication_remote_suspended{{remote=\"{remote}\"}} 1"),
        "# TYPE astrum_replication_transfers_total counter".to_owned(),
    ] {
        assert!(
            text.contains(&expected),
            "{expected} is missing from\n{text}"
        );
    }
    let lag = text
        .lines()
        .find(|line| line.starts_with(&format!("astrum_replication_lag_seconds{{{label}}}")))
        .unwrap();
    let lag: f64 = lag.rsplit(' ').next().unwrap().parse().unwrap();
    assert!(lag >= 90.0, "{lag}");
    let success = text
        .lines()
        .find(|line| line.starts_with("astrum_replication_last_success_timestamp_seconds{"))
        .unwrap();
    let success: f64 = success.rsplit(' ').next().unwrap().parse().unwrap();
    assert!(success > 1.0e9, "{success}");

    sender.sql("DELETE FROM replication_item").await;
    sender
        .sql("UPDATE replication SET last_success_at = NULL")
        .await;
    let idle = scrape(&sender).await;
    for expected in [
        format!("astrum_replication_lag_seconds{{{label}}} 0"),
        format!("astrum_replication_last_success_timestamp_seconds{{{label}}} 0"),
    ] {
        assert!(
            idle.contains(&expected),
            "{expected} is missing from\n{idle}"
        );
    }
}
