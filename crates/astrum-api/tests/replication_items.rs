mod accounts;
mod cluster;
mod status_support;
mod support;

use accounts::{audit, create_role, create_user};
use astrum_replication::protocol::component_key;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use status_support::{REPLICATIONS, Setup, UNKNOWN};
use support::Auth;

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
async fn the_items_need_the_view_right_on_the_source_and_the_status_does_not() {
    let setup = Setup::new().await;
    let fixture = &setup.sender.fixture;
    for (user, privileges) in [
        ("reader", vec!["as-replication-read"]),
        (
            "viewer",
            vec!["as-replication-read", "as-repository-view-raw-source-read"],
        ),
    ] {
        create_role(fixture, user, &privileges, &[]).await;
        create_user(fixture, user, "pw-pw-pw-1", &[user]).await;
    }
    setup.item("dead", "a", 10).await;
    for (user, items) in [("reader", false), ("viewer", true)] {
        let auth = || Auth::Basic(user, "pw-pw-pw-1");
        let status = setup.call(Method::GET, "/status", auth(), None).await;
        assert_eq!(status.status, StatusCode::OK, "{user} status");
        let listed = setup
            .call(Method::GET, "/items?state=dead", auth(), None)
            .await;
        match items {
            true => assert_eq!(listed.status, StatusCode::OK, "{user} {}", listed.text()),
            false => assert_eq!(listed.status, StatusCode::FORBIDDEN, "{user}"),
        }
    }
    let unknown = fixture
        .call(
            Method::GET,
            &format!("{REPLICATIONS}/{UNKNOWN}/items?state=dead"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}
