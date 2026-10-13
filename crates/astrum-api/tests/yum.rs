mod support;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_yum::repodata::Repomd;
use astrum_format_yum::testing::TestRpm;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "yumboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_yum::Yum::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/yum/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, depth: i32, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "yum": {"repodataDepth": depth, "deployPolicy": policy}
    })
}

async fn put(fixture: &Fixture, path: &str, data: Vec<u8>) -> Reply {
    fixture
        .send_body(Method::PUT, path, Auth::Admin, Body::from(data), &[])
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn rpm(name: &str, version: &str) -> Vec<u8> {
    TestRpm::new(name, version, "1")
        .file(&format!("/usr/bin/{name}"), 10)
        .changelog(1_700_000_000, "Jane <jane@example.org>", "- built")
        .build()
}

fn gunzip(data: &[u8]) -> String {
    let mut text = String::new();
    flate2::read::GzDecoder::new(data)
        .read_to_string(&mut text)
        .unwrap();
    text
}

async fn repomd(fixture: &Fixture, repository: &str, base: &str) -> Repomd {
    let reply = get(
        fixture,
        &format!("/repository/{repository}{base}/repodata/repomd.xml"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    Repomd::parse(&reply.body).unwrap()
}

async fn primary(fixture: &Fixture, repository: &str, base: &str) -> String {
    let parsed = repomd(fixture, repository, base).await;
    let location = &parsed.data("primary").unwrap().location;
    let reply = get(
        fixture,
        &format!("/repository/{repository}{base}/{location}"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    gunzip(&reply.body)
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
async fn hosted_repositories_generate_repodata_for_their_packages() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("rpms", 0, "STRICT")).await;
    let hello = rpm("hello", "1.0");
    let reply = put(
        &fixture,
        "/repository/rpms/Packages/hello-1.0-1.x86_64.rpm",
        hello.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let text = primary(&fixture, "rpms", "").await;
    assert!(text.contains("packages=\"1\""), "{text}");
    assert!(text.contains("<location href=\"Packages/hello-1.0-1.x86_64.rpm\"/>"));
    assert!(text.contains("<file>/usr/bin/hello</file>"));
    let first = repomd(&fixture, "rpms", "").await;
    assert_eq!(repomd(&fixture, "rpms", "").await, first);

    let reply = upload(
        &fixture,
        "rpms",
        form(
            &[("yum.directory", "Packages/extra")],
            ("yum.asset", "anything.bin", &rpm("world", "2.0")),
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let text = primary(&fixture, "rpms", "").await;
    assert!(text.contains("packages=\"2\""));
    assert!(text.contains("Packages/extra/world-2.0-1.x86_64.rpm"));
    let other = repomd(&fixture, "rpms", "").await;
    let location = &other.data("other").unwrap().location;
    let changelog = gunzip(
        &get(&fixture, &format!("/repository/rpms/{location}"))
            .await
            .body,
    );
    assert!(changelog.contains("<changelog author=\"Jane &lt;jane@example.org&gt;\""));

    let reply = get(
        &fixture,
        "/service/rest/v1/search?repository=rpms&yum.architecture=x86_64&name=world",
    )
    .await;
    let items = reply.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["version"], "2.0-1");
    assert_eq!(items[0]["assets"][0]["yum"]["summary"], "The world package");

    for (path, data) in [
        ("/repository/rpms/notes.txt", b"plain".to_vec()),
        ("/repository/rpms/repodata/x.rpm", hello.clone()),
        ("/repository/rpms/broken.rpm", b"not an rpm".to_vec()),
    ] {
        let reply = put(&fixture, path, data).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{path}: {}",
            reply.text()
        );
    }
    let reply = fixture
        .call(
            Method::DELETE,
            "/repository/rpms/repodata/repomd.xml",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::DELETE,
            "/repository/rpms/Packages/hello-1.0-1.x86_64.rpm",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let text = primary(&fixture, "rpms", "").await;
    assert!(text.contains("packages=\"1\"") && !text.contains("hello-1.0"));
    let assets = get(&fixture, "/service/rest/v1/search/assets?repository=rpms").await;
    let paths: Vec<String> = assets.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        paths
            .iter()
            .filter(|path| path.ends_with(".xml.gz"))
            .count(),
        6,
        "{paths:?}"
    );
    assert!(
        get(&fixture, "/repository/rpms/repodata/repomd.xml.asc")
            .await
            .status
            == StatusCode::NOT_FOUND
    );
    assert!(
        get(&fixture, "/repository/rpms/repodata/repomd.xml.key")
            .await
            .status
            == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn repodata_depth_and_layout_policy_place_packages() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("deep", 2, "STRICT")).await;
    let reply = put(
        &fixture,
        "/repository/deep/el9/hello-1.0-1.x86_64.rpm",
        rpm("hello", "1.0"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(
        reply.text().contains("2 directories deep"),
        "{}",
        reply.text()
    );
    let reply = put(
        &fixture,
        "/repository/deep/el9/x86_64/Packages/hello-1.0-1.x86_64.rpm",
        rpm("hello", "1.0"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    put(
        &fixture,
        "/repository/deep/el8/x86_64/hello-1.0-1.x86_64.rpm",
        rpm("hello", "1.0"),
    )
    .await;
    let text = primary(&fixture, "deep", "/el9/x86_64").await;
    assert!(text.contains("<location href=\"Packages/hello-1.0-1.x86_64.rpm\"/>"));
    assert!(
        primary(&fixture, "deep", "/el8/x86_64")
            .await
            .contains("<location href=\"hello-1.0-1.x86_64.rpm\"/>")
    );
    assert_eq!(
        get(&fixture, "/repository/deep/repodata/repomd.xml")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/deep/el9/repodata/repomd.xml")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    create(&fixture, "hosted", hosted("loose", 1, "PERMISSIVE")).await;
    assert_eq!(
        put(&fixture, "/repository/loose/notes.txt", b"plain".to_vec())
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/loose/top-1.0-1.x86_64.rpm",
            rpm("top", "1.0")
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(&fixture, "/repository/loose/repodata/repomd.xml")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/loose/notes.txt").await.body,
        b"plain"
    );
}

#[tokio::test]
async fn signed_repositories_publish_a_signature_and_their_key() {
    let fixture = ready().await;
    let mut body = hosted("signed", 0, "STRICT");
    body["yumSigning"] = json!({
        "keypair": astrum_format_yum::testing::signing_key(Some("pass")),
        "passphrase": "pass"
    });
    create(&fixture, "hosted", body).await;
    put(
        &fixture,
        "/repository/signed/hello-1.0-1.x86_64.rpm",
        rpm("hello", "1.0"),
    )
    .await;
    repomd(&fixture, "signed", "").await;
    let signature = get(&fixture, "/repository/signed/repodata/repomd.xml.asc").await;
    assert_eq!(signature.status, StatusCode::OK);
    assert!(
        signature
            .text()
            .starts_with("-----BEGIN PGP SIGNATURE-----")
    );
    let key = get(&fixture, "/repository/signed/repodata/repomd.xml.key").await;
    assert_eq!(key.status, StatusCode::OK);
    assert!(
        key.text()
            .starts_with("-----BEGIN PGP PUBLIC KEY BLOCK-----")
    );
    assert!(!key.text().contains("PRIVATE"));
    let stored = get(&fixture, "/service/rest/v1/repositories/yum/hosted/signed")
        .await
        .json();
    assert!(stored["yumSigning"].get("keypair").is_none(), "{stored}");
}

#[tokio::test]
async fn groups_merge_the_repodata_of_their_members() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("one", 0, "STRICT")).await;
    create(&fixture, "hosted", hosted("two", 0, "STRICT")).await;
    put(
        &fixture,
        "/repository/one/a-1.0-1.x86_64.rpm",
        rpm("a", "1.0"),
    )
    .await;
    put(
        &fixture,
        "/repository/one/shared-1.0-1.x86_64.rpm",
        rpm("shared", "1.0"),
    )
    .await;
    put(
        &fixture,
        "/repository/two/b-1.0-1.x86_64.rpm",
        rpm("b", "1.0"),
    )
    .await;
    put(
        &fixture,
        "/repository/two/shared-1.0-1.x86_64.rpm",
        rpm("shared", "2.0"),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({"name": "both", "online": true, "storage": {"blobStoreName": "default", "strictContentTypeValidation": true}, "group": {"memberNames": ["one", "two"]}}),
    )
    .await;
    let text = primary(&fixture, "both", "").await;
    assert!(text.contains("packages=\"3\""), "{text}");
    assert!(text.contains("<name>a</name>") && text.contains("<name>b</name>"));
    assert!(text.contains("ver=\"1.0\"") && !text.contains("<version epoch=\"0\" ver=\"2.0\""));
    let first = repomd(&fixture, "both", "").await;
    assert_eq!(repomd(&fixture, "both", "").await, first);
    let package = get(&fixture, "/repository/both/b-1.0-1.x86_64.rpm").await;
    assert_eq!(package.status, StatusCode::OK);

    put(
        &fixture,
        "/repository/two/c-1.0-1.x86_64.rpm",
        rpm("c", "1.0"),
    )
    .await;
    assert!(
        primary(&fixture, "both", "")
            .await
            .contains("packages=\"4\"")
    );

    create(
        &fixture,
        "group",
        json!({"name": "single", "online": true, "storage": {"blobStoreName": "default", "strictContentTypeValidation": true}, "group": {"memberNames": ["two"]}}),
    )
    .await;
    assert_eq!(
        repomd(&fixture, "single", "").await,
        repomd(&fixture, "two", "").await
    );
    assert_eq!(
        fixture
            .call(
                Method::PUT,
                "/repository/both/x-1-1.x86_64.rpm",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::METHOD_NOT_ALLOWED
    );
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
async fn proxies_cache_metadata_and_packages_of_their_remote() {
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
    create(&fixture, "hosted", hosted("origin", 0, "STRICT")).await;
    let data = rpm("far", "3.0");
    put(
        &fixture,
        "/repository/origin/Packages/far-3.0-1.x86_64.rpm",
        data.clone(),
    )
    .await;
    let upstream = Upstream::default();
    let parsed = repomd(&fixture, "origin", "").await;
    let mut files = upstream.files.lock().unwrap().clone();
    files.insert(
        "mirror/repodata/repomd.xml".to_owned(),
        get(&fixture, "/repository/origin/repodata/repomd.xml")
            .await
            .body
            .to_vec(),
    );
    for entry in &parsed.data {
        files.insert(
            format!("mirror/{}", entry.location),
            get(&fixture, &format!("/repository/origin/{}", entry.location))
                .await
                .body
                .to_vec(),
        );
    }
    files.insert(
        "mirror/Packages/far-3.0-1.x86_64.rpm".to_owned(),
        data.clone(),
    );
    *upstream.files.lock().unwrap() = files;
    let address = serve_upstream(upstream).await;
    create(
        &fixture,
        "proxy",
        json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/mirror/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1},
            "httpClient": {"blocked": false, "autoBlock": false}
        }),
    )
    .await;
    assert_eq!(repomd(&fixture, "mirror", "").await, parsed);
    assert!(
        primary(&fixture, "mirror", "")
            .await
            .contains("far-3.0-1.x86_64.rpm")
    );
    let package = get(&fixture, "/repository/mirror/Packages/far-3.0-1.x86_64.rpm").await;
    assert_eq!(package.status, StatusCode::OK);
    assert_eq!(package.body, data);
    assert_eq!(
        package.headers.get(header::CONTENT_TYPE).unwrap(),
        "application/x-rpm"
    );
    let search = get(
        &fixture,
        "/service/rest/v1/search?repository=mirror&name=far",
    )
    .await
    .json();
    assert_eq!(search["items"][0]["version"], "3.0-1");
    assert_eq!(
        search["items"][0]["assets"][0]["yum"]["architecture"],
        "x86_64"
    );
    assert_eq!(
        get(&fixture, "/repository/mirror/missing-1-1.x86_64.rpm")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn upload_specs_describe_the_directory_field() {
    let fixture = ready().await;
    let spec = get(&fixture, "/service/rest/v1/formats/yum/upload-specs")
        .await
        .json();
    assert_eq!(
        spec,
        json!({
            "format": "yum",
            "multipleUpload": false,
            "componentFields": [{
                "name": "directory", "type": "STRING",
                "description": "Directory of the package, for example el9/x86_64/Packages",
                "optional": true, "group": "Component attributes", "groupHelpText": null, "options": null
            }],
            "assetFields": [{
                "name": "asset", "type": "FILE", "description": null, "optional": false,
                "group": null, "groupHelpText": null, "options": null
            }]
        })
    );
}
