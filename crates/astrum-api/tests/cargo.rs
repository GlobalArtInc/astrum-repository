mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_cargo::Cargo::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/cargo/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, policy: &str, require: bool) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
        "cargo": {"requireAuthentication": require}
    })
}

fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn archive(name: &str, version: &str) -> Vec<u8> {
    gzip(format!("{name}-{version}/Cargo.toml").as_bytes())
}

fn publish_body(metadata: &Value, archive: &[u8]) -> Vec<u8> {
    let json = serde_json::to_vec(metadata).unwrap();
    let mut body = Vec::new();
    body.extend_from_slice(&(json.len() as u32).to_le_bytes());
    body.extend_from_slice(&json);
    body.extend_from_slice(&(archive.len() as u32).to_le_bytes());
    body.extend_from_slice(archive);
    body
}

fn metadata(name: &str, version: &str) -> Value {
    json!({
        "name": name,
        "vers": version,
        "deps": [{"name": "serde", "version_req": "^1", "features": [], "optional": true,
                  "default_features": true, "target": null, "kind": "normal",
                  "registry": "https://github.com/rust-lang/crates.io-index"}],
        "features": {"default": [], "json": ["dep:serde"]},
        "authors": ["Astrum Tests"],
        "description": format!("The {name} crate"),
        "keywords": ["demo"],
        "categories": [],
        "license": "MIT",
    })
}

async fn publish(
    fixture: &Fixture,
    repository: &str,
    auth: Auth<'_>,
    name: &str,
    version: &str,
) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/api/v1/crates/new"),
            auth,
            Body::from(publish_body(
                &metadata(name, version),
                &archive(name, version),
            )),
            &[],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn lines(reply: &Reply) -> Vec<Value> {
    reply
        .text()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
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

#[tokio::test]
async fn crates_are_published_listed_downloaded_and_yanked() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("crates", "ALLOW_ONCE", false)).await;

    let reply = publish(&fixture, "crates", Auth::Admin, "astrum-demo", "0.1.0").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["warnings"]["other"], json!([]));
    let again = publish(&fixture, "crates", Auth::Admin, "astrum-demo", "0.1.0").await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(
        again.json()["errors"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("does not allow redeployment")
    );
    let clash = publish(&fixture, "crates", Auth::Admin, "Astrum-Demo", "0.2.0").await;
    assert_eq!(clash.status, StatusCode::BAD_REQUEST, "{}", clash.text());
    let reply = publish(&fixture, "crates", Auth::Admin, "astrum-demo", "0.2.0").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());

    let config = get(&fixture, "/repository/crates/config.json").await;
    assert_eq!(config.status, StatusCode::OK);
    assert_eq!(
        config.json()["dl"],
        "http://localhost/repository/crates/api/v1/crates"
    );
    assert_eq!(config.json()["auth-required"], true);

    let index = get(&fixture, "/repository/crates/as/tr/astrum-demo").await;
    assert_eq!(index.status, StatusCode::OK, "{}", index.text());
    let entries = lines(&index);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["vers"], "0.1.0");
    assert_eq!(entries[0]["deps"][0]["req"], "^1");
    assert_eq!(entries[0]["features2"], json!({"json": ["dep:serde"]}));
    let archive = archive("astrum-demo", "0.1.0");
    assert_eq!(entries[0]["cksum"], hex::encode(Sha256::digest(&archive)));
    let etag = index.headers[header::ETAG].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/crates/as/tr/astrum-demo",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    assert_eq!(
        get(&fixture, "/repository/crates/as/tr/missing")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let download = get(
        &fixture,
        "/repository/crates/api/v1/crates/astrum-demo/0.1.0/download",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, archive);

    let yank = fixture
        .call(
            Method::DELETE,
            "/repository/crates/api/v1/crates/astrum-demo/0.1.0/yank",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(yank.status, StatusCode::OK, "{}", yank.text());
    assert_eq!(yank.json(), json!({"ok": true}));
    let entries = lines(&get(&fixture, "/repository/crates/as/tr/astrum-demo").await);
    assert_eq!(entries[0]["yanked"], true);
    let unyank = fixture
        .call(
            Method::PUT,
            "/repository/crates/api/v1/crates/astrum-demo/0.1.0/unyank",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unyank.status, StatusCode::OK);
    let entries = lines(&get(&fixture, "/repository/crates/as/tr/astrum-demo").await);
    assert_eq!(entries[0]["yanked"], false);

    let missing = fixture
        .call(
            Method::DELETE,
            "/repository/crates/api/v1/crates/astrum-demo/9.9.9/yank",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json()["errors"][0]["detail"],
        "crate `astrum-demo` does not have a version `9.9.9`"
    );
    let owners = get(
        &fixture,
        "/repository/crates/api/v1/crates/astrum-demo/owners",
    )
    .await;
    assert_eq!(owners.json()["users"][0]["login"], "admin");
    let file = get(
        &fixture,
        "/repository/crates/crates/astrum-demo/0.1.0/astrum-demo-0.1.0.crate",
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert!(entries[0]["pubtime"].as_str().unwrap().ends_with('Z'));
    let search = get(
        &fixture,
        "/repository/crates/api/v1/crates?q=demo&per_page=5",
    )
    .await;
    assert_eq!(
        search.json(),
        json!({"crates": [{"name": "astrum-demo", "max_version": "0.2.0", "description": "The astrum-demo crate"}], "meta": {"total": 1}})
    );

    let components = get(&fixture, "/service/rest/v1/components?repository=crates").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0]["assets"][0]["path"],
        "/crates/astrum-demo/0.1.0/astrum-demo-0.1.0.crate"
    );
}

