mod cluster;
mod support;

use std::time::Duration;

use astrum_replication::alert::State;
use astrum_replication::dispatch::Settings;
use axum::http::{Method, StatusCode};
use cluster::{Node, connect, replicate, wait_for};
use serde_json::json;
use support::Auth;
use tokio_util::sync::CancellationToken;

const CHANNELS: &str = "/service/rest/internal/ui/notifications";

struct Setup {
    sender: Node,
    channels: Vec<String>,
}

async fn channel(node: &Node, name: &str, language: &str) -> String {
    let reply = node
        .fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Admin,
            Some(json!({
                "name": name,
                "type": "webhook",
                "secret": "https://hooks.example.com/alerts",
                "events": ["published"],
                "language": language,
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["id"].as_str().unwrap().to_owned()
}

impl Setup {
    async fn new() -> Self {
        let sender = Node::start(&["source"]).await;
        let receiver = Node::start(&["mirror"]).await;
        let remote = connect(&sender, &receiver).await;
        let first = channel(&sender, "ops", "en").await;
        let second = channel(&sender, "oncall", "ru").await;
        replicate(
            &sender,
            &remote,
            "source",
            "mirror",
            json!({"name": "eu-mirror", "alertChannels": [first, second]}),
        )
        .await;
        Self {
            sender,
            channels: vec![first, second],
        }
    }

    async fn silent(&self, age: &str) {
        self.sender
            .sql(&format!(
                "UPDATE replication SET last_success_at = now() - interval '{age}'"
            ))
            .await;
    }

    async fn item(&self, state: &str, age: &str) {
        self.sender
            .sql(&format!(
                "INSERT INTO replication_item (replication_id, namespace, name, version, state, \
                 reason, last_error, first_queued_at) \
                 SELECT id, 'ns', 'n' || (SELECT count(*) FROM replication_item), '1', '{state}', \
                 'event', 'broken https://user:hunter2@remote.example/x', now() - interval '{age}' \
                 FROM replication"
            ))
            .await;
    }

    async fn evaluate(&self) -> Vec<(State, usize)> {
        self.sender
            .dispatcher()
            .evaluate_alerts()
            .await
            .unwrap()
            .into_iter()
            .map(|alerted| (alerted.to, alerted.messages))
            .collect()
    }

    async fn state(&self) -> String {
        self.sender
            .text("SELECT alert_state AS t FROM replication")
            .await
            .unwrap()
    }

    async fn queued(&self, event: &str) -> i64 {
        self.sender
            .number(&format!(
                "SELECT count(*) AS n FROM notification_delivery WHERE event = '{event}'"
            ))
            .await
    }

    async fn all_queued(&self) -> i64 {
        self.sender
            .number("SELECT count(*) AS n FROM notification_delivery")
            .await
    }
}

#[tokio::test]
async fn each_transition_queues_one_message_per_channel_and_recovery_follows() {
    let setup = Setup::new().await;
    assert!(setup.evaluate().await.is_empty());

    setup.silent("2 hours").await;
    setup.item("pending", "5 minutes").await;
    assert!(
        setup.evaluate().await.is_empty(),
        "a young item is no alarm"
    );

    setup.item("pending", "2 hours").await;
    assert_eq!(setup.evaluate().await, [(State::Failing, 2)]);
    assert_eq!(setup.state().await, "failing");
    assert_eq!(setup.queued("replication.failing").await, 2);
    assert!(setup.evaluate().await.is_empty());
    assert_eq!(setup.all_queued().await, 2);

    setup.item("dead", "1 hour").await;
    assert_eq!(setup.evaluate().await, [(State::DeadItems, 2)]);
    assert_eq!(setup.queued("replication.dead_items").await, 2);
    assert!(setup.evaluate().await.is_empty());

    setup.sender.sql("DELETE FROM replication_item").await;
    assert_eq!(setup.evaluate().await, [(State::Ok, 2)]);
    assert_eq!(setup.state().await, "ok");
    assert_eq!(setup.queued("replication.recovered").await, 2);
    assert!(setup.evaluate().await.is_empty());
    assert_eq!(setup.all_queued().await, 6);

    let texts = setup
        .sender
        .text(
            "SELECT string_agg(text, E'\\n---\\n' ORDER BY id) AS t FROM notification_delivery \
             WHERE event = 'replication.dead_items'",
        )
        .await
        .unwrap();
    assert!(texts.contains("eu-mirror") && texts.contains("Replication has dead items"));
    assert!(texts.contains("В репликации есть мёртвые элементы"));
    assert!(texts.contains("https://remote.example/x"), "{texts}");
    assert!(!texts.contains("hunter2"), "{texts}");
}

#[tokio::test]
async fn evaluations_that_race_send_each_message_once() {
    let setup = Setup::new().await;
    setup.silent("2 hours").await;
    setup.item("pending", "2 hours").await;
    let evaluators: Vec<_> = (0..4).map(|_| setup.sender.dispatcher()).collect();
    let results = futures_util::future::join_all(
        evaluators
            .iter()
            .map(|dispatcher| dispatcher.evaluate_alerts()),
    )
    .await;
    let moves: usize = results.into_iter().map(|moved| moved.unwrap().len()).sum();
    assert_eq!(moves, 1);
    assert_eq!(setup.all_queued().await, 2);
}

#[tokio::test]
async fn a_removed_channel_is_skipped_and_a_disabled_replication_stays_quiet() {
    let setup = Setup::new().await;
    let gone = setup
        .sender
        .fixture
        .call(
            Method::DELETE,
            &format!("{CHANNELS}/{}", setup.channels[1]),
            Auth::Admin,
            None,
        )
        .await;
    assert!(gone.status.is_success(), "{}", gone.text());
    setup.silent("2 hours").await;
    setup.item("pending", "2 hours").await;
    assert_eq!(setup.evaluate().await, [(State::Failing, 1)]);
    assert_eq!(setup.all_queued().await, 1);

    setup
        .sender
        .sql("UPDATE replication SET enabled = false")
        .await;
    assert_eq!(setup.evaluate().await, [(State::Ok, 0)]);
    assert_eq!(setup.state().await, "ok");
    setup.item("dead", "1 hour").await;
    assert!(setup.evaluate().await.is_empty());
    assert_eq!(setup.all_queued().await, 1);
}

#[tokio::test]
async fn a_suspended_remote_counts_as_failing_until_it_resumes() {
    let setup = Setup::new().await;
    setup
        .sender
        .sql(
            "UPDATE replication_remote SET failures = 1, last_error = 'remote down', \
             suspended_until = now() + interval '10 minutes'",
        )
        .await;
    assert_eq!(setup.evaluate().await, [(State::Failing, 2)]);
    let text = setup
        .sender
        .text(
            "SELECT text AS t FROM notification_delivery WHERE event = 'replication.failing' \
             ORDER BY id LIMIT 1",
        )
        .await
        .unwrap();
    assert!(
        text.contains("paused") && text.contains("remote down"),
        "{text}"
    );
    setup
        .sender
        .sql("UPDATE replication_remote SET suspended_until = NULL")
        .await;
    assert_eq!(setup.evaluate().await, [(State::Ok, 2)]);
    assert_eq!(setup.queued("replication.recovered").await, 2);
}

#[tokio::test]
async fn the_dispatcher_that_leads_evaluates_the_alerts_on_its_timer() {
    let setup = Setup::new().await;
    setup.silent("2 hours").await;
    setup.item("pending", "2 hours").await;
    setup
        .sender
        .sql("UPDATE replication_item SET not_before = now() + interval '1 day'")
        .await;
    let settings = Settings {
        alerts: Duration::from_millis(100),
        discover: Duration::from_millis(100),
        tick: Duration::from_millis(50),
        idle: Duration::from_millis(50),
        ..Settings::default()
    };
    let stop = CancellationToken::new();
    let running = setup
        .sender
        .dispatcher_with(settings)
        .spawn_until(stop.clone());
    wait_for(async || setup.state().await == "failing").await;
    stop.cancel();
    running.await.unwrap();
    assert_eq!(setup.queued("replication.failing").await, 2);
}
