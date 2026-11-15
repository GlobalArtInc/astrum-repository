#![allow(dead_code)]

use std::sync::Arc;

use astrum_db::entity::{repository, token_download_day};
use axum::http::{Method, StatusCode};
use chrono::{Days, Utc};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};

use crate::support::{Auth, Fixture, Reply};

pub const CUSTOMERS: &str = "/service/rest/v1/customers";

pub async fn shop() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    for name in ["files-a", "files-b", "files-c"] {
        raw_repository(&fixture, name).await;
    }
    group(&fixture, "files-all", &["files-a", "files-b"]).await;
    group(&fixture, "files-outer", &["files-all", "files-c"]).await;
    group(&fixture, "loop-one", &["files-a"]).await;
    group(&fixture, "loop-two", &["loop-one"]).await;
    close_loop(&fixture).await;
    fixture
}

pub async fn raw_repository(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let stored = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{name}/a.txt"),
            Auth::Admin,
            axum::body::Body::from(format!("{name} content")),
            &[],
        )
        .await;
    assert_eq!(stored.status, StatusCode::CREATED);
}

pub async fn group(fixture: &Fixture, name: &str, members: &[&str]) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/group",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": members}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn close_loop(fixture: &Fixture) {
    let row = repository::Entity::find_by_id("loop-one")
        .one(fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap();
    let mut changed: repository::ActiveModel = row.into();
    changed.attributes =
        ActiveValue::Set(json!({"group": {"memberNames": ["loop-two", "files-a"]}}));
    repository::Entity::update(changed)
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
}

pub async fn customer(fixture: &Fixture, id: &str) {
    let reply = fixture
        .call(
            Method::POST,
            CUSTOMERS,
            Auth::Admin,
            Some(json!({"id": id, "name": format!("{id} Ltd")})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

pub fn issue_body(owner: &str, repositories: &[&str]) -> Value {
    json!({"name": "release", "owner": owner, "repositories": repositories, "expiresInDays": 30})
}

pub async fn issue(fixture: &Fixture, auth: Auth<'_>, customer: &str, body: Value) -> Reply {
    fixture
        .call(
            Method::POST,
            &format!("{CUSTOMERS}/{customer}/tokens"),
            auth,
            Some(body),
        )
        .await
}

pub async fn usage_row(
    fixture: &Fixture,
    token_id: uuid::Uuid,
    customer: &str,
    repository: &str,
    days_ago: u64,
    downloads: i64,
    bytes: i64,
) {
    token_download_day::Entity::insert(token_download_day::ActiveModel {
        token_id: ActiveValue::Set(token_id),
        customer: ActiveValue::Set(Some(customer.to_owned())),
        repository: ActiveValue::Set(repository.to_owned()),
        day: ActiveValue::Set(
            Utc::now()
                .date_naive()
                .checked_sub_days(Days::new(days_ago))
                .unwrap(),
        ),
        downloads: ActiveValue::Set(downloads),
        bytes: ActiveValue::Set(bytes),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
}

pub async fn read_as(fixture: &Fixture, user: &str, repository: &str, token: &str) -> StatusCode {
    fixture
        .call(
            Method::GET,
            &format!("/repository/{repository}/a.txt"),
            Auth::Basic(user, token),
            None,
        )
        .await
        .status
}
