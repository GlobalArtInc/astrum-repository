mod accounts;
mod shop;
mod support;

use accounts::{create_role, create_user, error_ids};
use astrum_api::routes::customers::CUSTOMERS;
use astrum_db::entity::access_token;
use axum::http::{Method, StatusCode, header};
use chrono::{Duration, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use shop::{customer, issue, issue_body, shop};
use support::{Auth, Fixture};

async fn issuer(fixture: &Fixture, repositories: &[&str]) {
    let mut privileges = vec!["as-customers-create".to_owned()];
    privileges.extend(
        repositories
            .iter()
            .map(|name| format!("as-repository-view-raw-{name}-read")),
    );
    let privileges: Vec<&str> = privileges.iter().map(String::as_str).collect();
    create_role(fixture, "issuers", &privileges, &[]).await;
    create_user(fixture, "isabel", "correct horse", &["issuers"]).await;
}

fn isabel() -> Auth<'static> {
    Auth::Basic("isabel", "correct horse")
}

async fn read(fixture: &Fixture, repository: &str, token: &str) -> StatusCode {
    shop::read_as(fixture, "acme", repository, token).await
}

#[tokio::test]
async fn an_issued_token_reads_the_listed_repositories_and_is_shown_once() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let reply = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(reply.headers[header::CACHE_CONTROL], "no-store");
    let shown = reply.json();
    let token = shown["token"].as_str().unwrap().to_owned();
    assert_eq!(shown["owner"], "admin");
    assert_eq!(shown["repositories"], json!(["files-a"]));
    assert_eq!(shown["hint"], token[..11]);
    assert_eq!(read(&fixture, "files-a", &token).await, StatusCode::OK);
    assert_eq!(
        read(&fixture, "files-b", &token).await,
        StatusCode::FORBIDDEN
    );

    let listed = fixture
        .call(
            Method::GET,
            &format!("{CUSTOMERS}/acme/tokens"),
            Auth::Admin,
            None,
        )
        .await;
    assert!(!listed.text().contains(&token));
    let item = &listed.json()[0];
    assert_eq!(item["owner"], "admin");
    assert_eq!(item["createdBy"], "admin");
    assert_eq!(item["repositories"], json!(["files-a"]));
    assert!(item.get("token").is_none());
}

#[tokio::test]
async fn bad_requests_for_a_token_are_refused_with_the_field() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    fixture.add_user("locked", "correct horse", &[]).await;
    fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/users/locked",
            Auth::Admin,
            Some(json!({"userId": "locked", "firstName": "L", "lastName": "L", "emailAddress": "l@example.org", "source": "default", "status": "disabled", "roles": []})),
        )
        .await;
    let attempt = |body: Value| issue(&fixture, Auth::Admin, "acme", body);
    let unknown = attempt(issue_body("nobody-here", &["files-a"])).await;
    let disabled = attempt(issue_body("locked", &["files-a"])).await;
    let anonymous = attempt(issue_body("anonymous", &["files-a"])).await;
    for reply in [&unknown, &disabled, &anonymous] {
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert_eq!(error_ids(reply), ["owner"]);
    }
    assert_eq!(unknown.text(), disabled.text());

    let mut cases: Vec<(Value, &str)> = vec![
        (issue_body("admin", &[]), "repositories"),
        (issue_body("admin", &["no-such-repo"]), "repositories"),
        (issue_body("admin", &["files-*"]), "repositories"),
        (issue_body("admin", &["*"]), "repositories"),
        (issue_body("admin", &["files-a", "FILES-A"]), "repositories"),
    ];
    let mut no_expiry = issue_body("admin", &["files-a"]);
    no_expiry["expiresInDays"] = Value::Null;
    cases.push((no_expiry, "expiresInDays"));
    let mut too_long = issue_body("admin", &["files-a"]);
    too_long["expiresInDays"] = json!(3651);
    cases.push((too_long, "expiresInDays"));
    for (body, field) in cases {
        let reply = attempt(body.clone()).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(error_ids(&reply), [field], "{body}");
    }
    let missing = issue(
        &fixture,
        Auth::Admin,
        "nobody",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_customer_holds_at_most_twenty_tokens() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    for number in 0..20 {
        let reply = issue(
            &fixture,
            Auth::Admin,
            "acme",
            issue_body("admin", &["files-a"]),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{number}: {}",
            reply.text()
        );
    }
    let refused = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
}

