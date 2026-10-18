mod support;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use support::{Auth, Fixture};
use tower::ServiceExt;

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/raw/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
        "raw": {"contentDisposition": "INLINE"}
    })
}

async fn put(fixture: &Fixture, path: &str, body: &'static [u8], auth: Auth<'_>) -> StatusCode {
    fixture
        .send_body(Method::PUT, path, auth, Body::from(body), &[])
        .await
        .status
}

#[tokio::test]
async fn hosted_files_go_in_and_come_out() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("files", "ALLOW")).await;
    assert_eq!(
        put(
            &fixture,
            "/repository/files/a/b/hello.txt",
            b"hello world\n",
            Auth::Admin
        )
        .await,
        StatusCode::CREATED
    );

    let reply = fixture
        .call(
            Method::GET,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text(), "hello world\n");
    assert_eq!(
        reply.headers[header::ETAG],
        "\"22596363b3de40b06f981fb85d82312e8c0ed511\""
    );
    assert_eq!(reply.headers[header::CONTENT_TYPE], "text/plain");
    assert_eq!(reply.headers[header::CONTENT_DISPOSITION], "inline");
    assert_eq!(reply.headers[header::CONTENT_LENGTH], "12");

    let head = fixture
        .call(
            Method::HEAD,
            "/repository/files//a/b/hello.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!((head.status, head.body.len()), (StatusCode::OK, 0));

    let part = fixture
        .send(
            Method::GET,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
            &[("range", "bytes=6-")],
        )
        .await;
    assert_eq!(part.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(part.text(), "world\n");
    assert_eq!(part.headers[header::CONTENT_RANGE], "bytes 6-11/12");
    let outside = fixture
        .send(
            Method::GET,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
            &[("range", "bytes=100-")],
        )
        .await;
    assert_eq!(outside.status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(outside.headers[header::CONTENT_RANGE], "bytes */12");

    let etag = reply.headers[header::ETAG].to_str().unwrap().to_owned();
    let cached = fixture
        .send(
            Method::GET,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(cached.status, StatusCode::NOT_MODIFIED);
    let modified = reply.headers[header::LAST_MODIFIED]
        .to_str()
        .unwrap()
        .to_owned();
    let since = fixture
        .send(
            Method::GET,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
            &[("if-modified-since", &modified)],
        )
        .await;
    assert_eq!(since.status, StatusCode::NOT_MODIFIED);
    for (name, value, status) in [
        ("if-match", etag.as_str(), StatusCode::OK),
        ("if-match", "\"0\"", StatusCode::PRECONDITION_FAILED),
        ("if-match", "*", StatusCode::OK),
        ("if-unmodified-since", modified.as_str(), StatusCode::OK),
        (
            "if-unmodified-since",
            "Mon, 01 Jan 2001 00:00:00 GMT",
            StatusCode::PRECONDITION_FAILED,
        ),
    ] {
        let reply = fixture
            .send(
                Method::GET,
                "/repository/files/a/b/hello.txt",
                Auth::Admin,
                None,
                &[(name, value)],
            )
            .await;
        assert_eq!(reply.status, status, "{name}: {value}");
    }

    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/files/a/missing.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/files/a/", Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/nothing/x", Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let post = fixture
        .call(
            Method::POST,
            "/repository/files/a/b/hello.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(post.headers[header::ALLOW], "GET,HEAD,PUT,DELETE");

    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                "/repository/files/a/b/hello.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                "/repository/files/a/b/hello.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/files/a/b/hello.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn uploads_record_the_peer_address_unless_a_proxy_names_the_client() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("peers", "ALLOW")).await;
    let credentials = STANDARD.encode(format!("admin:{}", fixture.admin_password));
    for (path, forwarded) in [
        ("/repository/peers/direct.txt", None),
        ("/repository/peers/forwarded.txt", Some("203.0.113.9")),
    ] {
        let mut request = Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::AUTHORIZATION, format!("Basic {credentials}"))
            .header(header::CONTENT_TYPE, "text/plain");
        if let Some(address) = forwarded {
            request = request.header("x-forwarded-for", address);
        }
        let mut request = request.body(Body::from("peer\n")).unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([192, 0, 2, 7], 40000))));
        let response = astrum_api::router(fixture.state.clone())
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "{path}");
    }
    let assets = fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=peers",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let addresses: HashMap<String, String> = assets["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| {
            (
                asset["path"].as_str().unwrap().to_owned(),
                asset["uploaderIp"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(addresses["/direct.txt"], "192.0.2.7");
    assert_eq!(addresses["/forwarded.txt"], "203.0.113.9");
}

#[tokio::test]
async fn write_policies_and_content_types_are_enforced() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("once", "ALLOW_ONCE")).await;
    create(&fixture, "hosted", hosted("frozen", "DENY")).await;
    assert_eq!(
        put(&fixture, "/repository/once/x.txt", b"one", Auth::Admin).await,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, "/repository/once/x.txt", b"two", Auth::Admin).await,
        StatusCode::CONFLICT
    );
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/once/x.txt", Auth::Admin, None)
            .await
            .text(),
        "one"
    );
    assert_eq!(
        put(&fixture, "/repository/frozen/x.txt", b"one", Auth::Admin).await,
        StatusCode::BAD_REQUEST
    );
    let mismatch = fixture
        .send_body(
            Method::PUT,
            "/repository/once/lib.jar",
            Auth::Admin,
            Body::from("plain text"),
            &[],
        )
        .await;
    assert_eq!(mismatch.status, StatusCode::BAD_REQUEST);
    assert!(
        mismatch
            .text()
            .contains("expected [application/java-archive]"),
        "{}",
        mismatch.text()
    );
    let zip = b"PK\x03\x04\x14\x00\x00\x00\x08\x00rest-of-a-zip";
    assert_eq!(
        put(&fixture, "/repository/once/lib.jar", zip, Auth::Admin).await,
        StatusCode::CREATED
    );
    let json_file = fixture
        .call(Method::HEAD, "/repository/once/lib.jar", Auth::Admin, None)
        .await;
    assert_eq!(
        json_file.headers[header::CONTENT_TYPE],
        "application/java-archive"
    );
}

