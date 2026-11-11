mod support;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const SCAN: &str = "/service/rest/v1/security/scan";
const REPOSITORIES: &str = "/service/rest/v1/repositories/raw/hosted";
const TASKS: &str = "/service/rest/v1/tasks";
const UI_TASKS: &str = "/service/rest/internal/ui/tasks";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared("INSERT INTO content_policy (name) VALUES ('strict')")
        .await
        .unwrap();
    fixture
}

fn definition(name: &str, policy: Option<&str>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "ATTACHMENT"}
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    body
}

async fn repository(fixture: &Fixture, name: &str, policy: Option<&str>) {
    let reply = fixture
        .call(
            Method::POST,
            REPOSITORIES,
            Auth::Admin,
            Some(definition(name, policy)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn components(fixture: &Fixture, repository: &str, count: usize) -> Vec<i64> {
    let mut ids = Vec::new();
    for number in 0..count {
        let row = fixture
            .state
            .db()
            .conn()
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                format!(
                    "INSERT INTO component (repository, format, name, version) \
                     VALUES ('{repository}', 'raw', 'c{number}', '1') RETURNING id"
                ),
            ))
            .await
            .unwrap()
            .unwrap();
        ids.push(row.try_get("", "id").unwrap());
    }
    ids
}

async fn queued(fixture: &Fixture) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM scan_queue",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn wait_queued(fixture: &Fixture, expected: i64) {
    for _ in 0..50 {
        if queued(fixture).await == expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!(
        "{expected} components were expected in the queue, found {}",
        queued(fixture).await
    );
}

async fn clear(fixture: &Fixture) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared("DELETE FROM scan_queue")
        .await
        .unwrap();
}

#[tokio::test]
async fn a_rescan_needs_the_policies_right_and_a_repository_with_a_policy() {
    let fixture = ready().await;
    fixture.add_user("nobody", "secret-1", &[]).await;
    repository(&fixture, "guarded", Some("strict")).await;
    repository(&fixture, "open", None).await;
    let ids = components(&fixture, "guarded", 3).await;
    components(&fixture, "open", 1).await;
    clear(&fixture).await;
    let body = json!({"repository": "guarded"});

    let anonymous = fixture
        .call(Method::POST, SCAN, Auth::None, Some(body.clone()))
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let nobody = fixture
        .call(
            Method::POST,
            SCAN,
            Auth::Basic("nobody", "secret-1"),
            Some(body.clone()),
        )
        .await;
    assert_eq!(nobody.status, StatusCode::FORBIDDEN);
    assert_eq!(queued(&fixture).await, 0);

    let all = fixture
        .call(Method::POST, SCAN, Auth::Admin, Some(body))
        .await;
    assert_eq!(all.status, StatusCode::ACCEPTED, "{}", all.text());
    assert_eq!(
        all.json(),
        json!({"repository": "guarded", "started": true})
    );
    wait_queued(&fixture, 3).await;

    clear(&fixture).await;
    let one = fixture
        .call(
            Method::POST,
            SCAN,
            Auth::Admin,
            Some(json!({
                "repository": "guarded",
                "componentId": astrum_content::ids::encode("guarded", ids[1]),
            })),
        )
        .await;
    assert_eq!(one.status, StatusCode::ACCEPTED, "{}", one.text());
    assert_eq!(one.json()["queued"], 1);
    assert_eq!(queued(&fixture).await, 1);

    for (body, status) in [
        (json!({"repository": "missing"}), StatusCode::NOT_FOUND),
        (
            json!({"repository": "guarded", "componentId": astrum_content::ids::encode("guarded", 999_999)}),
            StatusCode::NOT_FOUND,
        ),
        (
            json!({"repository": "guarded", "componentId": astrum_content::ids::encode("open", ids[0])}),
            StatusCode::NOT_FOUND,
        ),
        (
            json!({"repository": "guarded", "componentId": "not an id"}),
            StatusCode::NOT_FOUND,
        ),
        (json!({"repository": "open"}), StatusCode::BAD_REQUEST),
        (
            json!({"repository": "guarded", "componentID": "x"}),
            StatusCode::BAD_REQUEST,
        ),
        (json!({"componentId": "x"}), StatusCode::BAD_REQUEST),
    ] {
        let reply = fixture
            .call(Method::POST, SCAN, Auth::Admin, Some(body.clone()))
            .await;
        assert_eq!(reply.status, status, "{body}: {}", reply.text());
    }
}

