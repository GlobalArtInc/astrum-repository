mod support;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_rubygems::index::md5_hex;
use astrum_format_rubygems::testing::gem;
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "gemboundary0123456789";
const BROWSE: &str = "/service/rest/internal/ui/browse";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_rubygems::Rubygems));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/rubygems/{kind}"),
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
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy}
    })
}

fn basic(fixture: &Fixture) -> String {
    format!(
        "Basic {}",
        STANDARD.encode(format!("{ADMIN_USER}:{}", fixture.admin_password))
    )
}

async fn push(fixture: &Fixture, repository: &str, key: &str, data: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/v1/gems"),
            Auth::None,
            Body::from(data),
            &[
                ("authorization", key),
                ("content-type", "application/octet-stream"),
            ],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn text(fixture: &Fixture, path: &str) -> String {
    let reply = get(fixture, path).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.text()
}

fn gunzip(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(data)
        .read_to_end(&mut out)
        .unwrap();
    out
}

fn inflate(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data)
        .read_to_end(&mut out)
        .unwrap();
    out
}

fn marshal_array(data: &[u8]) -> usize {
    assert_eq!(&data[..3], b"\x04\x08[");
    match data[3] {
        0 => 0,
        small => small as usize - 5,
    }
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn versions_line<'a>(versions: &'a str, name: &str) -> Vec<&'a str> {
    versions
        .lines()
        .find(|line| line.split(' ').next() == Some(name))
        .unwrap_or_else(|| panic!("{name} is not in {versions}"))
        .split(' ')
        .collect()
}

fn form(file: (&str, &str, &[u8])) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
        file.0, file.1
    )
    .unwrap();
    body.extend_from_slice(file.2);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn upload(fixture: &Fixture, repository: &str, body: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await
}

