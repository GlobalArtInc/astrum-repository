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
    for index in 0..55 {
        put(
            &fixture,
            &format!("/repository/many/n/file-{:02}.txt", 54 - index),
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
    assert_eq!(first["items"].as_array().unwrap().len(), 50);
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
    let expected: Vec<String> = (0..55).map(|n| format!("/n/file-{n:02}.txt")).collect();
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

#[tokio::test]
async fn keywords_match_whole_words_whatever_separates_them() {
    let fixture = catalogue().await;
    put(
        &fixture,
        "/repository/files/libs/report-builder-0.4.2.tar.gz",
        "report",
    )
    .await;
    for query in [
        "q=report_builder",
        "q=Report.Builder",
        "q=REPORT-BUILDER",
        "q=report-b",
        "q=builder%200.4",
        "q=nothing%20report_builder",
    ] {
        assert_eq!(
            names(&fixture, query).await,
            ["report-builder-0.4.2.tar.gz"],
            "{query}"
        );
    }
    for query in ["q=rep-builder", "q=builder-report", "q=report_jobs"] {
        assert_eq!(
            names(&fixture, query).await,
            Vec::<String>::new(),
            "{query}"
        );
    }
    let mut either = names(&fixture, "q=beta%20delta").await;
    either.sort();
    assert_eq!(
        either,
        ["beta.txt", "delta.txt"],
        "words separated by spaces are alternatives"
    );
}

#[tokio::test]
async fn strangers_must_sign_in_to_search_while_anonymous_access_is_off() {
    let fixture = catalogue().await;
    for path in [
        "/service/rest/v1/search?name=*",
        "/service/rest/v1/search/assets?name=*",
        "/service/rest/v1/search/assets/download?name=beta.txt",
    ] {
        let reply = fixture.call(Method::GET, path, Auth::None, None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
        assert!(reply.headers.contains_key("www-authenticate"), "{path}");
    }
    let enabled = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(serde_json::json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert!(enabled.status.is_success(), "{}", enabled.text());
    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?name=*",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

#[tokio::test]
async fn a_group_lists_the_content_of_its_members_under_its_own_name() {
    let fixture = catalogue().await;
    raw_group(&fixture, "inner", &["files"]).await;
    raw_group(&fixture, "outer", &["inner", "archive"]).await;
    let assets = get(
        &fixture,
        "/service/rest/v1/assets?repository=outer",
        Auth::Admin,
    )
    .await;
    assert_eq!(assets.status, StatusCode::OK, "{}", assets.text());
    let items = assets.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 4);
    for item in &items {
        assert_eq!(item["repository"], "outer");
        let url = item["downloadUrl"].as_str().unwrap();
        assert!(url.contains("/repository/outer/"), "{url}");
        let one = get(
            &fixture,
            &format!("/service/rest/v1/assets/{}", item["id"].as_str().unwrap()),
            Auth::Admin,
        )
        .await;
        assert_eq!(one.status, StatusCode::OK, "{}", one.text());
        assert_eq!(one.json()["repository"], "outer");
    }
    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=outer",
        Auth::Admin,
    )
    .await
    .json();
    let listed = components["items"].as_array().unwrap();
    assert_eq!(listed.len(), 4);
    let id = listed[0]["id"].as_str().unwrap();
    let component = get(
        &fixture,
        &format!("/service/rest/v1/components/{id}"),
        Auth::Admin,
    )
    .await;
    assert_eq!(component.status, StatusCode::OK, "{}", component.text());
    assert_eq!(component.json()["repository"], "outer");
    assert_eq!(component.json()["assets"][0]["repository"], "outer");
    let refused = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::NOT_FOUND);
    let still = get(
        &fixture,
        "/service/rest/v1/assets?repository=outer",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(still["items"].as_array().unwrap().len(), 4);
    let mut found: Vec<String> = names(&fixture, "repository=outer").await;
    found.sort();
    assert_eq!(
        found,
        ["Alpha.txt", "beta.txt", "delta.txt", "gamma*star.txt"]
    );
    let searched = get(
        &fixture,
        "/service/rest/v1/search/assets?repository=inner",
        Auth::Admin,
    )
    .await
    .json();
    let repositories: Vec<&str> = searched["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["repository"].as_str().unwrap())
        .collect();
    assert_eq!(repositories, ["files", "files"]);
    let empty = get(
        &fixture,
        "/service/rest/v1/assets?repository=inner",
        Auth::Admin,
    )
    .await;
    assert_eq!(empty.json()["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn a_reader_of_a_group_lists_its_members_through_it_but_searches_only_what_it_may_browse() {
    let fixture = catalogue().await;
    raw_group(&fixture, "everything", &["files", "archive"]).await;
    raw_hosted(&fixture, "void").await;
    raw_group(&fixture, "nothing", &["void"]).await;
    user_with(
        &fixture,
        "reader",
        &[
            "as-repository-view-raw-everything-browse",
            "as-repository-view-raw-nothing-browse",
        ],
    )
    .await;
    let listed = get(
        &fixture,
        "/service/rest/v1/assets?repository=everything",
        user("reader"),
    )
    .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    assert_eq!(listed.json()["items"].as_array().unwrap().len(), 4);
    let empty = get(
        &fixture,
        "/service/rest/v1/components?repository=nothing",
        user("reader"),
    )
    .await;
    assert_eq!(empty.status, StatusCode::OK, "{}", empty.text());
    assert_eq!(empty.json()["items"].as_array().unwrap().len(), 0);
    let member = get(
        &fixture,
        "/service/rest/v1/assets?repository=files",
        user("reader"),
    )
    .await;
    assert_eq!(member.status, StatusCode::NOT_FOUND);
    let searched = get(
        &fixture,
        "/service/rest/v1/search?repository=everything",
        user("reader"),
    )
    .await;
    assert_eq!(searched.status, StatusCode::OK, "{}", searched.text());
    assert_eq!(searched.json()["items"].as_array().unwrap().len(), 0);
}
