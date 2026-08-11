mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_content::zip;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "composerboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_composer::Composer));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/composer/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW_ONCE"}
    })
}

fn package(name: &str, folder: &str) -> Vec<u8> {
    let manifest = json!({"name": name, "description": "A test package", "require": {"php": ">=8.1"}, "autoload": {"psr-4": {"Test\\": "src/"}}}).to_string();
    let manifest_path = format!("{folder}/composer.json");
    let source_path = format!("{folder}/src/Hello.php");
    zip::write(&[
        (manifest_path.as_str(), manifest.as_bytes()),
        (source_path.as_str(), b"<?php\n"),
    ])
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn form(fields: &[(&str, &str)], file: (&str, &[u8])) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        )
        .unwrap();
    }
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"package.zip\"\r\nContent-Type: application/zip\r\n\r\n",
        file.0
    )
    .unwrap();
    body.extend_from_slice(file.1);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

fn expanded(document: &Value, package: &str) -> Vec<Value> {
    astrum_format_composer::metadata::entries_of(document, package)
}

#[tokio::test]
async fn hosted_packages_answer_packages_json_and_p2_metadata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("php")).await;
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/php/packages/upload/acme/hello/1.0.0",
            Auth::Admin,
            Body::from(package("acme/hello", "hello-1.0.0")),
            &[("content-type", "application/zip")],
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());
    let again = fixture
        .send_body(
            Method::PUT,
            "/repository/php/packages/upload/acme/hello/1.0.0",
            Auth::Admin,
            Body::from(package("acme/hello", "hello-1.0.0")),
            &[("content-type", "application/zip")],
        )
        .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    let not_composer = fixture
        .send_body(
            Method::PUT,
            "/repository/php/packages/upload/acme/hello/2.0.0",
            Auth::Admin,
            Body::from(zip::write(&[("readme.txt", b"no manifest")])),
            &[("content-type", "application/zip")],
        )
        .await;
    assert_eq!(not_composer.status, StatusCode::BAD_REQUEST);
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=php",
            Auth::Admin,
            Body::from(form(
                &[
                    ("composer.asset.vendor", "acme"),
                    ("composer.asset.project", "hello"),
                    ("composer.asset.version", "1.1.0"),
                ],
                ("composer.asset", &package("acme/hello", "x")),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());

    let root = get(&fixture, "/repository/php/packages.json").await.json();
    assert_eq!(
        root["metadata-url"],
        "http://localhost/repository/php/p2/%package%.json"
    );
    assert_eq!(
        root["providers-url"],
        "http://localhost/repository/php/p/%package%.json"
    );
    assert_eq!(root["available-packages"], json!(["acme/hello"]));
    assert_eq!(
        get(&fixture, "/repository/php/packages/list.json")
            .await
            .json(),
        json!({"packageNames": ["acme/hello"]})
    );
    let p2 = get(&fixture, "/repository/php/p2/acme/hello.json")
        .await
        .json();
    assert_eq!(p2["minified"], "composer/2.0");
    let entries = expanded(&p2, "acme/hello");
    let versions: Vec<&str> = entries
        .iter()
        .map(|entry| entry["version"].as_str().unwrap())
        .collect();
    assert_eq!(versions, ["1.1.0", "1.0.0"]);
    assert_eq!(entries[1]["name"], "acme/hello");
    assert_eq!(entries[1]["require"], json!({"php": ">=8.1"}));
    assert_eq!(
        entries[1]["dist"]["url"],
        "http://localhost/repository/php/acme/hello/1.0.0/acme-hello-1.0.0.zip"
    );
    assert_eq!(entries[1]["dist"]["type"], "zip");
    assert!(entries[1]["dist"]["shasum"].as_str().unwrap().len() == 40);
    let provider = get(&fixture, "/repository/php/p/acme/hello.json")
        .await
        .json();
    assert_eq!(
        provider["packages"]["acme/hello"]["1.0.0"]["version"],
        "1.0.0"
    );
    assert_eq!(
        get(&fixture, "/repository/php/p2/acme/hello~dev.json")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let dist = get(
        &fixture,
        "/repository/php/acme/hello/1.0.0/acme-hello-1.0.0.zip",
    )
    .await;
    assert_eq!(dist.status, StatusCode::OK);
    assert_eq!(dist.body, package("acme/hello", "hello-1.0.0"));

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/php/acme/hello/1.1.0/acme-hello-1.1.0.zip",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let p2 = get(&fixture, "/repository/php/p2/acme/hello.json")
        .await
        .json();
    assert_eq!(expanded(&p2, "acme/hello").len(), 1);
    let stored = get(&fixture, "/service/rest/v1/assets?repository=php")
        .await
        .json();
    let paths: Vec<&str> = stored["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"/p2/acme/hello.json"), "{paths:?}");
    assert!(paths.contains(&"/p/acme/hello.json"), "{paths:?}");
}

#[derive(Clone, Default)]
struct Upstream {
    address: Arc<Mutex<Option<SocketAddr>>>,
    archive: Arc<Vec<u8>>,
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    let address = upstream.address.lock().unwrap().unwrap();
    match path.as_str() {
        "packages.json" => json!({"metadata-url": "/p2/%package%.json", "notify-batch": "x"})
            .to_string()
            .into_response(),
        "p2/psr/log.json" => json!({
            "minified": "composer/2.0",
            "packages": {"psr/log": [
                {"name": "psr/log", "version": "3.0.2", "description": "PSR-3", "dist": {"url": format!("http://{address}/zipball/302"), "type": "zip", "reference": "abc", "shasum": ""}},
                {"version": "3.0.1", "dist": {"url": format!("http://{address}/zipball/301"), "type": "zip", "reference": "def", "shasum": ""}}
            ]}
        })
        .to_string()
        .into_response(),
        "zipball/301" => upstream.archive.as_ref().clone().into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::test]
async fn proxies_rewrite_dist_addresses_and_groups_merge_members() {
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
    let upstream = Upstream {
        archive: Arc::new(package("psr/log", "log-3.0.1")),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    *upstream.address.lock().unwrap() = Some(address);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "packagist",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "packagist"]},
        }),
    )
    .await;
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/own/packages/upload/psr/log/9.0.0",
            Auth::Admin,
            Body::from(package("psr/log", "log")),
            &[("content-type", "application/zip")],
        )
        .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.text());

    let root = get(&fixture, "/repository/packagist/packages.json")
        .await
        .json();
    assert_eq!(
        root["list"],
        "http://localhost/repository/packagist/packages/list.json"
    );
    let p2 = get(&fixture, "/repository/packagist/p2/psr/log.json")
        .await
        .json();
    let entries = expanded(&p2, "psr/log");
    assert_eq!(
        entries[1]["dist"]["url"],
        "http://localhost/repository/packagist/psr/log/3.0.1/psr-log-3.0.1.zip"
    );
    assert_eq!(entries[1]["description"], "PSR-3");
    let dist = get(
        &fixture,
        "/repository/packagist/psr/log/3.0.1/psr-log-3.0.1.zip",
    )
    .await;
    assert_eq!(dist.status, StatusCode::OK, "{}", dist.text());
    assert_eq!(dist.body, *upstream.archive);
    assert_eq!(
        get(
            &fixture,
            "/repository/packagist/psr/log/3.0.2/psr-log-3.0.2.zip"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let merged = get(&fixture, "/repository/all/p2/psr/log.json")
        .await
        .json();
    let entries = expanded(&merged, "psr/log");
    let versions: Vec<&str> = entries
        .iter()
        .map(|entry| entry["version"].as_str().unwrap())
        .collect();
    assert_eq!(versions, ["9.0.0", "3.0.2", "3.0.1"]);
    assert!(entries.iter().all(|entry| {
        entry["dist"]["url"]
            .as_str()
            .unwrap()
            .starts_with("http://localhost/repository/all/psr/log/")
    }));
    let through = get(&fixture, "/repository/all/psr/log/3.0.1/psr-log-3.0.1.zip").await;
    assert_eq!(through.body, *upstream.archive);
    let own = get(&fixture, "/repository/all/psr/log/9.0.0/psr-log-9.0.0.zip").await;
    assert_eq!(own.status, StatusCode::OK);
    let root = get(&fixture, "/repository/all/packages.json").await.json();
    assert_eq!(
        root["metadata-url"],
        "http://localhost/repository/all/p2/%package%.json"
    );
}
