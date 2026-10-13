mod support;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::http::{Method, StatusCode};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};

const TASKS: &str = "/service/rest/v1/tasks";
const POLICIES: &str = "/service/rest/internal/cleanup-policies";
const UI_TASKS: &str = "/service/rest/internal/ui/tasks";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    let tasks = fixture.state.tasks();
    tasks.register(Arc::new(astrum_format_docker::gc::Gc));
    tasks.add_expiry(Arc::new(astrum_format_docker::gc::UploadExpiry));
    fixture
}

async fn call(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    fixture.call(method, path, Auth::Admin, body).await
}

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

async fn count(fixture: &Fixture, sql: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

async fn raw_hosted(fixture: &Fixture, name: &str, cleanup: Option<&[&str]>) -> Reply {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    });
    if let Some(names) = cleanup {
        body["cleanup"] = json!({"policyNames": names});
    }
    call(
        fixture,
        Method::POST,
        "/service/rest/v1/repositories/raw/hosted",
        Some(body),
    )
    .await
}

async fn upload(fixture: &Fixture, repository: &str, path: &str, body: &'static [u8]) {
    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/{path}"),
            Auth::Admin,
            Body::from(body),
            &[],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn create_task(fixture: &Fixture, body: Value) -> Value {
    let reply = call(fixture, Method::POST, TASKS, Some(body)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()
}

async fn run_now(fixture: &Fixture, id: &str) -> Value {
    let reply = call(fixture, Method::POST, &format!("{TASKS}/{id}/run"), None).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let mut document = call(fixture, Method::GET, &format!("{TASKS}/{id}"), None)
        .await
        .json();
    document["runMessage"] = last_run(fixture, id).await["message"].clone();
    document
}

async fn last_run(fixture: &Fixture, id: &str) -> Value {
    let runs = call(fixture, Method::GET, &format!("{UI_TASKS}/{id}/runs"), None).await;
    assert_eq!(runs.status, StatusCode::OK, "{}", runs.text());
    runs.json()[0].clone()
}

#[tokio::test]
async fn tasks_are_created_read_changed_and_deleted_like_the_public_api() {
    let fixture = ready().await;
    let templates = call(&fixture, Method::GET, &format!("{TASKS}/templates"), None).await;
    assert_eq!(templates.status, StatusCode::OK);
    let templates = templates.json();
    let types: Vec<&str> = templates
        .as_array()
        .unwrap()
        .iter()
        .map(|template| template["type"].as_str().unwrap())
        .collect();
    for wanted in [
        "blobstore.compact",
        "repository.cleanup",
        "repository.docker.gc",
        "repository.purge-unused",
        "blobstore.metrics.reconcile",
        "security.purge-expired",
    ] {
        assert!(types.contains(&wanted), "{wanted} in {types:?}");
    }
    let compact = call(
        &fixture,
        Method::GET,
        &format!("{TASKS}/templates/blobstore.compact"),
        None,
    )
    .await
    .json();
    assert_eq!(compact["frequency"]["schedule"], "manual");
    assert_eq!(compact["properties"]["blobsOlderThan"], "1");
    assert_eq!(compact["formFields"][0]["id"], "blobstoreName");
    assert_eq!(compact["formFields"][0]["type"], "combobox");
    assert_eq!(compact["formFields"][0]["required"], true);
    assert_eq!(
        call(
            &fixture,
            Method::GET,
            &format!("{TASKS}/templates/nope"),
            None
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let created = create_task(
        &fixture,
        json!({
            "type": "blobstore.compact",
            "name": "Compact default",
            "enabled": true,
            "alertEmail": "ops@example.org",
            "notificationCondition": "FAILURE",
            "frequency": {"schedule": "manual"},
            "properties": {"blobstoreName": "default", "blobsOlderThan": 2}
        }),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["typeName"], "Admin - Compact blob store");
    assert_eq!(created["currentState"], "WAITING");
    assert_eq!(created["schedule"], "manual");
    assert_eq!(created["lastRunResult"], Value::Null);
    assert_eq!(created["nextRun"], Value::Null);
    assert_eq!(created["properties"]["blobsOlderThan"], "2");
    assert_eq!(created["alertEmail"], "ops@example.org");

    let duplicate = call(
        &fixture,
        Method::POST,
        TASKS,
        Some(json!({"type": "blobstore.compact", "name": "compact DEFAULT", "properties": {"blobstoreName": "default"}})),
    )
    .await;
    assert_eq!(
        duplicate.status,
        StatusCode::CONFLICT,
        "{}",
        duplicate.text()
    );

    for (body, field) in [
        (json!({"name": "x"}), "type"),
        (json!({"type": "nope", "name": "x"}), "type"),
        (
            json!({"type": "blobstore.compact", "name": "x"}),
            "properties.blobstoreName",
        ),
        (
            json!({"type": "blobstore.compact", "name": "x", "properties": {"blobstoreName": "missing"}}),
            "properties.blobstoreName",
        ),
        (
            json!({"type": "blobstore.compact", "name": "x", "properties": {"blobstoreName": "default", "blobsOlderThan": -1}}),
            "properties.blobsOlderThan",
        ),
        (
            json!({"type": "blobstore.compact", "name": "", "properties": {"blobstoreName": "default"}}),
            "name",
        ),
        (
            json!({"type": "blobstore.compact", "name": "x", "properties": {"blobstoreName": "default"}, "frequency": {"schedule": "cron", "cronExpression": "0 15 10 * *"}}),
            "frequency.cronExpression",
        ),
        (
            json!({"type": "blobstore.compact", "name": "x", "properties": {"blobstoreName": "default"}, "alertEmail": "nobody"}),
            "alertEmail",
        ),
    ] {
        let refused = call(&fixture, Method::POST, TASKS, Some(body.clone())).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(refused.json()[0]["id"], field, "{body}: {}", refused.text());
    }

    let changed = call(
        &fixture,
        Method::PUT,
        &format!("{TASKS}/{id}"),
        Some(json!({"frequency": {"schedule": "cron", "cronExpression": "0 0 3 * * ?"}})),
    )
    .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.text());
    let changed = changed.json();
    assert_eq!(changed["schedule"], "advanced");
    assert_eq!(changed["cronExpression"], "0 0 3 * * ?");
    assert_eq!(changed["timeZoneOffset"], "Z");
    assert!(
        changed["nextRun"]
            .as_str()
            .unwrap()
            .ends_with("T03:00:00.000+00:00")
    );
    assert_eq!(changed["name"], "Compact default");
    assert_eq!(changed["properties"]["blobstoreName"], "default");
    let weekly = call(
        &fixture,
        Method::PUT,
        &format!("{TASKS}/{id}"),
        Some(json!({"enabled": false, "frequency": {"schedule": "weekly", "startDate": 1791900000000_i64, "recurringDays": [2]}})),
    )
    .await
    .json();
    assert_eq!(weekly["enabled"], false);
    assert_eq!(weekly["nextRun"], Value::Null);
    assert_eq!(weekly["recurringDays"], json!([2]));
    assert_eq!(weekly["startDate"], "2026-10-13T14:00:00.000+00:00");
    let retyped = call(
        &fixture,
        Method::PUT,
        &format!("{TASKS}/{id}"),
        Some(json!({"type": "repository.cleanup"})),
    )
    .await;
    assert_eq!(retyped.status, StatusCode::BAD_REQUEST);
    assert_eq!(retyped.json()[0]["id"], "type");

    let listed = call(
        &fixture,
        Method::GET,
        &format!("{TASKS}?type=blobstore.compact"),
        None,
    )
    .await
    .json();
    assert_eq!(listed["items"].as_array().unwrap().len(), 1);
    assert_eq!(listed["continuationToken"], Value::Null);
    assert_eq!(
        call(
            &fixture,
            Method::GET,
            &format!("{TASKS}?type=repository.cleanup"),
            None
        )
        .await
        .json()["items"],
        json!([])
    );

    let deleted = call(&fixture, Method::DELETE, &format!("{TASKS}/{id}"), None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    for method in [Method::GET, Method::DELETE] {
        assert_eq!(
            call(&fixture, method, &format!("{TASKS}/{id}"), None)
                .await
                .status,
            StatusCode::NOT_FOUND
        );
    }
    let audit = count(
        &fixture,
        "SELECT count(*) AS n FROM audit_event WHERE domain = 'tasks' AND context = 'Compact default'",
    )
    .await;
    assert!(audit >= 3, "{audit}");
}

#[tokio::test]
async fn tasks_run_on_request_stop_and_keep_a_log() {
    let fixture = ready().await;
    let created = create_task(
        &fixture,
        json!({"type": "blobstore.metrics.reconcile", "name": "Recount", "properties": {"blobstoreName": "*"}}),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(
        call(&fixture, Method::POST, &format!("{TASKS}/{id}/stop"), None)
            .await
            .status,
        StatusCode::CONFLICT
    );
    let requested = call(&fixture, Method::POST, &format!("{TASKS}/{id}/run"), None).await;
    assert_eq!(requested.status, StatusCode::NO_CONTENT);
    let queued = call(&fixture, Method::GET, &format!("{UI_TASKS}/{id}"), None)
        .await
        .json();
    assert_eq!(queued["state"], "queued");
    assert_eq!(
        call(&fixture, Method::POST, &format!("{TASKS}/{id}/run"), None)
            .await
            .status,
        StatusCode::CONFLICT
    );
    fixture.state.tasks().run_pending().await.unwrap();
    let after = call(&fixture, Method::GET, &format!("{TASKS}/{id}"), None)
        .await
        .json();
    assert_eq!(after["currentState"], "WAITING");
    assert_eq!(after["lastRunResult"], "OK");
    assert!(after["lastRun"].as_str().is_some());
    assert!(
        last_run(&fixture, &id).await["message"]
            .as_str()
            .unwrap()
            .contains("counted the blobs")
    );
    assert!(
        after["message"]
            .as_str()
            .unwrap()
            .starts_with("Counts the blobs")
    );
    let run = last_run(&fixture, &id).await;
    assert_eq!(run["state"], "done");
    assert_eq!(run["trigger"], "manual");
    assert_eq!(run["initiator"], "admin");
    assert!(run["durationMs"].as_i64().is_some());
    let log = call(
        &fixture,
        Method::GET,
        &format!("{UI_TASKS}/{id}/runs/{}/log", run["id"]),
        None,
    )
    .await;
    assert_eq!(log.status, StatusCode::OK, "{}", log.text());
    let log = log.json();
    assert_eq!(log["finished"], true);
    let lines = log["lines"].as_array().unwrap();
    assert!(lines.len() >= 3, "{lines:?}");
    assert!(
        lines[0]["message"]
            .as_str()
            .unwrap()
            .starts_with("Recount started")
    );
    assert!(
        lines
            .iter()
            .any(|line| line["message"].as_str().unwrap().starts_with("default:"))
    );
    let later = call(
        &fixture,
        Method::GET,
        &format!(
            "{UI_TASKS}/{id}/runs/{}/log?after={}",
            run["id"],
            lines.len()
        ),
        None,
    )
    .await
    .json();
    assert_eq!(later["lines"], json!([]));

    call(&fixture, Method::POST, &format!("{TASKS}/{id}/run"), None).await;
    let stopped = call(&fixture, Method::POST, &format!("{TASKS}/{id}/stop"), None).await;
    assert_eq!(stopped.status, StatusCode::NO_CONTENT);
    let cancelled = last_run(&fixture, &id).await;
    assert_eq!(cancelled["state"], "cancelled");
    fixture.state.tasks().run_pending().await.unwrap();
    let doc = call(&fixture, Method::GET, &format!("{TASKS}/{id}"), None)
        .await
        .json();
    assert_eq!(doc["lastRunResult"], "CANCELED");

    call(
        &fixture,
        Method::PUT,
        &format!("{TASKS}/{id}"),
        Some(json!({"enabled": false})),
    )
    .await;
    assert_eq!(
        call(&fixture, Method::POST, &format!("{TASKS}/{id}/run"), None)
            .await
            .status,
        StatusCode::METHOD_NOT_ALLOWED
    );
    let events = count(
        &fixture,
        "SELECT count(*) AS n FROM audit_event WHERE domain = 'tasks' AND type IN ('started', 'finished', 'stopped')",
    )
    .await;
    assert!(events >= 3, "{events}");
}

#[tokio::test]
async fn schedules_fire_when_due_and_abandoned_runs_fail() {
    let fixture = ready().await;
    let created = create_task(
        &fixture,
        json!({
            "type": "blobstore.metrics.reconcile",
            "name": "Hourly recount",
            "frequency": {"schedule": "hourly", "startDate": (Utc::now() - chrono::Duration::minutes(90)).timestamp_millis()},
            "properties": {"blobstoreName": "default"}
        }),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_owned();
    let next = created["nextRun"].as_str().unwrap().to_owned();
    execute(
        &fixture,
        &format!("UPDATE task SET next_run_at = now() - interval '1 minute' WHERE id = '{id}'"),
    )
    .await;
    fixture.state.tasks().run_pending().await.unwrap();
    let run = last_run(&fixture, &id).await;
    assert_eq!(run["trigger"], "schedule");
    assert_eq!(run["state"], "done");
    let after = call(&fixture, Method::GET, &format!("{TASKS}/{id}"), None)
        .await
        .json();
    assert_eq!(after["nextRun"], next.as_str());

    execute(
        &fixture,
        &format!(
            "INSERT INTO task_run (task_id, state, trigger, started_at) VALUES ('{id}', 'running', 'manual', now())"
        ),
    )
    .await;
    let running = call(&fixture, Method::GET, &format!("{TASKS}/{id}"), None)
        .await
        .json();
    assert_eq!(running["currentState"], "RUNNING");
    assert_eq!(fixture.state.tasks().fail_orphans().await.unwrap(), 1);
    let orphan = last_run(&fixture, &id).await;
    assert_eq!(orphan["state"], "failed");
    assert!(orphan["message"].as_str().unwrap().contains("stopped"));

    let lock = astrum_tasks::scheduler::lock_key(uuid::Uuid::parse_str(&id).unwrap());
    let pool = fixture.state.db().conn().get_postgres_connection_pool();
    let mut other = pool.acquire().await.unwrap();
    let (taken,): (bool,) = sea_orm::sqlx::query_as("SELECT pg_try_advisory_lock($1)")
        .bind(lock)
        .fetch_one(&mut *other)
        .await
        .unwrap();
    assert!(taken);
    call(&fixture, Method::POST, &format!("{TASKS}/{id}/run"), None).await;
    let started = fixture.state.tasks().tick().await.unwrap();
    assert!(
        started.is_empty(),
        "another holder of the lock keeps the run waiting"
    );
    assert_eq!(last_run(&fixture, &id).await["state"], "waiting");
    sea_orm::sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(lock)
        .execute(&mut *other)
        .await
        .unwrap();
    fixture.state.tasks().run_pending().await.unwrap();
    assert_eq!(last_run(&fixture, &id).await["state"], "done");
}

#[tokio::test]
async fn built_in_tasks_are_created_once_and_stay_editable() {
    let fixture = ready().await;
    let db = fixture.state.db().conn();
    assert_eq!(astrum_tasks::bootstrap(db).await.unwrap(), 4);
    assert_eq!(astrum_tasks::bootstrap(db).await.unwrap(), 0);
    let listed = call(&fixture, Method::GET, TASKS, None).await.json();
    let names: Vec<&str> = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Cleanup service",
            "Compact blob stores",
            "Delete expired sessions and uploads",
            "Delete unused Docker manifests and layers"
        ]
    );
    let cleanup = &listed["items"][0];
    assert_eq!(cleanup["type"], "repository.cleanup");
    assert_eq!(cleanup["cronExpression"], "0 0 1 * * ?");
    let templates = call(&fixture, Method::GET, &format!("{TASKS}/templates"), None)
        .await
        .json();
    assert!(
        !templates
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["type"] == "repository.cleanup"),
        "only one cleanup task"
    );
    let second = call(
        &fixture,
        Method::POST,
        TASKS,
        Some(json!({"type": "repository.cleanup", "name": "Another cleanup"})),
    )
    .await;
    assert_eq!(second.status, StatusCode::BAD_REQUEST);
    let id = cleanup["id"].as_str().unwrap();
    let edited = call(
        &fixture,
        Method::PUT,
        &format!("{TASKS}/{id}"),
        Some(json!({"frequency": {"schedule": "cron", "cronExpression": "0 30 2 * * ?"}})),
    )
    .await;
    assert_eq!(edited.status, StatusCode::OK);
    fixture.add_user("viewer", "viewer-password", &[]).await;
    let refused = fixture
        .call(
            Method::GET,
            TASKS,
            Auth::Basic("viewer", "viewer-password"),
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cleanup_policies_are_managed_previewed_assigned_and_applied() {
    let fixture = ready().await;
    let formats = call(
        &fixture,
        Method::GET,
        &format!("{POLICIES}/criteria/formats"),
        None,
    )
    .await
    .json();
    assert_eq!(formats[0]["id"], "*");
    let maven = formats
        .as_array()
        .unwrap()
        .iter()
        .find(|format| format["id"] == "maven2")
        .unwrap();
    assert!(
        maven["availableCriteria"]
            .as_array()
            .unwrap()
            .contains(&json!("isPrerelease"))
    );

    let created = call(
        &fixture,
        Method::POST,
        POLICIES,
        Some(json!({"name": "old-raw", "format": "raw", "notes": "older than a week", "criteriaLastBlobUpdated": 7, "criteriaAssetRegex": "docs/.*"})),
    )
    .await;
    assert_eq!(created.status, StatusCode::OK, "{}", created.text());
    assert_eq!(created.json()["criteriaLastBlobUpdated"], 7);
    assert_eq!(created.json()["inUseCount"], 0);
    let again = call(
        &fixture,
        Method::POST,
        POLICIES,
        Some(json!({"name": "OLD-RAW", "format": "raw", "criteriaLastBlobUpdated": 7})),
    )
    .await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert_eq!(again.json()[0]["id"], "name");
    let wrong = call(
        &fixture,
        Method::POST,
        POLICIES,
        Some(json!({"name": "pre", "format": "raw", "criteriaReleaseType": "PRERELEASES"})),
    )
    .await;
    assert_eq!(wrong.json()[0]["id"], "criteriaReleaseType");
    let snapshots = call(
        &fixture,
        Method::POST,
        "/service/rest/v1/cleanup-policies",
        Some(json!({"name": "snapshots", "format": "maven2", "criteriaReleaseType": "PRERELEASES", "criteriaLastDownloaded": 30})),
    )
    .await;
    assert_eq!(
        snapshots.status,
        StatusCode::CREATED,
        "{}",
        snapshots.text()
    );
    assert_eq!(snapshots.json().get("inUseCount"), None);

    assert_eq!(
        raw_hosted(&fixture, "docs", Some(&["snapshots"]))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        raw_hosted(&fixture, "docs", Some(&["old-raw"]))
            .await
            .status,
        StatusCode::CREATED
    );
    for (path, body) in [
        ("docs/a.txt", b"first" as &'static [u8]),
        ("docs/b.txt", b"second"),
        ("other/c.txt", b"third"),
        ("docs/new.txt", b"fresh"),
    ] {
        upload(&fixture, "docs", path, body).await;
    }
    execute(
        &fixture,
        "UPDATE asset SET blob_updated_at = now() - interval '10 days' WHERE path <> '/docs/new.txt'",
    )
    .await;

    let used = call(&fixture, Method::GET, &format!("{POLICIES}/old-raw"), None)
        .await
        .json();
    assert_eq!(used["repositories"], json!(["docs"]));
    let users = call(
        &fixture,
        Method::GET,
        &format!("{POLICIES}/old-raw/repositories"),
        None,
    )
    .await
    .json();
    assert_eq!(
        users,
        json!([{"name": "docs", "format": "raw", "type": "hosted"}])
    );

    let preview_body = json!({"name": "old-raw", "format": "raw", "criteriaLastBlobUpdated": 7, "criteriaAssetRegex": "docs/.*", "repository": "docs"});
    let preview = call(
        &fixture,
        Method::POST,
        &format!("{POLICIES}/preview/components?page=0&size=1"),
        Some(preview_body.clone()),
    )
    .await;
    assert_eq!(preview.status, StatusCode::OK, "{}", preview.text());
    let preview = preview.json();
    assert_eq!(preview["total"], 2);
    assert_eq!(preview["results"].as_array().unwrap().len(), 1);
    assert_eq!(preview["results"][0]["name"], "/docs/a.txt");
    let second = call(
        &fixture,
        Method::POST,
        &format!("{POLICIES}/preview/components?page=1&size=1"),
        Some(preview_body.clone()),
    )
    .await
    .json();
    assert_eq!(second["results"][0]["name"], "/docs/b.txt");
    let summary = call(
        &fixture,
        Method::POST,
        &format!("{POLICIES}/preview/summary"),
        Some(preview_body.clone()),
    )
    .await
    .json();
    assert_eq!(
        summary,
        json!({"repository": "docs", "componentCount": 2, "assetCount": 2, "totalSize": 11, "complete": true})
    );
    let csv = call(
        &fixture,
        Method::GET,
        &format!("{POLICIES}/preview/components/csv?repository=docs&format=raw&criteriaLastBlobUpdated=7&criteriaAssetRegex=docs/.*"),
        None,
    )
    .await;
    assert_eq!(csv.status, StatusCode::OK);
    assert!(
        csv.headers["content-disposition"]
            .to_str()
            .unwrap()
            .starts_with("attachment; filename=\"CleanupPreview-docs-")
    );
    let text = csv.text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(lines[0].starts_with("Namespace,Name,Version,Path"));
    assert!(lines[1].contains(",/docs/a.txt,default,5,"));
    let group = call(
        &fixture,
        Method::POST,
        &format!("{POLICIES}/preview/summary"),
        Some(json!({"format": "raw", "criteriaLastBlobUpdated": 7, "repository": "nowhere"})),
    )
    .await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST);

    let cleanup = create_task(
        &fixture,
        json!({"type": "repository.cleanup", "name": "Cleanup"}),
    )
    .await;
    let done = run_now(&fixture, cleanup["id"].as_str().unwrap()).await;
    assert_eq!(done["lastRunResult"], "OK", "{done}");
    assert!(
        done["runMessage"]
            .as_str()
            .unwrap()
            .starts_with("deleted 2 components")
    );
    let left: Vec<String> =
        astrum_content::store::under(fixture.state.db().conn(), "docs", "/", 10)
            .await
            .unwrap()
            .into_iter()
            .map(|asset| asset.path)
            .collect();
    assert_eq!(left, ["/docs/new.txt", "/other/c.txt"]);
    let audited = count(
        &fixture,
        "SELECT count(*) AS n FROM audit_event WHERE domain = 'component' AND type = 'deleted' AND initiator = '*TASK' AND attributes ->> 'policy' = 'old-raw'",
    )
    .await;
    assert_eq!(audited, 2);
    let run = last_run(&fixture, cleanup["id"].as_str().unwrap()).await;
    assert_eq!(
        run["counts"],
        json!({"assets": 2, "bytes": 11, "components": 2})
    );

    let changed_format = call(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/old-raw"),
        Some(json!({"name": "old-raw", "format": "npm", "criteriaLastBlobUpdated": 7})),
    )
    .await;
    assert_eq!(changed_format.status, StatusCode::BAD_REQUEST);
    let updated = call(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/old-raw"),
        Some(
            json!({"name": "renamed", "format": "raw", "notes": "", "criteriaLastDownloaded": 30}),
        ),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    assert_eq!(updated.json()["name"], "old-raw");
    assert_eq!(updated.json()["criteriaLastBlobUpdated"], Value::Null);

    let removed = call(
        &fixture,
        Method::DELETE,
        &format!("{POLICIES}/old-raw"),
        None,
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let repository = call(
        &fixture,
        Method::GET,
        "/service/rest/v1/repositories/raw/hosted/docs",
        None,
    )
    .await
    .json();
    assert_eq!(repository["cleanup"], Value::Null);
    let names = call(
        &fixture,
        Method::GET,
        "/service/rest/internal/ui/cleanup-policies/names?format=maven2",
        None,
    )
    .await
    .json();
    assert_eq!(
        names,
        json!([{"name": "snapshots", "format": "maven2", "notes": ""}])
    );
}

#[tokio::test]
async fn compaction_deletes_old_blobs_nothing_refers_to() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files", None).await;
    upload(&fixture, "files", "kept.txt", b"kept").await;
    let store = fixture.state.blob_stores().get("default").await.unwrap();
    let orphan = store
        .put(astrum_blob::content_of(b"orphan".to_vec()))
        .await
        .unwrap();
    let fresh = store
        .put(astrum_blob::content_of(b"fresh".to_vec()))
        .await
        .unwrap();
    let blobs = fixture.data.path().join("blobs/default/content");
    let old = SystemTime::now() - Duration::from_secs(3 * 86_400);
    for entry in walk(&blobs) {
        if !entry.ends_with(fresh.id.to_string()) {
            std::fs::File::options()
                .write(true)
                .open(&entry)
                .unwrap()
                .set_modified(old)
                .unwrap();
        }
    }
    let task = create_task(
        &fixture,
        json!({"type": "blobstore.compact", "name": "Compact", "properties": {"blobstoreName": "*", "blobsOlderThan": "1"}}),
    )
    .await;
    let done = run_now(&fixture, task["id"].as_str().unwrap()).await;
    assert_eq!(done["lastRunResult"], "OK", "{done}");
    assert!(!store.exists(orphan.id).await.unwrap());
    assert!(store.exists(fresh.id).await.unwrap());
    let kept = astrum_content::store::find(fixture.state.db().conn(), "files", "/kept.txt")
        .await
        .unwrap()
        .unwrap();
    assert!(store.exists(kept.blob_id.into()).await.unwrap());
    assert_eq!(
        last_run(&fixture, task["id"].as_str().unwrap()).await["counts"],
        json!({"blobs": 1, "bytes": 6})
    );

    let reconcile = create_task(
        &fixture,
        json!({"type": "blobstore.metrics.reconcile", "name": "Recount", "properties": {"blobstoreName": "default"}}),
    )
    .await;
    execute(
        &fixture,
        "UPDATE blob_store_metrics SET blob_count = 99, total_bytes = 99",
    )
    .await;
    run_now(&fixture, reconcile["id"].as_str().unwrap()).await;
    let totals = store.totals().await.unwrap();
    assert_eq!((totals.blob_count, totals.total_bytes), (2, 9));
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

async fn push_blob(fixture: &Fixture, bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let reply = fixture
        .send_body(
            Method::POST,
            &format!("/repository/images/v2/app/blobs/uploads/?digest={digest}"),
            Auth::Admin,
            Body::from(bytes.to_vec()),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    digest
}

async fn push_image(fixture: &Fixture, tag: &str, layer: &[u8]) -> String {
    let config = push_blob(fixture, b"{\"os\":\"linux\"}").await;
    let layer = push_blob(fixture, layer).await;
    let manifest = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {"mediaType": "application/vnd.oci.image.config.v1+json", "digest": config, "size": 14},
        "layers": [{"mediaType": "application/vnd.oci.image.layer.v1.tar+gzip", "digest": layer, "size": 1}],
    });
    let bytes = serde_json::to_vec(&manifest).unwrap();
    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/images/v2/app/manifests/{tag}"),
            Auth::Admin,
            Body::from(bytes.clone()),
            &[("content-type", "application/vnd.oci.image.manifest.v1+json")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    layer
}

#[tokio::test]
async fn docker_garbage_collection_removes_untagged_manifests_and_their_layers() {
    let fixture = ready().await;
    let created = call(
        &fixture,
        Method::POST,
        "/service/rest/v1/repositories/docker/hosted",
        Some(json!({
            "name": "images",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            "docker": {"v1Enabled": false, "forceBasicAuth": true}
        })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let kept_layer = push_image(&fixture, "stable", b"layer of stable").await;
    let gone_layer = push_image(&fixture, "old", b"layer of old").await;
    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/images/v2/app/manifests/old",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED, "{}", deleted.text());
    let task = create_task(
        &fixture,
        json!({"type": "repository.docker.gc", "name": "GC", "properties": {"repositoryName": "images", "deployOffset": 24}}),
    )
    .await;
    let id = task["id"].as_str().unwrap().to_owned();
    let young = run_now(&fixture, &id).await;
    assert!(
        young["runMessage"]
            .as_str()
            .unwrap()
            .starts_with("deleted 0 untagged manifests and 0")
    );
    execute(
        &fixture,
        "UPDATE asset SET blob_updated_at = now() - interval '2 days'",
    )
    .await;
    let done = run_now(&fixture, &id).await;
    assert_eq!(done["lastRunResult"], "OK", "{done}");
    assert_eq!(
        last_run(&fixture, &id).await["counts"]["manifests"],
        1,
        "{}",
        done["runMessage"]
    );
    let blob = |digest: &str| format!("/repository/images/v2/app/blobs/{digest}");
    assert_eq!(
        fixture
            .call(Method::HEAD, &blob(&kept_layer), Auth::Admin, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .call(Method::HEAD, &blob(&gone_layer), Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/images/v2/app/manifests/stable",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::OK
    );
    let wrong = call(
        &fixture,
        Method::POST,
        TASKS,
        Some(json!({"type": "repository.docker.gc", "name": "GC raw", "properties": {"repositoryName": "missing"}})),
    )
    .await;
    assert_eq!(wrong.json()[0]["id"], "properties.repositoryName");
}

#[tokio::test]
async fn expired_sessions_tokens_and_uploads_are_deleted() {
    let fixture = ready().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        "security.sessionTimeoutMinutes",
        &30u32,
    )
    .await
    .unwrap();
    execute(
        &fixture,
        "INSERT INTO user_session (token_hash, user_id, realm, credential_stamp, csrf_token, created_at, last_seen_at, client_ip, user_agent) VALUES \
         ('old', 'admin', 'AstrumAuthenticatingRealm', 's', 'c', now() - interval '2 hours', now() - interval '2 hours', '', ''), \
         ('new', 'admin', 'AstrumAuthenticatingRealm', 's', 'c', now(), now(), '', '')",
    )
    .await;
    fixture
        .state
        .security()
        .issue_format_token("DockerToken", "admin", Some(chrono::Duration::seconds(-5)))
        .await
        .unwrap();
    execute(
        &fixture,
        "INSERT INTO docker_upload (id, repository, image, blob_store, updated_at) VALUES \
         ('00000000-0000-4000-8000-000000000001', 'images', 'app', 'default', now() - interval '25 hours')",
    )
    .await;
    let task = create_task(
        &fixture,
        json!({"type": "security.purge-expired", "name": "Expire"}),
    )
    .await;
    let done = run_now(&fixture, task["id"].as_str().unwrap()).await;
    assert_eq!(done["lastRunResult"], "OK", "{done}");
    assert_eq!(
        done["runMessage"],
        "deleted 1 session, 1 client token and 1 unfinished upload"
    );
    assert_eq!(
        count(&fixture, "SELECT count(*) AS n FROM user_session").await,
        1
    );
}

#[tokio::test]
async fn purge_unused_deletes_cached_content_nobody_downloaded() {
    let fixture = ready().await;
    let created = call(
        &fixture,
        Method::POST,
        "/service/rest/v1/repositories/raw/proxy",
        Some(json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "proxy": {"remoteUrl": "http://127.0.0.1:9/", "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": true, "autoBlock": false}
        })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    raw_hosted(&fixture, "local", None).await;
    upload(&fixture, "local", "stays.txt", b"local").await;
    execute(
        &fixture,
        "INSERT INTO asset (repository, format, path, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, created_at, last_downloaded_at) VALUES \
         ('mirror', 'raw', '/unused.txt', 'default', '00000000-0000-4000-8000-0000000000aa', 3, 'text/plain', '', '', '', '', now() - interval '40 days', NULL), \
         ('mirror', 'raw', '/used.txt', 'default', '00000000-0000-4000-8000-0000000000bb', 4, 'text/plain', '', '', '', '', now() - interval '40 days', now())",
    )
    .await;
    execute(
        &fixture,
        "UPDATE asset SET created_at = now() - interval '40 days' WHERE repository = 'local'",
    )
    .await;
    let task = create_task(
        &fixture,
        json!({"type": "repository.purge-unused", "name": "Purge", "properties": {"repositoryName": "*", "lastUsed": 30}}),
    )
    .await;
    let done = run_now(&fixture, task["id"].as_str().unwrap()).await;
    assert_eq!(done["lastRunResult"], "OK", "{done}");
    let left = count(&fixture, "SELECT count(*) AS n FROM asset").await;
    assert_eq!(left, 2);
    let hosted = call(
        &fixture,
        Method::POST,
        TASKS,
        Some(json!({"type": "repository.purge-unused", "name": "Purge local", "properties": {"repositoryName": "local", "lastUsed": 30}})),
    )
    .await;
    assert_eq!(hosted.json()[0]["id"], "properties.repositoryName");
}
