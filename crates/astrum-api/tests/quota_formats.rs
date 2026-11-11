mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const JAR: &[u8] = b"PK\x03\x04\x14\x00\x08\x00\x08\x00jar-content";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    fixture
}

async fn create(fixture: &Fixture, format: &str, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/{format}/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn storage(quota: Value) -> Value {
    json!({"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW", "quota": quota})
}

async fn used(fixture: &Fixture, repository: &str) -> i64 {
    astrum_content::usage::current(fixture.state.db().conn(), repository)
        .await
        .unwrap()
        .unwrap()
        .bytes
}

async fn send(
    fixture: &Fixture,
    method: Method,
    path: &str,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    fixture
        .send_body(method, path, Auth::Admin, Body::from(body), headers)
        .await
}

#[tokio::test]
async fn maven_deploys_over_the_limit_are_refused() {
    let fixture = ready().await;
    create(
        &fixture,
        "maven",
        "hosted",
        json!({
            "name": "releases",
            "online": true,
            "storage": storage(json!({"limitBytes": 10, "enforce": true})),
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"}
        }),
    )
    .await;
    let path = "/repository/releases/org/acme/lib/1.0/lib-1.0.jar";
    let refused = send(&fixture, Method::PUT, path, JAR.to_vec(), &[]).await;
    assert_eq!(
        refused.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        refused.text()
    );
    assert!(refused.text().contains("releases"), "{}", refused.text());
    assert_eq!(used(&fixture, "releases").await, 0);
}

#[tokio::test]
async fn npm_publishes_over_the_limit_are_refused() {
    let fixture = ready().await;
    create(
        &fixture,
        "npm",
        "hosted",
        json!({"name": "packages", "online": true, "storage": storage(json!({"limitBytes": 10, "enforce": true}))}),
    )
    .await;
    let data = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03package lib 1.0.0".to_vec();
    let publication = json!({
        "_id": "lib",
        "name": "lib",
        "dist-tags": {"latest": "1.0.0"},
        "versions": {"1.0.0": {"name": "lib", "version": "1.0.0", "dist": {"tarball": "http://client.example/lib/-/lib-1.0.0.tgz"}}},
        "_attachments": {"lib-1.0.0.tgz": {"content_type": "application/octet-stream", "data": STANDARD.encode(&data), "length": data.len()}}
    });
    let refused = send(
        &fixture,
        Method::PUT,
        "/repository/packages/lib",
        serde_json::to_vec(&publication).unwrap(),
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        refused.text()
    );
    assert_eq!(refused.json()["success"], false);
    assert!(
        refused.json()["error"]
            .as_str()
            .unwrap()
            .contains("packages")
    );
    assert_eq!(used(&fixture, "packages").await, 0);
}

#[tokio::test]
async fn component_uploads_over_the_limit_are_refused() {
    let fixture = ready().await;
    create(
        &fixture,
        "raw",
        "hosted",
        json!({
            "name": "uploads",
            "online": true,
            "storage": storage(json!({"limitBytes": 20, "enforce": true})),
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
    let upload = |content: &str| {
        let boundary = "QUOTABOUNDARY";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"raw.directory\"\r\n\r\n/up\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"raw.asset1\"; filename=\"f.txt\"\r\nContent-Type: text/plain\r\n\r\n{content}\r\n\
             --{boundary}--\r\n"
        );
        let content_type = format!("multipart/form-data; boundary={boundary}");
        let fixture = &fixture;
        async move {
            fixture
                .send_body(
                    Method::POST,
                    "/service/rest/v1/components?repository=uploads",
                    Auth::Admin,
                    Body::from(body),
                    &[("content-type", content_type.as_str())],
                )
                .await
        }
    };
    let refused = upload(&"x".repeat(21)).await;
    assert_eq!(
        refused.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        refused.text()
    );
    assert_eq!(used(&fixture, "uploads").await, 0);
    let accepted = upload(&"x".repeat(20)).await;
    assert_eq!(
        accepted.status,
        StatusCode::NO_CONTENT,
        "{}",
        accepted.text()
    );
    assert_eq!(used(&fixture, "uploads").await, 20);
}

#[tokio::test]
async fn a_proxy_over_its_soft_limit_keeps_caching() {
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
    let hits = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route(
            "/{*path}",
            get(|State(hits): State<Arc<AtomicUsize>>| async move {
                hits.fetch_add(1, Ordering::SeqCst);
                "tool v1"
            }),
        )
        .with_state(hits.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "raw",
        "proxy",
        json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "quota": {"limitBytes": 1, "enforce": false}},
            "proxy": {"remoteUrl": format!("http://{address}/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": true},
            "raw": {"contentDisposition": "ATTACHMENT"}
        }),
    )
    .await;
    for _ in 0..2 {
        let read = fixture
            .call(
                Method::GET,
                "/repository/mirror/tools/tool.txt",
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(
            (read.status, read.text()),
            (StatusCode::OK, "tool v1".to_owned())
        );
    }
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "the second read comes from the cache"
    );
    assert_eq!(used(&fixture, "mirror").await, 7);
}
