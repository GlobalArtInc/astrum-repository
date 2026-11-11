mod support;

use std::sync::Arc;

use astrum_content::share::{self, Link, NewLink};
use astrum_content::{PutAsset, store};
use astrum_db::entity::{audit_event, share_link};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use uuid::Uuid;

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    for name in ["files", "other"] {
        create(&fixture, name, "hosted", hosted(name)).await;
        put(&fixture, name, "/notes/one.txt", b"first file").await;
        put(&fixture, name, "/notes/deep/two.txt", b"second file").await;
        put(&fixture, name, "/notes2/three.txt", b"third file").await;
        put(&fixture, name, "/top%20&%20%3Cb%3E.txt", b"odd name").await;
    }
    fixture
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    })
}

async fn create(fixture: &Fixture, name: &str, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/raw/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::CREATED,
        "{name}: {}",
        reply.text()
    );
}

async fn put(fixture: &Fixture, repository: &str, path: &str, body: &'static [u8]) {
    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}{path}"),
            Auth::Admin,
            Body::from(body),
            &[],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{path}");
}

async fn plant(fixture: &Fixture, repository: &str, kind: &str, path: &str, body: &'static [u8]) {
    let content = fixture.state.content();
    let found = content.repository(repository).await.unwrap().unwrap();
    let ctx = content.context(
        found,
        fixture.state.security().model(),
        None,
        String::new(),
        String::new(),
    );
    let mut asset = PutAsset::hosted(&ctx, path);
    asset.kind = kind.to_owned();
    asset.check_content_type = false;
    store::put(&ctx, asset, astrum_blob::content_of(body))
        .await
        .unwrap();
}

