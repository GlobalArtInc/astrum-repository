mod accounts;
mod cluster;
mod support;

use accounts::{audit, create_role, create_user};
use astrum_replication::protocol::component_key;
use axum::http::{Method, StatusCode};
use cluster::{Node, connect, replicate};
use serde_json::{Value, json};
use support::Auth;

const REPLICATIONS: &str = "/service/rest/v1/replications";
const UNKNOWN: &str = "00000000-0000-0000-0000-000000000001";

struct Setup {
    sender: Node,
    _receiver: Node,
    id: String,
}

impl Setup {
    async fn new() -> Self {
        let sender = Node::start(&["source"]).await;
        let receiver = Node::start(&["mirror"]).await;
        let remote = connect(&sender, &receiver).await;
        let id = replicate(&sender, &remote, "source", "mirror", json!({})).await;
        Self {
            sender,
            _receiver: receiver,
            id,
        }
    }

    fn path(&self, tail: &str) -> String {
        format!("{REPLICATIONS}/{}{tail}", self.id)
    }

    async fn get(&self, tail: &str) -> support::Reply {
        self.call(Method::GET, tail, Auth::Admin, None).await
    }

    async fn post(&self, tail: &str, body: Value) -> support::Reply {
        self.call(Method::POST, tail, Auth::Admin, Some(body)).await
    }

    async fn call(
        &self,
        method: Method,
        tail: &str,
        auth: Auth<'_>,
        body: Option<Value>,
    ) -> support::Reply {
        self.sender
            .fixture
            .call(method, &self.path(tail), auth, body)
            .await
    }

    async fn item(&self, state: &str, name: &str, age_seconds: i64) -> i64 {
        let sql = format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, state, \
             reason, attempts, last_error, first_queued_at, leased_until) \
             VALUES ('{}', 'ns', '{name}', '1.0', '{state}', 'event', 4, 'broken {name}', \
             now() - interval '{age_seconds} seconds', \
             CASE WHEN '{state}' = 'leased' THEN now() + interval '10 minutes' END) RETURNING id AS n",
            self.id
        );
        let db = self.sender.fixture.state.db().conn();
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()
            .unwrap();
        row.try_get("", "n").unwrap()
    }

    async fn states(&self) -> Vec<(i64, String, i32, String)> {
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let sql = "SELECT id, state, attempts, reason FROM replication_item ORDER BY id";
        let rows = self
            .sender
            .fixture
            .state
            .db()
            .conn()
            .query_all(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
        rows.iter()
            .map(|row| {
                (
                    row.try_get("", "id").unwrap(),
                    row.try_get("", "state").unwrap(),
                    row.try_get("", "attempts").unwrap(),
                    row.try_get("", "reason").unwrap(),
                )
            })
            .collect()
    }
}