#[tokio::test]
async fn hosted_repositories_serve_the_indexes_of_their_gems() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("gems", "ALLOW_ONCE")).await;
    let key = basic(&fixture);
    let hola = gem("hola", "1.0.0", "ruby", &[("dep", "~> 0.1")]);
    for (data, expected) in [
        (
            gem("dep", "0.1.0", "ruby", &[]),
            "Successfully registered gem: dep (0.1.0)",
        ),
        (hola.clone(), "Successfully registered gem: hola (1.0.0)"),
        (
            gem("hola", "1.1.0", "x86_64-linux", &[]),
            "Successfully registered gem: hola (1.1.0-x86_64-linux)",
        ),
        (
            gem("hola", "2.0.0.rc1", "ruby", &[]),
            "Successfully registered gem: hola (2.0.0.rc1)",
        ),
    ] {
        let reply = push(&fixture, "gems", &key, data).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        assert_eq!(reply.text(), expected);
    }
    let again = push(&fixture, "gems", &key, hola.clone()).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(
        again.text(),
        "Repushing of gem versions is not allowed: repository gems does not allow redeployment"
    );
    let garbage = push(&fixture, "gems", &key, b"not a gem at all".to_vec()).await;
    assert_eq!(garbage.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        garbage.text().starts_with("Not a gem file"),
        "{}",
        garbage.text()
    );
    let anonymous = fixture
        .send_body(
            Method::POST,
            "/repository/gems/api/v1/gems",
            Auth::None,
            Body::from(hola.clone()),
            &[],
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let info = text(&fixture, "/repository/gems/info/hola").await;
    assert_eq!(
        info,
        format!(
            "---\n1.0.0 dep:~> 0.1|checksum:{}\n1.1.0-x86_64-linux |checksum:{}\n2.0.0.rc1 |checksum:{}\n",
            sha256(&hola),
            sha256(&gem("hola", "1.1.0", "x86_64-linux", &[])),
            sha256(&gem("hola", "2.0.0.rc1", "ruby", &[]))
        )
    );
    let versions = get(&fixture, "/repository/gems/versions").await;
    assert_eq!(versions.status, StatusCode::OK);
    let body = versions.text();
    assert!(body.starts_with("created_at: "), "{body}");
    assert_eq!(
        versions_line(&body, "hola"),
        [
            "hola",
            "1.0.0,1.1.0-x86_64-linux,2.0.0.rc1",
            &md5_hex(info.as_bytes())
        ]
    );
    let etag = versions.headers[header::ETAG].to_str().unwrap().to_owned();
    assert_eq!(etag, format!("\"{}\"", md5_hex(body.as_bytes())));
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/gems/versions",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    assert_eq!(
        text(&fixture, "/repository/gems/names").await,
        "---\ndep\nhola\n"
    );
    let missing = get(&fixture, "/repository/gems/info/nope").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.text(), "gem nope not found");

    for (path, count) in [
        ("/specs.4.8.gz", 3),
        ("/latest_specs.4.8.gz", 3),
        ("/prerelease_specs.4.8.gz", 1),
    ] {
        let reply = get(&fixture, &format!("/repository/gems{path}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(marshal_array(&gunzip(&reply.body)), count, "{path}");
    }
    let quick = get(
        &fixture,
        "/repository/gems/quick/Marshal.4.8/hola-1.1.0-x86_64-linux.gemspec.rz",
    )
    .await;
    assert_eq!(quick.status, StatusCode::OK);
    let spec = inflate(&quick.body);
    assert!(spec.starts_with(b"\x04\x08u:\x17Gem::Specification"));
    assert!(spec.windows(13).any(|part| part == b"Gem::Platform"));
    assert_eq!(
        get(
            &fixture,
            "/repository/gems/quick/Marshal.4.8/hola-9.gemspec.rz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let dependencies = get(
        &fixture,
        "/repository/gems/api/v1/dependencies?gems=hola,dep",
    )
    .await;
    assert_eq!(dependencies.status, StatusCode::OK);
    assert_eq!(marshal_array(&dependencies.body), 4);
    assert_eq!(
        get(&fixture, "/repository/gems/api/v1/dependencies")
            .await
            .body,
        b"\x04\x08[\x00"
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/gems/api/v1/dependencies?gems=bad%20name"
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    let download = get(&fixture, "/repository/gems/gems/hola-1.0.0.gem").await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, hola);
    assert_eq!(download.headers[header::CONTENT_TYPE], "application/x-tar");
    let found = get(
        &fixture,
        "/service/rest/v1/search?repository=gems&rubygems.platform=x86_64-linux",
    )
    .await
    .json();
    assert_eq!(found["items"].as_array().unwrap().len(), 1);
    assert_eq!(found["items"][0]["name"], "hola");
    assert_eq!(found["items"][0]["version"], "1.1.0");
    assert_eq!(
        found["items"][0]["assets"][0]["rubygems"]["summary"],
        "hola summary"
    );
    let page = get(&fixture, "/repository/gems/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("RubyGems repository"));

    let yank = fixture
        .send_body(
            Method::DELETE,
            "/repository/gems/api/v1/gems/yank",
            Auth::None,
            Body::from("gem_name=hola&version=1.1.0&platform=x86_64-linux"),
            &[
                ("authorization", key.as_str()),
                ("content-type", "application/x-www-form-urlencoded"),
            ],
        )
        .await;
    assert_eq!(yank.status, StatusCode::OK, "{}", yank.text());
    assert_eq!(
        yank.text(),
        "Successfully deleted gem: hola (1.1.0-x86_64-linux)"
    );
    let again = fixture
        .send_body(
            Method::DELETE,
            "/repository/gems/api/v1/gems/yank?gem_name=hola&version=1.1.0&platform=x86_64-linux",
            Auth::Admin,
            Body::empty(),
            &[],
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert_eq!(
        again.text(),
        "The version 1.1.0-x86_64-linux of hola does not exist."
    );
    let body = text(&fixture, "/repository/gems/versions").await;
    assert_eq!(versions_line(&body, "hola")[1], "1.0.0,2.0.0.rc1");

    let components = get(&fixture, "/service/rest/v1/search?repository=gems&name=dep")
        .await
        .json();
    let id = components["items"][0]["id"].as_str().unwrap().to_owned();
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    assert_eq!(
        text(&fixture, "/repository/gems/names").await,
        "---\nhola\n"
    );
    assert_eq!(
        get(&fixture, "/repository/gems/info/dep").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn api_keys_are_kept_per_user_and_authenticate_pushes() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("gems", "ALLOW")).await;
    let path = "/repository/gems/api/v1/api_key";
    let inactive = get(&fixture, path).await;
    assert_eq!(inactive.status, StatusCode::CONFLICT);
    assert!(inactive.text().contains("RubyGems API Key Realm"));
    let anonymous = fixture.call(Method::GET, path, Auth::None, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    let realms = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!([
                "AstrumAuthenticatingRealm",
                "AccessTokenRealm",
                "RubyGemsApiKey"
            ])),
        )
        .await;
    assert_eq!(realms.status, StatusCode::NO_CONTENT, "{}", realms.text());

    let key = text(&fixture, path).await;
    assert!(key.starts_with("RubyGemsApiKey."), "{key}");
    assert_eq!(text(&fixture, path).await, key);
    let signed_in = fixture
        .send_body(
            Method::POST,
            path,
            Auth::Admin,
            Body::from("name=laptop&index_rubygems=true"),
            &[("content-type", "application/x-www-form-urlencoded")],
        )
        .await;
    assert_eq!(signed_in.status, StatusCode::OK);
    assert_eq!(signed_in.text(), key);

    let pushed = push(&fixture, "gems", &key, gem("keyed", "1.0.0", "ruby", &[])).await;
    assert_eq!(pushed.status, StatusCode::OK, "{}", pushed.text());
    let bearer = push(
        &fixture,
        "gems",
        &format!("Bearer {key}"),
        gem("keyed", "1.0.1", "ruby", &[]),
    )
    .await;
    assert_eq!(bearer.status, StatusCode::OK, "{}", bearer.text());

    let revoked = fixture.call(Method::DELETE, path, Auth::Admin, None).await;
    assert_eq!(revoked.status, StatusCode::OK);
    assert_eq!(revoked.text(), "RubyGems API key revoked");
    let none = fixture.call(Method::DELETE, path, Auth::Admin, None).await;
    assert_eq!(none.status, StatusCode::NOT_FOUND);
    let refused = push(&fixture, "gems", &key, gem("keyed", "1.0.2", "ruby", &[])).await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
    let fresh = text(&fixture, path).await;
    assert_ne!(fresh, key);
}

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    match upstream.files.lock().unwrap().get(&path) {
        Some(data) => data.clone().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_upstream(upstream: Upstream) -> SocketAddr {
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

#[tokio::test]
async fn proxies_cache_and_groups_merge_their_members() {
    let fixture = ready().await;
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
    let key = basic(&fixture);
    create(&fixture, "hosted", hosted("origin", "ALLOW")).await;
    let far = gem("hola", "1.0.0", "ruby", &[("dep", ">= 0.1")]);
    push(&fixture, "origin", &key, far.clone()).await;
    push(&fixture, "origin", &key, gem("dep", "0.1.0", "ruby", &[])).await;
    let upstream = Upstream::default();
    let mut files = HashMap::new();
    for path in [
        "versions",
        "names",
        "info/hola",
        "info/dep",
        "specs.4.8.gz",
        "latest_specs.4.8.gz",
        "prerelease_specs.4.8.gz",
        "quick/Marshal.4.8/hola-1.0.0.gemspec.rz",
        "gems/hola-1.0.0.gem",
        "gems/dep-0.1.0.gem",
    ] {
        let reply = get(&fixture, &format!("/repository/origin/{path}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        files.insert(format!("remote/{path}"), reply.body);
    }
    *upstream.files.lock().unwrap() = files.clone();
    let address = serve_upstream(upstream).await;
    create(
        &fixture,
        "proxy",
        json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/remote/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1},
            "httpClient": {"blocked": false, "autoBlock": false}
        }),
    )
    .await;
    let versions = get(&fixture, "/repository/mirror/versions").await;
    assert_eq!(versions.status, StatusCode::OK);
    assert_eq!(versions.body, files["remote/versions"]);
    assert_eq!(
        versions.headers[header::ETAG].to_str().unwrap(),
        format!("\"{}\"", md5_hex(&versions.body))
    );
    assert_eq!(
        get(&fixture, "/repository/mirror/info/hola").await.body,
        files["remote/info/hola"]
    );
    let download = get(&fixture, "/repository/mirror/gems/hola-1.0.0.gem").await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, far);
    let found = get(
        &fixture,
        "/service/rest/v1/search?repository=mirror&name=hola",
    )
    .await
    .json();
    assert_eq!(found["items"][0]["version"], "1.0.0");
    assert_eq!(
        found["items"][0]["assets"][0]["rubygems"]["platform"],
        "ruby"
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/mirror/quick/Marshal.4.8/hola-1.0.0.gemspec.rz"
        )
        .await
        .body,
        files["remote/quick/Marshal.4.8/hola-1.0.0.gemspec.rz"]
    );
    assert_eq!(
        get(&fixture, "/repository/mirror/gems/missing-1.0.gem")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let refused = push(&fixture, "mirror", &key, gem("x", "1.0.0", "ruby", &[])).await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        refused.text(),
        "Repository mirror is a proxy repository; push gems to a rubygems hosted repository instead"
    );

    create(&fixture, "hosted", hosted("local", "ALLOW")).await;
    let newer = gem("hola", "2.0.0", "ruby", &[]);
    push(&fixture, "local", &key, newer.clone()).await;
    push(&fixture, "local", &key, gem("mine", "1.0.0", "ruby", &[])).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["local", "mirror"]}
        }),
    )
    .await;
    let merged = get(&fixture, "/repository/all/versions").await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    let body = merged.text();
    let info = text(&fixture, "/repository/all/info/hola").await;
    assert_eq!(
        versions_line(&body, "hola"),
        ["hola", "1.0.0,2.0.0", &md5_hex(info.as_bytes())]
    );
    assert_eq!(versions_line(&body, "mine")[1], "1.0.0");
    assert_eq!(versions_line(&body, "dep")[1], "0.1.0");
    let cached = astrum_content::store::find(fixture.state.db().conn(), "all", "/versions")
        .await
        .unwrap()
        .unwrap();
    assert!(cached.attributes["rubygems"]["signature"].is_string());
    let again = get(&fixture, "/repository/all/versions").await;
    assert_eq!(again.headers[header::ETAG], merged.headers[header::ETAG]);
    assert_eq!(
        text(&fixture, "/repository/all/names").await,
        "---\ndep\nhola\nmine\n"
    );
    let specs = get(&fixture, "/repository/all/specs.4.8.gz").await;
    assert_eq!(marshal_array(&gunzip(&specs.body)), 4);
    let latest = get(&fixture, "/repository/all/latest_specs.4.8.gz").await;
    assert_eq!(marshal_array(&gunzip(&latest.body)), 3);
    for full in ["hola-2.0.0", "hola-1.0.0"] {
        let quick = get(
            &fixture,
            &format!("/repository/all/quick/Marshal.4.8/{full}.gemspec.rz"),
        )
        .await;
        assert_eq!(quick.status, StatusCode::OK, "{full}");
    }
    assert_eq!(
        get(&fixture, "/repository/all/gems/hola-2.0.0.gem")
            .await
            .body,
        newer
    );
    assert_eq!(
        get(&fixture, "/repository/all/gems/hola-1.0.0.gem")
            .await
            .body,
        far
    );
    let dependencies = get(&fixture, "/repository/all/api/v1/dependencies?gems=hola").await;
    assert_eq!(marshal_array(&dependencies.body), 2);

    push(&fixture, "local", &key, gem("hola", "3.0.0", "ruby", &[])).await;
    let changed = text(&fixture, "/repository/all/versions").await;
    assert_eq!(versions_line(&changed, "hola")[1], "1.0.0,2.0.0,3.0.0");
}

