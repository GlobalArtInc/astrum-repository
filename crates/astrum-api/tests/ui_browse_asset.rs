mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const BROWSE: &str = "/service/rest/internal/ui/browse";

async fn details(fixture: &Fixture, repository: &str, path: &str, auth: Auth<'_>) -> Reply {
    get(
        fixture,
        &format!("{BROWSE}/{repository}/asset?path={path}"),
        auth,
    )
    .await
}

async fn shared() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "first").await;
    raw_hosted(&fixture, "second").await;
    put(&fixture, "/repository/first/docs/same.txt", "first").await;
    put(
        &fixture,
        "/repository/second/docs/same.txt",
        "second-longer",
    )
    .await;
    put(&fixture, "/repository/second/docs/only-second.txt", "2").await;
    raw_group(&fixture, "both", &["first", "second"]).await;
    fixture
}

#[tokio::test]
async fn hosted_assets_answer_with_their_details() {
    let fixture = ready().await;
    maven_hosted(&fixture, "releases").await;
    deploy(&fixture, "releases", "org.acme", "lib", "1.0").await;
    let reply = details(
        &fixture,
        "releases",
        "org/acme/lib/1.0/lib-1.0.jar",
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let asset = reply.json();
    assert_eq!(asset["path"], "/org/acme/lib/1.0/lib-1.0.jar");
    assert_eq!(asset["repository"], "releases");
    assert_eq!(asset["browsedRepository"], "releases");
    assert_eq!(asset["format"], "maven2");
    assert_eq!(asset["fileSize"], JAR.len());
    assert_eq!(asset["uploader"], "admin");
    assert_eq!(asset["downloadCount"], 0);
    assert!(asset["checksum"]["sha1"].as_str().unwrap().len() == 40);
    assert!(asset["checksum"]["sha256"].as_str().unwrap().len() == 64);
    assert!(asset["blobCreated"].is_string());
    assert_eq!(
        asset["downloadUrl"],
        "http://localhost/repository/releases/org/acme/lib/1.0/lib-1.0.jar"
    );
    assert_eq!(asset["maven2"]["artifactId"], "lib");
    assert_eq!(asset["canDelete"], true);
    let component = &asset["component"];
    assert_eq!(component["group"], "org.acme");
    assert_eq!(component["name"], "lib");
    assert_eq!(component["version"], "1.0");
    assert_eq!(component["repository"], "releases");

    let public = get(
        &fixture,
        &format!("/service/rest/v1/assets/{}", asset["id"].as_str().unwrap()),
        Auth::Admin,
    )
    .await;
    assert_eq!(public.status, StatusCode::OK);
    assert_eq!(public.json()["path"], asset["path"]);
    let owner = get(
        &fixture,
        &format!(
            "/service/rest/v1/components/{}",
            component["id"].as_str().unwrap()
        ),
        Auth::Admin,
    )
    .await;
    assert_eq!(owner.status, StatusCode::OK);
}

#[tokio::test]
async fn group_viewers_read_member_assets_through_the_group() {
    let fixture = shared().await;
    user_with(
        &fixture,
        "viewer",
        &[
            "as-repository-view-raw-both-browse",
            "as-repository-view-raw-both-read",
        ],
    )
    .await;
    let same = details(&fixture, "both", "/docs/same.txt", user("viewer")).await;
    assert_eq!(same.status, StatusCode::OK, "{}", same.text());
    let same = same.json();
    assert_eq!(same["repository"], "first");
    assert_eq!(same["browsedRepository"], "both");
    assert_eq!(same["fileSize"], 5);
    assert_eq!(
        same["downloadUrl"],
        "http://localhost/repository/both/docs/same.txt"
    );
    assert_eq!(same["canDelete"], false);
    let only = details(&fixture, "both", "docs/only-second.txt", user("viewer"))
        .await
        .json();
    assert_eq!(only["repository"], "second");
    assert_eq!(only["component"]["name"], "/docs/only-second.txt");

    let direct = get(
        &fixture,
        &format!("/service/rest/v1/assets/{}", same["id"].as_str().unwrap()),
        user("viewer"),
    )
    .await;
    assert_eq!(direct.status, StatusCode::NOT_FOUND);
    assert_eq!(
        details(&fixture, "first", "docs/same.txt", user("viewer"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let download = get(&fixture, "/repository/both/docs/same.txt", user("viewer")).await;
    assert_eq!(download.status, StatusCode::OK);

    let admin = details(&fixture, "both", "docs/same.txt", Auth::Admin)
        .await
        .json();
    assert_eq!(admin["canDelete"], true);
}

#[tokio::test]
async fn content_selectors_limit_what_is_shown() {
    let fixture = shared().await;
    selector(&fixture, "only-second", "path == \"/docs/only-second.txt\"").await;
    selector_privilege(
        &fixture,
        "second-one",
        "only-second",
        "raw",
        "second",
        &["BROWSE", "READ", "DELETE"],
    )
    .await;
    user_with(&fixture, "selected", &["second-one"]).await;
    let shown = details(&fixture, "second", "docs/only-second.txt", user("selected")).await;
    assert_eq!(shown.status, StatusCode::OK, "{}", shown.text());
    assert_eq!(shown.json()["canDelete"], true);
    assert_eq!(
        details(&fixture, "second", "docs/same.txt", user("selected"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn missing_and_hidden_assets_answer_like_the_tree() {
    let fixture = shared().await;
    let cases: [(&str, &str, Auth<'_>, StatusCode); 6] = [
        ("first", "docs/none.txt", Auth::Admin, StatusCode::NOT_FOUND),
        ("first", "docs/", Auth::Admin, StatusCode::NOT_FOUND),
        ("first", "", Auth::Admin, StatusCode::NOT_FOUND),
        ("first", "docs/../x", Auth::Admin, StatusCode::BAD_REQUEST),
        (
            "missing",
            "docs/same.txt",
            Auth::Admin,
            StatusCode::NOT_FOUND,
        ),
        (
            "first",
            "docs/same.txt",
            Auth::None,
            StatusCode::UNAUTHORIZED,
        ),
    ];
    for (repository, path, auth, status) in cases {
        assert_eq!(
            details(&fixture, repository, path, auth).await.status,
            status,
            "{repository} {path}"
        );
    }
    fixture.add_user("nobody", "password-1", &[]).await;
    assert_eq!(
        details(&fixture, "first", "docs/same.txt", user("nobody"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    anonymous(&fixture, true).await;
    let anonymous = details(&fixture, "both", "docs/same.txt", Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::OK);
    assert_eq!(anonymous.json()["canDelete"], false);
    let removed = fixture
        .call(
            Method::DELETE,
            &format!(
                "/service/rest/v1/assets/{}",
                details(&fixture, "first", "docs/same.txt", Auth::Admin)
                    .await
                    .json()["id"]
                    .as_str()
                    .unwrap()
            ),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let fallback: Value = details(&fixture, "both", "docs/same.txt", Auth::Admin)
        .await
        .json();
    assert_eq!(fallback["repository"], "second");
    assert_eq!(fallback["fileSize"], json!(13));
}

async fn preview(fixture: &Fixture, repository: &str, path: &str, auth: Auth<'_>) -> Reply {
    get(
        fixture,
        &format!("{BROWSE}/{repository}/content?path={path}"),
        auth,
    )
    .await
}

fn header<'a>(reply: &'a Reply, name: &str) -> &'a str {
    reply
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[tokio::test]
async fn previews_show_content_without_counting_downloads() {
    let fixture = shared().await;
    let shown = preview(&fixture, "first", "docs/same.txt", Auth::Admin).await;
    assert_eq!(shown.status, StatusCode::OK, "{}", shown.text());
    assert_eq!(shown.body, b"first");
    assert_eq!(header(&shown, "content-type"), "text/plain; charset=utf-8");
    assert_eq!(header(&shown, "x-content-type-options"), "nosniff");
    assert!(header(&shown, "content-security-policy").starts_with("sandbox"));
    assert_eq!(header(&shown, "cache-control"), "private, no-store");
    fixture.state.content().flush_downloads().await.unwrap();
    let counted = |reply: Reply| reply.json()["downloadCount"].clone();
    assert_eq!(
        counted(details(&fixture, "first", "docs/same.txt", Auth::Admin).await),
        0
    );
    let download = get(&fixture, "/repository/first/docs/same.txt", Auth::Admin).await;
    assert_eq!(download.status, StatusCode::OK);
    fixture.state.content().flush_downloads().await.unwrap();
    assert_eq!(
        counted(details(&fixture, "first", "docs/same.txt", Auth::Admin).await),
        1
    );

    let pixel = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    put(&fixture, "/repository/first/docs/pixel.png", pixel.clone()).await;
    let image = preview(&fixture, "first", "docs/pixel.png", Auth::Admin).await;
    assert_eq!(image.status, StatusCode::OK, "{}", image.text());
    assert_eq!(header(&image, "content-type"), "image/png");
    assert_eq!(image.body, pixel);
}

#[tokio::test]
async fn previews_need_the_right_to_read() {
    let fixture = shared().await;
    user_with(
        &fixture,
        "reader",
        &[
            "as-repository-view-raw-both-browse",
            "as-repository-view-raw-both-read",
        ],
    )
    .await;
    user_with(&fixture, "browser", &["as-repository-view-raw-both-browse"]).await;
    let through_group = preview(&fixture, "both", "docs/same.txt", user("reader")).await;
    assert_eq!(
        through_group.status,
        StatusCode::OK,
        "{}",
        through_group.text()
    );
    assert_eq!(through_group.body, b"first");
    assert_eq!(
        details(&fixture, "both", "docs/same.txt", user("browser"))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        preview(&fixture, "both", "docs/same.txt", user("browser"))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let cases: [(&str, &str, Auth<'_>, StatusCode); 4] = [
        ("first", "docs/none.txt", Auth::Admin, StatusCode::NOT_FOUND),
        ("first", "docs/", Auth::Admin, StatusCode::NOT_FOUND),
        (
            "missing",
            "docs/same.txt",
            Auth::Admin,
            StatusCode::NOT_FOUND,
        ),
        (
            "first",
            "docs/same.txt",
            Auth::None,
            StatusCode::UNAUTHORIZED,
        ),
    ];
    for (repository, path, auth, status) in cases {
        assert_eq!(
            preview(&fixture, repository, path, auth).await.status,
            status,
            "{repository} {path}"
        );
    }
}
