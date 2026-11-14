mod support;

use std::sync::Arc;

use astrum_content::zip;
use astrum_db::entity::audit_event;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const FEEDS: &str = "/service/rest/v1/security/advisories/feeds";
const IMPORT: &str = "/service/rest/v1/security/advisories/import";
const TASKS: &str = "/service/rest/v1/tasks";
const UI_TASKS: &str = "/service/rest/internal/ui/tasks";

fn record(id: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": id,
        "modified": "2024-03-01T00:00:00Z",
        "database_specific": {"severity": "CRITICAL"},
        "affected": [{"package": {"ecosystem": "npm", "name": "left-pad"},
                      "ranges": [{"type": "SEMVER", "events": [{"introduced": "0"}, {"fixed": "1.3.0"}]}]}]
    }))
    .unwrap()
}

async fn post_zip(fixture: &Fixture, path: &str, auth: Auth<'_>, body: Vec<u8>) -> support::Reply {
    fixture
        .send_body(
            Method::POST,
            path,
            auth,
            Body::from(body),
            &[("content-type", "application/zip")],
        )
        .await
}

async fn advisories(fixture: &Fixture) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM advisory",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
async fn the_routes_need_a_signed_in_caller_with_the_right() {
    let fixture = Fixture::new().await;
    fixture.add_user("nobody", "secret-1", &[]).await;
    let nobody = || Auth::Basic("nobody", "secret-1");
    let archive = zip::write(&[("GHSA-1.json", &record("GHSA-1"))]);
    let url = format!("{IMPORT}?ecosystem=npm");
    assert_eq!(
        fixture
            .call(Method::GET, FEEDS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        fixture
            .call(Method::GET, FEEDS, nobody(), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        post_zip(&fixture, &url, Auth::None, archive.clone())
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post_zip(&fixture, &url, nobody(), archive).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(advisories(&fixture).await, 0);
}

#[tokio::test]
async fn an_uploaded_dump_is_imported_listed_and_audited() {
    let fixture = Fixture::new().await;
    let feeds = fixture.call(Method::GET, FEEDS, Auth::Admin, None).await;
    assert_eq!(feeds.status, StatusCode::OK);
    assert_eq!(feeds.json(), json!([]));

    for (path, body) in [
        (
            IMPORT.to_owned(),
            zip::write(&[("A-1.json", &record("A-1"))]),
        ),
        (
            format!("{IMPORT}?ecosystem=../x"),
            zip::write(&[("A-1.json", &record("A-1"))]),
        ),
        (format!("{IMPORT}?ecosystem=npm"), Vec::new()),
        (
            format!("{IMPORT}?ecosystem=npm"),
            b"not a zip, only text".to_vec(),
        ),
        (
            format!("{IMPORT}?ecosystem=npm"),
            zip::write(&[("A-1.json", b"{broken")]),
        ),
    ] {
        let reply = post_zip(&fixture, &path, Auth::Admin, body).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{path}: {}",
            reply.text()
        );
    }
    assert_eq!(advisories(&fixture).await, 0);

    let archive = zip::write(&[
        ("GHSA-1.json", &record("GHSA-1")),
        ("GHSA-2.json", &record("GHSA-2")),
        ("../GHSA-3.json", &record("GHSA-3")),
    ]);
    let reply = post_zip(
        &fixture,
        &format!("{IMPORT}?ecosystem=npm"),
        Auth::Admin,
        archive,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let result = reply.json();
    assert_eq!(result["records"], 2);
    assert_eq!(result["changed"], 2);
    assert_eq!(result["skipped"], 1);
    assert_eq!(advisories(&fixture).await, 2);

    let feeds = fixture
        .call(Method::GET, FEEDS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(feeds[0]["ecosystem"], "npm");
    assert_eq!(feeds[0]["records"], 2);
    assert!(feeds[0]["syncedAt"].is_string());
    assert_eq!(feeds[0]["lastError"], Value::Null);

    let events = audit_event::Entity::find()
        .filter(sea_orm::ColumnTrait::eq(
            &audit_event::Column::Kind,
            "imported",
        ))
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].context, "npm");
}

#[tokio::test]
async fn the_sync_task_is_a_singleton_and_a_private_source_fails_it() {
    let fixture = Fixture::new().await;
    fixture.state.tasks().register(Arc::new(
        astrum_tasks::kinds::advisories::AdvisorySync::new(fixture.data.path().join("tmp")),
    ));
    let created = fixture
        .call(
            Method::POST,
            TASKS,
            Auth::Admin,
            Some(json!({
                "type": "advisories.sync", "name": "advisories",
                "frequency": {"schedule": "manual"},
                "properties": {"url": "http://127.0.0.1:9/osv", "ecosystems": "npm,PyPI"}
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    assert_eq!(created.json()["properties"]["staleAfterDays"], "7");
    let second = fixture
        .call(
            Method::POST,
            TASKS,
            Auth::Admin,
            Some(json!({"type": "advisories.sync", "name": "again", "frequency": {"schedule": "manual"}})),
        )
        .await;
    assert_eq!(second.status, StatusCode::BAD_REQUEST, "{}", second.text());

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
    let message = runs[0]["message"].as_str().unwrap();
    assert!(message.contains("2 ecosystems of 2 failed"), "{message}");
    let feeds = fixture
        .call(Method::GET, FEEDS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(feeds.as_array().unwrap().len(), 2);
    assert!(
        feeds[0]["lastError"].as_str().unwrap().contains("SSRF"),
        "{feeds}"
    );
    assert_eq!(advisories(&fixture).await, 0);
}

#[tokio::test]
async fn a_shrinking_dump_needs_force_and_a_running_import_answers_409() {
    let fixture = Fixture::new().await;
    let url = format!("{IMPORT}?ecosystem=npm");
    let many: Vec<(String, Vec<u8>)> = (0..120)
        .map(|n| (format!("R-{n}.json"), record(&format!("R-{n}"))))
        .collect();
    let listed: Vec<(&str, &[u8])> = many
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    let first = post_zip(&fixture, &url, Auth::Admin, zip::write(&listed)).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    let one = zip::write(&[("R-0.json", &record("R-0"))]);
    let refused = post_zip(&fixture, &url, Auth::Admin, one.clone()).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(refused.text().contains("force=true"), "{}", refused.text());
    assert_eq!(advisories(&fixture).await, 120);
    let bad = post_zip(
        &fixture,
        &format!("{url}&force=maybe"),
        Auth::Admin,
        one.clone(),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let forced = post_zip(
        &fixture,
        &format!("{url}&force=true"),
        Auth::Admin,
        one.clone(),
    )
    .await;
    assert_eq!(forced.status, StatusCode::OK, "{}", forced.text());
    assert_eq!(forced.json()["removed"], 119);

    let held = astrum_scan::osv::EcosystemLock::try_take(fixture.state.db().conn(), "npm")
        .await
        .unwrap()
        .unwrap();
    let busy = post_zip(&fixture, &url, Auth::Admin, one).await;
    assert_eq!(busy.status, StatusCode::CONFLICT, "{}", busy.text());
    drop(held);
}