#[tokio::test]
async fn attaching_a_policy_queues_the_components_of_the_repository() {
    let fixture = ready().await;
    repository(&fixture, "files", None).await;
    components(&fixture, "files", 4).await;
    assert_eq!(queued(&fixture).await, 0);

    let unchanged = fixture
        .call(
            Method::PUT,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            Some(definition("files", None)),
        )
        .await;
    assert_eq!(
        unchanged.status,
        StatusCode::NO_CONTENT,
        "{}",
        unchanged.text()
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(queued(&fixture).await, 0);

    let attached = fixture
        .call(
            Method::PUT,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            Some(definition("files", Some("strict"))),
        )
        .await;
    assert_eq!(
        attached.status,
        StatusCode::NO_CONTENT,
        "{}",
        attached.text()
    );
    wait_queued(&fixture, 4).await;
}

#[tokio::test]
async fn the_scan_task_queues_a_repository_or_every_repository_with_a_policy() {
    let fixture = ready().await;
    fixture
        .state
        .tasks()
        .register(Arc::new(astrum_tasks::kinds::scan::ScanRepository));
    repository(&fixture, "guarded", Some("strict")).await;
    repository(&fixture, "open", None).await;
    components(&fixture, "guarded", 3).await;
    components(&fixture, "open", 2).await;
    clear(&fixture).await;

    let mut messages = Vec::new();
    for (name, repository) in [("all", "*"), ("named", "guarded"), ("refused", "open")] {
        let created = fixture
            .call(
                Method::POST,
                TASKS,
                Auth::Admin,
                Some(json!({
                    "type": "scan.repository", "name": name,
                    "frequency": {"schedule": "manual"},
                    "properties": {"repositoryName": repository, "reextract": "true"}
                })),
            )
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
        let id = created.json()["id"].as_str().unwrap().to_owned();
        let run = fixture
            .call(
                Method::POST,
                &format!("{TASKS}/{id}/run"),
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(run.status, StatusCode::NO_CONTENT);
        fixture.state.tasks().run_pending().await.unwrap();
        let runs = fixture
            .call(
                Method::GET,
                &format!("{UI_TASKS}/{id}/runs"),
                Auth::Admin,
                None,
            )
            .await
            .json();
        messages.push(runs[0]["message"].as_str().unwrap().to_owned());
        if name == "all" {
            assert_eq!(queued(&fixture).await, 3);
        }
    }
    assert!(
        messages[0].contains("queued 3 of 3 components"),
        "{messages:?}"
    );
    assert!(
        messages[1].contains("queued 3 of 3 components"),
        "{messages:?}"
    );
    assert!(messages[2].contains("no content policy"), "{messages:?}");
}

#[tokio::test]
async fn the_scrape_reports_the_queue_and_the_scan_durations() {
    let fixture = ready().await;
    repository(&fixture, "guarded", Some("strict")).await;
    components(&fixture, "guarded", 3).await;
    let enabled = fixture
        .call(
            Method::POST,
            "/service/rest/v1/capabilities",
            Auth::Admin,
            Some(json!({"type": "metrics.prometheus", "enabled": true, "properties": {}})),
        )
        .await;
    assert_eq!(enabled.status, StatusCode::CREATED, "{}", enabled.text());
    fixture.state.scan_metrics().observe(0.02);
    fixture.state.scan_metrics().failed();

    let text = fixture
        .call(
            Method::GET,
            "/service/metrics/prometheus",
            Auth::Admin,
            None,
        )
        .await
        .text();
    for expected in [
        "astrum_scan_queue_depth 3",
        "astrum_scan_seconds_bucket{le=\"0.025\"} 1",
        "astrum_scan_seconds_count 1",
        "astrum_scan_failures_total 1",
    ] {
        assert!(
            text.lines().any(|line| line == expected),
            "{expected}\n{text}"
        );
    }
    assert!(text.contains("astrum_scan_queue_oldest_seconds "), "{text}");
}
