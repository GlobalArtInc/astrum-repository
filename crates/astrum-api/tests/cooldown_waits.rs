mod support;
mod ui_support;

use astrum_tasks::Expiry;
use astrum_tasks::kinds::cooldown::CooldownExpiry;
use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::json;
use support::{Auth, Fixture};
use ui_support::*;

const WAITS: &str = "/service/rest/v1/security/cooldown";
const SELECTORS: &str = "/service/rest/v1/security/content-selectors";
const SELECTOR_PRIVILEGES: &str =
    "/service/rest/v1/security/privileges/repository-content-selector";

async fn selected_only(fixture: &Fixture) {
    for (path, body) in [
        (
            SELECTORS,
            json!({"name": "first-only", "description": "", "expression": "format == \"raw\" && path =^ \"/first\""}),
        ),
        (
            SELECTOR_PRIVILEGES,
            json!({"name": "first-only-read", "description": "", "actions": ["READ", "BROWSE"], "format": "raw", "repository": "files", "contentSelector": "first-only"}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.text());
    }
    user_with(fixture, "selected", &["first-only-read"]).await;
}

#[tokio::test]
async fn anonymous_callers_are_refused() {
    let fixture = ready().await;
    let reply = get(&fixture, WAITS, Auth::None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{}", reply.text());
}

#[tokio::test]
async fn the_list_shows_only_repositories_the_caller_may_browse() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "private").await;
    waiting(&fixture, "files", vec!["alpha".into()], 3600.0, true).await;
    waiting(&fixture, "private", vec!["beta".into()], 3600.0, true).await;
    user_with(
        &fixture,
        "browser",
        &["as-repository-view-raw-files-browse"],
    )
    .await;
    fixture.add_user("nobody", "password-1", &[]).await;
    selected_only(&fixture).await;

    let seen = get(&fixture, WAITS, user("browser")).await;
    assert_eq!(seen.status, StatusCode::OK, "{}", seen.text());
    let items = seen.json()["items"].clone();
    assert_eq!(items.as_array().unwrap().len(), 1, "{items}");
    assert_eq!(items[0]["repository"], "files");
    assert_eq!(items[0]["name"], "alpha");
    assert_eq!(items[0]["blocked"], true);
    assert_eq!(items[0]["refusals"], 1);

    let narrowed = get(
        &fixture,
        &format!("{WAITS}?repository=private"),
        user("browser"),
    )
    .await;
    assert_eq!(narrowed.json()["items"], json!([]));
    let nobody = get(&fixture, WAITS, user("nobody")).await;
    assert_eq!(nobody.json()["items"], json!([]));
    let selector = get(&fixture, WAITS, user("selected")).await;
    assert_eq!(selector.json()["items"], json!([]));
}

#[tokio::test]
async fn the_list_pages_through_every_wait_with_the_continuation_token() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    let names: Vec<String> = (1..=60).map(|n| format!("pkg-{n:02}")).collect();
    waiting(&fixture, "files", names, 3600.0, false).await;
    user_with(
        &fixture,
        "browser",
        &["as-repository-view-raw-files-browse"],
    )
    .await;

    let first = get(&fixture, WAITS, user("browser")).await.json();
    let token = first["continuationToken"].as_str().unwrap().to_owned();
    let second = get(
        &fixture,
        &format!("{WAITS}?continuationToken={token}"),
        user("browser"),
    )
    .await
    .json();
    assert_eq!(first["items"].as_array().unwrap().len(), 50);
    assert_eq!(second["items"].as_array().unwrap().len(), 10);
    assert!(second["continuationToken"].is_null());
    let mut seen: Vec<String> = first["items"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["items"].as_array().unwrap())
        .map(|item| item["name"].as_str().unwrap().to_owned())
        .collect();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 60);
}

#[tokio::test]
async fn the_release_announces_each_due_blocked_wait_once() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    waiting(&fixture, "files", vec!["due".into()], -60.0, true).await;
    waiting(&fixture, "files", vec!["report".into()], -60.0, false).await;
    waiting(&fixture, "files", vec!["later".into()], 3600.0, true).await;
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            "INSERT INTO notification_channel (id, name, type, events) \
             VALUES (gen_random_uuid(), 'releases', 'webhook', ARRAY['cooldown.released'])",
        ))
        .await
        .unwrap();

    assert_eq!(
        CooldownExpiry
            .expire(fixture.state.content())
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        CooldownExpiry
            .expire(fixture.state.content())
            .await
            .unwrap(),
        0
    );

    let db = fixture.state.db().conn();
    let remaining = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*)::bigint AS n FROM cooldown_wait",
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remaining.try_get::<i64>("", "n").unwrap(), 1);
    let changes = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT repository, format, namespace, name, version, action, initiator \
             FROM notification_change WHERE action = 'cooldown.released' ORDER BY id",
        ))
        .await
        .unwrap();
    assert_eq!(changes.len(), 1);
    let change = &changes[0];
    assert_eq!(change.try_get::<String>("", "repository").unwrap(), "files");
    assert_eq!(change.try_get::<String>("", "format").unwrap(), "raw");
    assert_eq!(change.try_get::<String>("", "namespace").unwrap(), "");
    assert_eq!(change.try_get::<String>("", "name").unwrap(), "due");
    assert_eq!(change.try_get::<String>("", "version").unwrap(), "1.0");
    assert_eq!(
        change.try_get::<Option<String>>("", "initiator").unwrap(),
        None
    );
}
