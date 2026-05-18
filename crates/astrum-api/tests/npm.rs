mod support;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Fixture, Reply};
use tower::ServiceExt;

const HOSTED: &str = "/repository/npm-hosted";
const ABBREVIATED: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

#[derive(Clone, Copy)]
enum Who<'a> {
    Nobody,
    Admin,
    Basic(&'a str, &'a str),
    Bearer(&'a str),
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_raw::Raw));
    fixture
}

async fn send(
    fixture: &Fixture,
    method: Method,
    path: &str,
    who: Who<'_>,
    body: Option<Value>,
    headers: &[(&str, &str)],
) -> Reply {
    let mut request = axum::http::Request::builder().method(method).uri(path);
    let authorization = match who {
        Who::Nobody => None,
        Who::Admin => Some(format!(
            "Basic {}",
            STANDARD.encode(format!("admin:{}", fixture.admin_password))
        )),
        Who::Basic(user, password) => Some(format!(
            "Basic {}",
            STANDARD.encode(format!("{user}:{password}"))
        )),
        Who::Bearer(token) => Some(format!("Bearer {token}")),
    };
    if let Some(value) = authorization {
        request = request.header(header::AUTHORIZATION, value);
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let body = match body {
        Some(value) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).unwrap())
        }
        None => Body::empty(),
    };
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_npm::routes());
    let response = app.oneshot(request.body(body).unwrap()).await.unwrap();
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

