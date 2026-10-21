mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_openvsx::names;
use astrum_format_openvsx::testing::{Spec, vsix};
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const GALLERY: &str = "application/json;api-version=3.0-preview.1";
const BOUNDARY: &str = "openvsx-form-boundary-0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_openvsx::OpenVsx));
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    fixture
}

fn app(fixture: &Fixture) -> Router {
    astrum_api::router_with(fixture.state.clone(), astrum_format_openvsx::routes())
}

async fn call(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    use base64::Engine;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let mut request = axum::http::Request::builder().method(method).uri(path);
    let credentials = match auth {
        Auth::None => None,
        Auth::Admin => Some(("admin".to_owned(), fixture.admin_password.clone())),
        Auth::Basic(user, password) => Some((user.to_owned(), password.to_owned())),
    };
    if let Some((user, password)) = credentials {
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"));
        request = request.header("authorization", format!("Basic {encoded}"));
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app(fixture)
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Reply {
        status,
        headers,
        body,
    }
}

async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    call(fixture, Method::GET, path, auth, Vec::new(), &[]).await
}

async fn publish(fixture: &Fixture, repository: &str, auth: Auth<'_>, package: &[u8]) -> Reply {
    call(
        fixture,
        Method::POST,
        &format!("/repository/{repository}/api/-/publish"),
        auth,
        package.to_vec(),
        &[("content-type", "application/octet-stream")],
    )
    .await
}

