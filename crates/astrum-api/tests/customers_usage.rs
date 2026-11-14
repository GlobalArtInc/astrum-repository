mod shop;
mod support;

use astrum_security::customers::CustomerError;
use axum::http::{Method, StatusCode, header};
use serde_json::json;
use shop::{CUSTOMERS, customer, issue, shop, usage_row};
use support::{Auth, Fixture};
use uuid::Uuid;

async fn get(fixture: &Fixture, path: &str) -> support::Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn lines(text: &str) -> Vec<String> {
    text.split("\r\n")
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[tokio::test]
async fn usage_is_listed_per_day_and_repository_with_totals() {
    let fixture = shop().await;
    let (token, _) = fixture
        .customer_token("acme", &["files-a", "files-b"])
        .await;
    let (other, _) = fixture.customer_token("globex", &["files-a"]).await;
    usage_row(&fixture, token, "acme", "files-b", 0, 1, 10).await;
    usage_row(&fixture, token, "acme", "files-a", 0, 2, 20).await;
    usage_row(&fixture, token, "acme", "files-a", 3, 4, 40).await;
    usage_row(&fixture, token, "acme", "files-a", 40, 100, 1000).await;
    usage_row(&fixture, other, "globex", "files-a", 0, 9, 90).await;

    let reply = get(&fixture, &format!("{CUSTOMERS}/acme/usage")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["customer"], "acme");
    assert_eq!(body["days"], 30);
    assert_eq!(body["totals"], json!({"downloads": 7, "bytes": 70}));
    let daily = body["daily"].as_array().unwrap();
    let shape: Vec<(String, i64)> = daily
        .iter()
        .map(|row| {
            (
                row["repository"].as_str().unwrap().to_owned(),
                row["downloads"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        [
            ("files-a".into(), 4),
            ("files-a".into(), 2),
            ("files-b".into(), 1)
        ]
    );
    assert!(daily[0]["day"].as_str().unwrap() < daily[1]["day"].as_str().unwrap());

    let short = get(&fixture, &format!("{CUSTOMERS}/acme/usage?days=1")).await;
    assert_eq!(short.json()["totals"]["downloads"], 3);
    let long = get(&fixture, &format!("{CUSTOMERS}/acme/usage?days=730")).await;
    assert_eq!(long.json()["totals"]["downloads"], 107);
    for days in ["0", "731", "-1"] {
        let refused = get(&fixture, &format!("{CUSTOMERS}/acme/usage?days={days}")).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{days}");
        assert_eq!(refused.json()[0]["id"], "days");
    }
    let unknown = get(&fixture, &format!("{CUSTOMERS}/nobody/usage")).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_csv_export_has_one_safe_row_per_token_repository_and_day() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    customer(&fixture, "globex").await;
    let mut body = shop::issue_body("admin", &["files-a"]);
    body["name"] = json!("=1+1");
    let issued = issue(&fixture, Auth::Admin, "acme", body).await;
    let id: Uuid = issued.json()["id"].as_str().unwrap().parse().unwrap();
    let hint = issued.json()["hint"].as_str().unwrap().to_owned();
    usage_row(&fixture, id, "acme", "files-a", 1, 3, 300).await;
    usage_row(&fixture, Uuid::new_v4(), "acme", "-odd", 2, 1, 5).await;
    let (other, _) = fixture.customer_token("globex", &["files-a"]).await;
    usage_row(&fixture, other, "globex", "files-a", 1, 8, 80).await;

    let reply = get(&fixture, &format!("{CUSTOMERS}/usage.csv?customer=acme")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.headers[header::CONTENT_TYPE],
        "text/csv; charset=utf-8"
    );
    assert_eq!(
        reply.headers[header::CONTENT_DISPOSITION],
        "attachment; filename=\"customer-usage.csv\""
    );
    let rows = lines(&reply.text());
    assert_eq!(
        rows[0],
        "customer,token,tokenHint,repository,day,downloads,bytes"
    );
    assert_eq!(rows.len(), 3, "{rows:?}");
    let day =
        |ago: i64| (chrono::Utc::now().date_naive() - chrono::Duration::days(ago)).to_string();
    assert_eq!(rows[1], format!("acme,,,'-odd,{},1,5", day(2)));
    assert_eq!(
        rows[2],
        format!("acme,'=1+1,{hint},files-a,{},3,300", day(1))
    );

    let all = get(&fixture, &format!("{CUSTOMERS}/usage.csv?days=7")).await;
    assert_eq!(lines(&all.text()).len(), 4);
    assert!(all.text().contains("globex,"));
}

#[tokio::test]
async fn csv_parameters_are_validated() {
    let fixture = shop().await;
    for query in ["days=0", "days=731", "customer=Not%20A%20Slug"] {
        let reply = get(&fixture, &format!("{CUSTOMERS}/usage.csv?{query}")).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{query}");
    }
    let unknown = get(&fixture, &format!("{CUSTOMERS}/usage.csv?customer=nobody")).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    let empty = get(&fixture, &format!("{CUSTOMERS}/usage.csv")).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(lines(&empty.text()).len(), 1);
}

#[tokio::test]
async fn an_export_beyond_the_row_cap_is_refused() {
    let fixture = shop().await;
    let (token, _) = fixture.customer_token("acme", &["files-a"]).await;
    for ago in 0..3 {
        usage_row(&fixture, token, "acme", "files-a", ago, 1, 1).await;
    }
    let security = fixture.state.security();
    assert_eq!(security.usage_export(30, None, 3).await.unwrap().len(), 3);
    assert!(matches!(
        security.usage_export(30, None, 2).await,
        Err(CustomerError::TooManyRows(2))
    ));
    assert!(matches!(
        security.customer_usage("acme", 30, 2).await,
        Err(CustomerError::TooManyRows(2))
    ));
}