#[tokio::test]
async fn content_permissions_follow_repository_views_and_selectors() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("team", "ALLOW")).await;
    assert_eq!(
        put(&fixture, "/repository/team/public/a.txt", b"a", Auth::Admin).await,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, "/repository/team/secret/b.txt", b"b", Auth::Admin).await,
        StatusCode::CREATED
    );

    let anonymous = fixture
        .call(
            Method::GET,
            "/repository/team/public/a.txt",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));

    fixture.add_user("reader", "reader-pass", &[]).await;
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/team/public/a.txt",
                Auth::Basic("reader", "reader-pass"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );

    let selector = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/content-selectors",
            Auth::Admin,
            Some(json!({"name": "public-only", "description": "", "expression": "format == \"raw\" and path =^ \"/public/\""})),
        )
        .await;
    assert!(selector.status.is_success(), "{}", selector.text());
    let privilege = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/privileges/repository-content-selector",
            Auth::Admin,
            Some(json!({"name": "team-public-read", "description": "", "actions": ["READ", "BROWSE"], "format": "raw", "repository": "team", "contentSelector": "public-only"})),
        )
        .await;
    assert!(privilege.status.is_success(), "{}", privilege.text());
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "public-readers", "name": "public-readers", "description": "", "privileges": ["team-public-read"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("selected", "selected-pass", &["public-readers"])
        .await;
    let selected = Auth::Basic("selected", "selected-pass");
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/team/public/a.txt",
                Auth::Basic("selected", "selected-pass"),
                None
            )
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/team/secret/b.txt",
                Auth::Basic("selected", "selected-pass"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(&fixture, "/repository/team/public/c.txt", b"c", selected).await,
        StatusCode::FORBIDDEN
    );

    let found = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search/assets?repository=team",
            Auth::Basic("selected", "selected-pass"),
            None,
        )
        .await
        .json();
    let paths: Vec<&str> = found["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["/public/a.txt"]);
}

