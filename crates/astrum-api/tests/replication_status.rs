mod accounts;
mod cluster;
mod status_support;
mod support;

use accounts::{create_role, create_user};
use axum::http::{Method, StatusCode};
use serde_json::json;
use status_support::{REPLICATIONS, Setup, UNKNOWN};
use support::Auth;

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
    assert!(queued.json().get("note").is_none());
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

#[tokio::test]
async fn a_disabled_replication_is_told_that_the_run_will_be_skipped() {
    let setup = Setup::new().await;
    setup
        .sender
        .fixture
        .state
        .tasks()
        .run_pending()
        .await
        .unwrap();
    setup
        .sender
        .sql("UPDATE replication SET enabled = false")
        .await;
    let queued = setup.post("/reconcile", json!({})).await;
    assert_eq!(queued.status, StatusCode::ACCEPTED, "{}", queued.text());
    assert!(queued.json()["run"].is_i64());
    assert_eq!(
        queued.json()["note"],
        "the replication is disabled: the run will be skipped"
    );
}
