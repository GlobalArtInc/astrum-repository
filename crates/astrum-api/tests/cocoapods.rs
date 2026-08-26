mod support;

use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BROWSE: &str = "/service/rest/internal/ui/browse";

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Arc<Mutex<HashMap<String, usize>>>,
    authorizations: Arc<Mutex<Vec<(String, String)>>>,
    mirror: Arc<Mutex<Option<SocketAddr>>>,
}

impl Upstream {
    fn put(&self, path: &str, body: impl Into<Vec<u8>>) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_owned(), body.into());
    }

    fn hits(&self, path: &str) -> usize {
        self.hits.lock().unwrap().get(path).copied().unwrap_or(0)
    }

    fn note(&self, host: &str, path: &str, headers: &HeaderMap) {
        *self
            .hits
            .lock()
            .unwrap()
            .entry(path.to_owned())
            .or_default() += 1;
        if let Some(value) = headers.get(header::AUTHORIZATION) {
            self.authorizations
                .lock()
                .unwrap()
                .push((host.to_owned(), value.to_str().unwrap().to_owned()));
        }
    }
}

async fn cdn(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.note("cdn", &path, &headers);
    if path.starts_with("Specs/") {
        let mirror = upstream.mirror.lock().unwrap().unwrap();
        return (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, format!("http://{mirror}/cocoa/{path}"))],
        )
            .into_response();
    }
    match upstream.files.lock().unwrap().get(&path) {
        Some(body) => (
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            body.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn mirror(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.note("mirror", &path, &headers);
    if path.starts_with("archives/")
        && headers
            .get("x-astrum-fixture")
            .and_then(|v| v.to_str().ok())
            != Some("archive")
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    match upstream.files.lock().unwrap().get(&path) {
        Some(body) => body.clone().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn targz(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (name, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(&mut header, name, *data).unwrap();
    }
    let mut encoder = archive.into_inner().unwrap();
    encoder.flush().unwrap();
    encoder.finish().unwrap()
}

async fn listener() -> (tokio::net::TcpListener, SocketAddr) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    (listener, address)
}

struct Setup {
    fixture: Fixture,
    upstream: Upstream,
    cdn: SocketAddr,
    archive: Vec<u8>,
}

const KIT_SPEC: &str = "/Specs/3/f/4/AstrumKit/1.2.0/AstrumKit.podspec.json";
const KIT_ARCHIVE: &str = "/pods/AstrumKit/1.2.0/kit-1.2.0.tar.gz";

async fn setup() -> Setup {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_cocoapods::CocoaPods));
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
    let upstream = Upstream::default();
    let (cdn_listener, cdn) = listener().await;
    let (mirror_listener, mirror_address) = listener().await;
    *upstream.mirror.lock().unwrap() = Some(mirror_address);
    let cdn_app = Router::new()
        .route("/{*path}", routing::get(self::cdn))
        .with_state(upstream.clone());
    let mirror_app = Router::new()
        .route("/{*path}", routing::get(self::mirror))
        .with_state(upstream.clone());
    tokio::spawn(async move { axum::serve(cdn_listener, cdn_app).await.unwrap() });
    tokio::spawn(async move { axum::serve(mirror_listener, mirror_app).await.unwrap() });

    assert_eq!(
        astrum_format_cocoapods::podspec::podspec_path(
            "AstrumKit",
            "1.2.0",
            &astrum_format_cocoapods::podspec::DEFAULT_PREFIX_LENGTHS
        ),
        KIT_SPEC
    );
    let archive = targz(&[
        ("kit-1.2.0/LICENSE", b"MIT\n"),
        ("kit-1.2.0/Sources/Kit.swift", b"public let kit = 1\n"),
    ]);
    upstream.put(
        "CocoaPods-version.yml",
        "---\nmin: 1.0.0\nlast: 1.17.0\nprefix_lengths:\n- 1\n- 1\n- 1\n",
    );
    upstream.put("all_pods.txt", "AstrumGit\nAstrumKit\n");
    upstream.put("all_pods_versions_3_f_4.txt", "AstrumKit/1.1.0/1.2.0\n");
    upstream.put(
        "deprecated_podspecs.txt",
        "Specs/3/f/4/AstrumKit/1.1.0/AstrumKit.podspec.json\n",
    );
    let spec = format!(
        "{{\n  \"name\": \"AstrumKit\",\n  \"version\": \"1.2.0\",\n  \"summary\": \"A kit of the tests.\",\n  \"source\": {{\n    \"http\": \"http://{mirror_address}/archives/v1.2.0/kit-1.2.0.tar.gz\",\n    \"headers\": [\n      \"X-Astrum-Fixture: archive\"\n    ],\n    \"flatten\": true\n  }},\n  \"source_files\": \"Sources/**/*.swift\"\n}}\n"
    );
    upstream.put(&format!("cocoa{KIT_SPEC}"), spec);
    upstream.put("archives/v1.2.0/kit-1.2.0.tar.gz", archive.clone());
    upstream.put(
        "cocoa/Specs/6/3/3/AstrumGit/5.0.0/AstrumGit.podspec.json",
        r#"{"name":"AstrumGit","version":"5.0.0","source":{"git":"https://github.invalid/astrum/git.git","tag":"5.0.0"}}"#,
    );
    Setup {
        fixture,
        upstream,
        cdn,
        archive,
    }
}

async fn create(fixture: &Fixture, name: &str, remote: &str, metadata_max_age: i64) {
    let reply = fixture
        .call(
            axum::http::Method::POST,
            "/service/rest/v1/repositories/cocoapods/proxy",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
                "negativeCache": {"enabled": true, "timeToLive": 1440},
                "httpClient": {"blocked": false, "autoBlock": true, "authentication": {"type": "username", "username": "cdn-user", "password": "cdn-secret"}},
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture
        .call(axum::http::Method::GET, path, Auth::Admin, None)
        .await
}

#[tokio::test]
async fn proxies_serve_the_cdn_and_fetch_http_sources_through_themselves() {
    let Setup {
        fixture,
        upstream,
        cdn,
        archive,
    } = setup().await;
    create(&fixture, "trunk", &format!("http://{cdn}/"), 1440).await;

    let version = get(&fixture, "/repository/trunk/CocoaPods-version.yml").await;
    assert_eq!(version.status, StatusCode::OK, "{}", version.text());
    assert!(version.text().contains("prefix_lengths"));
    for (path, expected) in [
        ("all_pods.txt", "AstrumGit\nAstrumKit\n"),
        ("all_pods_versions_3_f_4.txt", "AstrumKit/1.1.0/1.2.0\n"),
        (
            "deprecated_podspecs.txt",
            "Specs/3/f/4/AstrumKit/1.1.0/AstrumKit.podspec.json\n",
        ),
    ] {
        let reply = get(&fixture, &format!("/repository/trunk/{path}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.text(), expected, "{path}");
    }
    let etag = version.headers[header::ETAG].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            axum::http::Method::GET,
            "/repository/trunk/CocoaPods-version.yml",
            Auth::Admin,
            None,
            &[("if-none-match", etag.as_str())],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let spec = get(&fixture, &format!("/repository/trunk{KIT_SPEC}")).await;
    assert_eq!(spec.status, StatusCode::OK, "{}", spec.text());
    assert_eq!(spec.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        spec.text(),
        "{\n  \"name\": \"AstrumKit\",\n  \"version\": \"1.2.0\",\n  \"summary\": \"A kit of the tests.\",\n  \"source\": {\n    \"flatten\": true,\n    \"http\": \"http://localhost/repository/trunk/pods/AstrumKit/1.2.0/kit-1.2.0.tar.gz\"\n  },\n  \"source_files\": \"Sources/**/*.swift\"\n}\n"
    );
    let etag = spec.headers[header::ETAG].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            axum::http::Method::GET,
            &format!("/repository/trunk{KIT_SPEC}"),
            Auth::Admin,
            None,
            &[("if-none-match", etag.as_str())],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let git_path = "/Specs/6/3/3/AstrumGit/5.0.0/AstrumGit.podspec.json";
    let git = get(&fixture, &format!("/repository/trunk{git_path}")).await;
    assert_eq!(git.status, StatusCode::OK);
    assert_eq!(
        git.json()["source"],
        json!({"git": "https://github.invalid/astrum/git.git", "tag": "5.0.0"})
    );

    let first = get(&fixture, &format!("/repository/trunk{KIT_ARCHIVE}")).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert_eq!(first.body, archive);
    let again = get(&fixture, &format!("/repository/trunk{KIT_ARCHIVE}")).await;
    assert_eq!(again.body, archive);
    assert_eq!(upstream.hits("archives/v1.2.0/kit-1.2.0.tar.gz"), 1);
    assert_eq!(upstream.hits(&format!("cocoa{KIT_SPEC}")), 1);
    assert_eq!(
        get(
            &fixture,
            "/repository/trunk/pods/AstrumKit/1.2.0/other.tar.gz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/trunk/pods/AstrumGit/5.0.0/5.0.0.tar.gz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let authorizations = upstream.authorizations.lock().unwrap().clone();
    assert!(
        authorizations.iter().all(|(host, _)| host == "cdn"),
        "{authorizations:?}"
    );
    assert!(!authorizations.is_empty());

    let components = get(&fixture, "/service/rest/v1/components?repository=trunk")
        .await
        .json();
    let items = components["items"].as_array().unwrap();
    let kit = items
        .iter()
        .find(|item| item["name"] == "AstrumKit")
        .unwrap();
    assert_eq!(kit["format"], "cocoapods");
    assert_eq!(kit["version"], "1.2.0");
    let mut paths: Vec<&str> = kit["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap())
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        [
            "/Specs/3/f/4/AstrumKit/1.2.0/AstrumKit.podspec.json",
            "/pods/AstrumKit/1.2.0/kit-1.2.0.tar.gz"
        ]
    );
    let pod = kit["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"].as_str().unwrap().starts_with("/pods/"))
        .unwrap();
    assert_eq!(
        pod["cocoapods"]["sourceUrl"],
        format!(
            "http://{}/archives/v1.2.0/kit-1.2.0.tar.gz",
            upstream.mirror.lock().unwrap().unwrap()
        )
    );
    let search = get(
        &fixture,
        "/service/rest/v1/search?format=cocoapods&name=AstrumKit",
    )
    .await
    .json();
    assert_eq!(search["items"].as_array().unwrap().len(), 1);

    let root: Vec<(String, String)> = get(&fixture, &format!("{BROWSE}/trunk?path=/"))
        .await
        .json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert!(
        root.contains(&("folder".to_owned(), "AstrumKit".to_owned())),
        "{root:?}"
    );
    assert!(
        root.contains(&("asset".to_owned(), "all_pods.txt".to_owned())),
        "{root:?}"
    );
    let versions = get(&fixture, &format!("{BROWSE}/trunk?path=/AstrumKit/"))
        .await
        .json();
    assert_eq!(versions["children"][0]["kind"], "component");
    assert_eq!(versions["children"][0]["name"], "1.2.0");
    let files = get(&fixture, &format!("{BROWSE}/trunk?path=/AstrumKit/1.2.0/"))
        .await
        .json();
    let names: Vec<&str> = files["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["AstrumKit.podspec.json", "kit-1.2.0.tar.gz"]);

    let page = get(&fixture, "/repository/trunk/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("pod repo add-cdn trunk"));
    assert_eq!(
        get(&fixture, "/repository/trunk/other.txt").await.status,
        StatusCode::NOT_FOUND
    );
    let put = fixture
        .call(
            axum::http::Method::PUT,
            "/repository/trunk/all_pods.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED);
    let deleted = fixture
        .call(
            axum::http::Method::DELETE,
            &format!("/repository/trunk{KIT_ARCHIVE}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let refetched = get(&fixture, &format!("/repository/trunk{KIT_ARCHIVE}")).await;
    assert_eq!(refetched.body, archive);
    assert_eq!(upstream.hits("archives/v1.2.0/kit-1.2.0.tar.gz"), 2);
}

#[tokio::test]
async fn archives_resolve_without_an_earlier_podspec_and_metadata_follows_its_max_age() {
    let Setup {
        fixture,
        upstream,
        cdn,
        archive,
    } = setup().await;
    create(&fixture, "fresh", &format!("http://{cdn}"), 0).await;
    create(&fixture, "kept", &format!("http://{cdn}"), 1440).await;

    let direct = get(&fixture, &format!("/repository/fresh{KIT_ARCHIVE}")).await;
    assert_eq!(direct.status, StatusCode::OK, "{}", direct.text());
    assert_eq!(direct.body, archive);

    for name in ["fresh", "kept"] {
        assert_eq!(
            get(&fixture, &format!("/repository/{name}/all_pods.txt"))
                .await
                .text(),
            "AstrumGit\nAstrumKit\n"
        );
    }
    upstream.put("all_pods.txt", "AstrumGit\nAstrumKit\nAstrumNew\n");
    assert_eq!(
        get(&fixture, "/repository/fresh/all_pods.txt").await.text(),
        "AstrumGit\nAstrumKit\nAstrumNew\n"
    );
    assert_eq!(
        get(&fixture, "/repository/kept/all_pods.txt").await.text(),
        "AstrumGit\nAstrumKit\n"
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/kept/Specs/0/0/0/Missing/1.0/Missing.podspec.json"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_rest_api_keeps_cocoapods_to_proxies() {
    let Setup { fixture, cdn, .. } = setup().await;
    create(&fixture, "trunk", &format!("http://{cdn}/"), 1440).await;
    let read = get(
        &fixture,
        "/service/rest/v1/repositories/cocoapods/proxy/trunk",
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    let document = read.json();
    assert_eq!(document["format"], "cocoapods");
    assert_eq!(document["type"], "proxy");
    assert_eq!(document["proxy"]["remoteUrl"], format!("http://{cdn}/"));
    assert_eq!(document["url"], "http://localhost/repository/trunk");
    let mut changed = document.clone();
    changed["proxy"]["metadataMaxAge"] = json!(60);
    let updated = fixture
        .call(
            axum::http::Method::PUT,
            "/service/rest/v1/repositories/cocoapods/proxy/trunk",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(updated.status, StatusCode::NO_CONTENT, "{}", updated.text());
    assert_eq!(
        get(
            &fixture,
            "/service/rest/v1/repositories/cocoapods/proxy/trunk"
        )
        .await
        .json()["proxy"]["metadataMaxAge"],
        60
    );

    let missing = fixture
        .call(
            axum::http::Method::POST,
            "/service/rest/v1/repositories/cocoapods/proxy",
            Auth::Admin,
            Some(json!({
                "name": "broken",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": true, "timeToLive": 1440},
                "httpClient": {"blocked": false, "autoBlock": true},
            })),
        )
        .await;
    assert_eq!(
        missing.status,
        StatusCode::BAD_REQUEST,
        "{}",
        missing.text()
    );
    for kind in ["hosted", "group"] {
        let reply = fixture
            .call(
                axum::http::Method::POST,
                &format!("/service/rest/v1/repositories/cocoapods/{kind}"),
                Auth::Admin,
                Some(json!({
                    "name": format!("pods-{kind}"),
                    "online": true,
                    "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                    "group": {"memberNames": ["trunk"]},
                })),
            )
            .await;
        assert!(
            reply.status.is_client_error(),
            "{kind}: {} {}",
            reply.status,
            reply.text()
        );
    }

    let recipes: Value = get(&fixture, "/service/rest/internal/ui/repositories/recipes")
        .await
        .json();
    let ids: Vec<&str> = recipes
        .as_array()
        .unwrap()
        .iter()
        .map(|recipe| recipe["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"cocoapods-proxy"), "{ids:?}");
    assert!(!ids.contains(&"cocoapods-hosted"), "{ids:?}");

    fixture.add_user("reader", "reader-password", &[]).await;
    let refused = fixture
        .call(
            axum::http::Method::GET,
            "/repository/trunk/all_pods.txt",
            Auth::Basic("reader", "reader-password"),
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn pods_with_unusual_names_keep_their_names_through_the_proxy() {
    let Setup {
        fixture,
        upstream,
        cdn,
        archive,
    } = setup().await;
    create(&fixture, "trunk", &format!("http://{cdn}/"), 1440).await;
    let mirror = upstream.mirror.lock().unwrap().unwrap();
    let name = "!Proto Kit";
    let spec_path = astrum_format_cocoapods::podspec::podspec_path(
        name,
        "1.0",
        &astrum_format_cocoapods::podspec::DEFAULT_PREFIX_LENGTHS,
    );
    upstream.put(
        &format!("cocoa{spec_path}"),
        json!({"name": name, "version": "1.0", "source": {"http": format!("http://{mirror}/archives/proto%20kit.zip"), "headers": ["X-Astrum-Fixture: archive"]}}).to_string(),
    );
    upstream.put("archives/proto kit.zip", archive.clone());
    let encoded = spec_path.replace(' ', "%20");
    let spec = get(&fixture, &format!("/repository/trunk{encoded}")).await;
    assert_eq!(spec.status, StatusCode::OK, "{}", spec.text());
    let location = spec.json()["source"]["http"].as_str().unwrap().to_owned();
    assert_eq!(
        location,
        "http://localhost/repository/trunk/pods/!Proto%20Kit/1.0/proto_kit.zip"
    );
    let path = location.trim_start_matches("http://localhost");
    let downloaded = get(&fixture, path).await;
    assert_eq!(downloaded.status, StatusCode::OK, "{}", downloaded.text());
    assert_eq!(downloaded.body, archive);
    let components = get(&fixture, "/service/rest/v1/components?repository=trunk")
        .await
        .json();
    assert!(
        components["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == name && item["assets"].as_array().unwrap().len() == 2),
        "{components}"
    );
}
