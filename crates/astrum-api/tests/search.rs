mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::Value;
use support::{Auth, Fixture};
use ui_support::*;

async fn catalogue() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "archive").await;
    put(&fixture, "/repository/files/tools/beta.txt", "beta").await;
    put(&fixture, "/repository/archive/tools/Alpha.txt", "alpha").await;
    put(&fixture, "/repository/files/docs/gamma*star.txt", "gamma").await;
    put(&fixture, "/repository/archive/docs/delta.txt", "delta").await;
    fixture
}

async fn names(fixture: &Fixture, query: &str) -> Vec<String> {
    let reply = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/search?{query}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            let name = item["name"].as_str().unwrap_or_default();
            name.rsplit('/').next().unwrap_or_default().to_owned()
        })
        .collect()
}

#[tokio::test]
async fn components_sort_by_name_group_and_repository_in_both_directions() {
    let fixture = catalogue().await;
    assert_eq!(
        names(&fixture, "sort=name").await,
        ["delta.txt", "gamma*star.txt", "Alpha.txt", "beta.txt"]
    );
    assert_eq!(
        names(&fixture, "sort=name&direction=desc").await,
        ["beta.txt", "Alpha.txt", "gamma*star.txt", "delta.txt"]
    );
    assert_eq!(
        names(&fixture, "sort=repository").await,
        ["Alpha.txt", "delta.txt", "beta.txt", "gamma*star.txt"]
    );
    assert_eq!(
        names(&fixture, "sort=group&direction=desc").await,
        ["Alpha.txt", "beta.txt", "delta.txt", "gamma*star.txt"]
    );
}

#[tokio::test]
async fn components_without_a_sort_come_in_the_order_they_were_added() {
    let fixture = catalogue().await;
    assert_eq!(
        names(&fixture, "").await,
        ["beta.txt", "Alpha.txt", "gamma*star.txt", "delta.txt"]
    );
    assert_eq!(
        names(&fixture, "direction=desc").await,
        ["delta.txt", "gamma*star.txt", "Alpha.txt", "beta.txt"]
    );
}

#[tokio::test]
async fn sorted_pages_continue_where_the_last_one_stopped() {
    let fixture = ready().await;
    raw_hosted(&fixture, "many").await;
    for index in 0..13 {
        put(
            &fixture,
            &format!("/repository/many/n/file-{:02}.txt", 12 - index),
            "x",
        )
        .await;
    }
    let first = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?sort=name",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let token = first["continuationToken"].as_str().unwrap().to_owned();
    let second = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/search?sort=name&continuationToken={token}"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    let all: Vec<String> = [first, second]
        .iter()
        .flat_map(|page| page["items"].as_array().unwrap().clone())
        .map(|item: Value| item["name"].as_str().unwrap().to_owned())
        .collect();
    let expected: Vec<String> = (0..13).map(|n| format!("/n/file-{n:02}.txt")).collect();
    assert_eq!(all, expected);
}

#[tokio::test]
async fn keywords_take_an_asterisk_for_any_characters() {
    let fixture = catalogue().await;
    assert_eq!(names(&fixture, "q=al*a&sort=name").await, ["Alpha.txt"]);
    assert_eq!(
        names(&fixture, "q=tools*txt&sort=name").await,
        ["Alpha.txt", "beta.txt"]
    );
    assert_eq!(names(&fixture, "q=a%25a").await, Vec::<String>::new());
}