async fn gallery(fixture: &Fixture, repository: &str, auth: Auth<'_>, query: Value) -> Reply {
    call(
        fixture,
        Method::POST,
        &format!("/repository/{repository}/vscode/gallery/extensionquery"),
        auth,
        query.to_string().into_bytes(),
        &[("content-type", "application/json"), ("accept", GALLERY)],
    )
    .await
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/openvsx/{kind}"),
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

fn package(name: &str, version: &str, platform: Option<&str>) -> Vec<u8> {
    vsix(&Spec {
        name,
        version,
        platform,
        ..Spec::default()
    })
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn query(criteria: Value, flags: u64) -> Value {
    json!({
        "filters": [{"criteria": criteria, "pageNumber": 1, "pageSize": 10, "sortBy": 0, "sortOrder": 0}],
        "assetTypes": [],
        "flags": flags,
    })
}

fn versions_of(extension: &Value) -> Vec<(String, String)> {
    extension["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["version"].as_str().unwrap().to_owned(),
                entry["targetPlatform"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

fn source(entry: &Value, asset_type: &str) -> String {
    entry["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["assetType"] == asset_type)
        .and_then(|file| file["source"].as_str())
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn hosted_repositories_take_ovsx_publish_and_serve_the_registry_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("vsx", "ALLOW_ONCE")).await;
    let universal = package("hello", "1.0.0", None);
    let linux = package("hello", "1.0.0", Some("linux-x64"));
    let pre = vsix(&Spec {
        version: "1.1.0",
        pre_release: true,
        ..Spec::default()
    });

    let done = publish(&fixture, "vsx", Auth::Admin, &universal).await;
    assert_eq!(done.status, StatusCode::CREATED, "{}", done.text());
    let document = done.json();
    assert_eq!(document["namespace"], "acme");
    assert_eq!(document["version"], "1.0.0");
    assert_eq!(
        document["files"]["download"],
        "http://localhost/repository/vsx/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix"
    );
    assert_eq!(
        done.headers["location"],
        "http://localhost/repository/vsx/api/acme/hello/universal/1.0.0"
    );
    for package in [&linux, &pre] {
        let done = publish(&fixture, "vsx", Auth::Admin, package).await;
        assert_eq!(done.status, StatusCode::CREATED, "{}", done.text());
    }
    let again = publish(&fixture, "vsx", Auth::Admin, &universal).await;
    assert_eq!(again.status, StatusCode::CREATED);
    assert!(
        again.json()["warning"]
            .as_str()
            .unwrap()
            .contains("already published")
    );
    let changed = publish(
        &fixture,
        "vsx",
        Auth::Admin,
        &vsix(&Spec {
            extra: ", \"homepage\": \"https://changed.example\"",
            ..Spec::default()
        }),
    )
    .await;
    assert_eq!(changed.status, StatusCode::CONFLICT);
    assert!(
        changed.json()["error"]
            .as_str()
            .unwrap()
            .contains("is already published in vsx"),
        "{}",
        changed.text()
    );
    for (body, message) in [
        (b"not a package".to_vec(), "extension package"),
        (Vec::new(), "the body of the request"),
        (
            vsix(&Spec {
                version: "1.0",
                ..Spec::default()
            }),
            "semantic version",
        ),
    ] {
        let refused = publish(&fixture, "vsx", Auth::Admin, &body).await;
        assert_eq!(
            refused.status,
            StatusCode::BAD_REQUEST,
            "{}",
            refused.text()
        );
        let text = refused.json()["error"].as_str().unwrap().to_owned();
        assert!(text.contains(message), "{text}");
    }

    let latest = get(&fixture, "/repository/vsx/api/acme/hello", Auth::Admin).await;
    assert_eq!(latest.status, StatusCode::OK, "{}", latest.text());
    assert_eq!(latest.headers["content-type"], "application/json");
    assert_eq!(latest.headers["access-control-allow-origin"], "*");
    let latest = latest.json();
    assert_eq!(latest["version"], "1.0.0");
    assert_eq!(latest["targetPlatform"], "universal");
    assert_eq!(latest["versionAlias"], json!(["latest"]));
    assert_eq!(latest["displayName"], "Hello hello");
    assert_eq!(latest["engines"]["vscode"], "^1.80.0");
    assert_eq!(latest["dependencies"][0]["extension"], "base");
    assert_eq!(
        latest["downloads"]["linux-x64"],
        "http://localhost/repository/vsx/api/acme/hello/linux-x64/1.0.0/file/acme.hello-1.0.0@linux-x64.vsix"
    );
    assert_eq!(
        latest["allVersions"]["1.1.0"],
        "http://localhost/repository/vsx/api/acme/hello/1.1.0"
    );
    assert_eq!(
        latest["allVersions"]["pre-release"],
        "http://localhost/repository/vsx/api/acme/hello/pre-release"
    );
    let pre_doc = get(
        &fixture,
        "/repository/vsx/api/acme/hello/pre-release",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(pre_doc["version"], "1.1.0");
    assert_eq!(pre_doc["preRelease"], true);
    for (path, version, platform) in [
        (
            "/repository/vsx/api/acme/hello/linux-x64",
            "1.0.0",
            "linux-x64",
        ),
        (
            "/repository/vsx/api/acme/hello/darwin-arm64",
            "1.0.0",
            "universal",
        ),
        (
            "/repository/vsx/api/acme/hello/linux-x64/1.0.0",
            "1.0.0",
            "linux-x64",
        ),
        (
            "/repository/vsx/api/acme/hello/universal/1.1.0",
            "1.1.0",
            "universal",
        ),
        ("/repository/vsx/api/acme/hello/1.0.0", "1.0.0", "universal"),
    ] {
        let reply = get(&fixture, path, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
        assert_eq!(reply.json()["version"], version, "{path}");
        assert_eq!(reply.json()["targetPlatform"], platform, "{path}");
    }
    let missing = get(
        &fixture,
        "/repository/vsx/api/acme/hello/9.9.9",
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json()["error"],
        "Extension not found: acme.hello 9.9.9"
    );
    let namespace = get(&fixture, "/repository/vsx/api/acme", Auth::Admin)
        .await
        .json();
    assert_eq!(
        namespace["extensions"]["hello"],
        "http://localhost/repository/vsx/api/acme/hello"
    );
    let versions = get(
        &fixture,
        "/repository/vsx/api/acme/hello/versions",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions["totalSize"], 2);
    assert!(versions["versions"]["1.1.0"].is_string());
    let references = get(
        &fixture,
        "/repository/vsx/api/acme/hello/version-references?size=2",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(references["totalSize"], 3);
    assert_eq!(references["versions"].as_array().unwrap().len(), 2);
    assert_eq!(references["versions"][0]["version"], "1.1.0");
    let reviews = get(
        &fixture,
        "/repository/vsx/api/acme/hello/reviews",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(reviews["reviews"], json!([]));

    let search = get(
        &fixture,
        "/repository/vsx/api/-/search?query=hello",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(search["totalSize"], 1);
    assert_eq!(search["extensions"][0]["version"], "1.0.0");
    assert_eq!(
        search["extensions"][0]["url"],
        "http://localhost/repository/vsx/api/acme/hello"
    );
    let nothing = get(
        &fixture,
        "/repository/vsx/api/-/search?query=nothing",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(nothing["totalSize"], 0);
    let bad = get(
        &fixture,
        "/repository/vsx/api/-/search?targetPlatform=mars",
        Auth::Admin,
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let all = get(
        &fixture,
        "/repository/vsx/api/-/query?extensionId=acme.hello&includeAllVersions=true",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(all["totalSize"], 3);
    let one = get(
        &fixture,
        "/repository/vsx/api/v2/-/query?extensionName=hello&namespaceName=ACME",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(one["totalSize"], 1);
    assert_eq!(one["extensions"][0]["version"], "1.0.0");
    let posted = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/-/query",
        Auth::Admin,
        json!({"extensionId": "acme.hello", "extensionVersion": "1.0.0"})
            .to_string()
            .into_bytes(),
        &[("content-type", "application/json")],
    )
    .await
    .json();
    assert_eq!(posted["totalSize"], 2);

    let download = get(
        &fixture,
        "/repository/vsx/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix",
        Auth::Admin,
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.headers["content-type"], "application/vsix");
    assert_eq!(download.body, universal);
    let linux_download = get(
        &fixture,
        "/repository/vsx/api/acme/hello/linux-x64/1.0.0/file/acme.hello-1.0.0@linux-x64.vsix",
        Auth::Admin,
    )
    .await;
    assert_eq!(linux_download.body, linux);
    for (file, expected) in [
        ("README.md", b"# hello 1.0.0\n".to_vec()),
        ("CHANGELOG.md", b"# Changes\n".to_vec()),
        ("LICENSE.txt", b"MIT\n".to_vec()),
        ("acme.hello-1.0.0.sha256", sha256(&universal).into_bytes()),
    ] {
        let reply = get(
            &fixture,
            &format!("/repository/vsx/api/acme/hello/1.0.0/file/{file}"),
            Auth::Admin,
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{file}");
        assert_eq!(reply.body, expected, "{file}");
    }
    let manifest = get(
        &fixture,
        "/repository/vsx/api/acme/hello/1.0.0/file/package.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(manifest.headers["content-type"], "application/json");
    assert_eq!(manifest.json()["publisher"], "acme");
    let vsixmanifest = get(
        &fixture,
        "/repository/vsx/api/acme/hello/1.0.0/file/extension.vsixmanifest",
        Auth::Admin,
    )
    .await;
    assert!(vsixmanifest.text().contains("PackageManifest"));
    let unknown = get(
        &fixture,
        "/repository/vsx/api/acme/hello/1.0.0/file/main.js",
        Auth::Admin,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    let registry = get(&fixture, "/repository/vsx/api/version", Auth::None).await;
    assert_eq!(registry.status, StatusCode::OK);
    assert!(registry.json()["version"].is_string());
    let verified = get(&fixture, "/repository/vsx/api/acme/verify-pat", Auth::Admin).await;
    assert_eq!(verified.status, StatusCode::OK, "{}", verified.text());
    assert!(verified.json()["success"].is_string());
    let namespace = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/-/namespace/create",
        Auth::Admin,
        br#"{"name": "acme"}"#.to_vec(),
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(
        namespace.status,
        StatusCode::CREATED,
        "{}",
        namespace.text()
    );
    let item = get(
        &fixture,
        "/repository/vsx/vscode/item?itemName=acme.hello",
        Auth::Admin,
    )
    .await;
    assert_eq!(item.status, StatusCode::FOUND);
    assert_eq!(
        item.headers["location"],
        "http://localhost/browse/vsx/acme/hello"
    );
    let page = get(&fixture, "/repository/vsx/", Auth::Admin).await;
    assert!(
        page.text()
            .contains("http://localhost/repository/vsx/vscode/gallery")
    );

    let searched = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=vsx&openvsx.targetPlatform=linux-x64",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(searched["items"].as_array().unwrap().len(), 1, "{searched}");
    assert_eq!(searched["items"][0]["group"], "acme");
    assert_eq!(searched["items"][0]["name"], "hello");
}

#[tokio::test]
async fn the_gallery_answers_editor_queries() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("vsx", "ALLOW_ONCE")).await;
    let universal = package("hello", "1.0.0", None);
    let linux = package("hello", "1.0.0", Some("linux-x64"));
    for package in [
        universal.clone(),
        linux.clone(),
        package("hello", "0.9.0", None),
        package("other", "2.0.0", None),
    ] {
        let done = publish(&fixture, "vsx", Auth::Admin, &package).await;
        assert_eq!(done.status, StatusCode::CREATED, "{}", done.text());
    }
    let named = gallery(
        &fixture,
        "vsx",
        Auth::Admin,
        query(
            json!([
                {"filterType": 8, "value": "Microsoft.VisualStudio.Code"},
                {"filterType": 7, "value": "Acme.Hello"},
                {"filterType": 12, "value": "4096"}
            ]),
            950,
        ),
    )
    .await;
    assert_eq!(named.status, StatusCode::OK, "{}", named.text());
    assert_eq!(named.headers["content-type"], GALLERY);
    assert_eq!(named.headers["access-control-allow-origin"], "*");
    let result = &named.json()["results"][0];
    assert_eq!(result["resultMetadata"][0]["metadataItems"][0]["count"], 1);
    let extension = &result["extensions"][0];
    let uuid = names::extension_uuid("acme", "hello");
    assert_eq!(extension["extensionId"], uuid);
    assert_eq!(extension["extensionName"], "hello");
    assert_eq!(extension["publisher"]["publisherName"], "acme");
    assert_eq!(
        versions_of(extension),
        pairs(&[("1.0.0", "universal"), ("1.0.0", "linux-x64")])
    );
    let entry = &extension["versions"][0];
    assert_eq!(
        entry["assetUri"],
        "http://localhost/repository/vsx/vscode/asset/acme/hello/1.0.0"
    );
    assert_eq!(
        source(entry, "Microsoft.VisualStudio.Services.VSIXPackage"),
        "http://localhost/repository/vsx/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix"
    );
    assert_eq!(
        source(entry, "Microsoft.VisualStudio.Code.Manifest"),
        "http://localhost/repository/vsx/api/acme/hello/1.0.0/file/package.json"
    );
    assert!(
        entry["properties"]
            .as_array()
            .unwrap()
            .contains(&json!({"key": "Microsoft.VisualStudio.Code.Engine", "value": "^1.80.0"}))
    );
    let by_id = gallery(
        &fixture,
        "vsx",
        Auth::Admin,
        query(json!([{"filterType": 4, "value": uuid}]), 1),
    )
    .await
    .json();
    assert_eq!(
        by_id["results"][0]["extensions"][0]["extensionName"],
        "hello"
    );
    assert_eq!(
        by_id["results"][0]["extensions"][0]["versions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let text = gallery(
        &fixture,
        "vsx",
        Auth::Admin,
        query(json!([{"filterType": 10, "value": "other"}]), 0),
    )
    .await
    .json();
    assert_eq!(
        text["results"][0]["resultMetadata"][0]["metadataItems"][0]["count"],
        1
    );
    assert_eq!(
        text["results"][0]["extensions"][0]["extensionName"],
        "other"
    );
    assert_eq!(text["results"][0]["extensions"][0]["versions"], json!([]));
    let paged = gallery(
        &fixture,
        "vsx",
        Auth::Admin,
        json!({"filters": [{"criteria": [], "pageNumber": 2, "pageSize": 1, "sortBy": 2, "sortOrder": 1}], "flags": 0}),
    )
    .await
    .json();
    assert_eq!(
        paged["results"][0]["resultMetadata"][0]["metadataItems"][0]["count"],
        2
    );
    assert_eq!(
        paged["results"][0]["extensions"][0]["extensionName"],
        "other"
    );
    let q = url::form_urlencoded::Serializer::new(String::new())
        .append_pair(
            "q",
            &query(json!([{"filterType": 7, "value": "acme.other"}]), 512).to_string(),
        )
        .finish();
    let by_get = get(
        &fixture,
        &format!("/repository/vsx/vscode/gallery/extensionquery?{q}"),
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        by_get["results"][0]["extensions"][0]["extensionName"],
        "other"
    );
    let latest = get(
        &fixture,
        "/repository/vsx/vscode/gallery/acme/hello/latest",
        Auth::Admin,
    )
    .await;
    assert_eq!(latest.status, StatusCode::OK);
    assert_eq!(
        versions_of(&latest.json()),
        pairs(&[("1.0.0", "universal"), ("1.0.0", "linux-x64")])
    );

    for (path, expected) in [
        (
            "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Services.VSIXPackage?targetPlatform=linux-x64",
            linux.clone(),
        ),
        (
            "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Services.VSIXPackage",
            universal.clone(),
        ),
        (
            "/repository/vsx/vscode/gallery/publishers/acme/vsextensions/hello/1.0.0/vspackage?targetPlatform=darwin-arm64",
            universal.clone(),
        ),
        (
            "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Services.Content.Details",
            b"# hello 1.0.0\n".to_vec(),
        ),
        (
            "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Services.Icons.Default",
            b"\x89PNG\r\n\x1a\nicon".to_vec(),
        ),
        (
            "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Code.WebResources/extension/out/main.js",
            b"exports.activate = () => {};\n".to_vec(),
        ),
        (
            "/repository/vsx/vscode/unpkg/acme/hello/1.0.0/extension/CHANGELOG.md",
            b"# Changes\n".to_vec(),
        ),
    ] {
        let reply = get(&fixture, path, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
        assert_eq!(reply.body, expected, "{path}");
    }
    let manifest = get(
        &fixture,
        "/repository/vsx/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Code.Manifest",
        Auth::Admin,
    )
    .await;
    assert_eq!(manifest.json()["name"], "hello");
    let listing = get(
        &fixture,
        "/repository/vsx/vscode/unpkg/acme/hello/1.0.0/extension/",
        Auth::Admin,
    )
    .await
    .json();
    let listed: Vec<&str> = listing
        .as_array()
        .unwrap()
        .iter()
        .map(|url| url.as_str().unwrap())
        .collect();
    assert!(listed.contains(
        &"http://localhost/repository/vsx/vscode/unpkg/acme/hello/1.0.0/extension/README.md"
    ));
    assert!(
        listed.contains(
            &"http://localhost/repository/vsx/vscode/unpkg/acme/hello/1.0.0/extension/out/"
        )
    );
    let missing = get(
        &fixture,
        "/repository/vsx/vscode/asset/acme/hello/7.0.0/Microsoft.VisualStudio.Services.VSIXPackage",
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let preflight = call(
        &fixture,
        Method::OPTIONS,
        "/repository/vsx/vscode/gallery/extensionquery",
        Auth::None,
        Vec::new(),
        &[
            ("origin", "https://editor.example"),
            ("access-control-request-method", "POST"),
            (
                "access-control-request-headers",
                "content-type,x-market-client-id",
            ),
        ],
    )
    .await;
    assert_eq!(preflight.status, StatusCode::NO_CONTENT);
    assert_eq!(preflight.headers["access-control-allow-origin"], "*");
    assert_eq!(
        preflight.headers["access-control-allow-headers"],
        "content-type,x-market-client-id"
    );
}

#[tokio::test]
async fn tokens_challenges_and_privileges_follow_the_ovsx_client() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("closed", "ALLOW")).await;
    let first = package("hello", "1.0.0", None);
    assert_eq!(
        publish(&fixture, "closed", Auth::Admin, &first)
            .await
            .status,
        StatusCode::CREATED
    );

    let anonymous = get(&fixture, "/repository/closed/api/acme/hello", Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous.headers["www-authenticate"],
        "Basic realm=\"Astrum Repository\""
    );
    assert!(anonymous.json()["error"].is_string());
    let anonymous_gallery = gallery(&fixture, "closed", Auth::None, query(json!([]), 0)).await;
    assert_eq!(anonymous_gallery.status, StatusCode::UNAUTHORIZED);
    let anonymous_file = get(
        &fixture,
        "/repository/closed/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix",
        Auth::None,
    )
    .await;
    assert_eq!(anonymous_file.status, StatusCode::UNAUTHORIZED);
    let anonymous_publish = publish(&fixture, "closed", Auth::None, &first).await;
    assert_eq!(anonymous_publish.status, StatusCode::UNAUTHORIZED);

    fixture.add_user("stranger", "stranger-password", &[]).await;
    let stranger = Auth::Basic("stranger", "stranger-password");
    assert_eq!(
        get(&fixture, "/repository/closed/api/acme/hello", stranger)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        publish(
            &fixture,
            "closed",
            Auth::Basic("stranger", "stranger-password"),
            &first
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );

    for (role, actions) in [
        ("vsx-readers", vec!["read", "browse"]),
        ("vsx-deployers", vec!["read", "browse", "add"]),
    ] {
        let privileges: Vec<String> = actions
            .iter()
            .map(|action| format!("as-repository-view-openvsx-closed-{action}"))
            .collect();
        let reply = fixture
            .call(
                Method::POST,
                "/service/rest/v1/security/roles",
                Auth::Admin,
                Some(json!({"id": role, "name": role, "description": "", "privileges": privileges, "roles": []})),
            )
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-password", &["vsx-readers"])
        .await;
    fixture
        .add_user("deployer", "deployer-password", &["vsx-deployers"])
        .await;
    let read = get(
        &fixture,
        "/repository/closed/api/acme/hello",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    let refused = publish(
        &fixture,
        "closed",
        Auth::Basic("reader", "reader-password"),
        &package("hello", "1.1.0", None),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert!(refused.json()["error"].is_string());
    let verify = get(
        &fixture,
        "/repository/closed/api/acme/verify-pat",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(verify.status, StatusCode::FORBIDDEN);
    let deployed = publish(
        &fixture,
        "closed",
        Auth::Basic("deployer", "deployer-password"),
        &package("hello", "1.1.0", None),
    )
    .await;
    assert_eq!(deployed.status, StatusCode::CREATED, "{}", deployed.text());

    let (_, token) = fixture
        .state
        .security()
        .create_token("deployer", "ovsx", Scope::Write, None)
        .await
        .unwrap();
    let bearer = format!("Bearer {token}");
    let with_bearer = call(
        &fixture,
        Method::POST,
        "/repository/closed/api/-/publish",
        Auth::None,
        package("hello", "1.2.0", Some("linux-x64")),
        &[
            ("content-type", "application/octet-stream"),
            ("authorization", &bearer),
        ],
    )
    .await;
    assert_eq!(
        with_bearer.status,
        StatusCode::CREATED,
        "{}",
        with_bearer.text()
    );
    let verified = call(
        &fixture,
        Method::GET,
        &format!("/repository/closed/api/acme/verify-pat?token={token}"),
        Auth::None,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(verified.status, StatusCode::OK, "{}", verified.text());
    let in_query = call(
        &fixture,
        Method::POST,
        &format!("/repository/closed/api/-/publish?token={token}"),
        Auth::None,
        package("hello", "1.3.0", None),
        &[("content-type", "application/octet-stream")],
    )
    .await;
    assert_eq!(in_query.status, StatusCode::CREATED, "{}", in_query.text());
    let password_token = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("token", "deployer:deployer-password")
        .finish();
    let with_password = call(
        &fixture,
        Method::POST,
        &format!("/repository/closed/api/-/publish?{password_token}"),
        Auth::None,
        package("hello", "1.4.0", None),
        &[
            ("content-type", "application/octet-stream"),
            ("authorization", "Bearer deployer:deployer-password"),
        ],
    )
    .await;
    assert_eq!(
        with_password.status,
        StatusCode::CREATED,
        "{}",
        with_password.text()
    );
    let header_only = call(
        &fixture,
        Method::POST,
        "/repository/closed/api/-/publish",
        Auth::None,
        package("hello", "1.4.1", None),
        &[
            ("content-type", "application/octet-stream"),
            ("authorization", "Bearer deployer:deployer-password"),
        ],
    )
    .await;
    assert_eq!(
        header_only.status,
        StatusCode::CREATED,
        "{}",
        header_only.text()
    );
    for (wrong, message) in [
        ("deployer:wrong", "user name or password is wrong"),
        (
            "astrum_0000000000000000000000000000000000000000",
            "access token is not valid",
        ),
        ("plain", "access token of Astrum Repository"),
    ] {
        let rejected = call(
            &fixture,
            Method::POST,
            &format!("/repository/closed/api/-/publish?token={wrong}"),
            Auth::None,
            package("hello", "1.5.0", None),
            &[("content-type", "application/octet-stream")],
        )
        .await;
        assert_eq!(rejected.status, StatusCode::UNAUTHORIZED, "{wrong}");
        assert!(
            rejected.json()["error"].as_str().unwrap().contains(message),
            "{}",
            rejected.text()
        );
    }
    let (_, read_token) = fixture
        .state
        .security()
        .create_token("reader", "ovsx", Scope::Write, None)
        .await
        .unwrap();
    let unpublish_refused = call(
        &fixture,
        Method::POST,
        &format!("/repository/closed/api/acme/hello/delete?token={read_token}&allVersions=true"),
        Auth::None,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(unpublish_refused.status, StatusCode::FORBIDDEN);
    let versions = get(
        &fixture,
        "/repository/closed/api/acme/hello/versions",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions["totalSize"], 6);
    let ui = call(
        &fixture,
        Method::GET,
        "/repository/closed/api/acme/hello",
        Auth::None,
        Vec::new(),
        &[("x-astrum-ui", "true")],
    )
    .await;
    assert_eq!(ui.status, StatusCode::UNAUTHORIZED);
    assert!(ui.headers.get("www-authenticate").is_none());
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<(String, String)>>>,
    address: Arc<Mutex<String>>,
    latest: Arc<Vec<u8>>,
    linux: Arc<Vec<u8>>,
    old: Arc<Vec<u8>>,
}

fn version_document(base: &str, version: &str, platform: &str) -> Value {
    let (dir, stem) = match platform {
        "universal" => (
            format!("http://{base}/api/far/away/{version}"),
            format!("far.away-{version}"),
        ),
        _ => (
            format!("http://{base}/api/far/away/{platform}/{version}"),
            format!("far.away-{version}@{platform}"),
        ),
    };
    let mut downloads = json!({"universal": format!("http://{base}/api/far/away/{version}/file/far.away-{version}.vsix")});
    if version == "2.0.0" {
        downloads["linux-x64"] = json!(format!(
            "http://{base}/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix"
        ));
    }
    json!({
        "namespace": "far",
        "name": "away",
        "version": version,
        "targetPlatform": platform,
        "displayName": "Far Away",
        "description": "From far away.",
        "categories": ["Themes"],
        "tags": ["far"],
        "engines": {"vscode": "^1.70.0"},
        "homepage": "https://example.org/far",
        "namespaceUrl": format!("http://{base}/api/far"),
        "url": format!("http://{base}/api/far/away/{platform}/{version}"),
        "files": {
            "download": format!("{dir}/file/{stem}.vsix"),
            "readme": format!("{dir}/file/README.md"),
            "sha256": format!("{dir}/file/{stem}.sha256"),
        },
        "downloads": downloads,
        "allVersions": {
            "latest": format!("http://{base}/api/far/away/latest"),
            "2.0.0": format!("http://{base}/api/far/away/2.0.0"),
            "1.0.0": format!("http://{base}/api/far/away/1.0.0"),
        },
        "dependencies": [{"namespace": "acme", "extension": "base", "url": format!("http://{base}/api/acme/base")}],
        "preRelease": false,
        "timestamp": "2025-01-01T00:00:00Z",
    })
}

fn gallery_entry(base: &str, version: &str, platform: &str) -> Value {
    let vsix = match platform {
        "universal" => format!("http://{base}/api/far/away/{version}/file/far.away-{version}.vsix"),
        _ => format!(
            "http://{base}/api/far/away/{platform}/{version}/file/far.away-{version}@{platform}.vsix"
        ),
    };
    json!({
        "version": version,
        "targetPlatform": platform,
        "assetUri": format!("http://{base}/vscode/asset/far/away/{version}"),
        "fallbackAssetUri": format!("http://{base}/vscode/asset/far/away/{version}"),
        "files": [{"assetType": "Microsoft.VisualStudio.Services.VSIXPackage", "source": vsix}],
        "properties": [{"key": "Microsoft.VisualStudio.Code.Engine", "value": "^1.70.0"}],
    })
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let query = query.unwrap_or_default();
    upstream
        .hits
        .lock()
        .unwrap()
        .push((path.clone(), query.clone()));
    let base = upstream.address.lock().unwrap().clone();
    let document = |value: Value| value.to_string().into_response();
    match path.trim_end_matches('/') {
        "api/far" => document(
            json!({"name": "far", "extensions": {"away": format!("http://{base}/api/far/away")}}),
        ),
        "api/far/away"
        | "api/far/away/latest"
        | "api/far/away/2.0.0"
        | "api/far/away/universal/2.0.0" => document(version_document(&base, "2.0.0", "universal")),
        "api/far/away/linux-x64" | "api/far/away/linux-x64/2.0.0" => {
            document(version_document(&base, "2.0.0", "linux-x64"))
        }
        "api/far/away/1.0.0" | "api/far/away/universal/1.0.0" => {
            document(version_document(&base, "1.0.0", "universal"))
        }
        "api/far/away/versions" => document(json!({
            "offset": 0,
            "totalSize": 2,
            "versions": {"2.0.0": format!("http://{base}/api/far/away/2.0.0"), "1.0.0": format!("http://{base}/api/far/away/1.0.0")},
        })),
        "api/far/away/2.0.0/file/far.away-2.0.0.vsix" => {
            upstream.latest.as_ref().clone().into_response()
        }
        "api/far/away/2.0.0/file/far.away-2.0.0.sha256" => sha256(&upstream.latest).into_response(),
        "api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix" => {
            upstream.linux.as_ref().clone().into_response()
        }
        "api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.sha256" => {
            sha256(&upstream.linux).into_response()
        }
        "api/far/away/1.0.0/file/far.away-1.0.0.vsix" => {
            upstream.old.as_ref().clone().into_response()
        }
        "api/far/away/1.0.0/file/far.away-1.0.0.sha256" => "00".repeat(32).into_response(),
        "api/far/away/2.0.0/file/README.md" => "# far away\n".into_response(),
        "api/-/search" => document(json!({
            "offset": 0,
            "totalSize": 1,
            "extensions": [{
                "namespace": "far",
                "name": "away",
                "version": "2.0.0",
                "url": format!("http://{base}/api/far/away"),
                "files": {"download": format!("http://{base}/api/far/away/2.0.0/file/far.away-2.0.0.vsix")},
            }],
        })),
        "api/-/query" => document(json!({
            "offset": 0,
            "totalSize": 1,
            "extensions": [version_document(&base, "2.0.0", "universal")],
        })),
        "vscode/gallery/extensionquery" => document(json!({"results": [{
            "extensions": [{
                "extensionId": "11111111-2222-3333-4444-555555555555",
                "extensionName": "away",
                "displayName": "Far Away",
                "shortDescription": "From far away.",
                "publisher": {"publisherId": "66666666-7777-8888-9999-000000000000", "publisherName": "far", "displayName": "Far"},
                "versions": [
                    gallery_entry(&base, "2.0.0", "universal"),
                    gallery_entry(&base, "2.0.0", "linux-x64"),
                    gallery_entry(&base, "1.0.0", "universal"),
                ],
                "statistics": [],
                "tags": ["far"],
                "categories": ["Themes"],
                "flags": "validated, public",
            }],
            "resultMetadata": [{"metadataType": "ResultCount", "metadataItems": [{"name": "TotalCount", "count": 1}]}],
        }]})),
        "vscode/unpkg/far/away/2.0.0/extension/package.json" => {
            r#"{"name": "away", "publisher": "far"}"#.into_response()
        }
        "vscode/unpkg/far/away/2.0.0/extension" => document(json!([format!(
            "http://{base}/vscode/unpkg/far/away/2.0.0/extension/package.json"
        )])),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

fn hit_count(upstream: &Upstream, path: &str) -> usize {
    upstream
        .hits
        .lock()
        .unwrap()
        .iter()
        .filter(|(hit, _)| hit == path)
        .count()
}

#[tokio::test]
async fn proxies_rewrite_addresses_and_groups_merge_their_members() {
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
    let far = |version: &'static str, platform: Option<&'static str>, extra: &'static str| {
        vsix(&Spec {
            namespace: "far",
            name: "away",
            version,
            platform,
            extra,
            ..Spec::default()
        })
    };
    let upstream = Upstream {
        latest: Arc::new(far("2.0.0", None, "")),
        linux: Arc::new(far("2.0.0", Some("linux-x64"), "")),
        old: Arc::new(far("1.0.0", None, "")),
        ..Default::default()
    };
    let remote = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    *upstream.address.lock().unwrap() = address.to_string();
    tokio::spawn(async move { axum::serve(listener, remote).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "vsx-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW_ONCE")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "vsx-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "vsx-proxy"]},
        }),
    )
    .await;
    let own = far("2.0.0", None, ", \"homepage\": \"https://own.example\"");
    for package in [own.clone(), package("hello", "1.0.0", None)] {
        let done = publish(&fixture, "own", Auth::Admin, &package).await;
        assert_eq!(done.status, StatusCode::CREATED, "{}", done.text());
    }

    let latest = get(&fixture, "/repository/vsx-proxy/api/far/away", Auth::Admin).await;
    assert_eq!(latest.status, StatusCode::OK, "{}", latest.text());
    let latest = latest.json();
    assert_eq!(
        latest["files"]["download"],
        "http://localhost/repository/vsx-proxy/api/far/away/2.0.0/file/far.away-2.0.0.vsix"
    );
    assert_eq!(
        latest["downloads"]["linux-x64"],
        "http://localhost/repository/vsx-proxy/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix"
    );
    assert_eq!(
        latest["namespaceUrl"],
        "http://localhost/repository/vsx-proxy/api/far"
    );
    assert_eq!(latest["homepage"], "https://example.org/far");
    for _ in 0..2 {
        let download = get(
            &fixture,
            "/repository/vsx-proxy/api/far/away/2.0.0/file/far.away-2.0.0.vsix",
            Auth::Admin,
        )
        .await;
        assert_eq!(download.status, StatusCode::OK, "{}", download.text());
        assert_eq!(download.body, *upstream.latest);
    }
    assert_eq!(
        hit_count(&upstream, "api/far/away/2.0.0/file/far.away-2.0.0.vsix"),
        1
    );
    assert_eq!(
        hit_count(&upstream, "api/far/away/2.0.0/file/far.away-2.0.0.sha256"),
        1
    );
    let mismatch = get(
        &fixture,
        "/repository/vsx-proxy/api/far/away/1.0.0/file/far.away-1.0.0.vsix",
        Auth::Admin,
    )
    .await;
    assert_eq!(
        mismatch.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        mismatch.text()
    );
    let cached = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=vsx-proxy&name=away",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(cached["items"].as_array().unwrap().len(), 1, "{cached}");
    assert_eq!(cached["items"][0]["group"], "far");
    assert_eq!(cached["items"][0]["version"], "2.0.0");

    let search = get(
        &fixture,
        "/repository/vsx-proxy/api/-/search?query=far",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        search["extensions"][0]["url"],
        "http://localhost/repository/vsx-proxy/api/far/away"
    );
    assert!(
        upstream
            .hits
            .lock()
            .unwrap()
            .iter()
            .any(|(path, query)| path == "api/-/search" && query.contains("query=far"))
    );
    let lookup = get(
        &fixture,
        "/repository/vsx-proxy/api/-/query?extensionId=far.away",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        lookup["extensions"][0]["files"]["readme"],
        "http://localhost/repository/vsx-proxy/api/far/away/2.0.0/file/README.md"
    );
    let found = gallery(
        &fixture,
        "vsx-proxy",
        Auth::Admin,
        query(json!([{"filterType": 7, "value": "far.away"}]), 1),
    )
    .await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.text());
    assert_eq!(found.headers["content-type"], GALLERY);
    let extension = found.json()["results"][0]["extensions"][0].clone();
    assert_eq!(
        extension["versions"][0]["assetUri"],
        "http://localhost/repository/vsx-proxy/vscode/asset/far/away/2.0.0"
    );
    assert_eq!(
        source(
            &extension["versions"][1],
            "Microsoft.VisualStudio.Services.VSIXPackage"
        ),
        "http://localhost/repository/vsx-proxy/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix"
    );
    let asked = upstream
        .hits
        .lock()
        .unwrap()
        .iter()
        .find(|(path, _)| path == "vscode/gallery/extensionquery")
        .map(|(_, query)| query.clone())
        .unwrap();
    let forwarded: Value = serde_json::from_str(
        &url::form_urlencoded::parse(asked.as_bytes())
            .find(|(key, _)| key == "q")
            .unwrap()
            .1,
    )
    .unwrap();
    assert_eq!(forwarded["filters"][0]["criteria"][0]["value"], "far.away");
    assert_eq!(forwarded["flags"], 1);

    let readme = get(
        &fixture,
        "/repository/vsx-proxy/vscode/asset/far/away/2.0.0/Microsoft.VisualStudio.Services.Content.Details",
        Auth::Admin,
    )
    .await;
    assert_eq!(readme.status, StatusCode::OK, "{}", readme.text());
    assert_eq!(readme.text(), "# far away\n");
    let linux = get(
        &fixture,
        "/repository/vsx-proxy/vscode/asset/far/away/2.0.0/Microsoft.VisualStudio.Services.VSIXPackage?targetPlatform=linux-x64",
        Auth::Admin,
    )
    .await;
    assert_eq!(linux.body, *upstream.linux);
    let manifest = get(
        &fixture,
        "/repository/vsx-proxy/vscode/unpkg/far/away/2.0.0/extension/package.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(manifest.status, StatusCode::OK, "{}", manifest.text());
    assert_eq!(manifest.json()["name"], "away");
    let listing = get(
        &fixture,
        "/repository/vsx-proxy/vscode/unpkg/far/away/2.0.0/extension/",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        listing,
        json!([
            "http://localhost/repository/vsx-proxy/vscode/unpkg/far/away/2.0.0/extension/package.json"
        ])
    );
    let item = get(
        &fixture,
        "/repository/vsx-proxy/vscode/item?itemName=far.away",
        Auth::Admin,
    )
    .await;
    assert_eq!(item.status, StatusCode::FOUND);
    assert_eq!(
        item.headers["location"].to_str().unwrap(),
        format!("http://{address}/vscode/item?itemName=far.away")
    );
    let refused = publish(&fixture, "vsx-proxy", Auth::Admin, &own).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let merged = get(&fixture, "/repository/vsx-all/api/far/away", Auth::Admin).await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    let merged = merged.json();
    assert_eq!(merged["version"], "2.0.0");
    assert_eq!(merged["homepage"], "https://own.example");
    assert_eq!(
        merged["files"]["download"],
        "http://localhost/repository/vsx-all/api/far/away/2.0.0/file/far.away-2.0.0.vsix"
    );
    assert_eq!(
        merged["downloads"]["linux-x64"],
        "http://localhost/repository/vsx-all/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix"
    );
    assert_eq!(
        merged["allVersions"]["1.0.0"],
        "http://localhost/repository/vsx-all/api/far/away/1.0.0"
    );
    let first_wins = get(
        &fixture,
        "/repository/vsx-all/api/far/away/2.0.0/file/far.away-2.0.0.vsix",
        Auth::Admin,
    )
    .await;
    assert_eq!(first_wins.body, own);
    let from_proxy = get(
        &fixture,
        "/repository/vsx-all/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix",
        Auth::Admin,
    )
    .await;
    assert_eq!(from_proxy.body, *upstream.linux);
    let group_gallery = gallery(
        &fixture,
        "vsx-all",
        Auth::Admin,
        query(json!([{"filterType": 7, "value": "far.away"}]), 1),
    )
    .await
    .json();
    let extension = &group_gallery["results"][0]["extensions"][0];
    assert_eq!(
        extension["extensionId"],
        names::extension_uuid("far", "away")
    );
    assert_eq!(
        versions_of(extension),
        pairs(&[
            ("2.0.0", "universal"),
            ("2.0.0", "linux-x64"),
            ("1.0.0", "universal")
        ])
    );
    assert_eq!(
        source(
            &extension["versions"][0],
            "Microsoft.VisualStudio.Services.VSIXPackage"
        ),
        "http://localhost/repository/vsx-all/api/far/away/2.0.0/file/far.away-2.0.0.vsix"
    );
    assert_eq!(
        source(
            &extension["versions"][1],
            "Microsoft.VisualStudio.Services.VSIXPackage"
        ),
        "http://localhost/repository/vsx-all/api/far/away/linux-x64/2.0.0/file/far.away-2.0.0@linux-x64.vsix"
    );
    let group_asset = get(
        &fixture,
        "/repository/vsx-all/vscode/asset/far/away/2.0.0/Microsoft.VisualStudio.Services.VSIXPackage",
        Auth::Admin,
    )
    .await;
    assert_eq!(group_asset.body, own);
    let group_search = get(&fixture, "/repository/vsx-all/api/-/search", Auth::Admin)
        .await
        .json();
    assert_eq!(group_search["totalSize"], 2, "{group_search}");
    let group_versions = get(
        &fixture,
        "/repository/vsx-all/api/far/away/versions",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(group_versions["totalSize"], 2, "{group_versions}");
    let group_namespace = get(&fixture, "/repository/vsx-all/api/far", Auth::Admin)
        .await
        .json();
    assert_eq!(
        group_namespace["extensions"]["away"],
        "http://localhost/repository/vsx-all/api/far/away"
    );
    let hello = get(&fixture, "/repository/vsx-all/api/acme/hello", Auth::Admin).await;
    assert_eq!(hello.status, StatusCode::OK);
    let group_publish = publish(&fixture, "vsx-all", Auth::Admin, &own).await;
    assert_eq!(group_publish.status, StatusCode::BAD_REQUEST);
    assert!(
        group_publish.json()["error"]
            .as_str()
            .unwrap()
            .contains("hosted")
    );
    let nothing = get(
        &fixture,
        "/repository/vsx-all/api/nobody/nothing",
        Auth::Admin,
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deleting_packages_updates_the_metadata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("vsx", "ALLOW_ONCE")).await;
    for package in [
        package("hello", "1.0.0", None),
        package("hello", "1.1.0", None),
        package("hello", "1.1.0", Some("linux-x64")),
    ] {
        assert_eq!(
            publish(&fixture, "vsx", Auth::Admin, &package).await.status,
            StatusCode::CREATED
        );
    }
    let deleted = call(
        &fixture,
        Method::DELETE,
        "/repository/vsx/api/acme/hello/1.1.0/file/acme.hello-1.1.0.vsix",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let latest = get(&fixture, "/repository/vsx/api/acme/hello", Auth::Admin)
        .await
        .json();
    assert_eq!(latest["version"], "1.1.0");
    assert_eq!(latest["targetPlatform"], "linux-x64");
    assert!(latest["downloads"].get("universal").is_none());
    let unpublished = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/acme/hello/delete",
        Auth::Admin,
        json!([{"version": "1.1.0", "targetPlatform": "linux-x64"}])
            .to_string()
            .into_bytes(),
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(unpublished.status, StatusCode::OK, "{}", unpublished.text());
    let latest = get(&fixture, "/repository/vsx/api/acme/hello", Auth::Admin)
        .await
        .json();
    assert_eq!(latest["version"], "1.0.0");
    let found = gallery(
        &fixture,
        "vsx",
        Auth::Admin,
        query(json!([{"filterType": 7, "value": "acme.hello"}]), 1),
    )
    .await
    .json();
    assert_eq!(
        versions_of(&found["results"][0]["extensions"][0]),
        pairs(&[("1.0.0", "universal")])
    );
    let unnamed = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/acme/hello/delete",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(unnamed.status, StatusCode::BAD_REQUEST);
    let everything = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/acme/hello/delete?allVersions=true",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(everything.status, StatusCode::OK, "{}", everything.text());
    for path in [
        "/repository/vsx/api/acme/hello",
        "/repository/vsx/api/acme",
        "/repository/vsx/vscode/gallery/acme/hello/latest",
    ] {
        assert_eq!(
            get(&fixture, path, Auth::Admin).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    let search = get(&fixture, "/repository/vsx/api/-/search", Auth::Admin)
        .await
        .json();
    assert_eq!(search["totalSize"], 0);
    let again = call(
        &fixture,
        Method::POST,
        "/repository/vsx/api/acme/hello/delete?allVersions=true",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
}

fn form(field: &str, file: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"extension.vsix\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

#[tokio::test]
async fn the_rest_api_and_the_upload_form_take_open_vsx_repositories() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("vsx-hosted", "ALLOW_ONCE")).await;
    let stored = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/openvsx/hosted/vsx-hosted",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.status, StatusCode::OK);
    let document = stored.json();
    assert_eq!(document["format"], "openvsx");
    assert_eq!(document["type"], "hosted");
    assert_eq!(document["url"], "http://localhost/repository/vsx-hosted");
    let spec = fixture
        .call(
            Method::GET,
            "/service/rest/v1/formats/openvsx/upload-specs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(spec["assetFields"][0]["name"], "asset");
    let package = package("uploaded", "0.3.0", Some("web"));
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=vsx-hosted",
            Auth::Admin,
            Body::from(form("openvsx.asset", &package)),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());
    let uploaded = get(
        &fixture,
        "/repository/vsx-hosted/api/acme/uploaded",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(uploaded["targetPlatform"], "web");
    let broken = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=vsx-hosted",
            Auth::Admin,
            Body::from(form("openvsx.asset", b"plain text")),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);

    let mut changed = document.clone();
    changed["storage"]["writePolicy"] = json!("DENY");
    let update = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/openvsx/hosted/vsx-hosted",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT, "{}", update.text());
    let read_only = publish(&fixture, "vsx-hosted", Auth::Admin, &vsix(&Spec::default())).await;
    assert_eq!(read_only.status, StatusCode::BAD_REQUEST);
    assert!(read_only.text().contains("read-only"));

    let proxy = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/openvsx/proxy",
            Auth::Admin,
            Some(json!({
                "name": "no-remote",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "negativeCache": {"enabled": true, "timeToLive": 1440},
                "httpClient": {"blocked": false, "autoBlock": true},
            })),
        )
        .await;
    assert_eq!(proxy.status, StatusCode::BAD_REQUEST);
    let raw = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(hosted("files", "ALLOW")),
        )
        .await;
    assert_eq!(raw.status, StatusCode::CREATED);
    let mixed = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/openvsx/group",
            Auth::Admin,
            Some(json!({
                "name": "mixed",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["vsx-hosted", "files"]},
            })),
        )
        .await;
    assert_eq!(mixed.status, StatusCode::BAD_REQUEST, "{}", mixed.text());

    for path in ["api/acme/verify-pat", "api/-/publish", "api/a/b/delete"] {
        let put = call(
            &fixture,
            Method::PUT,
            &format!("/repository/files/{path}"),
            Auth::Admin,
            b"raw content".to_vec(),
            &[("content-type", "text/plain")],
        )
        .await;
        assert!(put.status.is_success(), "{path}: {}", put.text());
        let read = get(&fixture, &format!("/repository/files/{path}"), Auth::Admin).await;
        assert_eq!(read.status, StatusCode::OK, "{path}");
        assert_eq!(read.body, b"raw content", "{path}");
    }
    let other = get(
        &fixture,
        "/repository/vsx-hosted/api/acme/uploaded/web/0.3.0",
        Auth::Admin,
    )
    .await;
    assert_eq!(other.status, StatusCode::OK, "{}", other.text());
}