async fn row(fixture: &Fixture, link: &Link) -> share_link::Model {
    share_link::Entity::find_by_id(link.id)
        .one(fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap()
}

async fn ranged(fixture: &Fixture, secret: &str, range: &str) -> Reply {
    fetch(fixture, Method::GET, &url(secret, ""), &[("range", range)]).await
}

async fn link(
    fixture: &Fixture,
    repository: &str,
    path: &str,
    prefix: bool,
    max_downloads: Option<i32>,
) -> (Link, String) {
    share::create(
        fixture.state.db().conn(),
        NewLink {
            repository: repository.to_owned(),
            path: path.to_owned(),
            prefix,
            note: String::new(),
            created_by: "admin".to_owned(),
            expires_at: Utc::now() + Duration::hours(1),
            max_downloads,
        },
    )
    .await
    .unwrap()
}

async fn expire(fixture: &Fixture, link: &Link, ago: Duration) {
    share_link::ActiveModel {
        id: ActiveValue::Unchanged(link.id),
        expires_at: ActiveValue::Set((Utc::now() - ago).fixed_offset()),
        ..Default::default()
    }
    .update(fixture.state.db().conn())
    .await
    .unwrap();
}

async fn fetch(fixture: &Fixture, method: Method, path: &str, headers: &[(&str, &str)]) -> Reply {
    fixture.send(method, path, Auth::None, None, headers).await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fetch(fixture, Method::GET, path, &[]).await
}

fn url(secret: &str, rest: &str) -> String {
    format!("/service/share/{secret}{rest}")
}

#[tokio::test]
async fn a_single_asset_link_serves_that_asset_with_leak_protection() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    let reply = get(&fixture, &url(&secret, "")).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text(), "first file");
    let expected = [
        (
            "content-disposition",
            "attachment; filename*=UTF-8''one.txt",
        ),
        ("cache-control", "private, no-store"),
        ("referrer-policy", "no-referrer"),
        ("x-content-type-options", "nosniff"),
        ("content-security-policy", "sandbox"),
        ("x-robots-tag", "noindex"),
    ];
    for (name, value) in expected {
        assert_eq!(reply.headers[name], value, "{name}");
    }
    let other = get(&fixture, &url(&secret, "/notes/deep/two.txt")).await;
    assert_eq!(other.text(), "first file");
    let odd = link(&fixture, "files", "/top & <b>.txt", false, None)
        .await
        .1;
    let reply = get(&fixture, &url(&odd, "/ignored")).await;
    assert_eq!(
        reply.headers[header::CONTENT_DISPOSITION],
        "attachment; filename*=UTF-8''top%20&%20%3Cb%3E.txt"
    );
    let direct = get(&fixture, "/repository/files/notes/one.txt").await;
    assert_eq!(direct.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_prefix_link_serves_files_and_an_escaped_index_inside_its_prefix_only() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/", true, None).await;
    let file = get(&fixture, &url(&secret, "/one.txt")).await;
    assert_eq!(
        (file.status, file.text()),
        (StatusCode::OK, "first file".to_owned())
    );
    let deep = get(&fixture, &url(&secret, "//deep///two.txt")).await;
    assert_eq!(deep.text(), "second file");
    for index in [url(&secret, ""), url(&secret, "/")] {
        let page = get(&fixture, &index).await;
        assert_eq!(page.status, StatusCode::OK, "{index}");
        let text = page.text();
        assert!(text.contains(&format!("href=\"/service/share/{secret}/one.txt\"")));
        assert!(text.contains("deep/two.txt"));
        assert!(!text.contains("three.txt"));
        assert_eq!(page.headers[header::CACHE_CONTROL], "private, no-store");
    }
    let wide = link(&fixture, "files", "/", true, None).await.1;
    let page = get(&fixture, &url(&wide, "/")).await.text();
    assert!(page.contains("top%20%26%20%3Cb%3E.txt"), "{page}");
    assert!(page.contains("top &amp; &lt;b&gt;.txt"), "{page}");
    let missing = get(&fixture, &url(&secret, "/absent.txt")).await;
    for escape in [
        "/../notes2/three.txt",
        "/%2e%2e/notes2/three.txt",
        "/%2E%2E%2Fnotes2%2Fthree.txt",
        "/deep/../../notes2/three.txt",
        "/three.txt",
        "/notes2/three.txt",
        "/deep/",
        "/%00",
    ] {
        let reply = get(&fixture, &url(&secret, escape)).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{escape}");
        assert_eq!(reply.body, missing.body, "{escape}");
    }
    let sibling = link(&fixture, "files", "/notes2/", true, None).await.1;
    assert_eq!(
        get(&fixture, &url(&sibling, "/three.txt")).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &url(&sibling, "/../notes/one.txt"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let other = link(&fixture, "other", "/notes/", true, None).await.1;
    assert_eq!(
        get(&fixture, &url(&other, "/one.txt")).await.text(),
        "first file"
    );
}

#[tokio::test]
async fn request_paths_of_a_prefix_link_are_normalised_before_the_lookup() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/", true, None).await;
    for (rest, expected) in [
        ("//one.txt", "first file"),
        ("/deep//two.txt", "second file"),
        ("//deep///two.txt", "second file"),
        ("/deep%2F%2Ftwo.txt", "second file"),
    ] {
        let reply = get(&fixture, &url(&secret, rest)).await;
        assert_eq!(reply.status, StatusCode::OK, "{rest}");
        assert_eq!(reply.text(), expected, "{rest}");
    }
    let missing = get(&fixture, &url(&secret, "/absent.txt")).await;
    for rest in [
        "/./one.txt",
        "/deep/./two.txt",
        "/%2e/one.txt",
        "/%2E%2E/notes2/three.txt",
        "/deep/%2e%2e/%2e%2e/notes2/three.txt",
        "/..%2Fnotes2/three.txt",
        "//..//notes2//three.txt",
        "/one.txt/",
    ] {
        let reply = get(&fixture, &url(&secret, rest)).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{rest}");
        assert_eq!(reply.body, missing.body, "{rest}");
    }
}

#[tokio::test]
async fn unknown_secrets_answer_like_missing_paths_and_expired_links_are_gone() {
    let fixture = ready().await;
    let (expired, secret) = link(&fixture, "files", "/notes/", true, None).await;
    let missing = get(&fixture, &url(&secret, "/absent.txt")).await;
    for bad in [
        "astrum_share_".to_owned(),
        format!("{}x", secret),
        secret.replace('a', "b"),
        "x".to_owned(),
    ] {
        let reply = get(&fixture, &url(&bad, "/one.txt")).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{bad}");
        assert_eq!(reply.body, missing.body);
        assert_eq!(
            reply.headers[header::CONTENT_TYPE],
            missing.headers[header::CONTENT_TYPE]
        );
    }
    expire(&fixture, &expired, Duration::seconds(5)).await;
    for path in ["/one.txt", "", "/absent.txt"] {
        assert_eq!(
            get(&fixture, &url(&secret, path)).await.status,
            StatusCode::GONE,
            "{path}"
        );
    }
}

#[tokio::test]
async fn only_get_and_head_are_served_and_head_does_not_count() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    for method in [Method::PUT, Method::POST, Method::DELETE, Method::PATCH] {
        let reply = fetch(&fixture, method, &url(&secret, ""), &[]).await;
        assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(reply.headers[header::ALLOW], "GET, HEAD");
    }
    let unknown = fetch(&fixture, Method::PUT, &url("astrum_share_nothing", ""), &[]).await;
    assert_eq!(unknown.status, StatusCode::METHOD_NOT_ALLOWED);
    for _ in 0..3 {
        let head = fetch(&fixture, Method::HEAD, &url(&secret, ""), &[]).await;
        assert_eq!((head.status, head.body.len()), (StatusCode::OK, 0));
        assert_eq!(head.headers[header::CONTENT_LENGTH], "10");
        assert_eq!(head.headers[header::CACHE_CONTROL], "private, no-store");
    }
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_download_budget_counts_started_downloads_and_ignores_conditional_answers() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::OK
    );
    for range in [None, Some("bytes=5-"), Some("bytes=0-3")] {
        let headers: Vec<(&str, &str)> = range.map(|value| ("range", value)).into_iter().collect();
        let reply = fetch(&fixture, Method::GET, &url(&secret, ""), &headers).await;
        assert_eq!(reply.status, StatusCode::GONE, "{range:?}");
    }
    let cached = fetch(
        &fixture,
        Method::GET,
        &url(&secret, ""),
        &[("if-none-match", "\"*\""), ("if-match", "\"nope\"")],
    )
    .await;
    assert_eq!(cached.status, StatusCode::PRECONDITION_FAILED);

    let (counted, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(5)).await;
    let etag = get(&fixture, &url(&secret, "")).await.headers[header::ETAG]
        .to_str()
        .unwrap()
        .to_owned();
    let not_modified = fetch(
        &fixture,
        Method::GET,
        &url(&secret, ""),
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(not_modified.status, StatusCode::NOT_MODIFIED);
    assert_eq!(
        ranged(&fixture, &secret, "bytes=500-").await.status,
        StatusCode::RANGE_NOT_SATISFIABLE
    );
    let counted = row(&fixture, &counted).await;
    assert_eq!((counted.downloads, counted.bytes_served), (1, 10));
    assert!(counted.last_used_at.is_some());
}

