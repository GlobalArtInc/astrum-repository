mod support;

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use astrum_format_firefox::testing::{manifest, signed_xpi, xpi};
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "firefoxboundary0123456789";
const NAME: &str = "name@example.org";
const UUID: &str = "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}";
const UUID_PATH: &str = "%7Bec8030f7-c20a-464f-9b0e-13a3a9e97384%7D";
const UUID_FILE: &str = "ec8030f7-c20a-464f-9b0e-13a3a9e97384";
const UBLOCK: &str = "uBlock0@raymondhill.net";
const REMOTE_UUID: &str = "{11111111-2222-3333-4444-555555555555}";
const REMOTE_UUID_PATH: &str = "%7B11111111-2222-3333-4444-555555555555%7D";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_firefox::Firefox::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/firefox/{kind}"),
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

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
    })
}

fn proxy(name: &str, remote: &str, metadata_max_age: i32) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": false},
    })
}

async fn token(fixture: &Fixture, scope: Scope) -> String {
    format!(
        "Bearer {}",
        fixture
            .state
            .security()
            .create_token("admin", "firefox", scope, None)
            .await
            .unwrap()
            .1
    )
}

async fn put(fixture: &Fixture, path: &str, authorization: Option<&str>, data: &[u8]) -> Reply {
    let mut headers = vec![("content-type", "application/x-xpinstall")];
    if let Some(authorization) = authorization {
        headers.push(("authorization", authorization));
    }
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::None,
            Body::from(data.to_vec()),
            &headers,
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn header_of<'a>(reply: &'a Reply, name: &str) -> Option<&'a str> {
    reply
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn sha256(data: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(data)))
}

fn manifest_of(reply: &Reply) -> Value {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(header_of(reply, "content-type"), Some("application/json"));
    reply.json()
}

fn versions(document: &Value, id: &str) -> Vec<String> {
    document["addons"][id]["updates"]
        .as_array()
        .unwrap_or_else(|| panic!("{id} is not in {document}"))
        .iter()
        .map(|update| update["version"].as_str().unwrap().to_owned())
        .collect()
}

fn local(link: &str) -> String {
    let at = link
        .find("/repository/")
        .unwrap_or_else(|| panic!("{link} is not a repository address"));
    link[at..].to_owned()
}

fn addon(id: &str, version: &str) -> Vec<u8> {
    xpi(&manifest(id, version))
}