#[tokio::test]
async fn expired_tokens_do_not_count_against_the_limit() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let mut ids = Vec::new();
    for _ in 0..20 {
        let reply = issue(
            &fixture,
            Auth::Admin,
            "acme",
            issue_body("admin", &["files-a"]),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        ids.push(uuid::Uuid::parse_str(reply.json()["id"].as_str().unwrap()).unwrap());
    }
    access_token::Entity::update_many()
        .col_expr(
            access_token::Column::ExpiresAt,
            Expr::value((Utc::now() - Duration::days(1)).fixed_offset()),
        )
        .filter(access_token::Column::Id.is_in(ids[..5].to_vec()))
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    let listed = fixture
        .call(Method::GET, &format!("{CUSTOMERS}/acme"), Auth::Admin, None)
        .await;
    assert_eq!(listed.json()["tokens"], 15);
    for number in 0..5 {
        let reply = issue(
            &fixture,
            Auth::Admin,
            "acme",
            issue_body("admin", &["files-a"]),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{number}: {}",
            reply.text()
        );
    }
    let refused = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
}

#[tokio::test]
async fn a_disabled_customer_gets_no_token_until_it_is_enabled() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let path = format!("{CUSTOMERS}/acme");
    let put = |disabled: bool| {
        fixture.call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "acme", "reference": "", "note": "", "disabled": disabled})),
        )
    };
    assert_eq!(put(true).await.status, StatusCode::OK);
    let refused = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert_eq!(error_ids(&refused), ["customer"]);
    assert_eq!(
        refused.json()[0]["message"],
        "enable the customer before issuing tokens"
    );
    assert_eq!(put(false).await.status, StatusCode::OK);
    let allowed = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
}

#[tokio::test]
async fn the_issuer_must_read_every_listed_repository() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    issuer(&fixture, &["files-a"]).await;
    let allowed = issue(
        &fixture,
        isabel(),
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
    let refused = issue(
        &fixture,
        isabel(),
        "acme",
        issue_body("admin", &["files-a", "files-b"]),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert!(!refused.text().contains("files-b"), "{}", refused.text());
}

#[tokio::test]
async fn a_listed_group_needs_read_on_every_member() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    issuer(
        &fixture,
        &[
            "files-all",
            "files-outer",
            "files-a",
            "files-c",
            "loop-one",
            "loop-two",
        ],
    )
    .await;
    for listed in ["files-all", "files-outer"] {
        let refused = issue(&fixture, isabel(), "acme", issue_body("admin", &[listed])).await;
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{listed}");
        assert!(
            refused
                .text()
                .contains("everything a listed group contains")
        );
        assert!(!refused.text().contains("files-b"), "{}", refused.text());
    }
    let looped = issue(
        &fixture,
        isabel(),
        "acme",
        issue_body("admin", &["loop-one"]),
    )
    .await;
    assert_eq!(looped.status, StatusCode::CREATED, "{}", looped.text());

    create_role(
        &fixture,
        "member-b",
        &["as-repository-view-raw-files-b-read"],
        &["issuers"],
    )
    .await;
    create_user(&fixture, "ivan", "correct horse", &["member-b"]).await;
    let allowed = issue(
        &fixture,
        Auth::Basic("ivan", "correct horse"),
        "acme",
        issue_body("admin", &["files-outer"]),
    )
    .await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
}

#[tokio::test]
async fn a_token_limited_to_repositories_issues_nothing() {
    let fixture = shop().await;
    let (_, token) = fixture.customer_token("acme", &["files-a"]).await;
    let reply = issue(
        &fixture,
        Auth::Basic("acme", &token),
        "acme",
        issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
}
