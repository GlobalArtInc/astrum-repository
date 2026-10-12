mod support;
mod ui_support;

use astrum_db::entity::audit_event;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture};
use ui_support::*;

const AUDIT: &str = "/service/rest/internal/audit";

async fn event(
    fixture: &Fixture,
    days_ago: i64,
    domain: &str,
    kind: &str,
    context: &str,
    initiator: &str,
    attributes: Value,
) {
    audit_event::Entity::insert(audit_event::ActiveModel {
        id: ActiveValue::NotSet,
        at: ActiveValue::Set((Utc::now() - Duration::days(days_ago)).fixed_offset()),
        domain: ActiveValue::Set(domain.to_owned()),
        kind: ActiveValue::Set(kind.to_owned()),
        context: ActiveValue::Set(context.to_owned()),
        initiator: ActiveValue::Set(initiator.to_owned()),
        attributes: ActiveValue::Set(attributes),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
}

async fn history() -> Fixture {
    let fixture = ready().await;
    event(
        &fixture,
        10,
        "repository",
        "created",
        "files",
        "admin",
        json!({}),
    )
    .await;
    event(
        &fixture,
        5,
        "security.user",
        "created",
        "jane",
        "admin",
        json!({}),
    )
    .await;
    event(
        &fixture,
        3,
        "asset",
        "deleted",
        "/a/b.txt",
        "jane",
        json!({"repository": "files"}),
    )
    .await;
    event(
        &fixture,
        2,
        "repository",
        "updated",
        "files",
        "Jane_Ops",
        json!({}),
    )
    .await;
    event(
        &fixture,
        1,
        "repository",
        "updated",
        "other",
        "admin",
        json!({}),
    )
    .await;
    event(
        &fixture,
        0,
        "asset",
        "deleted",
        "/x.txt",
        "admin",
        json!({"repository": "other"}),
    )
    .await;
    fixture
}

async fn contexts(fixture: &Fixture, query: &str) -> Vec<String> {
    let reply = get(fixture, &format!("{AUDIT}?{query}"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{query}: {}", reply.text());
    names(&reply.json(), "context")
}

#[tokio::test]
async fn audit_events_filter_by_every_field() {
    let fixture = history().await;
    assert_eq!(contexts(&fixture, "").await.len(), 6);
    assert_eq!(
        contexts(&fixture, "q=JANE").await,
        ["files", "/a/b.txt", "jane"]
    );
    assert_eq!(contexts(&fixture, "q=b.t").await, ["/a/b.txt"]);
    assert_eq!(contexts(&fixture, "q=100%25").await, Vec::<String>::new());
    assert_eq!(contexts(&fixture, "initiator=jane").await, ["/a/b.txt"]);
    assert_eq!(contexts(&fixture, "kind=updated").await, ["other", "files"]);
    assert_eq!(
        contexts(&fixture, "type=created&domain=repository").await,
        ["files"]
    );
    assert_eq!(
        contexts(&fixture, "context=files").await,
        ["files", "files"]
    );
    assert_eq!(
        contexts(&fixture, "repository=files").await,
        ["files", "/a/b.txt", "files"]
    );
    let since = (Utc::now() - Duration::days(4)).date_naive();
    let until = (Utc::now() - Duration::days(2)).date_naive();
    assert_eq!(
        contexts(&fixture, &format!("since={since}&until={until}")).await,
        ["files", "/a/b.txt"]
    );
    let moment =
        (Utc::now() - Duration::hours(36)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    assert_eq!(
        contexts(&fixture, &format!("since={moment}")).await,
        ["/x.txt", "other"]
    );
    let first = get(&fixture, &format!("{AUDIT}?limit=2"), Auth::Admin)
        .await
        .json();
    assert_eq!(names(&first, "context"), ["/x.txt", "other"]);
    let before = first[1]["id"].as_i64().unwrap();
    assert_eq!(
        contexts(
            &fixture,
            &format!("limit=2&before={before}&domain=repository")
        )
        .await,
        ["files", "files"]
    );
}

#[tokio::test]
async fn audit_filters_refuse_bad_dates_and_need_audit_read() {
    let fixture = history().await;
    let reply = get(&fixture, &format!("{AUDIT}?since=yesterday"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(names(&reply.json(), "id"), ["since"]);
    let reply = get(&fixture, &format!("{AUDIT}?until=2026-13-01"), Auth::Admin).await;
    assert_eq!(names(&reply.json(), "id"), ["until"]);
    user_with(&fixture, "auditor", &["as-audit-read"]).await;
    assert_eq!(
        get(&fixture, &format!("{AUDIT}?q=files"), user("auditor"))
            .await
            .status,
        StatusCode::OK
    );
    fixture.add_user("nobody", "password-1", &[]).await;
    assert_eq!(
        get(&fixture, AUDIT, user("nobody")).await.status,
        StatusCode::FORBIDDEN
    );
}
