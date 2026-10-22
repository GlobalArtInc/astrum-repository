mod support;

use std::collections::BTreeMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::Scope;
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

const JSON: &str = "application/json";
const BOUNDARY: &str = "bazel-form-boundary-0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_bazel::Bazel));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/bazel/{kind}"),
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
    })
}

fn proxy(name: &str, remote: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 0, "metadataMaxAge": 0},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": true},
    })
}

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
    })
}

fn sri(data: &[u8]) -> String {
    format!("sha256-{}", STANDARD.encode(Sha256::digest(data)))
}

fn module_bazel(name: &str, version: &str) -> String {
    format!("module(name = \"{name}\", version = \"{version}\")\n")
}

fn source_json(archive: &[u8]) -> String {
    json!({
        "url": "https://example.org/archive.tar.gz",
        "integrity": sri(archive),
        "strip_prefix": "archive-1.0",
    })
    .to_string()
}

async fn put(
    fixture: &Fixture,
    repository: &str,
    path: &str,
    auth: Auth<'_>,
    body: &[u8],
) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}{path}"),
            auth,
            Body::from(body.to_vec()),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    fixture.send(Method::GET, path, auth, None, &[]).await
}

async fn publish(fixture: &Fixture, repository: &str, module: &str, version: &str) {
    for (file, body) in [
        ("MODULE.bazel", module_bazel(module, version)),
        ("source.json", source_json(version.as_bytes())),
    ] {
        let reply = put(
            fixture,
            repository,
            &format!("/modules/{module}/{version}/{file}"),
            Auth::Admin,
            body.as_bytes(),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{module} {version} {file}: {}",
            reply.text()
        );
    }
}

async fn metadata(fixture: &Fixture, repository: &str, module: &str) -> Reply {
    get(
        fixture,
        &format!("/repository/{repository}/modules/{module}/metadata.json"),
        Auth::Admin,
    )
    .await
}

fn versions(document: &Value) -> Vec<String> {
    document["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| version.as_str().unwrap().to_owned())
        .collect()
}

fn form(fields: &[(&str, &str)], files: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        )
        .unwrap();
    }
    for (name, filename, content) in files {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .unwrap();
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn upload(
    fixture: &Fixture,
    repository: &str,
    fields: &[(&str, &str)],
    files: &[(&str, &str, &[u8])],
) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(form(fields, files)),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await
}

#[tokio::test]
async fn hosted_repositories_list_versions_in_bazel_order_once_a_version_is_whole() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;

    let registry = get(
        &fixture,
        "/repository/modules/bazel_registry.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(registry.status, StatusCode::OK);
    assert_eq!(registry.headers["content-type"], JSON);
    assert_eq!(registry.json(), json!({"mirrors": []}));
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.status,
        StatusCode::NOT_FOUND
    );

    let uploaded = put(
        &fixture,
        "modules",
        "/modules/rules_demo/1.0.0/MODULE.bazel",
        Auth::Admin,
        module_bazel("rules_demo", "1.0.0").as_bytes(),
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::CREATED, "{}", uploaded.text());
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.status,
        StatusCode::NOT_FOUND,
        "a version without source.json is not listed"
    );
    let source = put(
        &fixture,
        "modules",
        "/modules/rules_demo/1.0.0/source.json",
        Auth::Admin,
        source_json(b"one").as_bytes(),
    )
    .await;
    assert_eq!(source.status, StatusCode::CREATED, "{}", source.text());
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.json(),
        json!({"versions": ["1.0.0"], "yanked_versions": {}})
    );

    for version in ["1.10.0", "1.2.0", "1.2.0-rc1", "1.2.0.bcr.1"] {
        publish(&fixture, "modules", "rules_demo", version).await;
    }
    let listing = metadata(&fixture, "modules", "rules_demo").await;
    assert_eq!(listing.status, StatusCode::OK);
    assert_eq!(listing.headers["content-type"], JSON);
    assert_eq!(
        versions(&listing.json()),
        ["1.0.0", "1.2.0-rc1", "1.2.0", "1.2.0.bcr.1", "1.10.0"]
    );
    let probe = fixture
        .send(
            Method::HEAD,
            "/repository/modules/modules/rules_demo/metadata.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(probe.status, StatusCode::OK);
    assert!(probe.body.is_empty());
    assert_eq!(
        probe.headers["content-length"].to_str().unwrap(),
        listing.body.len().to_string()
    );
    let etag = listing.headers["etag"].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/modules/modules/rules_demo/metadata.json",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let module = get(
        &fixture,
        "/repository/modules/modules/rules_demo/1.2.0/MODULE.bazel",
        Auth::Admin,
    )
    .await;
    assert_eq!(module.status, StatusCode::OK);
    assert_eq!(module.body, module_bazel("rules_demo", "1.2.0").as_bytes());
    assert_eq!(module.headers["content-type"], "text/plain");
    let source = get(
        &fixture,
        "/repository/modules/modules/rules_demo/1.2.0/source.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(source.headers["content-type"], JSON);
    assert_eq!(source.json()["integrity"], sri(b"1.2.0"));
    let ranged = fixture
        .send(
            Method::GET,
            "/repository/modules/modules/rules_demo/1.2.0/MODULE.bazel",
            Auth::Admin,
            None,
            &[("range", "bytes=0-5")],
        )
        .await;
    assert_eq!(ranged.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(ranged.body, b"module");
    for missing in [
        "/modules/rules_demo/9.9.9/MODULE.bazel",
        "/modules/rules_demo/1.2.0/presubmit.yml",
        "/modules/rules_demo",
        "/modules/",
        "/other.json",
    ] {
        assert_eq!(
            get(
                &fixture,
                &format!("/repository/modules{missing}"),
                Auth::Admin
            )
            .await
            .status,
            StatusCode::NOT_FOUND,
            "{missing}"
        );
    }
    let page = get(&fixture, "/repository/modules/", Auth::Admin).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.text()
            .contains("--registry=http://localhost/repository/modules")
    );

    let tree = get(
        &fixture,
        "/service/rest/internal/ui/browse/modules?path=",
        Auth::Admin,
    )
    .await
    .json();
    let root: Vec<(&str, &str)> = tree["children"]
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
    assert_eq!(
        root,
        [("folder", "modules"), ("asset", "bazel_registry.json")]
    );

    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=modules",
        Auth::Admin,
    )
    .await
    .json();
    let items = components["items"].as_array().unwrap();
    assert_eq!(items.len(), 5);
    assert!(items.iter().all(
        |item| item["name"] == "rules_demo" && item["group"].as_str().unwrap_or("").is_empty()
    ));
    let first = items
        .iter()
        .find(|item| item["version"] == "1.0.0")
        .unwrap();
    let mut paths: Vec<&str> = first["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            "/modules/rules_demo/1.0.0/MODULE.bazel",
            "/modules/rules_demo/1.0.0/source.json"
        ]
    );
}

