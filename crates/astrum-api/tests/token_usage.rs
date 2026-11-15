mod support;

use std::sync::Arc;

use astrum_db::entity::token_download_day;
use astrum_security::{NewToken, Scope};
use axum::http::{Method, StatusCode, header};
use chrono::{Days, Utc};
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, QueryOrder, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BODY: &str = "hello world!";
const PATH: &str = "/repository/files/a.txt";

async fn shop() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": "files",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let stored = fixture
        .send_body(
            Method::PUT,
            PATH,
            Auth::Admin,
            axum::body::Body::from(BODY),
            &[],
        )
        .await;
    assert_eq!(stored.status, StatusCode::CREATED);
    fixture
}

async fn fetch(fixture: &Fixture, auth: Auth<'_>, headers: &[(&str, &str)]) -> Reply {
    let reply = fixture.send(Method::GET, PATH, auth, None, headers).await;
    assert!(reply.status.is_success(), "{}", reply.text());
    reply
}

async fn rows(fixture: &Fixture) -> Vec<token_download_day::Model> {
    fixture.state.content().flush_downloads().await.unwrap();
    token_download_day::Entity::find()
        .order_by_asc(token_download_day::Column::TokenId)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn sql(fixture: &Fixture, statement: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, statement))
        .await
        .unwrap();
}

async fn set_days_kept(fixture: &Fixture, value: Value) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        "customers.usage.daysKept",
        &value,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn full_requests_count_downloads_and_ranged_resumes_count_only_bytes() {
    let fixture = shop().await;
    let (id, token) = fixture.customer_token("acme", &["files"]).await;
    let auth = || Auth::Basic("acme", &token);
    let first = fetch(&fixture, auth(), &[]).await;
    let second = fetch(&fixture, auth(), &[]).await;
    let resumed = fetch(&fixture, auth(), &[("range", "bytes=5-")]).await;
    assert_eq!(resumed.status, StatusCode::PARTIAL_CONTENT);
    let sent = first.body.len() + second.body.len() + resumed.body.len();
    let rows = rows(&fixture).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token_id, id);
    assert_eq!(rows[0].repository, "files");
    assert_eq!(rows[0].downloads, 2);
    assert_eq!(rows[0].bytes, sent as i64);
    assert_eq!(sent, BODY.len() * 2 + BODY.len() - 5);
}

#[tokio::test]
async fn heads_not_modified_and_unsatisfiable_ranges_count_nothing() {
    let fixture = shop().await;
    let (_, token) = fixture.customer_token("acme", &["files"]).await;
    let auth = || Auth::Basic("acme", &token);
    let head = fixture.send(Method::HEAD, PATH, auth(), None, &[]).await;
    assert_eq!(head.status, StatusCode::OK);
    let etag = head.headers.get(header::ETAG).unwrap().to_str().unwrap();
    let unchanged = fixture
        .send(Method::GET, PATH, auth(), None, &[("if-none-match", etag)])
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    let past_the_end = fixture
        .send(Method::GET, PATH, auth(), None, &[("range", "bytes=100-")])
        .await;
    assert_eq!(past_the_end.status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert!(rows(&fixture).await.is_empty());
    fetch(&fixture, auth(), &[]).await;
    let rows = rows(&fixture).await;
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].downloads, rows[0].bytes), (1, BODY.len() as i64));
}

#[tokio::test]
async fn anonymous_and_session_callers_write_no_row() {
    let fixture = shop().await;
    let enabled = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(enabled.status, StatusCode::OK);
    fetch(&fixture, Auth::None, &[]).await;
    fetch(&fixture, Auth::Admin, &[]).await;
    let signed_in = fixture
        .call(
            Method::POST,
            "/service/rest/internal/session",
            Auth::None,
            Some(json!({"username": "admin", "password": fixture.admin_password})),
        )
        .await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.text());
    let session = signed_in
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|value| {
            value
                .split(';')
                .next()
                .filter(|pair| pair.starts_with("astrum_session="))
        })
        .unwrap()
        .to_owned();
    fetch(&fixture, Auth::None, &[("cookie", &session)]).await;
    assert!(rows(&fixture).await.is_empty());
}