#[tokio::test]
async fn components_assets_and_search_answer_like_the_reference() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("docs", "ALLOW")).await;
    for index in 0..12 {
        let path = format!("/repository/docs/guides/part-{index:02}.txt");
        let reply = fixture
            .send_body(
                Method::PUT,
                &path,
                Auth::Admin,
                Body::from(format!("part {index}")),
                &[],
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED);
    }
    for index in 0..90 {
        let reply = fixture
            .send_body(
                Method::PUT,
                &format!("/repository/docs/extra/x-{index:02}.txt"),
                Auth::Admin,
                Body::from(format!("extra {index}")),
                &[],
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED);
    }
    let first = fixture
        .call(
            Method::GET,
            "/service/rest/v1/components?repository=docs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let items = first["items"].as_array().unwrap();
    assert_eq!(items.len(), 100);
    assert_eq!(items[0]["group"], "/guides");
    assert_eq!(items[0]["name"], "/guides/part-00.txt");
    assert_eq!(items[0]["version"], "");
    let asset = &items[0]["assets"][0];
    assert_eq!(asset["path"], "/guides/part-00.txt");
    assert_eq!(
        asset["downloadUrl"],
        "http://localhost/repository/docs/guides/part-00.txt"
    );
    assert_eq!(asset["checksum"]["sha1"].as_str().unwrap().len(), 40);
    assert_eq!(asset["checksum"]["sha512"].as_str().unwrap().len(), 128);
    assert_eq!(asset["uploader"], "admin");
    assert_eq!(asset["blobStoreName"], "default");
    assert_eq!(asset["raw"], json!({}));
    let token = first["continuationToken"].as_str().unwrap();
    let second = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/components?repository=docs&continuationToken={token}"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(second["items"].as_array().unwrap().len(), 2);
    assert_eq!(second["continuationToken"], json!(null));

    let id = items[0]["id"].as_str().unwrap();
    let one = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(one.json()["name"], "/guides/part-00.txt");
    let asset_id = asset["id"].as_str().unwrap();
    assert_eq!(
        fixture
            .call(
                Method::GET,
                &format!("/service/rest/v1/assets/{asset_id}"),
                Auth::Admin,
                None
            )
            .await
            .json()["path"],
        "/guides/part-00.txt"
    );

    let search = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?name=*part-1*",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(search["items"].as_array().unwrap().len(), 2);
    let sha1 = asset["checksum"]["sha1"].as_str().unwrap();
    let by_checksum = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/search/assets?sha1={sha1}"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(by_checksum["items"].as_array().unwrap().len(), 1);
    let download = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search/assets/download?repository=docs&name=/guides/part-03.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(download.status, StatusCode::FOUND);
    assert_eq!(
        download.headers[header::LOCATION],
        "http://localhost/repository/docs/guides/part-03.txt"
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/service/rest/v1/search/assets/download?repository=docs",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                &format!("/service/rest/v1/components/{id}"),
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                &format!("/service/rest/v1/components/{id}"),
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/docs/guides/part-00.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/service/rest/v1/components?repository=nope",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/service/rest/v1/components?repository=docs",
                Auth::None,
                None
            )
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );

    let page = fixture
        .call(
            Method::GET,
            "/service/rest/repository/browse/docs/guides/",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("part-11.txt"), "{}", page.text());
    let root = fixture
        .call(
            Method::GET,
            "/service/rest/repository/browse/docs/",
            Auth::Admin,
            None,
        )
        .await;
    assert!(root.text().contains("guides/"), "{}", root.text());
    assert!(
        root.text()
            .contains("href=\"/service/rest/repository/browse/docs/guides/\""),
        "{}",
        root.text()
    );
    assert!(
        root.text().contains("href=\"/browse/docs\""),
        "{}",
        root.text()
    );
    assert!(
        page.text()
            .contains("href=\"/service/rest/repository/browse/docs/\">Parent Directory"),
        "{}",
        page.text()
    );
    let bare = fixture
        .call(
            Method::GET,
            "/service/rest/repository/browse/docs",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(bare.status, StatusCode::FOUND);
    assert_eq!(
        bare.headers[header::LOCATION],
        "/service/rest/repository/browse/docs/"
    );

    let browser = [
        ("accept", "text/html,application/xhtml+xml,*/*;q=0.8"),
        ("sec-fetch-mode", "navigate"),
    ];
    for address in ["/repository/docs", "/repository/docs/guides/"] {
        let visit = fixture
            .send(Method::GET, address, Auth::Admin, None, &browser)
            .await;
        assert_eq!(visit.status, StatusCode::NOT_FOUND, "{address}");
        assert!(
            visit.headers[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html"),
            "{address}"
        );
        assert!(
            visit
                .text()
                .contains("/service/rest/repository/browse/docs/"),
            "{}",
            visit.text()
        );
    }
    fixture.add_user("stranger", "stranger-pass", &[]).await;
    let stranger = fixture
        .send(
            Method::GET,
            "/repository/docs/",
            Auth::Basic("stranger", "stranger-pass"),
            None,
            &browser,
        )
        .await;
    assert!(!stranger.text().contains("<html"), "{}", stranger.text());
    let tool = fixture
        .call(Method::GET, "/repository/docs/", Auth::Admin, None)
        .await;
    assert_eq!(tool.status, StatusCode::NOT_FOUND);
    assert_eq!(tool.text(), "You can't browse this way");
    let file = fixture
        .send(
            Method::GET,
            "/repository/docs/guides/missing.txt",
            Auth::Admin,
            None,
            &browser,
        )
        .await;
    assert_eq!(file.status, StatusCode::NOT_FOUND);
    assert!(!file.text().contains("<html"), "{}", file.text());
}

#[tokio::test]
async fn uploads_through_the_components_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("uploads", "ALLOW")).await;
    let boundary = "XBOUNDARYX";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"raw.directory\"\r\n\r\n/up\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"raw.asset1\"; filename=\"local.txt\"\r\nContent-Type: text/plain\r\n\r\nfirst file\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"raw.asset1.filename\"\r\n\r\none.txt\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"raw.asset2\"; filename=\"two.txt\"\r\nContent-Type: text/plain\r\n\r\nsecond file\r\n\
         --{boundary}--\r\n"
    );
    let reply = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=uploads",
            Auth::Admin,
            Body::from(body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={boundary}"),
            )],
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/uploads/up/one.txt",
                Auth::Admin,
                None
            )
            .await
            .text(),
        "first file"
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/uploads/up/two.txt",
                Auth::Admin,
                None
            )
            .await
            .text(),
        "second file"
    );
    let spec = fixture
        .call(
            Method::GET,
            "/service/rest/v1/formats/raw/upload-specs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(spec["format"], "raw");
    assert_eq!(spec["componentFields"][0]["name"], "directory");
    let leftovers: Vec<_> = std::fs::read_dir(fixture.data.path().join("tmp"))
        .unwrap()
        .collect();
    assert!(leftovers.is_empty(), "uploads leave no temporary files");
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<AtomicUsize>,
    files: Arc<HashMap<&'static str, &'static str>>,
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.hits.fetch_add(1, Ordering::SeqCst);
    match upstream.files.get(path.as_str()) {
        Some(body) => {
            let etag = format!("\"{}\"", body.len());
            if headers
                .get(header::IF_NONE_MATCH)
                .and_then(|v| v.to_str().ok())
                == Some(etag.as_str())
            {
                return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
            }
            (
                StatusCode::OK,
                [
                    (header::ETAG, etag),
                    (header::CONTENT_TYPE, "text/plain".to_owned()),
                ],
                *body,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start_upstream(files: &[(&'static str, &'static str)]) -> (SocketAddr, Upstream) {
    let upstream = Upstream {
        hits: Arc::default(),
        files: Arc::new(files.iter().copied().collect()),
    };
    let app = Router::new()
        .route("/{*path}", get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, upstream)
}

async fn allow_loopback(fixture: &Fixture) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

fn proxy_body(name: &str, remote: &str, max_age: i64) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
        "proxy": {"remoteUrl": remote, "contentMaxAge": max_age, "metadataMaxAge": max_age},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": true},
        "raw": {"contentDisposition": "ATTACHMENT"}
    })
}

#[tokio::test]
async fn proxies_cache_and_revalidate() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (address, upstream) = start_upstream(&[("tools/tool.txt", "tool v1")]).await;
    create(
        &fixture,
        "proxy",
        proxy_body("mirror", &format!("http://{address}/"), 1440),
    )
    .await;

    let first = fixture
        .call(
            Method::GET,
            "/repository/mirror/tools/tool.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        (first.status, first.text()),
        (StatusCode::OK, "tool v1".to_owned())
    );
    assert_eq!(first.headers[header::CONTENT_DISPOSITION], "attachment");
    let second = fixture
        .call(
            Method::GET,
            "/repository/mirror/tools/tool.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(second.text(), "tool v1");
    assert_eq!(
        upstream.hits.load(Ordering::SeqCst),
        1,
        "the second read comes from the cache"
    );

    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/mirror/missing.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/mirror/missing.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        upstream.hits.load(Ordering::SeqCst),
        2,
        "the negative cache answers the second miss"
    );

    let invalidate = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/mirror/invalidate-cache",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(invalidate.status, StatusCode::NO_CONTENT);
    let revalidated = fixture
        .call(
            Method::GET,
            "/repository/mirror/tools/tool.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(revalidated.text(), "tool v1");
    assert_eq!(
        upstream.hits.load(Ordering::SeqCst),
        3,
        "a stale copy is checked with the remote"
    );

    let components = fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=mirror",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(components["items"][0]["path"], "/tools/tool.txt");
    assert!(components["items"][0]["lastVerified"].is_string());
}

#[tokio::test]
async fn proxies_refuse_internal_addresses_by_default() {
    let fixture = ready().await;
    let (address, upstream) = start_upstream(&[("x.txt", "x")]).await;
    let refused = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/proxy",
            Auth::Admin,
            Some(proxy_body("inside", &format!("http://{address}/"), 1440)),
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert_eq!(refused.json()[0]["id"], "PARAMETER remoteUrl");
    assert_eq!(
        refused.json()[0]["message"],
        "Proxy URL blocked: Host resolves to private/local IP address(es): 127.0.0.1 (loopback address). To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection."
    );
    allow_loopback(&fixture).await;
    create(
        &fixture,
        "proxy",
        proxy_body("inside", &format!("http://{address}/"), 1440),
    )
    .await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection::default(),
    )
    .await
    .unwrap();
    let reply = fixture
        .call(Method::GET, "/repository/inside/x.txt", Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    assert_eq!(upstream.hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn groups_read_through_their_members_in_order() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (address, _upstream) = start_upstream(&[
        ("shared.txt", "from the remote"),
        ("remote-only.txt", "remote"),
    ])
    .await;
    create(&fixture, "hosted", hosted("local", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy_body("remote", &format!("http://{address}/"), 1440),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "everything",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["local", "remote"]}
        }),
    )
    .await;
    assert_eq!(
        put(
            &fixture,
            "/repository/local/shared.txt",
            b"from the hosted",
            Auth::Admin
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/everything/shared.txt",
                Auth::Admin,
                None
            )
            .await
            .text(),
        "from the hosted"
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/everything/remote-only.txt",
                Auth::Admin,
                None
            )
            .await
            .text(),
        "remote"
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/everything/none.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .call(
                Method::PUT,
                "/repository/everything/new.txt",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::METHOD_NOT_ALLOWED
    );

    fixture
        .add_user("group-reader", "group-pass", &["group-readers"])
        .await;
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "group-readers", "name": "group-readers", "description": "", "privileges": ["as-repository-view-raw-everything-read"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture.state.security().reload().await.unwrap();
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/everything/shared.txt",
                Auth::Basic("group-reader", "group-pass"),
                None
            )
            .await
            .status,
        StatusCode::OK,
        "reading a group needs the group's permission only"
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/local/shared.txt",
                Auth::Basic("group-reader", "group-pass"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn deleting_a_repository_removes_its_content_and_counts_downloads() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("short-lived", "ALLOW")).await;
    assert_eq!(
        put(&fixture, "/repository/short-lived/a.txt", b"a", Auth::Admin).await,
        StatusCode::CREATED
    );
    for _ in 0..3 {
        fixture
            .call(
                Method::GET,
                "/repository/short-lived/a.txt",
                Auth::Admin,
                None,
            )
            .await;
    }
    fixture.state.content().flush_downloads().await.unwrap();
    let assets = fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=short-lived",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(assets["items"][0]["downloadCount"], 3);
    assert!(assets["items"][0]["lastDownloaded"].is_string());

    let metrics_before = fixture
        .state
        .blob_stores()
        .metrics("default")
        .totals()
        .await
        .unwrap();
    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                "/service/rest/v1/repositories/short-lived",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let metrics_after = fixture
        .state
        .blob_stores()
        .metrics("default")
        .totals()
        .await
        .unwrap();
    assert_eq!(metrics_after.blob_count + 1, metrics_before.blob_count);
}
