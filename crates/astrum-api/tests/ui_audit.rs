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

#[tokio::test]
async fn audit_facets_list_the_domains_types_and_initiators_in_use() {
    let fixture = history().await;
    let reply = get(&fixture, &format!("{UI}/audit/facets"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!({
            "domains": ["asset", "repository", "security.user"],
            "types": ["created", "deleted", "updated"],
            "initiators": ["admin", "jane", "Jane_Ops"]
        })
    );
    fixture.add_user("nobody", "password-1", &[]).await;
    assert_eq!(
        get(&fixture, &format!("{UI}/audit/facets"), user("nobody"))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&fixture, &format!("{UI}/audit/facets"), Auth::None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

async fn log(fixture: &Fixture, query: &str) -> Value {
    let reply = get(fixture, &format!("{UI}/audit-log?{query}"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{query}: {}", reply.text());
    reply.json()
}

fn log_contexts(reply: &Value) -> Vec<String> {
    names(&reply["items"], "context")
}

#[tokio::test]
async fn audit_log_pages_and_filters_by_category_type_and_text() {
    let fixture = history().await;
    event(
        &fixture,
        0,
        "security.session",
        "created",
        "jane",
        "jane/10.0.0.1",
        json!({}),
    )
    .await;
    event(
        &fixture,
        0,
        "security.session",
        "failed",
        "bob",
        "*UNKNOWN/10.0.0.2",
        json!({}),
    )
    .await;
    event(
        &fixture,
        0,
        "security.contentselector",
        "created",
        "sel",
        "admin",
        json!({}),
    )
    .await;

    let all = log(&fixture, "limit=4&page=1").await;
    assert_eq!(
        all["pagination"],
        json!({"totalItems": 9, "totalPages": 3, "currentPage": 1, "itemsPerPage": 4})
    );
    assert_eq!(log_contexts(&all), ["sel", "bob", "jane", "/x.txt"]);
    assert!(all["items"][0]["timestamp"].as_str().unwrap().contains('T'));
    let last = log(&fixture, "limit=4&page=3").await;
    assert_eq!(log_contexts(&last), ["files"]);

    assert_eq!(
        log_contexts(&log(&fixture, "categories=security").await),
        ["bob", "jane", "jane"]
    );
    assert_eq!(
        log_contexts(&log(&fixture, "categories=repository").await),
        ["/x.txt", "other", "files", "/a/b.txt", "files"]
    );
    assert_eq!(
        log_contexts(&log(&fixture, "categories=configuration").await),
        ["sel"]
    );
    assert_eq!(
        log_contexts(&log(&fixture, "categories=protection").await),
        Vec::<String>::new()
    );
    assert_eq!(
        log_contexts(&log(&fixture, "categories=configuration,security").await).len(),
        4
    );

    assert_eq!(log_contexts(&log(&fixture, "types=login").await), ["jane"]);
    assert_eq!(
        log_contexts(&log(&fixture, "types=created").await),
        ["sel", "jane", "files"]
    );
    assert_eq!(log_contexts(&log(&fixture, "types=failed").await), ["bob"]);
    assert_eq!(
        log_contexts(&log(&fixture, "types=updated,deleted").await),
        ["/x.txt", "other", "files", "/a/b.txt"]
    );

    assert_eq!(
        log_contexts(&log(&fixture, "repository=files").await),
        ["files", "/a/b.txt", "files"]
    );
    assert_eq!(
        log_contexts(&log(&fixture, "initiator=JANE").await),
        ["jane", "files", "/a/b.txt"]
    );
    assert_eq!(log_contexts(&log(&fixture, "q=b.t").await), ["/a/b.txt"]);
    let since = (Utc::now() - Duration::days(4)).to_rfc3339();
    let until = (Utc::now() - Duration::days(2)).to_rfc3339();
    assert_eq!(
        log_contexts(
            &log(
                &fixture,
                &format!(
                    "startDate={}&endDate={}",
                    urlencoding(&since),
                    urlencoding(&until)
                )
            )
            .await
        ),
        ["files", "/a/b.txt"]
    );
    let reply = get(
        &fixture,
        &format!("{UI}/audit-log?startDate=yesterday"),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    fixture.add_user("nobody", "password-1", &[]).await;
    assert_eq!(
        get(&fixture, &format!("{UI}/audit-log"), user("nobody"))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

fn urlencoding(value: &str) -> String {
    value.replace('+', "%2B").replace(':', "%3A")
}