#[tokio::test]
async fn published_files_are_never_replaced_and_must_be_valid() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;
    publish(&fixture, "modules", "rules_demo", "1.0.0").await;

    let again = put(
        &fixture,
        "modules",
        "/modules/rules_demo/1.0.0/MODULE.bazel",
        Auth::Admin,
        module_bazel("rules_demo", "1.0.0").as_bytes(),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(again.text().contains("not replaced"), "{}", again.text());
    let kept = get(
        &fixture,
        "/repository/modules/modules/rules_demo/1.0.0/source.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(kept.json()["integrity"], sri(b"1.0.0"));

    let deleted = fixture
        .send(
            Method::DELETE,
            "/repository/modules/modules/rules_demo/1.0.0/source.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.status,
        StatusCode::NOT_FOUND,
        "a version that lost source.json is no longer listed"
    );
    let replaced = put(
        &fixture,
        "modules",
        "/modules/rules_demo/1.0.0/source.json",
        Auth::Admin,
        source_json(b"changed").as_bytes(),
    )
    .await;
    assert_eq!(replaced.status, StatusCode::CREATED, "{}", replaced.text());
    assert_eq!(
        versions(&metadata(&fixture, "modules", "rules_demo").await.json()),
        ["1.0.0"]
    );
    let missing = fixture
        .send(
            Method::DELETE,
            "/repository/modules/modules/rules_demo/7.0.0/source.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let cases: [(&str, &[u8], &str); 12] = [
        (
            "/modules/rules_demo/2.0.0/MODULE.bazel",
            b"module(name = \"other\", version = \"2.0.0\")",
            "declares the module other",
        ),
        (
            "/modules/rules_demo/2.0.0/MODULE.bazel",
            b"module(name = \"rules_demo\", version = \"2.0.1\")",
            "declares version 2.0.1",
        ),
        (
            "/modules/rules_demo/2.0.0/MODULE.bazel",
            &[0xff, 0xfe, 0x00],
            "UTF-8",
        ),
        (
            "/modules/rules_demo/2.0.0/source.json",
            b"not json",
            "not valid JSON",
        ),
        (
            "/modules/rules_demo/2.0.0/source.json",
            br#"{"url": "https://example.org/a.tar.gz"}"#,
            "integrity is required for the archive type",
        ),
        (
            "/modules/rules_demo/2.0.0/source.json",
            br#"{"type": "zip", "url": "https://example.org/a.tar.gz"}"#,
            "type zip",
        ),
        (
            "/modules/rules_demo/2.0.0/source.json",
            br#"{"url": "https://example.org/a", "integrity": "sha256-AAAA", "patches": {"../x": "sha256-AAAA"}}"#,
            "not a valid file name",
        ),
        (
            "/modules/Rules_Demo/1.0.0/MODULE.bazel",
            b"module()",
            "upload to /bazel_registry.json",
        ),
        (
            "/modules/rules_demo/1..0/MODULE.bazel",
            b"module()",
            "upload to /bazel_registry.json",
        ),
        (
            "/modules/rules_demo/1.0.0/presubmit.yml",
            b"x",
            "upload to /bazel_registry.json",
        ),
        (
            "/modules/rules_demo/1.0.0/patches/a%20b.patch",
            b"x",
            "upload to /bazel_registry.json",
        ),
        ("/anything", b"x", "upload to /bazel_registry.json"),
    ];
    for (path, body, expected) in cases {
        let reply = put(&fixture, "modules", path, Auth::Admin, body).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(reply.text().contains(expected), "{path}: {}", reply.text());
    }
    assert_eq!(
        versions(&metadata(&fixture, "modules", "rules_demo").await.json()),
        ["1.0.0"]
    );
}

#[tokio::test]
async fn metadata_and_registry_documents_are_set_by_hand_and_checked() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW_ONCE")).await;
    publish(&fixture, "modules", "rules_demo", "1.0.0").await;
    publish(&fixture, "modules", "rules_demo", "1.1.0").await;

    let document = json!({
        "homepage": "https://example.org/rules_demo",
        "maintainers": [{"name": "Roman", "email": "roman@example.org", "github": "roman"}],
        "repository": ["github:example/rules_demo"],
        "versions": ["9.9.9"],
        "yanked_versions": {"1.0.0": "broken, use 1.1.0"},
    });
    let stored = put(
        &fixture,
        "modules",
        "/modules/rules_demo/metadata.json",
        Auth::Admin,
        document.to_string().as_bytes(),
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    let served = metadata(&fixture, "modules", "rules_demo").await.json();
    assert_eq!(served["homepage"], "https://example.org/rules_demo");
    assert_eq!(served["maintainers"][0]["github"], "roman");
    assert_eq!(served["repository"], json!(["github:example/rules_demo"]));
    assert_eq!(
        served["yanked_versions"],
        json!({"1.0.0": "broken, use 1.1.0"})
    );
    assert_eq!(versions(&served), ["1.0.0", "1.1.0"]);

    let changed = put(
        &fixture,
        "modules",
        "/modules/rules_demo/metadata.json",
        Auth::Admin,
        json!({"homepage": "https://example.org/new"})
            .to_string()
            .as_bytes(),
    )
    .await;
    assert_eq!(
        changed.status,
        StatusCode::OK,
        "metadata can be replaced even where files cannot: {}",
        changed.text()
    );
    let served = metadata(&fixture, "modules", "rules_demo").await.json();
    assert_eq!(served["homepage"], "https://example.org/new");
    assert_eq!(served["yanked_versions"], json!({}));

    for (body, expected) in [
        (&b"[]"[..], "must be a JSON object"),
        (b"{", "not valid JSON"),
        (br#"{"homepage": 1}"#, "homepage must be a string"),
        (
            br#"{"maintainers": [1]}"#,
            "maintainers must be an array of objects",
        ),
        (br#"{"yanked_versions": {"1.0.0": 5}}"#, "yanked_versions"),
    ] {
        let reply = put(
            &fixture,
            "modules",
            "/modules/rules_demo/metadata.json",
            Auth::Admin,
            body,
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert!(reply.text().contains(expected), "{}", reply.text());
    }
    let only_metadata = put(
        &fixture,
        "modules",
        "/modules/announced/metadata.json",
        Auth::Admin,
        br#"{"homepage": "https://example.org/announced"}"#,
    )
    .await;
    assert_eq!(only_metadata.status, StatusCode::CREATED);
    assert_eq!(
        metadata(&fixture, "modules", "announced").await.json(),
        json!({
            "homepage": "https://example.org/announced",
            "versions": [],
            "yanked_versions": {},
        })
    );
    let dropped = fixture
        .send(
            Method::DELETE,
            "/repository/modules/modules/rules_demo/metadata.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(dropped.status, StatusCode::NO_CONTENT);
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.json(),
        json!({"versions": ["1.0.0", "1.1.0"], "yanked_versions": {}})
    );

    let registry = json!({
        "mirrors": ["https://mirror.example/archives/"],
        "module_base_path": "../modules",
    });
    let set = put(
        &fixture,
        "modules",
        "/bazel_registry.json",
        Auth::Admin,
        registry.to_string().as_bytes(),
    )
    .await;
    assert_eq!(set.status, StatusCode::CREATED, "{}", set.text());
    let served = get(
        &fixture,
        "/repository/modules/bazel_registry.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(served.json(), registry);
    for body in [
        &br#"{"mirrors": "https://mirror.example/"}"#[..],
        br#"{"mirrors": [""]}"#,
        br#"{"module_base_path": 4}"#,
        b"[]",
    ] {
        let reply = put(
            &fixture,
            "modules",
            "/bazel_registry.json",
            Auth::Admin,
            body,
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    }
    let reset = fixture
        .send(
            Method::DELETE,
            "/repository/modules/bazel_registry.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(
            &fixture,
            "/repository/modules/bazel_registry.json",
            Auth::Admin
        )
        .await
        .json(),
        json!({"mirrors": []})
    );
}

#[tokio::test]
async fn patches_and_overlays_must_match_the_integrity_source_json_lists() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;
    let patch = b"--- a.txt\n+++ a.txt\n@@ -1 +1 @@\n-one\n+two\n";
    let overlay = b"filegroup(name = \"all\", srcs = glob([\"**\"]))\n";
    let source = json!({
        "url": "https://example.org/archive.tar.gz",
        "integrity": sri(b"archive"),
        "patch_strip": 0,
        "patches": {"fix.patch": sri(patch)},
        "overlay": {"sub/BUILD.bazel": sri(overlay)},
    })
    .to_string();
    let base = "/modules/patched/1.0.0";
    assert_eq!(
        put(
            &fixture,
            "modules",
            &format!("{base}/source.json"),
            Auth::Admin,
            source.as_bytes()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let wrong = put(
        &fixture,
        "modules",
        &format!("{base}/patches/fix.patch"),
        Auth::Admin,
        b"another patch",
    )
    .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST, "{}", wrong.text());
    assert!(
        wrong.text().contains("patches/fix.patch does not match"),
        "{}",
        wrong.text()
    );
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/modules{base}/patches/fix.patch"),
            Auth::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    for (path, body) in [
        ("patches/fix.patch", &patch[..]),
        ("overlay/sub/BUILD.bazel", &overlay[..]),
        ("patches/unlisted.patch", b"anything"),
    ] {
        let reply = put(
            &fixture,
            "modules",
            &format!("{base}/{path}"),
            Auth::Admin,
            body,
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{path}: {}",
            reply.text()
        );
    }
    let fetched = get(
        &fixture,
        &format!("/repository/modules{base}/overlay/sub/BUILD.bazel"),
        Auth::Admin,
    )
    .await;
    assert_eq!(fetched.body, overlay);

    let other = "/modules/patched/2.0.0";
    assert_eq!(
        put(
            &fixture,
            "modules",
            &format!("{other}/patches/fix.patch"),
            Auth::Admin,
            b"uploaded before source.json"
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let mismatching = json!({
        "url": "https://example.org/archive.tar.gz",
        "integrity": sri(b"archive"),
        "patches": {"fix.patch": sri(patch)},
    })
    .to_string();
    let refused = put(
        &fixture,
        "modules",
        &format!("{other}/source.json"),
        Auth::Admin,
        mismatching.as_bytes(),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert!(
        refused.text().contains("patches/fix.patch does not match"),
        "{}",
        refused.text()
    );
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/modules{other}/source.json"),
            Auth::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn files_can_be_uploaded_together_through_the_components_form() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;
    let spec = get(
        &fixture,
        "/service/rest/v1/formats/bazel/upload-specs",
        Auth::Admin,
    )
    .await
    .json();
    let names = |key: &str| -> Vec<String> {
        spec[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field["name"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names("componentFields"), ["module", "version"]);
    assert_eq!(names("assetFields"), ["filename", "asset"]);
    assert_eq!(spec["multipleUpload"], true);

    let patch = b"--- a.txt\n+++ a.txt\n@@ -1 +1 @@\n-one\n+two\n";
    let source = json!({
        "url": "https://example.org/archive.tar.gz",
        "integrity": sri(b"archive"),
        "patches": {"form.patch": sri(patch)},
    })
    .to_string();
    let module = module_bazel("form_demo", "1.0.0");
    let reply = upload(
        &fixture,
        "modules",
        &[("bazel.module", "form_demo"), ("bazel.version", "1.0.0")],
        &[
            ("bazel.asset1", "source.json", source.as_bytes()),
            ("bazel.asset2", "MODULE.bazel", module.as_bytes()),
            ("bazel.asset3", "form.patch", patch),
        ],
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(
        versions(&metadata(&fixture, "modules", "form_demo").await.json()),
        ["1.0.0"]
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/modules/modules/form_demo/1.0.0/patches/form.patch",
            Auth::Admin
        )
        .await
        .body,
        patch
    );

    let described = upload(
        &fixture,
        "modules",
        &[
            ("bazel.module", "form_demo"),
            ("bazel.asset1.filename", "metadata.json"),
        ],
        &[(
            "bazel.asset1",
            "upload.bin",
            br#"{"homepage": "https://example.org/form", "yanked_versions": {"1.0.0": "uploaded"}}"#,
        )],
    )
    .await;
    assert_eq!(
        described.status,
        StatusCode::NO_CONTENT,
        "{}",
        described.text()
    );
    let served = metadata(&fixture, "modules", "form_demo").await.json();
    assert_eq!(served["homepage"], "https://example.org/form");
    assert_eq!(served["yanked_versions"], json!({"1.0.0": "uploaded"}));

    for (fields, files, expected) in [
        (
            vec![("bazel.module", "form_demo"), ("bazel.version", "1.0.0")],
            vec![("bazel.asset1", "MODULE.bazel", module.as_bytes())],
            "already exists",
        ),
        (
            vec![("bazel.module", "Form_Demo"), ("bazel.version", "1.0.0")],
            vec![("bazel.asset1", "MODULE.bazel", module.as_bytes())],
            "not a module name",
        ),
        (
            vec![("bazel.module", "form_demo")],
            vec![("bazel.asset1", "MODULE.bazel", module.as_bytes())],
            "bazel.version is required",
        ),
        (
            vec![("bazel.module", "form_demo"), ("bazel.version", "2.0.0")],
            vec![("bazel.asset1", "notes.txt", &b"x"[..])],
            "cannot tell where notes.txt goes",
        ),
        (
            vec![("bazel.module", "form_demo"), ("bazel.version", "2.0.0")],
            vec![],
            "no files",
        ),
    ] {
        let reply = upload(&fixture, "modules", &fields, &files).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert!(reply.text().contains(expected), "{}", reply.text());
    }
    assert_eq!(
        versions(&metadata(&fixture, "modules", "form_demo").await.json()),
        ["1.0.0"]
    );

    let (_, address) = serve_upstream(&fixture).await;
    create(
        &fixture,
        "proxy",
        proxy("remote", &format!("http://{address}")),
    )
    .await;
    let refused = upload(
        &fixture,
        "remote",
        &[("bazel.module", "form_demo"), ("bazel.version", "1.0.0")],
        &[("bazel.asset1", "MODULE.bazel", module.as_bytes())],
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(refused.text().contains("upload modules to a hosted"));
}

#[tokio::test]
async fn access_follows_the_privileges_of_the_caller() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;
    publish(&fixture, "modules", "rules_demo", "1.0.0").await;
    let reader = fixture
        .state
        .security()
        .create_token("admin", "bazel-read", Scope::Read, None)
        .await
        .unwrap()
        .1;
    let writer = fixture
        .state
        .security()
        .create_token("admin", "bazel-write", Scope::Write, None)
        .await
        .unwrap()
        .1;
    let file = "/modules/rules_demo/1.0.0/MODULE.bazel";

    let anonymous = get(&fixture, &format!("/repository/modules{file}"), Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(
        metadata(&fixture, "modules", "rules_demo").await.status,
        StatusCode::OK
    );
    let anonymous_metadata = get(
        &fixture,
        "/repository/modules/modules/rules_demo/metadata.json",
        Auth::None,
    )
    .await;
    assert_eq!(anonymous_metadata.status, StatusCode::UNAUTHORIZED);
    let anonymous_registry = get(
        &fixture,
        "/repository/modules/bazel_registry.json",
        Auth::None,
    )
    .await;
    assert_eq!(anonymous_registry.status, StatusCode::UNAUTHORIZED);

    let bearer = |token: &str| format!("Bearer {token}");
    let read = fixture
        .send(
            Method::GET,
            &format!("/repository/modules{file}"),
            Auth::Basic("admin", &reader),
            None,
            &[],
        )
        .await;
    assert_eq!(read.status, StatusCode::OK);
    let refused = fixture
        .send_body(
            Method::PUT,
            "/repository/modules/modules/rules_demo/2.0.0/MODULE.bazel",
            Auth::Basic("admin", &reader),
            Body::from(module_bazel("rules_demo", "2.0.0")),
            &[],
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let allowed = fixture
        .send_body(
            Method::PUT,
            "/repository/modules/modules/rules_demo/2.0.0/MODULE.bazel",
            Auth::None,
            Body::from(module_bazel("rules_demo", "2.0.0")),
            &[("authorization", &bearer(&writer))],
        )
        .await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
    let anonymous_write = put(
        &fixture,
        "modules",
        "/modules/rules_demo/3.0.0/MODULE.bazel",
        Auth::None,
        module_bazel("rules_demo", "3.0.0").as_bytes(),
    )
    .await;
    assert_eq!(anonymous_write.status, StatusCode::UNAUTHORIZED);
    let anonymous_delete = fixture
        .send(
            Method::DELETE,
            &format!("/repository/modules{file}"),
            Auth::None,
            None,
            &[],
        )
        .await;
    assert_eq!(anonymous_delete.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn selectors_hide_the_versions_a_caller_may_not_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("modules", "ALLOW")).await;
    publish(&fixture, "modules", "rules_demo", "1.0.0").await;
    publish(&fixture, "modules", "rules_demo", "2.0.0").await;
    let steps = [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "demo-one", "description": "", "expression": "format == \"bazel\" && (path == \"/modules/rules_demo/metadata.json\" || path =^ \"/modules/rules_demo/1.\")"}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "demo-one-read", "description": "", "actions": ["READ", "BROWSE"], "format": "bazel", "repository": "modules", "contentSelector": "demo-one"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "demo-readers", "name": "demo-readers", "description": "", "privileges": ["demo-one-read"], "roles": []}),
        ),
    ];
    for (path, body) in steps {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.text());
    }
    fixture
        .add_user("selected", "selected-pass", &["demo-readers"])
        .await;
    let selected = Auth::Basic("selected", "selected-pass");

    let listing = get(
        &fixture,
        "/repository/modules/modules/rules_demo/metadata.json",
        Auth::Basic("selected", "selected-pass"),
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(versions(&listing.json()), ["1.0.0"]);
    assert_eq!(
        get(
            &fixture,
            "/repository/modules/modules/rules_demo/1.0.0/MODULE.bazel",
            Auth::Basic("selected", "selected-pass")
        )
        .await
        .status,
        StatusCode::OK
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/modules/modules/rules_demo/2.0.0/MODULE.bazel",
            selected
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn read_only_and_derived_repositories_refuse_writes() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("frozen", "DENY")).await;
    let denied = put(
        &fixture,
        "frozen",
        "/modules/rules_demo/1.0.0/MODULE.bazel",
        Auth::Admin,
        module_bazel("rules_demo", "1.0.0").as_bytes(),
    )
    .await;
    assert_eq!(denied.status, StatusCode::BAD_REQUEST, "{}", denied.text());
    assert!(denied.text().contains("read-only"));
    let registry = put(
        &fixture,
        "frozen",
        "/bazel_registry.json",
        Auth::Admin,
        b"{}",
    )
    .await;
    assert_eq!(registry.status, StatusCode::BAD_REQUEST);

    let (_, address) = serve_upstream(&fixture).await;
    create(
        &fixture,
        "proxy",
        proxy("remote", &format!("http://{address}")),
    )
    .await;
    create(
        &fixture,
        "group",
        group("everything", &["frozen", "remote"]),
    )
    .await;
    for repository in ["remote", "everything"] {
        let reply = put(
            &fixture,
            repository,
            "/modules/rules_demo/1.0.0/MODULE.bazel",
            Auth::Admin,
            module_bazel("rules_demo", "1.0.0").as_bytes(),
        )
        .await;
        assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED, "{repository}");
    }
    let deleted = fixture
        .send(
            Method::DELETE,
            "/repository/everything/modules/rules_demo/1.0.0/MODULE.bazel",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(deleted.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(deleted.headers["allow"], "GET,HEAD");
    let posted = fixture
        .send(
            Method::POST,
            "/repository/frozen/modules/rules_demo/metadata.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(posted.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(posted.headers["allow"], "GET,HEAD,PUT,DELETE");
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<String>>>,
    files: Arc<Mutex<BTreeMap<String, String>>>,
}

impl Upstream {
    fn count(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|hit| hit.as_str() == path)
            .count()
    }

    fn publish(&self, path: &str, body: impl Into<String>) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_owned(), body.into());
    }

    fn module(&self, name: &str, versions: &[&str], extra: Value) {
        let mut document = json!({"versions": versions, "yanked_versions": {}});
        for (key, value) in extra.as_object().unwrap() {
            document[key] = value.clone();
        }
        self.publish(
            &format!("modules/{name}/metadata.json"),
            document.to_string(),
        );
        for version in versions {
            self.publish(
                &format!("modules/{name}/{version}/MODULE.bazel"),
                format!("module(name = \"{name}\", version = \"{version}\")\n# upstream\n"),
            );
            self.publish(
                &format!("modules/{name}/{version}/source.json"),
                source_json(format!("{name}{version}").as_bytes()),
            );
        }
    }
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    upstream.hits.lock().unwrap().push(path.clone());
    let found = upstream.files.lock().unwrap().get(&path).cloned();
    match found {
        Some(body) => ([(header::CONTENT_TYPE, "application/octet-stream")], body).into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn serve_upstream(fixture: &Fixture) -> (Upstream, SocketAddr) {
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
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (upstream, address)
}

#[tokio::test]
async fn proxies_keep_version_files_for_good_and_recheck_the_documents() {
    let fixture = ready().await;
    let (upstream, address) = serve_upstream(&fixture).await;
    upstream.publish(
        "bazel_registry.json",
        json!({"mirrors": ["https://mirror.example/"]}).to_string(),
    );
    upstream.module(
        "far_lib",
        &["1.0.0", "2.0.0"],
        json!({"homepage": "https://far.example", "yanked_versions": {"1.0.0": "old"}}),
    );
    upstream.publish("modules/far_lib/2.0.0/patches/fix.patch", "--- a\n+++ b\n");
    upstream.publish("modules/far_lib/2.0.0/overlay/dir/BUILD.bazel", "# build\n");
    create(
        &fixture,
        "proxy",
        proxy("bcr", &format!("http://{address}")),
    )
    .await;

    let registry = get(&fixture, "/repository/bcr/bazel_registry.json", Auth::Admin).await;
    assert_eq!(registry.status, StatusCode::OK);
    assert_eq!(
        registry.json(),
        json!({"mirrors": ["https://mirror.example/"]})
    );
    let listing = metadata(&fixture, "bcr", "far_lib").await;
    assert_eq!(listing.status, StatusCode::OK);
    assert_eq!(listing.headers["content-type"], "application/json");
    assert_eq!(listing.json()["homepage"], "https://far.example");
    assert_eq!(versions(&listing.json()), ["1.0.0", "2.0.0"]);
    upstream.module("far_lib", &["1.0.0", "2.0.0", "3.0.0"], json!({}));
    assert_eq!(
        versions(&metadata(&fixture, "bcr", "far_lib").await.json()),
        ["1.0.0", "2.0.0", "3.0.0"],
        "metadata is asked for again"
    );
    assert_eq!(upstream.count("modules/far_lib/metadata.json"), 2);
    assert_eq!(upstream.count("bazel_registry.json"), 1);

    for path in [
        "MODULE.bazel",
        "source.json",
        "patches/fix.patch",
        "overlay/dir/BUILD.bazel",
    ] {
        let url = format!("/repository/bcr/modules/far_lib/2.0.0/{path}");
        let first = get(&fixture, &url, Auth::Admin).await;
        assert_eq!(first.status, StatusCode::OK, "{path}: {}", first.text());
        let second = get(&fixture, &url, Auth::Admin).await;
        assert_eq!(second.body, first.body);
        assert_eq!(
            upstream.count(&format!("modules/far_lib/2.0.0/{path}")),
            1,
            "{path} is kept for good"
        );
    }
    let module = get(
        &fixture,
        "/repository/bcr/modules/far_lib/2.0.0/MODULE.bazel",
        Auth::Admin,
    )
    .await;
    assert_eq!(module.headers["content-type"], "text/plain");
    assert!(module.text().contains("# upstream"));
    assert_eq!(
        get(
            &fixture,
            "/repository/bcr/modules/far_lib/2.0.0/source.json",
            Auth::Admin
        )
        .await
        .headers["content-type"],
        "application/json"
    );

    for path in [
        "/repository/bcr/modules/far_lib/2.0.0/presubmit.yml",
        "/repository/bcr/modules/far_lib/2.0.0/patches/",
        "/repository/bcr/modules/Far_Lib/metadata.json",
        "/repository/bcr/index.html",
    ] {
        assert_eq!(
            get(&fixture, path, Auth::Admin).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    assert_eq!(upstream.count("modules/far_lib/2.0.0/presubmit.yml"), 0);
    assert_eq!(upstream.count("index.html"), 0);
    assert_eq!(
        metadata(&fixture, "bcr", "unknown_lib").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/bcr/modules/far_lib/9.0.0/MODULE.bazel",
            Auth::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let anonymous = get(
        &fixture,
        "/repository/bcr/modules/far_lib/1.0.0/MODULE.bazel",
        Auth::None,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(upstream.count("modules/far_lib/1.0.0/MODULE.bazel"), 0);

    let dropped = fixture
        .send(
            Method::DELETE,
            "/repository/bcr/modules/far_lib/2.0.0/MODULE.bazel",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(dropped.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(
            &fixture,
            "/repository/bcr/modules/far_lib/2.0.0/MODULE.bazel",
            Auth::Admin
        )
        .await
        .status,
        StatusCode::OK
    );
    assert_eq!(upstream.count("modules/far_lib/2.0.0/MODULE.bazel"), 2);
}

#[tokio::test]
async fn groups_merge_metadata_and_hosted_names_shadow_the_proxies() {
    let fixture = ready().await;
    let (upstream, address) = serve_upstream(&fixture).await;
    upstream.publish(
        "bazel_registry.json",
        json!({"mirrors": ["https://b.example/", "https://a.example/"]}).to_string(),
    );
    upstream.module(
        "shared",
        &["1.0.0", "2.0.0"],
        json!({"homepage": "https://far.example/shared"}),
    );
    upstream.module(
        "public_lib",
        &["3.0.0"],
        json!({"maintainers": [{"github": "far"}], "yanked_versions": {"3.0.0": "far reason"}}),
    );
    create(
        &fixture,
        "proxy",
        proxy("bcr", &format!("http://{address}")),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(&fixture, "hosted", hosted("second", "ALLOW")).await;
    publish(&fixture, "own", "shared", "1.0.0").await;
    publish(&fixture, "own", "private_lib", "1.0.0").await;
    publish(&fixture, "second", "shared", "1.0.0").await;
    publish(&fixture, "second", "shared", "1.5.0").await;
    let described = put(
        &fixture,
        "own",
        "/modules/shared/metadata.json",
        Auth::Admin,
        json!({"homepage": "https://own.example/shared", "yanked_versions": {"1.0.0": "own reason"}})
            .to_string()
            .as_bytes(),
    )
    .await;
    assert_eq!(described.status, StatusCode::CREATED);
    let second_note = put(
        &fixture,
        "second",
        "/modules/shared/metadata.json",
        Auth::Admin,
        json!({"maintainers": [{"github": "second"}], "yanked_versions": {"1.0.0": "second reason", "1.5.0": "too new"}})
            .to_string()
            .as_bytes(),
    )
    .await;
    assert_eq!(second_note.status, StatusCode::CREATED);
    assert_eq!(
        put(
            &fixture,
            "own",
            "/bazel_registry.json",
            Auth::Admin,
            json!({"mirrors": ["https://a.example/"]})
                .to_string()
                .as_bytes()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    create(&fixture, "group", group("all", &["own", "second", "bcr"])).await;
    create(
        &fixture,
        "group",
        group("all-reversed", &["bcr", "own", "second"]),
    )
    .await;

    for repository in ["all", "all-reversed"] {
        let shared = metadata(&fixture, repository, "shared").await;
        assert_eq!(
            shared.status,
            StatusCode::OK,
            "{repository}: {}",
            shared.text()
        );
        let shared = shared.json();
        assert_eq!(versions(&shared), ["1.0.0", "1.5.0"], "{repository}");
        assert_eq!(shared["homepage"], "https://own.example/shared");
        assert_eq!(shared["maintainers"][0]["github"], "second");
        assert_eq!(
            shared["yanked_versions"],
            json!({"1.0.0": "own reason", "1.5.0": "too new"})
        );
        let hidden = get(
            &fixture,
            &format!("/repository/{repository}/modules/shared/2.0.0/MODULE.bazel"),
            Auth::Admin,
        )
        .await;
        assert_eq!(hidden.status, StatusCode::NOT_FOUND, "{repository}");
        let first_wins = get(
            &fixture,
            &format!("/repository/{repository}/modules/shared/1.0.0/source.json"),
            Auth::Admin,
        )
        .await;
        assert_eq!(first_wins.json()["integrity"], sri(b"1.0.0"));
        let later_member = get(
            &fixture,
            &format!("/repository/{repository}/modules/shared/1.5.0/MODULE.bazel"),
            Auth::Admin,
        )
        .await;
        assert_eq!(later_member.status, StatusCode::OK);
        assert!(!later_member.text().contains("# upstream"));
    }
    assert_eq!(upstream.count("modules/shared/metadata.json"), 0);
    assert_eq!(upstream.count("modules/shared/2.0.0/MODULE.bazel"), 0);
    assert_eq!(upstream.count("modules/shared/1.0.0/MODULE.bazel"), 0);

    for repository in ["all", "all-reversed"] {
        let public = metadata(&fixture, repository, "public_lib").await;
        assert_eq!(public.status, StatusCode::OK, "{repository}");
        assert_eq!(versions(&public.json()), ["3.0.0"]);
        assert_eq!(public.json()["maintainers"][0]["github"], "far");
        assert_eq!(
            public.json()["yanked_versions"],
            json!({"3.0.0": "far reason"})
        );
        let module = get(
            &fixture,
            &format!("/repository/{repository}/modules/public_lib/3.0.0/MODULE.bazel"),
            Auth::Admin,
        )
        .await;
        assert_eq!(module.status, StatusCode::OK);
        assert!(module.text().contains("# upstream"));
        let private = get(
            &fixture,
            &format!("/repository/{repository}/modules/private_lib/1.0.0/MODULE.bazel"),
            Auth::Admin,
        )
        .await;
        assert_eq!(private.status, StatusCode::OK);
        assert_eq!(
            metadata(&fixture, repository, "no_such_lib").await.status,
            StatusCode::NOT_FOUND
        );
        let registry = get(
            &fixture,
            &format!("/repository/{repository}/bazel_registry.json"),
            Auth::Admin,
        )
        .await;
        let mirrors = registry.json()["mirrors"].clone();
        match repository {
            "all" => assert_eq!(mirrors, json!(["https://a.example/", "https://b.example/"])),
            _ => assert_eq!(mirrors, json!(["https://b.example/", "https://a.example/"])),
        }
    }

    let own_only = metadata(&fixture, "own", "public_lib").await;
    assert_eq!(own_only.status, StatusCode::NOT_FOUND);
    let probe = fixture
        .send(
            Method::HEAD,
            "/repository/all/modules/public_lib/metadata.json",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(probe.status, StatusCode::OK);
}