#[tokio::test]
async fn hosted_addons_are_stored_and_offered_to_firefox() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("addons", "ALLOW_ONCE")).await;
    let writer = token(&fixture, Scope::Write).await;

    let page = fixture
        .call(Method::GET, "/repository/addons/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("ExtensionSettings"));
    assert!(page.text().contains("updates.json"));

    let first = addon(NAME, "1.0.0");
    let stored = put(
        &fixture,
        "/repository/addons/build/my-addon.xpi",
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(stored.json()["id"], NAME);
    assert_eq!(stored.json()["version"], "1.0.0");
    assert_eq!(stored.json()["name"], "name@example.org name");
    assert_eq!(stored.json()["signed"], false);
    assert!(
        header_of(&stored, "location")
            .unwrap()
            .ends_with("/repository/addons/name@example.org/1.0.0/name-1.0.0.xpi"),
        "{:?}",
        stored.headers
    );

    let again = put(
        &fixture,
        "/repository/addons/again.xpi",
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let changed = xpi(&manifest(NAME, "1.0.0").replace("An add-on", "Another add-on"));
    let refused = put(
        &fixture,
        "/repository/addons/changed.xpi",
        Some(&writer),
        &changed,
    )
    .await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.text());
    assert!(refused.text().contains("redeploy is not allowed"));

    let misplaced = put(
        &fixture,
        "/repository/addons/name@example.org/1.0.1/name-1.0.1.xpi",
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(misplaced.status, StatusCode::BAD_REQUEST);
    assert!(
        misplaced.text().contains("belongs at"),
        "{}",
        misplaced.text()
    );
    let garbage = put(&fixture, "/repository/addons/x.xpi", Some(&writer), b"nope").await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert!(
        garbage.text().contains("not a Firefox add-on"),
        "{}",
        garbage.text()
    );
    let anonymous = xpi(r#"{"manifest_version": 3, "name": "x", "version": "1.0"}"#);
    let without_id = put(
        &fixture,
        "/repository/addons/y.xpi",
        Some(&writer),
        &anonymous,
    )
    .await;
    assert_eq!(without_id.status, StatusCode::BAD_REQUEST);
    assert!(
        without_id.text().contains("gecko.id"),
        "{}",
        without_id.text()
    );
    let unauthenticated = put(&fixture, "/repository/addons/z.xpi", None, &first).await;
    assert_eq!(unauthenticated.status, StatusCode::UNAUTHORIZED);

    let braced = signed_xpi(&manifest(UUID, "2.0"));
    let uuid_path = format!("/repository/addons/{UUID_PATH}/2.0/{UUID_FILE}-2.0.xpi");
    let signed = put(&fixture, &uuid_path, Some(&writer), &braced).await;
    assert_eq!(signed.status, StatusCode::CREATED, "{}", signed.text());
    assert_eq!(signed.json()["id"], UUID);
    assert_eq!(signed.json()["signed"], true);
    assert!(header_of(&signed, "location").unwrap().ends_with(&format!(
        "/repository/addons/{UUID_PATH}/2.0/{UUID_FILE}-2.0.xpi"
    )));
    let newer = addon(NAME, "1.10.0");
    let older = addon(NAME, "1.9.0");
    for data in [&newer, &older] {
        let reply = put(&fixture, "/repository/addons/next.xpi", Some(&writer), data).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    let all = manifest_of(&get(&fixture, "/repository/addons/updates.json").await);
    assert_eq!(versions(&all, NAME), ["1.0.0", "1.9.0", "1.10.0"]);
    assert_eq!(versions(&all, UUID), ["2.0"]);
    let latest = &all["addons"][NAME]["updates"][2];
    assert!(
        latest["update_link"]
            .as_str()
            .unwrap()
            .ends_with("/repository/addons/name@example.org/1.10.0/name-1.10.0.xpi"),
        "{latest}"
    );
    assert_eq!(latest["update_hash"], sha256(&newer));
    assert_eq!(
        latest["browser_specific_settings"]["gecko"]["strict_min_version"],
        "115.0"
    );
    assert!(
        latest["browser_specific_settings"]["gecko"]
            .get("strict_max_version")
            .is_none()
    );
    let uuid_link = all["addons"][UUID]["updates"][0]["update_link"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        uuid_link.ends_with(&format!("/{UUID_PATH}/2.0/{UUID_FILE}-2.0.xpi")),
        "{uuid_link}"
    );
    let through_link = get(&fixture, &local(&uuid_link)).await;
    assert_eq!(
        through_link.status,
        StatusCode::OK,
        "{}",
        through_link.text()
    );
    assert_eq!(through_link.body, braced);
    assert_eq!(
        header_of(&through_link, "content-type"),
        Some("application/x-xpinstall")
    );

    let narrowed = manifest_of(
        &get(
            &fixture,
            "/repository/addons/updates.json?id=name%40example.org",
        )
        .await,
    );
    assert_eq!(narrowed["addons"].as_object().unwrap().len(), 1);
    assert_eq!(versions(&narrowed, NAME).len(), 3);
    for path in [
        "/repository/addons/name@example.org/updates.json",
        "/repository/addons/name%40example.org/updates.json",
    ] {
        assert_eq!(manifest_of(&get(&fixture, path).await), narrowed);
    }
    let by_uuid = manifest_of(
        &get(
            &fixture,
            &format!("/repository/addons/{UUID_PATH}/updates.json"),
        )
        .await,
    );
    assert_eq!(versions(&by_uuid, UUID), ["2.0"]);
    let checked = manifest_of(
        &get(
            &fixture,
            &format!("/repository/addons/update/VersionCheck.php?reqVersion=2&id={UUID_PATH}&version=1.0&appID=%7Bec8030f7-c20a-464f-9b0e-13a3a9e97384%7D&appVersion=128.0"),
        )
        .await,
    );
    assert_eq!(checked, by_uuid);
    let unknown = manifest_of(
        &get(
            &fixture,
            "/repository/addons/update/VersionCheck.php?id=nobody%40example.org",
        )
        .await,
    );
    assert_eq!(unknown, json!({"addons": {}}));
    let missing_id = get(
        &fixture,
        "/repository/addons/update/VersionCheck.php?version=1",
    )
    .await;
    assert_eq!(missing_id.status, StatusCode::BAD_REQUEST);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/addons/updates.json",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
    assert!(
        header_of(&head, "content-length")
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 0
    );

    let newest = get(&fixture, "/repository/addons/name@example.org/latest.xpi").await;
    assert_eq!(newest.status, StatusCode::OK, "{}", newest.text());
    assert_eq!(newest.body, newer);
    assert_eq!(
        header_of(&newest, "content-type"),
        Some("application/x-xpinstall")
    );
    let braced_latest = get(
        &fixture,
        &format!("/repository/addons/{UUID_PATH}/latest.xpi"),
    )
    .await;
    assert_eq!(braced_latest.body, braced);
    let raw_braces = get(&fixture, &format!("/repository/addons/{UUID}/latest.xpi")).await;
    assert_eq!(raw_braces.body, braced);
    let exact = get(&fixture, "/repository/addons/name@example.org/1.0.0.xpi").await;
    assert_eq!(exact.body, first);
    assert_eq!(
        get(&fixture, "/repository/addons/nobody@example.org/latest.xpi")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/addons/ublock-origin/latest.xpi")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/addons/name@example.org/1.10.0/name-1.10.0.xpi",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let after = get(&fixture, "/repository/addons/name@example.org/latest.xpi").await;
    assert_eq!(after.body, older);
    let listed = manifest_of(&get(&fixture, "/repository/addons/updates.json").await);
    assert_eq!(versions(&listed, NAME), ["1.0.0", "1.9.0"]);
}

fn multipart(name: &str, file: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(data);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn upload_component(fixture: &Fixture, repository: &str, data: &[u8]) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(multipart("firefox.asset", "addon.xpi", data)),
            &[("content-type", content_type.as_str())],
        )
        .await
}

#[tokio::test]
async fn components_search_and_cleanup_cover_addons() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("addons", "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;

    let sent = upload_component(&fixture, "addons", &signed_xpi(&manifest(UUID, "3.0"))).await;
    assert_eq!(sent.status, StatusCode::NO_CONTENT, "{}", sent.text());
    let refused = upload_component(&fixture, "addons", b"not an add-on").await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    let localized = json!({
        "manifest_version": 2,
        "name": "__MSG_appName__",
        "version": "1.0b1",
        "default_locale": "de",
        "browser_specific_settings": {"gecko": {"id": NAME, "strict_max_version": "140.*"}}
    })
    .to_string();
    let messages = json!({"appName": {"message": "Werkzeugkasten"}}).to_string();
    let file = astrum_content::zip::write(&[
        ("manifest.json", localized.as_bytes()),
        ("_locales/de/messages.json", messages.as_bytes()),
    ]);
    let reply = put(
        &fixture,
        "/repository/addons/tool.xpi",
        Some(&writer),
        &file,
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(reply.json()["name"], "Werkzeugkasten");
    let replaced = put(
        &fixture,
        "/repository/addons/tool.xpi",
        Some(&writer),
        &xpi(&manifest(NAME, "1.0b1")),
    )
    .await;
    assert_eq!(replaced.status, StatusCode::CREATED, "{}", replaced.text());
    let reverted = put(
        &fixture,
        "/repository/addons/tool.xpi",
        Some(&writer),
        &file,
    )
    .await;
    assert_eq!(reverted.status, StatusCode::CREATED, "{}", reverted.text());

    let components = get(&fixture, "/service/rest/v1/components?repository=addons").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    let mut names: Vec<(String, String)> = items
        .iter()
        .map(|item| {
            (
                item["name"].as_str().unwrap().to_owned(),
                item["version"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            (NAME.to_owned(), "1.0b1".to_owned()),
            (UUID.to_owned(), "3.0".to_owned())
        ]
    );
    assert_eq!(items[0]["format"], "firefox");

    let count = |reply: Reply| reply.json()["items"].as_array().unwrap().len();
    assert_eq!(
        count(
            get(
                &fixture,
                "/service/rest/v1/search?repository=addons&firefox.signed=true"
            )
            .await
        ),
        1
    );
    assert_eq!(
        count(
            get(
                &fixture,
                "/service/rest/v1/search?repository=addons&firefox.minVersion=115.0"
            )
            .await
        ),
        1
    );
    let by_name = get(
        &fixture,
        "/service/rest/v1/search?repository=addons&firefox.name=Werkzeugkasten",
    )
    .await;
    assert_eq!(
        by_name.json()["items"][0]["name"],
        NAME,
        "{}",
        by_name.text()
    );
    let by_keyword = get(
        &fixture,
        "/service/rest/v1/search?repository=addons&q=werkzeug",
    )
    .await;
    assert_eq!(by_keyword.json()["items"][0]["name"], NAME);
    let manifest = manifest_of(&get(&fixture, "/repository/addons/updates.json").await);
    assert_eq!(
        manifest["addons"][NAME]["updates"][0]["browser_specific_settings"]["gecko"],
        json!({"strict_max_version": "140.*"})
    );

    let tree = get(&fixture, "/service/rest/internal/ui/browse/addons?path=/").await;
    assert_eq!(tree.status, StatusCode::OK, "{}", tree.text());
    let mut roots: Vec<String> = tree.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap().to_owned())
        .collect();
    roots.sort();
    assert_eq!(roots, [NAME, UUID]);

    let policy = fixture
        .call(
            Method::POST,
            "/service/rest/v1/cleanup-policies",
            Auth::Admin,
            Some(json!({"name": "addon-prereleases", "format": "firefox", "criteriaReleaseType": "PRERELEASES"})),
        )
        .await;
    assert!(policy.status.is_success(), "{}", policy.text());
    let id = items
        .iter()
        .find(|item| item["name"] == UUID)
        .and_then(|item| item["id"].as_str())
        .unwrap()
        .to_owned();
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let left = manifest_of(&get(&fixture, "/repository/addons/updates.json").await);
    assert!(left["addons"].get(UUID).is_none(), "{left}");
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/addons/{UUID_PATH}/latest.xpi")
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn content_selectors_decide_which_addons_a_reader_sees() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("addons", "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    for id in ["public@example.org", "secret@example.org"] {
        let reply = put(
            &fixture,
            "/repository/addons/a.xpi",
            Some(&writer),
            &addon(id, "1.0"),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "public-addons", "description": "", "expression": "format == \"firefox\" and path =^ \"/public@example.org/\""}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "public-addons-read", "description": "", "actions": ["READ"], "format": "firefox", "repository": "addons", "contentSelector": "public-addons"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "addon-readers", "name": "addon-readers", "description": "", "privileges": ["public-addons-read"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-pass", &["addon-readers"])
        .await;
    let call = |path: &'static str| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::GET,
                    path,
                    Auth::Basic("reader", "reader-pass"),
                    None,
                )
                .await
        }
    };
    let listed = manifest_of(&call("/repository/addons/updates.json").await);
    assert_eq!(
        listed["addons"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["public@example.org"]
    );
    let secret = manifest_of(
        &call("/repository/addons/update/VersionCheck.php?id=secret%40example.org").await,
    );
    assert_eq!(secret, json!({"addons": {}}));
    assert_eq!(
        call("/repository/addons/public@example.org/latest.xpi")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        call("/repository/addons/secret@example.org/latest.xpi")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call("/repository/addons/secret@example.org/1.0/secret-1.0.xpi")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call("/repository/addons/secret@example.org/1.0.xpi")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[derive(Clone, Default)]
struct Upstream {
    address: Arc<OnceLock<String>>,
    seen: Arc<Mutex<Vec<String>>>,
    broken: Arc<AtomicBool>,
    current: Arc<Vec<u8>>,
    previous: Arc<Vec<u8>>,
    braced: Arc<Vec<u8>>,
    plain: Arc<Vec<u8>>,
}

impl Upstream {
    fn count(&self, prefix: &str) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|seen| seen.starts_with(prefix))
            .count()
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.address.get().unwrap())
    }

    fn version(&self, version: &str, file: &str, data: &[u8]) -> Value {
        json!({
            "id": 7,
            "version": version,
            "channel": "listed",
            "compatibility": {"firefox": {"min": "109.0", "max": "*"}, "android": {"min": "120.0", "max": "*"}},
            "file": {"id": 9, "url": self.url(file), "hash": sha256(data), "size": data.len(), "status": "public"},
        })
    }

    fn addon(&self, guid: &str, slug: &str, current: Value) -> Response {
        axum::Json(json!({
            "id": 607454,
            "guid": guid,
            "slug": slug,
            "name": {"en-US": format!("{slug} name")},
            "current_version": current,
        }))
        .into_response()
    }
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "application/json")],
        json!({"detail": "Not found."}).to_string(),
    )
        .into_response()
}

async fn amo(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    upstream.seen.lock().unwrap().push(path.clone());
    if upstream.broken.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let xpi = |data: &Arc<Vec<u8>>| {
        (
            [(header::CONTENT_TYPE, "application/x-xpinstall")],
            data.as_ref().clone(),
        )
            .into_response()
    };
    let current = upstream.version(
        "1.2.0",
        "downloads/file/12/ublock-1.2.0.xpi",
        &upstream.current,
    );
    match path.as_str() {
        "api/v5/addons/addon/ublock-origin/" | "api/v5/addons/addon/uBlock0@raymondhill.net/" => {
            upstream.addon(UBLOCK, "ublock-origin", current)
        }
        "api/v5/addons/addon/uBlock0@raymondhill.net/versions/1.1.0/" => {
            axum::Json(upstream.version(
                "1.1.0",
                "downloads/file/11/ublock-1.1.0.xpi",
                &upstream.previous,
            ))
            .into_response()
        }
        "api/v5/addons/addon/braced/"
        | "api/v5/addons/addon/{11111111-2222-3333-4444-555555555555}/" => upstream.addon(
            REMOTE_UUID,
            "braced",
            upstream.version("4.0", "downloads/file/40/braced.xpi", &upstream.braced),
        ),
        "api/v5/addons/addon/bad-hash/" => {
            let mut version = upstream.version("1.0", "downloads/file/50/bad.xpi", &upstream.plain);
            version["file"]["hash"] = json!(format!("sha256:{}", "0".repeat(64)));
            upstream.addon("bad@example.org", "bad-hash", version)
        }
        "api/v5/addons/addon/not-an-addon/" => upstream.addon(
            "plain@example.org",
            "not-an-addon",
            upstream.version("1.0", "downloads/file/50/bad.xpi", &upstream.plain),
        ),
        "api/v5/addons/addon/unlisted/" => {
            upstream.addon("unlisted@example.org", "unlisted", Value::Null)
        }
        "downloads/file/12/ublock-1.2.0.xpi" => (
            StatusCode::FOUND,
            [(
                header::LOCATION,
                "/cdn/user-media/addons/12/ublock-1.2.0.xpi",
            )],
        )
            .into_response(),
        "cdn/user-media/addons/12/ublock-1.2.0.xpi" => xpi(&upstream.current),
        "downloads/file/11/ublock-1.1.0.xpi" => xpi(&upstream.previous),
        "downloads/file/40/braced.xpi" => xpi(&upstream.braced),
        "downloads/file/50/bad.xpi" => xpi(&upstream.plain),
        _ => not_found(),
    }
}

async fn serve_amo(fixture: &Fixture) -> (Upstream, String) {
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
    let state = Upstream {
        current: Arc::new(signed_xpi(&manifest(UBLOCK, "1.2.0"))),
        previous: Arc::new(signed_xpi(&manifest(UBLOCK, "1.1.0"))),
        braced: Arc::new(signed_xpi(&manifest(REMOTE_UUID, "4.0"))),
        plain: Arc::new(astrum_content::zip::write(&[("readme.txt", b"hello")])),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(amo))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    state.address.set(address.clone()).unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (state, address)
}

#[tokio::test]
async fn proxies_cache_addons_from_the_addons_api() {
    let fixture = ready().await;
    let (upstream, remote) = serve_amo(&fixture).await;
    create(&fixture, "proxy", proxy("amo", &remote, 1440)).await;
    create(&fixture, "proxy", proxy("live", &remote, 0)).await;

    let first = get(&fixture, "/repository/amo/ublock-origin/latest.xpi").await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert_eq!(first.body, *upstream.current);
    assert_eq!(
        header_of(&first, "content-type"),
        Some("application/x-xpinstall")
    );
    assert_eq!(upstream.count("api/v5/addons/addon/ublock-origin/"), 1);
    assert_eq!(upstream.count("cdn/"), 1);
    let again = get(&fixture, "/repository/amo/ublock-origin/latest.xpi").await;
    assert_eq!(again.body, *upstream.current);
    assert_eq!(upstream.count("api/"), 1);
    assert_eq!(upstream.count("cdn/"), 1);

    let by_guid = get(
        &fixture,
        "/repository/amo/uBlock0@raymondhill.net/latest.xpi",
    )
    .await;
    assert_eq!(by_guid.body, *upstream.current);
    assert_eq!(
        upstream.count("api/v5/addons/addon/uBlock0@raymondhill.net/"),
        1
    );
    assert_eq!(upstream.count("cdn/"), 1);
    let cached = get(
        &fixture,
        "/repository/amo/uBlock0@raymondhill.net/1.2.0/uBlock0-1.2.0.xpi",
    )
    .await;
    assert_eq!(cached.body, *upstream.current);
    assert_eq!(upstream.count("api/"), 2);

    let older = get(
        &fixture,
        "/repository/amo/uBlock0%40raymondhill.net/1.1.0.xpi",
    )
    .await;
    assert_eq!(older.status, StatusCode::OK, "{}", older.text());
    assert_eq!(older.body, *upstream.previous);
    assert_eq!(
        upstream.count("api/v5/addons/addon/uBlock0@raymondhill.net/versions/1.1.0/"),
        1
    );
    let older_again = get(
        &fixture,
        "/repository/amo/uBlock0@raymondhill.net/1.1.0.xpi",
    )
    .await;
    assert_eq!(older_again.body, *upstream.previous);
    assert_eq!(upstream.count("downloads/file/11/"), 1);
    let absent = "/repository/amo/uBlock0@raymondhill.net/9.9.9.xpi";
    assert_eq!(get(&fixture, absent).await.status, StatusCode::NOT_FOUND);
    assert_eq!(get(&fixture, absent).await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        upstream.count("api/v5/addons/addon/uBlock0@raymondhill.net/versions/9.9.9/"),
        1
    );

    let checked = manifest_of(
        &get(
            &fixture,
            "/repository/amo/update/VersionCheck.php?reqVersion=2&id=uBlock0%40raymondhill.net&version=1.0.0&appVersion=128.0",
        )
        .await,
    );
    assert_eq!(versions(&checked, UBLOCK), ["1.2.0"]);
    let update = &checked["addons"][UBLOCK]["updates"][0];
    assert!(
        update["update_link"]
            .as_str()
            .unwrap()
            .ends_with("/repository/amo/uBlock0@raymondhill.net/1.2.0/uBlock0-1.2.0.xpi"),
        "{update}"
    );
    assert_eq!(update["update_hash"], sha256(&upstream.current));
    assert_eq!(
        update["browser_specific_settings"]["gecko"],
        json!({"strict_min_version": "109.0"})
    );
    let own = manifest_of(
        &get(
            &fixture,
            "/repository/amo/uBlock0@raymondhill.net/updates.json",
        )
        .await,
    );
    assert_eq!(own, checked);
    let by_slug = manifest_of(&get(&fixture, "/repository/amo/ublock-origin/updates.json").await);
    assert_eq!(by_slug, checked);
    assert_eq!(upstream.count("api/"), 4);

    let nobody = "/repository/amo/update/VersionCheck.php?id=nobody%40example.org";
    assert_eq!(
        manifest_of(&get(&fixture, nobody).await),
        json!({"addons": {}})
    );
    assert_eq!(
        manifest_of(&get(&fixture, nobody).await),
        json!({"addons": {}})
    );
    assert_eq!(upstream.count("api/v5/addons/addon/nobody@example.org/"), 1);
    assert_eq!(
        get(&fixture, "/repository/amo/unlisted/latest.xpi")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let braced = get(
        &fixture,
        &format!("/repository/amo/{REMOTE_UUID_PATH}/latest.xpi"),
    )
    .await;
    assert_eq!(braced.status, StatusCode::OK, "{}", braced.text());
    assert_eq!(braced.body, *upstream.braced);
    assert_eq!(
        upstream.count("api/v5/addons/addon/{11111111-2222-3333-4444-555555555555}/"),
        1
    );
    let braced_manifest = manifest_of(&get(&fixture, "/repository/amo/braced/updates.json").await);
    let link = braced_manifest["addons"][REMOTE_UUID]["updates"][0]["update_link"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        link.ends_with(&format!(
            "/repository/amo/{REMOTE_UUID_PATH}/4.0/11111111-2222-3333-4444-555555555555-4.0.xpi"
        )),
        "{link}"
    );
    assert_eq!(get(&fixture, &local(&link)).await.body, *upstream.braced);

    let mismatch = get(&fixture, "/repository/amo/bad-hash/latest.xpi").await;
    assert_eq!(
        mismatch.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        mismatch.text()
    );
    assert!(
        mismatch.text().contains("does not match"),
        "{}",
        mismatch.text()
    );
    let broken = get(&fixture, "/repository/amo/not-an-addon/latest.xpi").await;
    assert_eq!(broken.status, StatusCode::BAD_GATEWAY, "{}", broken.text());
    assert!(
        broken.text().contains("did not send a Firefox add-on"),
        "{}",
        broken.text()
    );
    let invalid = get(&fixture, "/repository/amo/update/VersionCheck.php?id=a%20b").await;
    assert_eq!(
        invalid.status,
        StatusCode::BAD_REQUEST,
        "{}",
        invalid.text()
    );

    let components = get(&fixture, "/service/rest/v1/components?repository=amo").await;
    let mut names: Vec<String> = components.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            format!(
                "{}@{}",
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap()
            )
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            format!("{UBLOCK}@1.1.0"),
            format!("{UBLOCK}@1.2.0"),
            format!("{REMOTE_UUID}@4.0"),
        ]
    );
    let item = components.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["version"] == "1.2.0")
        .unwrap()
        .clone();
    assert_eq!(
        item["assets"][0]["contentType"], "application/x-xpinstall",
        "{item}"
    );
    let tree = get(&fixture, "/service/rest/internal/ui/browse/amo?path=/").await;
    let mut roots: Vec<String> = tree.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap().to_owned())
        .collect();
    roots.sort();
    assert_eq!(roots, [UBLOCK, REMOTE_UUID]);

    assert_eq!(
        get(&fixture, "/repository/amo/updates.json").await.status,
        StatusCode::NOT_FOUND
    );
    let upload = put(
        &fixture,
        "/repository/amo/x.xpi",
        Some(&token(&fixture, Scope::Write).await),
        &upstream.current,
    )
    .await;
    assert_eq!(upload.status, StatusCode::BAD_REQUEST);
    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/amo/uBlock0@raymondhill.net/1.2.0/uBlock0-1.2.0.xpi",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let refetched = get(&fixture, "/repository/amo/ublock-origin/latest.xpi").await;
    assert_eq!(refetched.body, *upstream.current);
    assert_eq!(upstream.count("cdn/"), 2);

    let live = get(
        &fixture,
        "/repository/live/uBlock0@raymondhill.net/latest.xpi",
    )
    .await;
    assert_eq!(live.status, StatusCode::OK, "{}", live.text());
    let asked = upstream.count("api/");
    upstream.broken.store(true, Ordering::SeqCst);
    let stale = get(
        &fixture,
        "/repository/live/uBlock0@raymondhill.net/latest.xpi",
    )
    .await;
    assert_eq!(stale.status, StatusCode::OK, "{}", stale.text());
    assert_eq!(stale.body, *upstream.current);
    assert_eq!(upstream.count("api/"), asked + 1);
    let stale_manifest = manifest_of(
        &get(
            &fixture,
            "/repository/live/update/VersionCheck.php?id=uBlock0%40raymondhill.net",
        )
        .await,
    );
    assert_eq!(versions(&stale_manifest, UBLOCK), ["1.2.0"]);
    let unknown = get(&fixture, "/repository/live/braced/latest.xpi").await;
    assert_eq!(
        unknown.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        unknown.text()
    );
}