#[tokio::test]
async fn ranged_requests_spend_a_byte_budget_that_never_exceeds_the_allowed_downloads() {
    let fixture = ready().await;
    let (single, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    let first = ranged(&fixture, &secret, "bytes=0-0").await;
    assert_eq!(
        (first.status, first.text()),
        (StatusCode::PARTIAL_CONTENT, "f".to_owned())
    );
    let rest = ranged(&fixture, &secret, "bytes=1-").await;
    assert_eq!(
        (rest.status, rest.text()),
        (StatusCode::PARTIAL_CONTENT, "irst file".to_owned())
    );
    assert_eq!(
        ranged(&fixture, &secret, "bytes=1-").await.status,
        StatusCode::GONE
    );
    assert_eq!(
        ranged(&fixture, &secret, "bytes=0-0").await.status,
        StatusCode::GONE
    );
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::GONE
    );
    let single = row(&fixture, &single).await;
    assert_eq!((single.downloads, single.bytes_served), (1, 10));

    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    let mut received = 0;
    for _ in 0..20 {
        let reply = ranged(&fixture, &secret, "bytes=1-").await;
        if reply.status == StatusCode::PARTIAL_CONTENT {
            received += reply.body.len();
        }
    }
    assert_eq!(received, 9);

    let (twice, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(2)).await;
    for _ in 0..2 {
        assert_eq!(
            get(&fixture, &url(&secret, "")).await.status,
            StatusCode::OK
        );
    }
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::GONE
    );
    assert_eq!(row(&fixture, &twice).await.downloads, 2);

    let (open, secret) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    for _ in 0..4 {
        assert_eq!(
            get(&fixture, &url(&secret, "")).await.status,
            StatusCode::OK
        );
        assert_eq!(
            ranged(&fixture, &secret, "bytes=1-").await.status,
            StatusCode::PARTIAL_CONTENT
        );
    }
    let open = row(&fixture, &open).await;
    assert_eq!((open.downloads, open.bytes_served), (4, 4 * 10 + 4 * 9));
}

