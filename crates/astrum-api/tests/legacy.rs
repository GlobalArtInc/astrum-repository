mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::{Auth, Fixture};

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_maven::Maven));
    fixture
}

async fn create(fixture: &Fixture, path: &str, body: Value) {
    let reply = fixture
        .call(Method::POST, path, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn repositories(fixture: &Fixture) {
    let storage = json!({"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"});
    create(
        fixture,
        "/service/rest/v1/repositories/raw/hosted",
        json!({"name": "files", "online": true, "storage": storage}),
    )
    .await;
    create(
        fixture,
        "/service/rest/v1/repositories/raw/group",
        json!({"name": "all-files", "online": true, "storage": storage, "group": {"memberNames": ["files"]}}),
    )
    .await;
    create(
        fixture,
        "/service/rest/v1/repositories/maven/hosted",
        json!({"name": "releases", "online": true, "storage": storage,
               "maven": {"versionPolicy": "MIXED", "layoutPolicy": "STRICT"}}),
    )
    .await;
}

async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> (StatusCode, String) {
    let reply = fixture.call(Method::GET, path, auth, None).await;
    (reply.status, reply.text())
}

#[tokio::test]
async fn old_content_urls_reach_the_repositories() {
    let fixture = ready().await;
    repositories(&fixture).await;
    for (path, body) in [
        ("/content/repositories/files/a/one.txt", "one"),
        ("/content/groups/files/a/two.txt", "two"),
        ("/content/sites/files/a/three.txt", "three"),
        ("/service/local/repositories/files/a/four.txt", "four"),
        ("/service/local/repo_groups/files/a/five.txt", "five"),
    ] {
        let reply = fixture
            .send_body(Method::PUT, path, Auth::Admin, Body::from(body), &[])
            .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{path}: {}",
            reply.text()
        );
        assert_eq!(
            get(
                &fixture,
                &format!("/repository/files/a/{}", path.rsplit('/').next().unwrap()),
                Auth::Admin
            )
            .await,
            (StatusCode::OK, body.to_owned())
        );
    }
    for prefix in [
        "/content/repositories/all-files",
        "/content/groups/all-files",
        "/content/sites/all-files",
        "/content/repositories/files",
        "/service/local/repositories/all-files",
        "/service/local/repo_groups/all-files",
    ] {
        assert_eq!(
            get(&fixture, &format!("{prefix}/a/one.txt"), Auth::Admin).await,
            (StatusCode::OK, "one".to_owned()),
            "{prefix}"
        );
    }
    let head = fixture
        .call(
            Method::HEAD,
            "/content/groups/all-files/a/two.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert_eq!(head.headers[header::CONTENT_LENGTH], "3");
    let deleted = fixture
        .call(
            Method::DELETE,
            "/content/repositories/files/a/one.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(
            &fixture,
            "/content/repositories/files/a/one.txt",
            Auth::Admin
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn sites_serve_only_raw_repositories() {
    let fixture = ready().await;
    repositories(&fixture).await;
    let jar = fixture
        .send_body(
            Method::PUT,
            "/content/repositories/releases/org/example/lib/1.0/lib-1.0.pom",
            Auth::Admin,
            Body::from("<project><modelVersion>4.0.0</modelVersion><groupId>org.example</groupId><artifactId>lib</artifactId><version>1.0</version></project>"),
            &[],
        )
        .await;
    assert_eq!(jar.status, StatusCode::CREATED, "{}", jar.text());
    let path = "releases/org/example/lib/1.0/lib-1.0.pom";
    assert_eq!(
        get(
            &fixture,
            &format!("/content/repositories/{path}"),
            Auth::Admin
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &format!("/content/groups/{path}"), Auth::Admin)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &format!("/content/sites/{path}"), Auth::Admin).await,
        (StatusCode::NOT_FOUND, "Repository not found".to_owned())
    );
}

#[tokio::test]
async fn malformed_and_unknown_addresses_answer_like_the_old_server() {
    let fixture = ready().await;
    repositories(&fixture).await;
    let empty = "Repository path must not be null or empty";
    let no_slash = "Repository path must have another '/' after initial '/'";
    for (path, status, body) in [
        ("/content/repositories", StatusCode::BAD_REQUEST, empty),
        ("/content/groups", StatusCode::BAD_REQUEST, empty),
        (
            "/service/local/repositories",
            StatusCode::BAD_REQUEST,
            empty,
        ),
        ("/service/local/repo_groups", StatusCode::BAD_REQUEST, empty),
        ("/content/sites/", StatusCode::BAD_REQUEST, no_slash),
        (
            "/content/repositories/files",
            StatusCode::BAD_REQUEST,
            no_slash,
        ),
        (
            "/content/repositories//a/one.txt",
            StatusCode::BAD_REQUEST,
            no_slash,
        ),
        (
            "/service/local/repositories/files",
            StatusCode::BAD_REQUEST,
            no_slash,
        ),
        (
            "/content/repositories/missing/a.txt",
            StatusCode::NOT_FOUND,
            "Repository not found",
        ),
        (
            "/content/sites/missing/a.txt",
            StatusCode::NOT_FOUND,
            "Repository not found",
        ),
        ("/content/shadows/files/a.txt", StatusCode::NOT_FOUND, ""),
        (
            "/service/local/groups/files/a.txt",
            StatusCode::NOT_FOUND,
            "",
        ),
        ("/service/local/status", StatusCode::NOT_FOUND, ""),
    ] {
        assert_eq!(
            get(&fixture, path, Auth::Admin).await,
            (status, body.to_owned()),
            "{path}"
        );
    }
}

#[tokio::test]
async fn old_content_urls_keep_the_repository_permissions() {
    let fixture = ready().await;
    repositories(&fixture).await;
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/files/a/one.txt",
            Auth::Admin,
            Body::from("one"),
            &[],
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED);
    let anonymous = fixture
        .call(
            Method::GET,
            "/content/repositories/files/a/one.txt",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(
        get(&fixture, "/content/repositories/missing/a.txt", Auth::None).await,
        (StatusCode::NOT_FOUND, "Repository not found".to_owned())
    );
    fixture.add_user("nobody", "nobody-pass", &[]).await;
    let refused = fixture
        .send_body(
            Method::PUT,
            "/content/repositories/files/a/two.txt",
            Auth::Basic("nobody", "nobody-pass"),
            Body::from("two"),
            &[],
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(
        get(
            &fixture,
            "/service/local/repositories/files/a/one.txt",
            Auth::Basic("nobody", "nobody-pass")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
