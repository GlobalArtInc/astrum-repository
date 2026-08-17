mod support;

use std::io::Write;
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

const BOUNDARY: &str = "goboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_go::Go));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/go/{kind}"),
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

fn module_zip(module: &str, version: &str, with_mod: bool) -> Vec<u8> {
    let go_mod = format!("module {module}\n\ngo 1.22\n");
    let source = b"package hello\n".to_vec();
    let mod_name = format!("{module}@{version}/go.mod");
    let source_name = format!("{module}@{version}/hello.go");
    let mut files: Vec<(&str, &[u8])> = vec![(source_name.as_str(), source.as_slice())];
    if with_mod {
        files.push((mod_name.as_str(), go_mod.as_bytes()));
    }
    zip::write(&files)
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn form(fields: &[(&str, &str)], file: (&str, &str, &[u8])) -> Vec<u8> {
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
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: application/zip\r\n\r\n",
        file.0, file.1
    )
    .unwrap();
    body.extend_from_slice(file.2);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

#[tokio::test]
async fn hosted_modules_answer_the_proxy_protocol() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules")).await;
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/modules/example.com/!astrum/hello/@v/v1.0.0.zip",
            Auth::Admin,
            Body::from(module_zip("example.com/Astrum/hello", "v1.0.0", true)),
            &[],
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.text());
    let again = fixture
        .send_body(
            Method::PUT,
            "/repository/modules/example.com/!astrum/hello/@v/v1.0.0.zip",
            Auth::Admin,
            Body::from(module_zip("example.com/Astrum/hello", "v1.0.0", true)),
            &[],
        )
        .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    let wrong = fixture
        .send_body(
            Method::PUT,
            "/repository/modules/example.com/!astrum/hello/@v/v1.1.0.zip",
            Auth::Admin,
            Body::from(module_zip("example.com/Astrum/hello", "v1.0.0", true)),
            &[],
        )
        .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST, "{}", wrong.text());

    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=modules",
            Auth::Admin,
            Body::from(form(
                &[],
                (
                    "go.asset",
                    "hello.zip",
                    &module_zip("example.com/Astrum/hello", "v1.2.0-rc.1", false),
                ),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());

    let base = "/repository/modules/example.com/!astrum/hello";
    assert_eq!(
        get(&fixture, &format!("{base}/@v/list")).await.text(),
        "v1.0.0\nv1.2.0-rc.1\n"
    );
    let latest = get(&fixture, &format!("{base}/@latest")).await;
    assert_eq!(latest.json()["Version"], "v1.0.0");
    let info = get(&fixture, &format!("{base}/@v/v1.2.0-rc.1.info")).await;
    assert_eq!(info.json()["Version"], "v1.2.0-rc.1");
    assert!(info.json()["Time"].as_str().unwrap().ends_with('Z'));
    let go_mod = get(&fixture, &format!("{base}/@v/v1.2.0-rc.1.mod")).await;
    assert_eq!(go_mod.text(), "module example.com/Astrum/hello\n");
    let go_mod = get(&fixture, &format!("{base}/@v/v1.0.0.mod")).await;
    assert!(go_mod.text().contains("go 1.22"));
    let archive = get(&fixture, &format!("{base}/@v/v1.0.0.zip")).await;
    assert_eq!(archive.status, StatusCode::OK);
    assert_eq!(
        archive.body,
        module_zip("example.com/Astrum/hello", "v1.0.0", true)
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/modules/example.com/Astrum/hello/@v/list"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/modules/example.com/other/@v/list")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let components = get(&fixture, "/service/rest/v1/components?repository=modules").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], "example.com/Astrum/hello");
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{base}/@v/v1.0.0.zip"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&fixture, &format!("{base}/@v/list")).await.text(),
        "v1.2.0-rc.1\n"
    );
}

#[derive(Clone, Default)]
struct Upstream {
    seen: Arc<Mutex<Vec<String>>>,
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    upstream.seen.lock().unwrap().push(format!("/{path}"));
    match path.as_str() {
        "github.com/!burnt!sushi/toml/@v/list" => "v1.0.0\nv1.1.0\n".into_response(),
        "github.com/!burnt!sushi/toml/@latest" => {
            json!({"Version": "v1.1.0", "Time": "2024-01-01T00:00:00Z"})
                .to_string()
                .into_response()
        }
        "github.com/!burnt!sushi/toml/@v/v1.1.0.info" => {
            json!({"Version": "v1.1.0", "Time": "2024-01-01T00:00:00Z"})
                .to_string()
                .into_response()
        }
        "github.com/!burnt!sushi/toml/@v/v1.1.0.mod" => {
            "module github.com/BurntSushi/toml\n".into_response()
        }
        "github.com/!burnt!sushi/toml/@v/v1.1.0.zip" => {
            module_zip("github.com/BurntSushi/toml", "v1.1.0", true).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
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
async fn proxies_keep_the_case_encoding_and_groups_merge_versions() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let upstream = Upstream::default();
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "golang",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
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
            "group": {"memberNames": ["own", "golang"]},
        }),
    )
    .await;
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/own/github.com/!burnt!sushi/toml/@v/v1.2.0-beta.1.zip",
            Auth::Admin,
            Body::from(module_zip(
                "github.com/BurntSushi/toml",
                "v1.2.0-beta.1",
                true,
            )),
            &[],
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.text());

    let base = "/repository/golang/github.com/!burnt!sushi/toml";
    let archive = get(&fixture, &format!("{base}/@v/v1.1.0.zip")).await;
    assert_eq!(archive.status, StatusCode::OK, "{}", archive.text());
    assert_eq!(
        archive.body,
        module_zip("github.com/BurntSushi/toml", "v1.1.0", true)
    );
    assert_eq!(
        get(&fixture, &format!("{base}/@v/v1.1.0.mod")).await.text(),
        "module github.com/BurntSushi/toml\n"
    );
    assert_eq!(
        get(&fixture, &format!("{base}/@v/v9.9.9.zip")).await.status,
        StatusCode::NOT_FOUND
    );
    assert!(
        upstream
            .seen
            .lock()
            .unwrap()
            .contains(&"/github.com/!burnt!sushi/toml/@v/v1.1.0.zip".to_owned())
    );

    let group = "/repository/all/github.com/!burnt!sushi/toml";
    assert_eq!(
        get(&fixture, &format!("{group}/@v/list")).await.text(),
        "v1.0.0\nv1.1.0\nv1.2.0-beta.1\n"
    );
    assert_eq!(
        get(&fixture, &format!("{group}/@latest")).await.json()["Version"],
        "v1.1.0"
    );
    let info = get(&fixture, &format!("{group}/@v/v1.2.0-beta.1.info")).await;
    assert_eq!(info.json()["Version"], "v1.2.0-beta.1");
    let cached = get(&fixture, "/service/rest/v1/assets?repository=golang").await;
    let paths: Vec<String> = cached.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        paths.contains(&"/github.com/!burnt!sushi/toml/@v/v1.1.0.zip".to_owned()),
        "{paths:?}"
    );
}