#[tokio::test]
async fn gems_upload_through_the_components_api_and_show_in_the_tree() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("gems", "ALLOW_ONCE")).await;
    let spec = get(&fixture, "/service/rest/v1/formats/rubygems/upload-specs")
        .await
        .json();
    assert_eq!(
        spec,
        json!({
            "format": "rubygems",
            "multipleUpload": false,
            "componentFields": [],
            "assetFields": [{
                "name": "asset", "type": "FILE", "description": null, "optional": false,
                "group": null, "groupHelpText": null, "options": null
            }]
        })
    );
    let data = gem("uploaded", "1.2.3", "ruby", &[]);
    let reply = upload(
        &fixture,
        "gems",
        form(("rubygems.asset", "uploaded-1.2.3.gem", &data)),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let twice = upload(
        &fixture,
        "gems",
        form(("rubygems.asset", "uploaded-1.2.3.gem", &data)),
    )
    .await;
    assert_eq!(twice.status, StatusCode::BAD_REQUEST);
    let broken = upload(
        &fixture,
        "gems",
        form(("rubygems.asset", "broken.gem", b"nothing")),
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    assert!(
        text(&fixture, "/repository/gems/versions")
            .await
            .contains("uploaded 1.2.3 ")
    );

    let root = get(&fixture, &format!("{BROWSE}/gems?path=/")).await.json();
    let names: Vec<(&str, &str)> = root["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap(),
                node["name"].as_str().unwrap(),
            )
        })
        .collect();
    assert!(names.contains(&("folder", "uploaded")), "{names:?}");
    assert!(names.contains(&("asset", "versions")), "{names:?}");
    let version = get(&fixture, &format!("{BROWSE}/gems?path=/uploaded/"))
        .await
        .json();
    assert_eq!(version["children"][0]["kind"], "component");
    assert_eq!(version["children"][0]["name"], "1.2.3");
    let files = get(&fixture, &format!("{BROWSE}/gems?path=/uploaded/1.2.3/"))
        .await
        .json();
    assert_eq!(files["children"][0]["name"], "uploaded-1.2.3.gem");
}