#[tokio::test]
async fn groups_take_each_addon_from_the_first_member_that_knows_it() {
    let fixture = ready().await;
    let (upstream, remote) = serve_amo(&fixture).await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(&fixture, "hosted", hosted("shadow", "ALLOW")).await;
    create(&fixture, "proxy", proxy("amo", &remote, 1440)).await;
    create(&fixture, "group", group("all", &["own", "amo"])).await;
    create(&fixture, "group", group("shadowed", &["shadow", "amo"])).await;
    let writer = token(&fixture, Scope::Write).await;
    let own = addon(NAME, "1.0.0");
    let shadow = addon(UBLOCK, "0.1.0");
    for (repository, data) in [("own", &own), ("shadow", &shadow)] {
        let reply = put(
            &fixture,
            &format!("/repository/{repository}/a.xpi"),
            Some(&writer),
            data,
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    let listed = manifest_of(&get(&fixture, "/repository/all/updates.json").await);
    assert_eq!(
        listed["addons"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        [NAME]
    );
    assert!(
        listed["addons"][NAME]["updates"][0]["update_link"]
            .as_str()
            .unwrap()
            .ends_with("/repository/all/name@example.org/1.0.0/name-1.0.0.xpi")
    );
    let hosted_check = manifest_of(
        &get(
            &fixture,
            "/repository/all/update/VersionCheck.php?id=name%40example.org",
        )
        .await,
    );
    assert_eq!(versions(&hosted_check, NAME), ["1.0.0"]);
    assert_eq!(upstream.count("api/"), 0);
    let remote_check = manifest_of(
        &get(
            &fixture,
            "/repository/all/update/VersionCheck.php?id=uBlock0%40raymondhill.net",
        )
        .await,
    );
    assert_eq!(versions(&remote_check, UBLOCK), ["1.2.0"]);
    let link = remote_check["addons"][UBLOCK]["updates"][0]["update_link"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        link.ends_with("/repository/all/uBlock0@raymondhill.net/1.2.0/uBlock0-1.2.0.xpi"),
        "{link}"
    );
    assert_eq!(get(&fixture, &local(&link)).await.body, *upstream.current);
    assert_eq!(
        get(&fixture, "/repository/all/name@example.org/latest.xpi")
            .await
            .body,
        own
    );
    assert_eq!(
        get(&fixture, "/repository/all/ublock-origin/latest.xpi")
            .await
            .body,
        *upstream.current
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/all/name@example.org/1.0.0/name-1.0.0.xpi"
        )
        .await
        .body,
        own
    );
    assert_eq!(
        get(&fixture, "/repository/all/nobody@example.org/latest.xpi")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let asked = upstream.count("api/");
    let shadowed = manifest_of(
        &get(
            &fixture,
            "/repository/shadowed/uBlock0@raymondhill.net/updates.json",
        )
        .await,
    );
    assert_eq!(versions(&shadowed, UBLOCK), ["0.1.0"]);
    assert_eq!(
        get(
            &fixture,
            "/repository/shadowed/uBlock0@raymondhill.net/latest.xpi"
        )
        .await
        .body,
        shadow
    );
    assert_eq!(upstream.count("api/"), asked);
    assert_eq!(
        get(
            &fixture,
            "/repository/shadowed/uBlock0@raymondhill.net/1.2.0.xpi"
        )
        .await
        .body,
        *upstream.current
    );

    let write = put(&fixture, "/repository/all/x.xpi", Some(&writer), &own).await;
    assert_eq!(write.status, StatusCode::BAD_REQUEST);
    let removal = fixture
        .call(
            Method::DELETE,
            "/repository/all/name@example.org/1.0.0/name-1.0.0.xpi",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removal.status, StatusCode::METHOD_NOT_ALLOWED);
}