#[tokio::test]
async fn the_status_counts_items_measures_the_lag_and_shows_the_remote() {
    let setup = Setup::new().await;
    let empty = setup.get("/status").await;
    assert_eq!(empty.status, StatusCode::OK, "{}", empty.text());
    let empty = empty.json();
    assert_eq!(
        (empty["pending"].as_i64(), empty["dead"].as_i64()),
        (Some(0), Some(0))
    );
    assert!(empty["lagSeconds"].is_null());
    assert_eq!(empty["alertState"], "ok");
    assert_eq!(empty["enabled"], true);
    assert!(empty["remote"]["suspendedUntil"].is_null());

    setup.item("pending", "a", 300).await;
    setup.item("pending", "b", 10).await;
    setup.item("leased", "c", 120).await;
    for name in ["d", "e", "f"] {
        setup.item("dead", name, 9000).await;
    }
    setup
        .sender
        .sql(
            "UPDATE replication_remote SET failures = 2, last_error = 'remote down', \
             suspended_until = now() + interval '10 minutes'",
        )
        .await;
    setup
        .sender
        .sql("UPDATE replication SET last_success_at = now() - interval '1 hour', last_error = 'item broke'")
        .await;
    let status = setup.get("/status").await.json();
    assert_eq!(
        (
            status["pending"].as_i64(),
            status["leased"].as_i64(),
            status["dead"].as_i64()
        ),
        (Some(2), Some(1), Some(3))
    );
    let lag = status["lagSeconds"].as_i64().unwrap();
    assert!((300..320).contains(&lag), "{lag}");
    assert_eq!(status["lastError"], "item broke");
    assert!(status["lastSuccessAt"].as_str().unwrap().contains('T'));
    assert_eq!(status["remote"]["failures"], 2);
    assert_eq!(status["remote"]["lastError"], "remote down");
    assert!(status["remote"]["suspendedUntil"].is_string());

    let unknown = setup
        .sender
        .fixture
        .call(
            Method::GET,
            &format!("{REPLICATIONS}/{UNKNOWN}/status"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reading_needs_read_and_the_actions_need_update_on_the_source() {
    let setup = Setup::new().await;
    let fixture = &setup.sender.fixture;
    for (user, privileges) in [
        ("reader", vec!["as-replication-read"]),
        ("updater", vec!["as-replication-update"]),
        (
            "operator",
            vec![
                "as-replication-update",
                "as-repository-admin-raw-source-read",
            ],
        ),
    ] {
        create_role(fixture, user, &privileges, &[]).await;
        create_user(fixture, user, "pw-pw-pw-1", &[user]).await;
    }
    let dead = setup.item("dead", "a", 10).await;
    let body = json!({"ids": [dead]});
    let cases = [
        (Method::GET, "/status", None),
        (Method::GET, "/items?state=dead", None),
        (Method::POST, "/items/retry", Some(body.clone())),
        (Method::POST, "/items/discard", Some(body)),
        (Method::POST, "/reconcile", Some(json!({}))),
    ];
    for (method, tail, body) in cases {
        let reading = method == Method::GET;
        let label = format!("{method} {tail}");
        let anonymous = setup
            .call(method.clone(), tail, Auth::None, body.clone())
            .await;
        assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED, "{label}");
        for (user, allowed) in [
            ("reader", reading),
            ("updater", reading),
            ("operator", true),
        ] {
            let reply = setup
                .call(
                    method.clone(),
                    tail,
                    Auth::Basic(user, "pw-pw-pw-1"),
                    body.clone(),
                )
                .await;
            match allowed {
                true => assert!(
                    reply.status.is_success() || reply.status == StatusCode::CONFLICT,
                    "{label} {user} {}",
                    reply.text()
                ),
                false => assert_eq!(reply.status, StatusCode::FORBIDDEN, "{label} {user}"),
            }
        }
    }
    for tail in ["/items/retry", "/items/discard", "/reconcile"] {
        let unknown = fixture
            .call(
                Method::POST,
                &format!("{REPLICATIONS}/{UNKNOWN}{tail}"),
                Auth::Admin,
                Some(json!({"all": true})),
            )
            .await;
        assert_eq!(unknown.status, StatusCode::NOT_FOUND, "{tail}");
    }
}

#[tokio::test]
async fn items_are_listed_by_state_in_pages_of_ids() {
    let setup = Setup::new().await;
    let mut dead = Vec::new();
    for name in ["a", "b", "c", "d", "e"] {
        dead.push(setup.item("dead", name, 10).await);
    }
    setup.item("pending", "p", 10).await;
    let first = setup.get("/items?state=dead&limit=2").await.json();
    let listed = |page: &Value| -> Vec<i64> {
        page["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_i64().unwrap())
            .collect()
    };
    assert_eq!(listed(&first), dead[..2]);
    assert_eq!(first["next"], dead[1]);
    let head = &first["items"][0];
    assert_eq!(head["key"], component_key("ns", "a", "1.0"));
    assert_eq!(
        (head["state"].as_str(), head["reason"].as_str()),
        (Some("dead"), Some("event"))
    );
    assert_eq!(head["attempts"], 4);
    assert_eq!(head["lastError"], "broken a");
    assert!(head["notBefore"].is_string() && head["firstQueuedAt"].is_string());
    let second = setup
        .get(&format!("/items?state=dead&limit=2&after={}", dead[1]))
        .await
        .json();
    assert_eq!(listed(&second), dead[2..4]);
    let last = setup
        .get(&format!("/items?state=dead&limit=2&after={}", dead[3]))
        .await
        .json();
    assert_eq!(listed(&last), dead[4..]);
    assert!(last["next"].is_null());
    let pending = setup.get("/items?state=pending").await.json();
    assert_eq!(pending["items"].as_array().unwrap().len(), 1);
    for bad in [
        "/items",
        "/items?state=gone",
        "/items?state=dead&limit=0",
        "/items?state=dead&limit=201",
        "/items?state=dead&after=x",
    ] {
        assert_eq!(
            setup.get(bad).await.status,
            StatusCode::BAD_REQUEST,
            "{bad}"
        );
    }
}

#[tokio::test]
async fn retrying_and_discarding_touch_dead_items_only() {
    let setup = Setup::new().await;
    let a = setup.item("dead", "a", 10).await;
    let b = setup.item("dead", "b", 10).await;
    let c = setup.item("dead", "c", 10).await;
    let pending = setup.item("pending", "p", 10).await;
    let leased = setup.item("leased", "l", 10).await;

    let retried = setup
        .post("/items/retry", json!({"ids": [a, pending, leased]}))
        .await;
    assert_eq!(retried.status, StatusCode::OK, "{}", retried.text());
    assert_eq!(retried.json()["retried"], 1);
    let states = setup.states().await;
    assert_eq!(states[0], (a, "pending".to_owned(), 0, "retry".to_owned()));
    assert_eq!(states[1].1, "dead");
    assert_eq!(
        states[3],
        (pending, "pending".to_owned(), 4, "event".to_owned())
    );
    assert_eq!(
        states[4],
        (leased, "leased".to_owned(), 4, "event".to_owned())
    );
    let kept = "SELECT last_error AS t FROM replication_item WHERE name = 'a'";
    assert_eq!(setup.sender.text(kept).await.as_deref(), Some("broken a"));
    let due = "SELECT count(*) AS n FROM replication_item WHERE name = 'a' AND not_before <= now()";
    assert_eq!(setup.sender.number(due).await, 1);

    let all = setup.post("/items/retry", json!({"all": true})).await;
    assert_eq!(all.json()["retried"], 2);
    assert!(setup.states().await.iter().all(|item| item.1 != "dead"));

    setup
        .sender
        .sql("UPDATE replication_item SET state = 'dead' WHERE name IN ('b', 'c', 'p')")
        .await;
    let discarded = setup
        .post("/items/discard", json!({"ids": [b, pending, leased]}))
        .await;
    assert_eq!(discarded.json()["discarded"], 2);
    let left: Vec<i64> = setup.states().await.iter().map(|item| item.0).collect();
    assert_eq!(left, [a, c, leased]);
    let rest = setup.post("/items/discard", json!({"all": true})).await;
    assert_eq!(rest.json()["discarded"], 1);
    let left: Vec<i64> = setup.states().await.iter().map(|item| item.0).collect();
    assert_eq!(left, [a, leased]);

    let events: Vec<String> = audit(&setup.sender.fixture, "replication")
        .await
        .into_iter()
        .map(|event| event.0)
        .collect();
    assert!(events.contains(&"replication.items.retried".to_owned()));
    assert!(events.contains(&"replication.items.discarded".to_owned()));
}

#[tokio::test]
async fn a_selection_must_name_ids_or_all_and_no_more_than_a_thousand() {
    let setup = Setup::new().await;
    let dead = setup.item("dead", "a", 10).await;
    let many: Vec<i64> = (0..=1000).collect();
    for tail in ["/items/retry", "/items/discard"] {
        for body in [
            json!({"ids": [dead], "all": true}),
            json!({}),
            json!({"all": false}),
            json!({"ids": []}),
            json!({"ids": many}),
            json!({"ids": ["x"]}),
        ] {
            let reply = setup.post(tail, body.clone()).await;
            assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{tail} {body}");
        }
    }
    let exact: Vec<i64> = (dead + 1..=dead + 1000).collect();
    let reply = setup.post("/items/retry", json!({"ids": exact})).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(setup.states().await[0].1, "dead");
}

#[tokio::test]
async fn the_reconcile_route_queues_one_run_and_refuses_a_second() {
    let setup = Setup::new().await;
    let tasks = setup.sender.fixture.state.tasks();
    tasks.run_pending().await.unwrap();
    let waiting =
        "SELECT count(*) AS n FROM task_run WHERE state = 'waiting' AND trigger = 'manual'";
    let before = setup.sender.number(waiting).await;
    let queued = setup.post("/reconcile", json!({})).await;
    assert_eq!(queued.status, StatusCode::ACCEPTED, "{}", queued.text());
    assert!(queued.json()["run"].is_i64());
    assert_eq!(setup.sender.number(waiting).await, before + 1);
    let again = setup.post("/reconcile", json!({})).await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(
        again.json()["message"]
            .as_str()
            .unwrap()
            .contains("already")
    );
    tasks.run_pending().await.unwrap();
    assert_eq!(setup.sender.number(waiting).await, 0);

    setup
        .sender
        .sql("DELETE FROM task WHERE type = 'replication.reconcile'")
        .await;
    let recreated = setup.post("/reconcile", json!({})).await;
    assert_eq!(
        recreated.status,
        StatusCode::ACCEPTED,
        "{}",
        recreated.text()
    );
    let tasks = "SELECT count(*) AS n FROM task WHERE type = 'replication.reconcile'";
    assert_eq!(setup.sender.number(tasks).await, 1);
    assert_eq!(setup.sender.number(waiting).await, 1);
}