#[tokio::test]
async fn customer_tokens_carry_their_customer_and_personal_tokens_none() {
    let fixture = shop().await;
    let (customer_id, customer_token) = fixture.customer_token("acme", &["files"]).await;
    let (info, personal) = fixture
        .state
        .security()
        .create_token("admin", NewToken::personal("ci", Scope::Read, Some(5)))
        .await
        .unwrap();
    fetch(&fixture, Auth::Basic("acme", &customer_token), &[]).await;
    fetch(&fixture, Auth::Basic("admin", &personal), &[]).await;
    let rows = rows(&fixture).await;
    assert_eq!(rows.len(), 2);
    let of = |id| rows.iter().find(|row| row.token_id == id).unwrap();
    assert_eq!(of(customer_id).customer.as_deref(), Some("acme"));
    assert_eq!(of(info.id).customer, None);
}

#[tokio::test]
async fn a_flush_adds_each_download_once() {
    let fixture = shop().await;
    let (_, token) = fixture.customer_token("acme", &["files"]).await;
    fetch(&fixture, Auth::Basic("acme", &token), &[]).await;
    let once = rows(&fixture).await;
    assert_eq!(rows(&fixture).await, once);
    assert_eq!((once[0].downloads, once[0].bytes), (1, BODY.len() as i64));
    fetch(&fixture, Auth::Basic("acme", &token), &[]).await;
    let twice = rows(&fixture).await;
    assert_eq!(twice.len(), 1);
    assert_eq!(
        (twice[0].downloads, twice[0].bytes),
        (2, 2 * BODY.len() as i64)
    );
}

#[tokio::test]
async fn usage_of_a_deleted_customer_does_not_fail_the_batch() {
    let fixture = shop().await;
    let (_, gone) = fixture.customer_token("acme", &["files"]).await;
    let (kept_id, kept) = fixture.customer_token("globex", &["files"]).await;
    fetch(&fixture, Auth::Basic("acme", &gone), &[]).await;
    fetch(&fixture, Auth::Basic("globex", &kept), &[]).await;
    sql(&fixture, "DELETE FROM customer WHERE id = 'acme'").await;
    let rows = rows(&fixture).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token_id, kept_id);
    assert_eq!(rows[0].customer.as_deref(), Some("globex"));
}

#[tokio::test]
async fn old_days_are_pruned_by_the_configured_number_of_days() {
    let fixture = shop().await;
    let content = fixture.state.content();
    let day = |offset: u64| {
        Utc::now()
            .date_naive()
            .checked_sub_days(Days::new(offset))
            .unwrap()
            .to_string()
    };
    for offset in [729, 730, 9, 10] {
        sql(
            &fixture,
            &format!(
                "INSERT INTO token_download_day (token_id, customer, repository, day, downloads, bytes) \
                 VALUES (gen_random_uuid(), NULL, 'r{offset}', DATE '{}', 1, 1)",
                day(offset)
            ),
        )
        .await;
    }
    let remaining = |rows: Vec<token_download_day::Model>| {
        let mut names: Vec<String> = rows.into_iter().map(|row| row.repository).collect();
        names.sort();
        names
    };
    let all = || async { remaining(rows(&fixture).await) };
    assert_eq!(content.prune_token_download_days().await.unwrap(), 1);
    assert_eq!(all().await, ["r10", "r729", "r9"]);

    set_days_kept(&fixture, json!(10)).await;
    assert_eq!(content.prune_token_download_days().await.unwrap(), 2);
    assert_eq!(all().await, ["r9"]);

    sql(
        &fixture,
        &format!(
            "INSERT INTO token_download_day (token_id, customer, repository, day, downloads, bytes) \
             VALUES (gen_random_uuid(), NULL, 'r730', DATE '{}', 1, 1), \
                    (gen_random_uuid(), NULL, 'r729', DATE '{}', 1, 1)",
            day(730),
            day(729)
        ),
    )
    .await;
    set_days_kept(&fixture, json!("many")).await;
    assert_eq!(content.prune_token_download_days().await.unwrap(), 1);
    assert_eq!(all().await, ["r729", "r9"]);
}