#[tokio::test]
async fn reads_follow_the_authentication_setting_and_tokens_work() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("open", "ALLOW_ONCE", false)).await;
    create(&fixture, "hosted", hosted("closed", "ALLOW_ONCE", true)).await;
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());

    let open = fixture
        .call(
            Method::GET,
            "/repository/open/config.json",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(open.status, StatusCode::OK);
    assert!(open.json().get("auth-required").is_none());
    let closed = fixture
        .call(
            Method::GET,
            "/repository/closed/config.json",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(closed.status, StatusCode::UNAUTHORIZED);
    let challenges: Vec<&str> = closed
        .headers
        .get_all(header::WWW_AUTHENTICATE)
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert!(challenges.contains(&"Cargo login_url=\"http://localhost/repository/closed/me\""));

    let anonymous_publish = publish(&fixture, "open", Auth::None, "demo", "1.0.0").await;
    assert_eq!(anonymous_publish.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous_publish.json()["errors"][0]["detail"].is_string());

    let me = fixture
        .call(Method::GET, "/repository/closed/me", Auth::Admin, None)
        .await;
    assert_eq!(me.status, StatusCode::CONFLICT);
    assert!(
        me.json()["errors"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("Cargo Bearer Token Realm is not active")
    );
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!(["AstrumAuthenticatingRealm", "CargoToken"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let me = fixture
        .call(Method::GET, "/repository/closed/me", Auth::Admin, None)
        .await;
    assert_eq!(me.status, StatusCode::OK);
    let token = me.text().trim().to_owned();
    assert!(token.starts_with("CargoToken."));
    assert_eq!(token.len(), "CargoToken.".len() + 36);
    let again = fixture
        .call(Method::GET, "/repository/closed/me", Auth::Admin, None)
        .await;
    assert_eq!(again.text().trim(), token);
    let anonymous = fixture
        .call(Method::GET, "/repository/closed/me", Auth::None, None)
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let published = fixture
        .send_body(
            Method::PUT,
            "/repository/closed/api/v1/crates/new",
            Auth::None,
            Body::from(publish_body(
                &metadata("demo", "1.0.0"),
                &archive("demo", "1.0.0"),
            )),
            &[("authorization", &token)],
        )
        .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    let read = fixture
        .send(
            Method::GET,
            "/repository/closed/3/d/dem",
            Auth::None,
            None,
            &[("authorization", &token)],
        )
        .await;
    assert_eq!(read.status, StatusCode::NOT_FOUND);
    let read = fixture
        .send(
            Method::GET,
            "/repository/closed/de/mo/demo",
            Auth::None,
            None,
            &[("authorization", &token)],
        )
        .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    let refused = fixture
        .send(
            Method::GET,
            "/repository/closed/de/mo/demo",
            Auth::None,
            None,
            &[(
                "authorization",
                "CargoToken.0000000000000000000000000000000000000000",
            )],
        )
        .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
}

type Seen = Arc<Mutex<Vec<(String, Option<String>)>>>;

#[derive(Clone, Default)]
struct Upstream {
    seen: Seen,
    address: Arc<Mutex<Option<SocketAddr>>>,
    archive: Arc<Vec<u8>>,
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.seen.lock().unwrap().push((
        format!("/{path}"),
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
    ));
    let address = upstream.address.lock().unwrap().unwrap();
    match path.as_str() {
        "index/config.json" => {
            Json(json!({"dl": format!("http://{address}/files/{{crate}}/{{crate}}-{{version}}.crate"), "api": format!("http://{address}")}))
                .into_response()
        }
        "index/re/mo/remote-demo" => {
            let checksum = hex::encode(Sha256::digest(upstream.archive.as_slice()));
            format!(
                "{}\n",
                json!({"name": "remote-demo", "vers": "1.0.0", "deps": [], "cksum": checksum, "features": {}, "yanked": false})
            )
            .into_response()
        }
        "files/remote-demo/remote-demo-1.0.0.crate" => upstream.archive.as_ref().clone().into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

use axum::Json;

async fn serve_upstream() -> Upstream {
    let upstream = Upstream {
        archive: Arc::new(archive("remote-demo", "1.0.0")),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    *upstream.address.lock().unwrap() = Some(listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    upstream
}

#[tokio::test]
async fn proxies_cache_the_remote_index_and_groups_merge_their_members() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let upstream = serve_upstream().await;
    let address = upstream.address.lock().unwrap().unwrap();
    create(&fixture, "hosted", hosted("local", "ALLOW_ONCE", false)).await;
    create(
        &fixture,
        "proxy",
        json!({
            "name": "remote",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/index"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true,
                           "authentication": {"type": "username", "username": "proxy-user", "password": "proxy-secret"}},
        }),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["local", "remote"], "writableMember": "local"},
        }),
    )
    .await;

    let index = get(&fixture, "/repository/remote/re/mo/remote-demo").await;
    assert_eq!(index.status, StatusCode::OK, "{}", index.text());
    assert_eq!(lines(&index)[0]["vers"], "1.0.0");
    let download = get(
        &fixture,
        "/repository/remote/api/v1/crates/remote-demo/1.0.0/download",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK, "{}", download.text());
    assert_eq!(download.body, *upstream.archive);
    let config = get(&fixture, "/repository/remote/config.json").await;
    assert_eq!(config.json()["api"], "http://localhost/repository/remote");
    let seen = upstream.seen.lock().unwrap().clone();
    assert!(
        seen.iter()
            .any(|(path, _)| path == "/files/remote-demo/remote-demo-1.0.0.crate")
    );
    assert!(seen.iter().all(|(_, authorization)| {
        authorization
            .as_deref()
            .is_some_and(|value| value.starts_with("Basic "))
    }));

    let reply = publish(&fixture, "all", Auth::Admin, "remote-demo", "2.0.0").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let local = lines(&get(&fixture, "/repository/local/re/mo/remote-demo").await);
    assert_eq!(local.len(), 1);
    let merged = lines(&get(&fixture, "/repository/all/re/mo/remote-demo").await);
    let versions: Vec<&str> = merged
        .iter()
        .map(|entry| entry["vers"].as_str().unwrap())
        .collect();
    assert_eq!(versions, ["2.0.0", "1.0.0"]);
    let through_group = get(
        &fixture,
        "/repository/all/api/v1/crates/remote-demo/1.0.0/download",
    )
    .await;
    assert_eq!(through_group.body, *upstream.archive);
    let config = get(&fixture, "/repository/all/config.json").await;
    assert_eq!(
        config.json()["dl"],
        "http://localhost/repository/all/api/v1/crates"
    );
    let proxy_publish = publish(&fixture, "remote", Auth::Admin, "other", "1.0.0").await;
    assert_eq!(proxy_publish.status, StatusCode::METHOD_NOT_ALLOWED);
}