#[tokio::test]
async fn answers_that_spend_nothing_do_not_depend_on_the_budget_left() {
    let fixture = ready().await;
    let (single, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    assert_eq!(
        ranged(&fixture, &secret, "bytes=0-5").await.status,
        StatusCode::PARTIAL_CONTENT
    );
    let head = fetch(&fixture, Method::HEAD, &url(&secret, ""), &[]).await;
    assert_eq!((head.status, head.body.len()), (StatusCode::OK, 0));
    assert_eq!(head.headers[header::CONTENT_LENGTH], "10");
    let etag = head.headers[header::ETAG].to_str().unwrap().to_owned();
    let not_modified = fetch(
        &fixture,
        Method::GET,
        &url(&secret, ""),
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(not_modified.status, StatusCode::NOT_MODIFIED);
    assert_eq!(
        ranged(&fixture, &secret, "bytes=500-").await.status,
        StatusCode::RANGE_NOT_SATISFIABLE
    );
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::GONE
    );
    let spent = row(&fixture, &single).await;
    assert_eq!((spent.downloads, spent.bytes_served), (1, 6));
    expire(&fixture, &single, Duration::seconds(1)).await;
    for method in [Method::HEAD, Method::GET] {
        let reply = fetch(&fixture, method, &url(&secret, ""), &[]).await;
        assert_eq!(reply.status, StatusCode::GONE);
    }
}

#[tokio::test]
async fn a_download_that_fails_before_any_byte_is_sent_is_given_back() {
    let fixture = ready().await;
    let (single, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
    let asset = store::find(fixture.state.db().conn(), "files", "/notes/one.txt")
        .await
        .unwrap()
        .unwrap();
    let blobs = fixture
        .state
        .content()
        .blob_stores()
        .get(&asset.blob_store)
        .await
        .unwrap();
    assert!(blobs.delete(asset.blob_id.into()).await.unwrap());
    for range in [None, Some("bytes=0-3"), Some("bytes=3-")] {
        let headers: Vec<(&str, &str)> = range.map(|value| ("range", value)).into_iter().collect();
        let reply = fetch(&fixture, Method::GET, &url(&secret, ""), &headers).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{range:?}");
        let after = row(&fixture, &single).await;
        assert_eq!((after.downloads, after.bytes_served), (0, 0), "{range:?}");
    }
    assert!(share::permits(
        &row(&fixture, &single).await,
        share::Portion::whole(10),
        Utc::now()
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_simultaneous_requests_never_both_use_the_last_download() {
    let fixture = Arc::new(ready().await);
    for round in 0..5 {
        let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, Some(1)).await;
        let tasks: Vec<_> = (0..8)
            .map(|_| {
                let fixture = fixture.clone();
                let path = url(&secret, "");
                tokio::spawn(async move { get(&fixture, &path).await.status })
            })
            .collect();
        let mut statuses = Vec::new();
        for task in tasks {
            statuses.push(task.await.unwrap());
        }
        let served = statuses.iter().filter(|s| **s == StatusCode::OK).count();
        let gone = statuses.iter().filter(|s| **s == StatusCode::GONE).count();
        assert_eq!((served, gone), (1, 7), "round {round}: {statuses:?}");
    }
}

#[tokio::test]
async fn links_to_offline_deleted_or_group_repositories_answer_not_found() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    let (_, doomed) = link(&fixture, "other", "/notes/one.txt", false, None).await;
    create(
        &fixture,
        "everything",
        "group",
        json!({
            "name": "everything",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["files"]}
        }),
    )
    .await;
    let (_, grouped) = link(&fixture, "everything", "/notes/one.txt", false, None).await;
    assert_eq!(
        get(&fixture, &url(&grouped, "")).await.status,
        StatusCode::NOT_FOUND
    );
    let mut mirror = hosted("mirror");
    mirror.as_object_mut().unwrap().remove("raw");
    mirror["proxy"] = json!({"remoteUrl": "https://mirror.example.org/", "contentMaxAge": 1440, "metadataMaxAge": 1440});
    mirror["negativeCache"] = json!({"enabled": true, "timeToLive": 1440});
    mirror["httpClient"] = json!({"blocked": false, "autoBlock": true});
    create(&fixture, "mirror", "proxy", mirror).await;
    plant(&fixture, "mirror", "file", "/cached.txt", b"cached file").await;
    let (_, proxied) = link(&fixture, "mirror", "/cached.txt", false, None).await;
    let (_, proxied_prefix) = link(&fixture, "mirror", "/", true, None).await;
    for target in [
        url(&proxied, ""),
        url(&proxied_prefix, "/cached.txt"),
        url(&proxied_prefix, ""),
    ] {
        assert_eq!(
            get(&fixture, &target).await.status,
            StatusCode::NOT_FOUND,
            "{target}"
        );
    }
    let mut offline = hosted("files");
    offline["online"] = json!(false);
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/raw/hosted/files",
            Auth::Admin,
            Some(offline),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(
        get(&fixture, &url(&secret, "")).await.status,
        StatusCode::NOT_FOUND
    );
    let reply = fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/repositories/other",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(
        get(&fixture, &url(&doomed, "")).await.status,
        StatusCode::NOT_FOUND
    );
    let left = share::list(fixture.state.db().conn(), None, Some("other"))
        .await
        .unwrap();
    assert!(left.is_empty());
}

#[tokio::test]
async fn a_repository_domain_serves_links_and_downloads_leave_no_secret_in_the_audit_log() {
    let fixture = ready().await;
    let mut rooted = hosted("rooted");
    rooted["domain"] = json!({"host": "files.example.org", "routing": "ROOT"});
    create(&fixture, "rooted", "hosted", rooted).await;
    put(&fixture, "rooted", "/notes/one.txt", b"first file").await;
    let (_, secret) = link(&fixture, "rooted", "/notes/one.txt", false, None).await;
    let reply = fetch(
        &fixture,
        Method::GET,
        &url(&secret, "/one.txt"),
        &[("host", "files.example.org")],
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.text(), "first file");
    let (_, other) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    assert_eq!(get(&fixture, &url(&other, "")).await.status, StatusCode::OK);
    let events = audit_event::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert!(events.iter().any(|event| event.kind.contains("downloaded")));
    let dump = format!("{events:?}");
    for secret in [&secret, &other, &share::hash(&secret), &share::hash(&other)] {
        assert!(!dump.contains(secret.as_str()));
    }
}

#[tokio::test]
async fn docker_repositories_are_never_served_through_links() {
    let fixture = ready().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/docker/hosted",
            Auth::Admin,
            Some(json!({
                "name": "images",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    plant(
        &fixture,
        "images",
        "blob",
        "/v2/app/blobs/sha256:abc",
        b"layer",
    )
    .await;
    let (_, single) = link(&fixture, "images", "/v2/app/blobs/sha256:abc", false, None).await;
    let (_, wide) = link(&fixture, "images", "/v2/", true, None).await;
    for target in [
        url(&single, ""),
        url(&wide, "/app/blobs/sha256:abc"),
        url(&wide, ""),
    ] {
        let reply = get(&fixture, &target).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{target}");
        assert_eq!(reply.headers[header::CACHE_CONTROL], "private, no-store");
    }
}

#[tokio::test]
async fn stale_credentials_do_not_break_a_public_link_or_leave_a_failed_sign_in() {
    let fixture = ready().await;
    let (_, secret) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    for authorization in ["Basic YWRtaW46d3Jvbmc=", "Bearer not-a-token", "Basic !!!"] {
        let reply = fetch(
            &fixture,
            Method::GET,
            &url(&secret, ""),
            &[("authorization", authorization)],
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{authorization}");
        assert_eq!(reply.text(), "first file");
        assert_eq!(reply.headers[header::CACHE_CONTROL], "private, no-store");
    }
    let events = audit_event::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert!(
        events.iter().all(|event| event.domain != "security.user"),
        "{events:?}"
    );
}

#[tokio::test]
async fn an_empty_secret_is_refused_with_the_same_protection_headers() {
    let fixture = ready().await;
    let reply = get(&fixture, "/service/share/").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let unknown = get(&fixture, &url("astrum_share_nothing", "")).await;
    assert_eq!(reply.body, unknown.body);
    for (name, value) in [
        ("cache-control", "private, no-store"),
        ("referrer-policy", "no-referrer"),
        ("x-content-type-options", "nosniff"),
        ("content-security-policy", "sandbox"),
        ("x-robots-tag", "noindex"),
    ] {
        assert_eq!(reply.headers[name], value, "{name}");
    }
}

#[tokio::test]
async fn links_are_created_validated_and_pruned() {
    let fixture = ready().await;
    let db = fixture.state.db().conn();
    let (first, secret) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    assert_eq!(first.secret_hash, share::hash(&secret));
    assert_ne!(first.secret_hash, secret);
    assert_eq!(first.secret_hash.len(), 64);
    assert!(secret.starts_with("astrum_share_") && secret.len() == 58);
    let by = share::by_secret(db, &secret).await.unwrap().unwrap();
    assert_eq!(by.id, first.id);
    assert!(
        share::by_secret(db, &first.secret_hash)
            .await
            .unwrap()
            .is_none()
    );
    let new = |path: &str, prefix: bool, max: Option<i32>, ttl: i64| NewLink {
        repository: "files".to_owned(),
        path: path.to_owned(),
        prefix,
        note: String::new(),
        created_by: "admin".to_owned(),
        expires_at: Utc::now() + Duration::seconds(ttl),
        max_downloads: max,
    };
    for bad in [
        new("/notes/", false, None, 60),
        new("/notes/one.txt", true, None, 60),
        new("/notes/", true, Some(3), 60),
        new("/notes/../x", false, None, 60),
        new("/notes/one.txt", false, Some(0), 60),
        new("/notes/one.txt", false, None, -60),
    ] {
        assert!(share::create(db, bad).await.is_err());
    }
    for good in [
        new("/notes/one.txt", false, Some(3), 60),
        new("/notes/", true, None, 60),
    ] {
        let (created, _) = share::create(db, good).await.unwrap();
        assert!(share::delete(db, created.id).await.unwrap());
    }
    let forced = share_link::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        secret_hash: ActiveValue::Set("forced".to_owned()),
        repository: ActiveValue::Set("files".to_owned()),
        path: ActiveValue::Set("/notes/".to_owned()),
        prefix: ActiveValue::Set(true),
        note: ActiveValue::Set(String::new()),
        created_by: ActiveValue::Set("admin".to_owned()),
        created_at: ActiveValue::Set(Utc::now().fixed_offset()),
        expires_at: ActiveValue::Set((Utc::now() + Duration::hours(1)).fixed_offset()),
        max_downloads: ActiveValue::Set(Some(1)),
        downloads: ActiveValue::Set(0),
        bytes_served: ActiveValue::Set(0),
        last_used_at: ActiveValue::Set(None),
        last_used_ip: ActiveValue::Set(None),
    };
    assert!(forced.insert(db).await.is_err());
    let (old, _) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    let (recent, _) = link(&fixture, "files", "/notes/one.txt", false, None).await;
    expire(&fixture, &old, Duration::days(31)).await;
    expire(&fixture, &recent, Duration::days(29)).await;
    assert_eq!(share::prune(db).await.unwrap(), 1);
    let ids: Vec<_> = share::list(db, Some("admin"), None)
        .await
        .unwrap()
        .into_iter()
        .map(|link| link.id)
        .collect();
    assert!(ids.contains(&recent.id) && ids.contains(&first.id) && !ids.contains(&old.id));
    assert_eq!(share::delete_of_user(db, "admin").await.unwrap(), 2);
}