async fn get(fixture: &Fixture, path: &str) -> Reply {
    send(fixture, Method::GET, path, Who::Admin, None, &[]).await
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    send(fixture, method, path, Who::Admin, body, &[]).await
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = admin(
        fixture,
        Method::POST,
        &format!("/service/rest/v1/repositories/npm/{kind}"),
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

fn tarball(name: &str, version: &str) -> Vec<u8> {
    let mut data = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03".to_vec();
    data.extend_from_slice(format!("package {name} {version}").as_bytes());
    data
}

fn publication(name: &str, version: &str, tag: &str) -> Value {
    let data = tarball(name, version);
    let bare = name.rsplit('/').next().unwrap();
    json!({
        "_id": name,
        "name": name,
        "description": format!("{bare} does things"),
        "dist-tags": {tag: version},
        "versions": {
            version: {
                "name": name,
                "version": version,
                "description": format!("{bare} does things"),
                "keywords": ["tooling", "astrum"],
                "readme": "# readme",
                "scripts": {"postinstall": "node setup.js"},
                "dependencies": {"left-pad": "^1.3.0"},
                "dist": {"tarball": format!("http://client.example/{name}/-/{name}-{version}.tgz")}
            }
        },
        "access": null,
        "_attachments": {
            format!("{name}-{version}.tgz"): {
                "content_type": "application/octet-stream",
                "data": STANDARD.encode(&data),
                "length": data.len()
            }
        }
    })
}

fn escaped(name: &str) -> String {
    name.replace('/', "%2f")
}

async fn publish(fixture: &Fixture, repository: &str, name: &str, version: &str, tag: &str) {
    let reply = admin(
        fixture,
        Method::PUT,
        &format!("{repository}/{}", escaped(name)),
        Some(publication(name, version, tag)),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json(), json!({"success": true}));
}

#[tokio::test]
async fn packages_are_published_and_read_back() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    publish(&fixture, HOSTED, "@team/tool", "1.0.0", "latest").await;
    publish(&fixture, HOSTED, "plain", "0.1.0", "latest").await;

    let full = get(&fixture, &format!("{HOSTED}/@team%2ftool")).await;
    assert_eq!(full.status, StatusCode::OK, "{}", full.text());
    assert_eq!(full.headers[header::CONTENT_TYPE], "application/json");
    let doc = full.json();
    assert_eq!(doc["name"], "@team/tool");
    assert_eq!(doc["_id"], "@team/tool");
    assert_eq!(doc["_rev"], "1");
    assert_eq!(doc["dist-tags"], json!({"latest": "1.0.0"}));
    assert!(doc.get("_attachments").is_none());
    let manifest = &doc["versions"]["1.0.0"];
    assert_eq!(
        manifest["dist"]["tarball"],
        "http://localhost/repository/npm-hosted/@team/tool/-/tool-1.0.0.tgz"
    );
    assert_eq!(manifest["dist"]["shasum"].as_str().unwrap().len(), 40);
    assert!(
        manifest["dist"]["integrity"]
            .as_str()
            .unwrap()
            .starts_with("sha512-")
    );
    assert_eq!(manifest["readme"], "# readme");
    assert!(doc["time"]["created"].is_string());
    assert_eq!(doc["time"]["1.0.0"], doc["time"]["created"]);
    assert_eq!(
        get(&fixture, &format!("{HOSTED}/@team/tool")).await.json(),
        doc
    );

    let short = send(
        &fixture,
        Method::GET,
        &format!("{HOSTED}/@team%2ftool"),
        Who::Admin,
        None,
        &[("accept", ABBREVIATED)],
    )
    .await;
    assert_eq!(
        short.headers[header::CONTENT_TYPE],
        "application/vnd.npm.install-v1+json"
    );
    assert_eq!(short.headers[header::VARY], "Accept");
    let short_doc = short.json();
    assert_eq!(short_doc["modified"], doc["time"]["modified"]);
    let short_manifest = &short_doc["versions"]["1.0.0"];
    assert_eq!(short_manifest["dependencies"]["left-pad"], "^1.3.0");
    assert_eq!(short_manifest["hasInstallScript"], true);
    assert_eq!(short_manifest["dist"], manifest["dist"]);
    assert!(short_manifest.get("readme").is_none());
    assert!(short_manifest.get("description").is_none());

    let etag = full.headers[header::ETAG].to_str().unwrap().to_owned();
    let cached = send(
        &fixture,
        Method::GET,
        &format!("{HOSTED}/@team%2ftool"),
        Who::Admin,
        None,
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(cached.status, StatusCode::NOT_MODIFIED);

    let version = get(&fixture, &format!("{HOSTED}/@team/tool/1.0.0")).await;
    assert_eq!(version.json(), *manifest);
    let latest = get(&fixture, &format!("{HOSTED}/@team%2ftool/latest")).await;
    assert_eq!(latest.json()["version"], "1.0.0");
    let missing_version = get(&fixture, &format!("{HOSTED}/@team/tool/9.9.9")).await;
    assert_eq!(missing_version.status, StatusCode::NOT_FOUND);
    assert_eq!(missing_version.json()["success"], false);

    let file = get(&fixture, &format!("{HOSTED}/@team/tool/-/tool-1.0.0.tgz")).await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, tarball("@team/tool", "1.0.0"));
    assert_eq!(file.headers[header::CONTENT_TYPE], "application/x-tgz");
    assert_eq!(
        file.headers[header::ETAG].to_str().unwrap(),
        format!("\"{}\"", manifest["dist"]["shasum"].as_str().unwrap())
    );
    let missing = get(&fixture, &format!("{HOSTED}/nothing-here")).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        missing.json(),
        json!({"success": false, "error": "package nothing-here not found"})
    );
    assert_eq!(
        get(&fixture, &format!("{HOSTED}/@team/tool/-/tool-9.9.9.tgz"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=npm-hosted",
    )
    .await
    .json();
    let items = components["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let tool = items.iter().find(|item| item["name"] == "tool").unwrap();
    assert_eq!(tool["group"], "team");
    assert_eq!(tool["version"], "1.0.0");
    assert_eq!(tool["format"], "npm");
    let asset = &tool["assets"][0];
    assert_eq!(asset["path"], "/@team/tool/-/tool-1.0.0.tgz");
    assert_eq!(asset["contentType"], "application/x-tgz");
    assert_eq!(
        asset["npm"],
        json!({"name": "@team/tool", "version": "1.0.0"})
    );
    let plain = items.iter().find(|item| item["name"] == "plain").unwrap();
    assert_eq!(plain["group"], "");

    let scoped = get(&fixture, "/service/rest/v1/search?npm.scope=team")
        .await
        .json();
    assert_eq!(scoped["items"].as_array().unwrap().len(), 1);
    assert_eq!(scoped["items"][0]["name"], "tool");

    let packuments = get(
        &fixture,
        "/service/rest/v1/search/assets?repository=npm-hosted&q=tool",
    )
    .await
    .json();
    assert!(
        packuments["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["path"] == "/@team/tool/-/tool-1.0.0.tgz")
    );

    publish(&fixture, HOSTED, "@team/tool", "1.1.0", "latest").await;
    let doc = get(&fixture, &format!("{HOSTED}/@team/tool")).await.json();
    assert_eq!(doc["_rev"], "2");
    assert_eq!(doc["dist-tags"]["latest"], "1.1.0");
    assert_eq!(doc["versions"].as_object().unwrap().len(), 2);

    let search = get(
        &fixture,
        &format!("{HOSTED}/-/v1/search?text=team%20tool&size=5"),
    )
    .await;
    assert_eq!(search.status, StatusCode::OK);
    let found = search.json();
    assert_eq!(found["total"], 1);
    assert_eq!(found["objects"][0]["package"]["name"], "@team/tool");
    assert_eq!(found["objects"][0]["package"]["version"], "1.1.0");
    assert_eq!(found["objects"][0]["package"]["scope"], "team");
    assert_eq!(
        found["objects"][0]["package"]["keywords"],
        json!(["tooling", "astrum"])
    );
    let by_keyword = get(&fixture, &format!("{HOSTED}/-/v1/search?text=astrum")).await;
    assert_eq!(by_keyword.json()["total"], 2);
    let nothing = get(&fixture, &format!("{HOSTED}/-/v1/search?text=zzz")).await;
    assert_eq!(nothing.json()["total"], 0);

    assert_eq!(
        get(&fixture, &format!("{HOSTED}/-/ping")).await.json(),
        json!({})
    );
}

#[tokio::test]
async fn bad_publications_are_refused() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    let mut wrong_sum = publication("tool", "1.0.0", "latest");
    wrong_sum["versions"]["1.0.0"]["dist"]["shasum"] = json!("0".repeat(40));
    let reply = admin(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Some(wrong_sum),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.json()["error"].as_str().unwrap().contains("shasum"));

    let other = admin(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Some(publication("other", "1.0.0", "latest")),
    )
    .await;
    assert_eq!(other.status, StatusCode::BAD_REQUEST);

    let not_json = send(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Who::Admin,
        None,
        &[],
    )
    .await;
    assert_eq!(not_json.status, StatusCode::BAD_REQUEST);

    let mut not_semver = publication("tool", "1.0.0", "latest");
    not_semver["versions"]["1.0"] = not_semver["versions"]["1.0.0"].take();
    not_semver["versions"]["1.0"]["version"] = json!("1.0");
    not_semver["versions"]
        .as_object_mut()
        .unwrap()
        .remove("1.0.0");
    let reply = admin(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Some(not_semver),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    let no_tarball = admin(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Some(json!({"name": "tool", "versions": {}})),
    )
    .await;
    assert_eq!(no_tarball.status, StatusCode::NOT_FOUND);

    assert_eq!(
        get(&fixture, &format!("{HOSTED}/tool")).await.status,
        StatusCode::NOT_FOUND
    );
    let assets = get(&fixture, "/service/rest/v1/assets?repository=npm-hosted")
        .await
        .json();
    assert_eq!(assets["items"], json!([]));
}

#[tokio::test]
async fn write_policies_permissions_and_repository_kinds_are_enforced() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    create(&fixture, "hosted", hosted("npm-once", "ALLOW_ONCE")).await;
    create(&fixture, "hosted", hosted("npm-frozen", "DENY")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "npm-all", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["npm-hosted"]}
        }),
    )
    .await;

    publish(&fixture, HOSTED, "tool", "1.0.0", "latest").await;
    publish(&fixture, HOSTED, "tool", "1.0.0", "latest").await;

    publish(&fixture, "/repository/npm-once", "tool", "1.0.0", "latest").await;
    let again = admin(
        &fixture,
        Method::PUT,
        "/repository/npm-once/tool",
        Some(publication("tool", "1.0.0", "latest")),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    let mut deprecated = get(&fixture, "/repository/npm-once/tool?write=true")
        .await
        .json();
    deprecated["versions"]["1.0.0"]["deprecated"] = json!("old");
    let reply = admin(
        &fixture,
        Method::PUT,
        "/repository/npm-once/tool",
        Some(deprecated),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());

    let frozen = admin(
        &fixture,
        Method::PUT,
        "/repository/npm-frozen/tool",
        Some(publication("tool", "1.0.0", "latest")),
    )
    .await;
    assert_eq!(frozen.status, StatusCode::BAD_REQUEST);

    let to_group = admin(
        &fixture,
        Method::PUT,
        "/repository/npm-all/tool",
        Some(publication("tool", "2.0.0", "latest")),
    )
    .await;
    assert_eq!(to_group.status, StatusCode::BAD_REQUEST);
    assert!(to_group.json()["error"].as_str().unwrap().contains("group"));

    let anonymous = send(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Who::Nobody,
        Some(publication("tool", "2.0.0", "latest")),
        &[],
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/tool"),
            Who::Nobody,
            None,
            &[]
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );

    let role = admin(
        &fixture,
        Method::POST,
        "/service/rest/v1/security/roles",
        Some(json!({"id": "npm-readers", "name": "npm-readers", "description": "", "privileges": ["as-repository-view-npm-npm-hosted-read", "as-repository-view-npm-npm-all-read"], "roles": []})),
    )
    .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("reader", "reader-pass", &["npm-readers"])
        .await;
    let reader = Who::Basic("reader", "reader-pass");
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/tool"),
            reader,
            None,
            &[]
        )
        .await
        .status,
        StatusCode::OK
    );
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            "/repository/npm-all/tool/-/tool-1.0.0.tgz",
            reader,
            None,
            &[]
        )
        .await
        .status,
        StatusCode::OK
    );
    let refused = send(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        reader,
        Some(publication("tool", "2.0.0", "latest")),
        &[],
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(
        send(
            &fixture,
            Method::DELETE,
            &format!("{HOSTED}/tool/-rev/1"),
            reader,
            None,
            &[]
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/-/whoami"),
            reader,
            None,
            &[]
        )
        .await
        .json(),
        json!({"username": "reader"})
    );
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/-/whoami"),
            Who::Nobody,
            None,
            &[]
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn tags_deprecations_and_unpublishing_follow_the_npm_client() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    publish(&fixture, HOSTED, "@team/tool", "1.0.0", "latest").await;
    publish(&fixture, HOSTED, "@team/tool", "1.1.0", "latest").await;
    publish(&fixture, HOSTED, "@team/tool", "2.0.0-beta.1", "beta").await;
    let tags = format!("{HOSTED}/-/package/@team%2ftool/dist-tags");

    assert_eq!(
        get(&fixture, &tags).await.json(),
        json!({"latest": "1.1.0", "beta": "2.0.0-beta.1"})
    );
    let set = admin(
        &fixture,
        Method::PUT,
        &format!("{tags}/stable"),
        Some(json!("1.0.0")),
    )
    .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.text());
    assert_eq!(get(&fixture, &tags).await.json()["stable"], "1.0.0");
    assert_eq!(
        admin(
            &fixture,
            Method::PUT,
            &format!("{tags}/stable"),
            Some(json!("9.9.9"))
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        admin(
            &fixture,
            Method::PUT,
            &format!("{tags}/1.0.0"),
            Some(json!("1.0.0"))
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        admin(&fixture, Method::DELETE, &format!("{tags}/stable"), None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        admin(&fixture, Method::DELETE, &format!("{tags}/latest"), None)
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        admin(&fixture, Method::DELETE, &format!("{tags}/missing"), None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let merged = admin(
        &fixture,
        Method::POST,
        &tags,
        Some(json!({"next": "2.0.0-beta.1"})),
    )
    .await;
    assert_eq!(merged.status, StatusCode::OK);
    assert_eq!(
        get(&fixture, &tags).await.json(),
        json!({"latest": "1.1.0", "beta": "2.0.0-beta.1", "next": "2.0.0-beta.1"})
    );

    let packument = format!("{HOSTED}/@team%2ftool");
    let mut doc = get(&fixture, &format!("{packument}?write=true"))
        .await
        .json();
    doc["versions"]["1.0.0"]["deprecated"] = json!("use 1.1");
    let reply = admin(&fixture, Method::PUT, &packument, Some(doc)).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let short = send(
        &fixture,
        Method::GET,
        &packument,
        Who::Admin,
        None,
        &[("accept", ABBREVIATED)],
    )
    .await
    .json();
    assert_eq!(short["versions"]["1.0.0"]["deprecated"], "use 1.1");
    assert!(short["versions"]["1.1.0"].get("deprecated").is_none());

    let mut doc = get(&fixture, &format!("{packument}?write=true"))
        .await
        .json();
    let rev = doc["_rev"].as_str().unwrap().to_owned();
    let tarball_url = doc["versions"]["1.0.0"]["dist"]["tarball"]
        .as_str()
        .unwrap()
        .to_owned();
    doc["versions"].as_object_mut().unwrap().remove("1.0.0");
    doc.as_object_mut().unwrap().remove("_attachments");
    let reply = admin(
        &fixture,
        Method::PUT,
        &format!("{packument}/-rev/{rev}"),
        Some(doc),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let after = get(&fixture, &packument).await.json();
    assert!(after["versions"].get("1.0.0").is_none());
    assert!(after["time"].get("1.0.0").is_none());
    let rev = after["_rev"].as_str().unwrap();
    let doubled = tarball_url.trim_start_matches("http://localhost");
    let delete = admin(
        &fixture,
        Method::DELETE,
        &format!("{HOSTED}{doubled}/-rev/{rev}"),
        None,
    )
    .await;
    assert_eq!(delete.status, StatusCode::NOT_FOUND, "{}", delete.text());
    assert_eq!(
        get(&fixture, &format!("{HOSTED}/@team/tool/-/tool-1.0.0.tgz"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let delete = admin(
        &fixture,
        Method::DELETE,
        &format!("{HOSTED}/@team/tool/-/tool-2.0.0-beta.1.tgz/-rev/9"),
        None,
    )
    .await;
    assert_eq!(delete.status, StatusCode::OK, "{}", delete.text());
    let after = get(&fixture, &packument).await.json();
    assert_eq!(
        after["versions"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["1.1.0"]
    );
    assert_eq!(after["dist-tags"], json!({"latest": "1.1.0"}));

    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=npm-hosted",
    )
    .await
    .json();
    assert_eq!(components["items"].as_array().unwrap().len(), 1);
    let id = components["items"][0]["id"].as_str().unwrap().to_owned();
    publish(&fixture, HOSTED, "@team/tool", "3.0.0", "latest").await;
    assert_eq!(
        admin(
            &fixture,
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    let after = get(&fixture, &packument).await.json();
    assert_eq!(
        after["versions"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["3.0.0"],
        "a version whose tarball was deleted elsewhere leaves the document"
    );

    let rev = after["_rev"].as_str().unwrap();
    let gone = admin(
        &fixture,
        Method::DELETE,
        &format!("{packument}/-rev/{rev}"),
        None,
    )
    .await;
    assert_eq!(gone.status, StatusCode::OK, "{}", gone.text());
    assert_eq!(
        get(&fixture, &packument).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/service/rest/v1/assets?repository=npm-hosted")
            .await
            .json()["items"],
        json!([])
    );
    assert_eq!(
        admin(
            &fixture,
            Method::DELETE,
            &format!("{packument}/-rev/{rev}"),
            None
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn npm_tokens_are_issued_used_and_revoked() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    let login = format!("{HOSTED}/-/user/org.couchdb.user:admin");
    let body = |password: &str| json!({"_id": "org.couchdb.user:admin", "name": "admin", "password": password, "type": "user", "roles": [], "date": "2026-10-12T00:00:00.000Z"});
    let password = fixture.admin_password.clone();

    let off = send(
        &fixture,
        Method::PUT,
        &login,
        Who::Nobody,
        Some(body(&password)),
        &[],
    )
    .await;
    assert_eq!(off.status, StatusCode::FORBIDDEN, "{}", off.text());
    assert!(off.json()["error"].as_str().unwrap().contains("realm"));

    let realms = admin(
        &fixture,
        Method::PUT,
        "/service/rest/v1/security/realms/active",
        Some(json!([
            "AstrumAuthenticatingRealm",
            "AccessTokenRealm",
            "NpmToken"
        ])),
    )
    .await;
    assert_eq!(realms.status, StatusCode::NO_CONTENT, "{}", realms.text());

    let wrong = send(
        &fixture,
        Method::PUT,
        &login,
        Who::Nobody,
        Some(body("nope")),
        &[],
    )
    .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.json()["success"], false);

    let signed_in = send(
        &fixture,
        Method::PUT,
        &login,
        Who::Nobody,
        Some(body(&password)),
        &[],
    )
    .await;
    assert_eq!(
        signed_in.status,
        StatusCode::CREATED,
        "{}",
        signed_in.text()
    );
    let answer = signed_in.json();
    assert_eq!(answer["ok"], true);
    assert_eq!(answer["id"], "org.couchdb.user:admin");
    let token = answer["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("NpmToken."), "{token}");

    let whoami = send(
        &fixture,
        Method::GET,
        &format!("{HOSTED}/-/whoami"),
        Who::Bearer(&token),
        None,
        &[],
    )
    .await;
    assert_eq!(whoami.json(), json!({"username": "admin"}));
    let published = send(
        &fixture,
        Method::PUT,
        &format!("{HOSTED}/tool"),
        Who::Bearer(&token),
        Some(publication("tool", "1.0.0", "latest")),
        &[],
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    let components = get(&fixture, "/service/rest/v1/assets?repository=npm-hosted")
        .await
        .json();
    assert!(
        components["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["uploader"] == "admin")
    );

    let audit = get(
        &fixture,
        "/service/rest/internal/audit?domain=security.npmtoken",
    )
    .await
    .json();
    assert!(audit.to_string().contains("created"), "{audit}");

    let logout = send(
        &fixture,
        Method::DELETE,
        &format!("{HOSTED}/-/user/token/{token}"),
        Who::Bearer(&token),
        None,
        &[],
    )
    .await;
    assert_eq!(logout.status, StatusCode::OK, "{}", logout.text());
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/-/whoami"),
            Who::Bearer(&token),
            None,
            &[]
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
    let again = send(
        &fixture,
        Method::PUT,
        &login,
        Who::Bearer(&token),
        Some(body(&password)),
        &[],
    )
    .await;
    assert_eq!(
        again.status,
        StatusCode::CREATED,
        "a revoked token left in .npmrc does not block signing in again: {}",
        again.text()
    );

    let created = admin(
        &fixture,
        Method::POST,
        "/service/rest/internal/access-tokens",
        Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let access = created.json()["token"].as_str().unwrap().to_owned();
    assert_eq!(
        send(
            &fixture,
            Method::GET,
            &format!("{HOSTED}/-/whoami"),
            Who::Bearer(&access),
            None,
            &[]
        )
        .await
        .json(),
        json!({"username": "admin"})
    );
    let exchanged = send(
        &fixture,
        Method::PUT,
        &login,
        Who::Nobody,
        Some(body(&access)),
        &[],
    )
    .await;
    assert_eq!(exchanged.status, StatusCode::UNAUTHORIZED);

    create(&fixture, "hosted", hosted("npm-other", "ALLOW")).await;
    let raw = admin(
        &fixture,
        Method::POST,
        "/service/rest/v1/repositories/raw/hosted",
        Some(json!({"name": "files", "online": true, "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"}})),
    )
    .await;
    assert_eq!(raw.status, StatusCode::CREATED, "{}", raw.text());
    let stored = send(
        &fixture,
        Method::PUT,
        "/repository/files/-/user/notes.txt",
        Who::Admin,
        Some(json!("not an npm sign-in")),
        &[],
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(
        get(&fixture, "/repository/files/-/user/notes.txt")
            .await
            .text(),
        "\"not an npm sign-in\""
    );
}

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<HashMap<String, (String, Vec<u8>)>>,
    hits: Arc<Mutex<Vec<String>>>,
    authorizations: Arc<Mutex<Vec<String>>>,
}

impl Upstream {
    fn hits(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|hit| hit.as_str() == path)
            .count()
    }
}

async fn upstream_file(State(upstream): State<Upstream>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    let decoded = path.replace("%2f", "/").replace("%2F", "/");
    upstream.hits.lock().unwrap().push(decoded.clone());
    if let Some(value) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        upstream
            .authorizations
            .lock()
            .unwrap()
            .push(value.to_owned());
    }
    let key = match request.uri().query() {
        Some(query) => format!("{decoded}?{query}"),
        None => decoded,
    };
    let found = upstream.files.get(&key).or_else(|| {
        upstream
            .files
            .get(key.split('?').next().unwrap_or_default())
    });
    match found {
        Some((content_type, body)) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
            (StatusCode::OK, headers, body.clone()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn remote_packument(address: SocketAddr) -> Value {
    let base = format!("http://{address}");
    json!({
        "_id": "@team/tool",
        "name": "@team/tool",
        "description": "the upstream tool",
        "dist-tags": {"latest": "2.0.0", "next": "3.0.0-rc.1"},
        "versions": {
            "1.0.0": {"name": "@team/tool", "version": "1.0.0", "description": "upstream 1", "dist": {"shasum": "a", "tarball": format!("{base}/@team/tool/-/tool-1.0.0.tgz")}},
            "2.0.0": {"name": "@team/tool", "version": "2.0.0", "description": "upstream 2", "dist": {"shasum": "b", "tarball": format!("{base}/@team/tool/-/tool-2.0.0.tgz")}},
            "3.0.0-rc.1": {"name": "@team/tool", "version": "3.0.0-rc.1", "dist": {"shasum": "c", "tarball": format!("{base}/@team/tool/-/tool-3.0.0-rc.1.tgz")}}
        },
        "time": {"created": "2020-01-01T00:00:00.000Z", "modified": "2030-01-01T00:00:00.000Z", "1.0.0": "2020-01-01T00:00:00.000Z", "2.0.0": "2021-01-01T00:00:00.000Z"}
    })
}

async fn start_upstream() -> (SocketAddr, Upstream) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut files = HashMap::new();
    files.insert(
        "/@team/tool".to_owned(),
        (
            "application/json".to_owned(),
            serde_json::to_vec(&remote_packument(address)).unwrap(),
        ),
    );
    for version in ["1.0.0", "2.0.0"] {
        files.insert(
            format!("/@team/tool/-/tool-{version}.tgz"),
            (
                "application/octet-stream".to_owned(),
                format!("upstream tarball {version}").into_bytes(),
            ),
        );
    }
    files.insert(
        "/-/v1/search".to_owned(),
        (
            "application/json".to_owned(),
            serde_json::to_vec(&json!({
                "objects": [
                    {"package": {"name": "@team/tool", "version": "2.0.0", "description": "the upstream tool"}, "score": {"final": 1.0}, "searchScore": 1.0},
                    {"package": {"name": "remote-only", "version": "1.0.0"}, "score": {"final": 0.5}, "searchScore": 0.5}
                ],
                "total": 2,
                "time": "now"
            }))
            .unwrap(),
        ),
    );
    let upstream = Upstream {
        files: Arc::new(files),
        ..Default::default()
    };
    let app = Router::new()
        .fallback(upstream_file)
        .with_state(upstream.clone());
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

fn proxy(name: &str, remote: &str, metadata_max_age: i64) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": false, "authentication": {"type": "bearerToken", "bearerToken": "upstream-secret"}},
        "npm": {"removeQuarantined": false, "removeNonCataloged": false}
    })
}

#[tokio::test]
async fn proxies_rewrite_and_cache_remote_packages() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (address, upstream) = start_upstream().await;
    create(
        &fixture,
        "proxy",
        proxy("npm-proxy", &format!("http://{address}/"), 1440),
    )
    .await;
    create(
        &fixture,
        "proxy",
        proxy("npm-live", &format!("http://{address}"), 0),
    )
    .await;

    let first = get(&fixture, "/repository/npm-proxy/@team%2ftool").await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    let doc = first.json();
    assert_eq!(
        doc["versions"]["2.0.0"]["dist"]["tarball"],
        "http://localhost/repository/npm-proxy/@team/tool/-/tool-2.0.0.tgz"
    );
    assert_eq!(doc["dist-tags"]["latest"], "2.0.0");
    let short = send(
        &fixture,
        Method::GET,
        "/repository/npm-proxy/@team/tool",
        Who::Admin,
        None,
        &[("accept", ABBREVIATED)],
    )
    .await
    .json();
    assert_eq!(short["modified"], "2030-01-01T00:00:00.000Z");
    assert!(short["versions"]["1.0.0"].get("description").is_none());
    assert_eq!(upstream.hits("/@team/tool"), 1, "metadata is cached");
    assert_eq!(
        upstream.authorizations.lock().unwrap()[0],
        "Bearer upstream-secret"
    );

    let file = get(
        &fixture,
        "/repository/npm-proxy/@team/tool/-/tool-2.0.0.tgz",
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.text(), "upstream tarball 2.0.0");
    assert_eq!(file.headers[header::CONTENT_TYPE], "application/x-tgz");
    get(
        &fixture,
        "/repository/npm-proxy/@team/tool/-/tool-2.0.0.tgz",
    )
    .await;
    assert_eq!(upstream.hits("/@team/tool/-/tool-2.0.0.tgz"), 1);
    let components = get(&fixture, "/service/rest/v1/components?repository=npm-proxy")
        .await
        .json();
    assert_eq!(components["items"][0]["group"], "team");
    assert_eq!(components["items"][0]["name"], "tool");
    assert_eq!(components["items"][0]["version"], "2.0.0");

    let version = get(&fixture, "/repository/npm-proxy/@team/tool/next").await;
    assert_eq!(version.json()["version"], "3.0.0-rc.1");
    assert_eq!(
        get(&fixture, "/repository/npm-proxy/missing").await.status,
        StatusCode::NOT_FOUND
    );

    get(&fixture, "/repository/npm-live/@team/tool").await;
    get(&fixture, "/repository/npm-live/@team/tool").await;
    assert_eq!(
        upstream.hits("/@team/tool"),
        3,
        "a metadata age of 0 asks the remote every time"
    );

    let search = get(&fixture, "/repository/npm-proxy/-/v1/search?text=tool").await;
    assert_eq!(search.status, StatusCode::OK, "{}", search.text());
    assert_eq!(search.json()["total"], 2);

    let publish = admin(
        &fixture,
        Method::PUT,
        "/repository/npm-proxy/tool",
        Some(publication("tool", "1.0.0", "latest")),
    )
    .await;
    assert_eq!(publish.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn groups_merge_their_members() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (address, upstream) = start_upstream().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy("npm-proxy", &format!("http://{address}"), 1440),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "npm-all", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["npm-hosted", "npm-proxy"]}
        }),
    )
    .await;
    publish(&fixture, HOSTED, "@team/tool", "1.0.0", "latest").await;
    publish(&fixture, HOSTED, "local-only", "0.1.0", "latest").await;

    let doc = get(&fixture, "/repository/npm-all/@team%2ftool")
        .await
        .json();
    let versions: Vec<&String> = doc["versions"].as_object().unwrap().keys().collect();
    assert_eq!(versions.len(), 3, "{versions:?}");
    assert_eq!(doc["versions"]["1.0.0"]["description"], "tool does things");
    assert_eq!(doc["versions"]["2.0.0"]["description"], "upstream 2");
    assert_eq!(
        doc["dist-tags"],
        json!({"latest": "1.0.0", "next": "3.0.0-rc.1"})
    );
    assert_eq!(doc["time"]["created"], "2020-01-01T00:00:00.000Z");
    assert_eq!(doc["time"]["modified"], "2030-01-01T00:00:00.000Z");
    for version in ["1.0.0", "2.0.0"] {
        assert_eq!(
            doc["versions"][version]["dist"]["tarball"],
            format!("http://localhost/repository/npm-all/@team/tool/-/tool-{version}.tgz")
        );
    }
    assert_eq!(
        get(&fixture, "/repository/npm-all/@team/tool/-/tool-1.0.0.tgz")
            .await
            .body,
        tarball("@team/tool", "1.0.0")
    );
    assert_eq!(
        get(&fixture, "/repository/npm-all/@team/tool/-/tool-2.0.0.tgz")
            .await
            .text(),
        "upstream tarball 2.0.0"
    );
    assert_eq!(upstream.hits("/@team/tool/-/tool-1.0.0.tgz"), 0);
    assert_eq!(
        get(
            &fixture,
            "/repository/npm-all/-/package/@team%2ftool/dist-tags"
        )
        .await
        .json(),
        json!({"latest": "1.0.0", "next": "3.0.0-rc.1"})
    );
    assert_eq!(
        get(&fixture, "/repository/npm-all/local-only").await.json()["name"],
        "local-only"
    );

    let search = get(&fixture, "/repository/npm-all/-/v1/search?text=tool")
        .await
        .json();
    let names: Vec<&str> = search["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|object| object["package"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["@team/tool", "local-only", "remote-only"]);
    assert_eq!(
        search["objects"][0]["package"]["description"],
        "tool does things"
    );
}

fn package_tgz(manifest: &str) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (name, data) in [
        ("package/package.json", manifest.as_bytes()),
        ("package/index.js", b"module.exports = 42;\n".as_slice()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, name, data).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

async fn upload_file(fixture: &Fixture, repository: &str, filename: &str, data: &[u8]) -> Reply {
    let boundary = "NPMUPLOADBOUNDARY";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"npm.asset\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let content_type = format!("multipart/form-data; boundary={boundary}");
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            support::Auth::Admin,
            Body::from(body),
            &[("content-type", content_type.as_str())],
        )
        .await
}

#[tokio::test]
async fn package_tarballs_upload_through_the_components_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW_ONCE")).await;
    let spec = get(&fixture, "/service/rest/v1/formats/npm/upload-specs").await;
    assert_eq!(spec.status, StatusCode::OK);
    assert_eq!(
        spec.json(),
        json!({
            "format": "npm",
            "multipleUpload": false,
            "componentFields": [],
            "assetFields": [{"name": "asset", "type": "FILE", "description": null, "optional": false, "group": null, "groupHelpText": null, "options": null}]
        })
    );

    let tgz = package_tgz(r#"{"name":"@team/widgets","version":"1.2.3","description":"Widgets"}"#);
    let reply = upload_file(&fixture, "npm-hosted", "local-name.tgz", &tgz).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());

    let document = get(&fixture, &format!("{HOSTED}/@team%2fwidgets"))
        .await
        .json();
    assert_eq!(document["dist-tags"]["latest"], "1.2.3");
    assert_eq!(document["description"], "Widgets");
    let dist = &document["versions"]["1.2.3"]["dist"];
    assert_eq!(
        dist["shasum"],
        hex::encode(<sha1::Sha1 as sha1::Digest>::digest(&tgz))
    );
    let tarball = get(
        &fixture,
        &format!("{HOSTED}/@team/widgets/-/widgets-1.2.3.tgz"),
    )
    .await;
    assert_eq!(tarball.status, StatusCode::OK);
    assert_eq!(tarball.body, tgz);

    let found = get(
        &fixture,
        "/service/rest/v1/search?repository=npm-hosted&name=widgets",
    )
    .await
    .json();
    assert_eq!(found["items"][0]["group"], "team");
    assert_eq!(found["items"][0]["version"], "1.2.3");

    let again = upload_file(&fixture, "npm-hosted", "local-name.tgz", &tgz).await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST, "{}", again.text());
    assert!(
        again.text().contains("already published"),
        "{}",
        again.text()
    );

    let plain = upload_file(&fixture, "npm-hosted", "notes.txt", b"just text").await;
    assert_eq!(plain.status, StatusCode::BAD_REQUEST);
    assert!(plain.text().contains("npm.asset"), "{}", plain.text());

    let nameless = package_tgz(r#"{"version":"1.0.0"}"#);
    let reply = upload_file(&fixture, "npm-hosted", "x.tgz", &nameless).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(
        reply.text().contains("names no package"),
        "{}",
        reply.text()
    );
}

#[derive(Clone, Default)]
struct Auditor {
    seen: Arc<Mutex<Vec<AuditCall>>>,
}

#[derive(Clone, Debug)]
struct AuditCall {
    path: String,
    encoding: Option<String>,
    authorization: Option<String>,
    body: Vec<u8>,
}

async fn audit_answer(State(auditor): State<Auditor>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let header_of = |name: header::HeaderName| {
        parts
            .headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    };
    let call = AuditCall {
        path: parts.uri.path().to_owned(),
        encoding: header_of(header::CONTENT_ENCODING),
        authorization: header_of(header::AUTHORIZATION),
        body: Vec::new(),
    };
    let method = parts.method.clone();
    let body = body.collect().await.unwrap().to_bytes();
    auditor.seen.lock().unwrap().push(AuditCall {
        body: body.to_vec(),
        ..call
    });
    if method != Method::POST {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"lodash":[{"id":1523,"url":"https://github.com/advisories/GHSA-p6mc-m468-83gw","title":"Prototype Pollution in lodash","severity":"high","vulnerable_versions":"<4.17.19","cwe":["CWE-1321"],"cvss":{"score":7.4}}]}"#,
    )
        .into_response()
}

async fn post_raw(fixture: &Fixture, path: &str, who: Who<'_>, body: Vec<u8>) -> Reply {
    let mut request = axum::http::Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_ENCODING, "gzip");
    if let Who::Admin = who {
        request = request.header(
            header::AUTHORIZATION,
            format!(
                "Basic {}",
                STANDARD.encode(format!("admin:{}", fixture.admin_password))
            ),
        );
    }
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_npm::routes());
    let response = app
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

#[tokio::test]
async fn security_audits_go_to_the_remote_of_a_proxy() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let auditor = Auditor::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .fallback(audit_answer)
        .with_state(auditor.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(&fixture, "hosted", hosted("npm-hosted", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy("npm-proxy", &format!("http://{address}/"), 1440),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "npm-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["npm-hosted", "npm-proxy"]}
        }),
    )
    .await;
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(
        &mut gzip,
        br#"{"lodash":["4.17.0"],"@private/app":["1.0.0"]}"#,
    )
    .unwrap();
    let packed = gzip.finish().unwrap();

    for repository in ["npm-proxy", "npm-all"] {
        let reply = post_raw(
            &fixture,
            &format!("/repository/{repository}/-/npm/v1/security/advisories/bulk"),
            Who::Admin,
            packed.clone(),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{repository}: {}",
            reply.text()
        );
        assert_eq!(
            reply.headers[header::CONTENT_TYPE],
            "application/json",
            "{repository}"
        );
        assert_eq!(
            reply.json()["lodash"][0]["vulnerable_versions"],
            "<4.17.19",
            "{repository}"
        );
    }
    let seen = auditor.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    for call in &seen {
        assert_eq!(call.path, "/-/npm/v1/security/advisories/bulk");
        assert_eq!(call.encoding.as_deref(), Some("gzip"));
        assert_eq!(call.body, packed);
        assert_eq!(
            call.authorization.as_deref(),
            Some("Bearer upstream-secret"),
            "the remote's own credentials go with the audit, never the caller's"
        );
    }

    let quick = post_raw(
        &fixture,
        "/repository/npm-all/-/npm/v1/security/audits/quick",
        Who::Admin,
        packed.clone(),
    )
    .await;
    assert_eq!(quick.status, StatusCode::OK, "{}", quick.text());
    assert_eq!(
        auditor.seen.lock().unwrap().last().unwrap().path,
        "/-/npm/v1/security/audits/quick"
    );

    let hosted = post_raw(
        &fixture,
        "/repository/npm-hosted/-/npm/v1/security/advisories/bulk",
        Who::Admin,
        packed.clone(),
    )
    .await;
    assert_eq!(hosted.status, StatusCode::BAD_REQUEST, "{}", hosted.text());
    assert!(
        hosted
            .text()
            .contains("only through npm proxy repositories"),
        "{}",
        hosted.text()
    );
    let read = send(
        &fixture,
        Method::GET,
        "/repository/npm-proxy/-/npm/v1/security/advisories/bulk",
        Who::Admin,
        None,
        &[],
    )
    .await;
    assert_eq!(read.status, StatusCode::METHOD_NOT_ALLOWED);
    let stranger = post_raw(
        &fixture,
        "/repository/npm-proxy/-/npm/v1/security/advisories/bulk",
        Who::Nobody,
        packed,
    )
    .await;
    assert_eq!(stranger.status, StatusCode::UNAUTHORIZED);
    assert_eq!(auditor.seen.lock().unwrap().len(), 3);
}
