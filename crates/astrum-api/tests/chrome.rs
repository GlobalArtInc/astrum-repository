mod support;

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use astrum_format_chrome::gupdate::{self, RemoteApp};
use astrum_format_chrome::testing::{crx, id, manifest};
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};
use url::form_urlencoded;

const BOUNDARY: &str = "chromeboundary0123456789";
const PRODVERSION: &str = "131.0.6778.85";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_chrome::Chrome::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/chrome/{kind}"),
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

fn proxy(name: &str, remote: &str, metadata_max_age: u32) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": false},
    })
}

async fn token(fixture: &Fixture) -> String {
    format!(
        "Bearer {}",
        fixture
            .state
            .security()
            .create_token("admin", "chrome", Scope::Write, None)
            .await
            .unwrap()
            .1
    )
}

async fn put(fixture: &Fixture, path: &str, authorization: &str, data: &[u8]) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::None,
            Body::from(data.to_vec()),
            &[
                ("content-type", "application/x-chrome-extension"),
                ("authorization", authorization),
            ],
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

fn package_path(id: &str, version: &str) -> String {
    format!("/{id}/{version}/{id}-{version}.crx")
}

fn check(entries: &[(&str, Option<&str>)], prodversion: Option<&str>) -> String {
    let mut query = form_urlencoded::Serializer::new(String::new());
    query.append_pair("response", "updatecheck");
    query.append_pair("os", "linux");
    query.append_pair("arch", "x64");
    if let Some(prodversion) = prodversion {
        query.append_pair("prodversion", prodversion);
    }
    query.append_pair("acceptformat", "crx3");
    for (id, version) in entries {
        let entry = match version {
            Some(version) => format!("id={id}&v={version}&installsource=policy&uc"),
            None => format!("id={id}&v=&uc"),
        };
        query.append_pair("x", &entry);
    }
    query.finish()
}

fn answer(reply: &Reply) -> BTreeMap<String, RemoteApp> {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        header_of(reply, "content-type"),
        Some("text/xml; charset=UTF-8")
    );
    gupdate::parse_answer(&reply.body).unwrap_or_else(|error| panic!("{error}: {}", reply.text()))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn multipart(name: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"extension.crx\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(data);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

#[tokio::test]
async fn hosted_extensions_are_stored_and_offered_to_browsers() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("extensions", "ALLOW_ONCE")).await;
    let writer = token(&fixture).await;
    let acme = id(b"acme key");
    let other = id(b"other key");
    let unknown = id(b"nobody");

    let page = fixture
        .call(Method::GET, "/repository/extensions/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("updates.xml"));
    assert!(page.text().contains("ExtensionInstallForcelist"));

    let first = crx(
        b"acme key",
        &manifest("Acme", "1.0.0"),
        &[("background.js", b"1")],
    );
    let stored = put(
        &fixture,
        &format!("/repository/extensions{}", package_path(&acme, "1.0.0")),
        &writer,
        &first,
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(stored.json()["id"], acme.as_str());
    assert_eq!(stored.json()["name"], "Acme");
    assert_eq!(stored.json()["manifestVersion"], 3);
    assert!(header_of(&stored, "location").unwrap().ends_with(&format!(
        "/repository/extensions{}",
        package_path(&acme, "1.0.0")
    )));

    let again = put(
        &fixture,
        &format!("/repository/extensions{}", package_path(&acme, "1.0.0")),
        &writer,
        &first,
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(again.text().contains("redeploy is not allowed"));

    let misplaced = put(
        &fixture,
        &format!("/repository/extensions{}", package_path(&acme, "1.0.1")),
        &writer,
        &first,
    )
    .await;
    assert_eq!(misplaced.status, StatusCode::BAD_REQUEST);
    assert!(
        misplaced.text().contains("belongs at"),
        "{}",
        misplaced.text()
    );

    let garbage = put(
        &fixture,
        "/repository/extensions/x.crx",
        &writer,
        b"PK\x03\x04 nope",
    )
    .await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert!(garbage.text().contains("Cr24"), "{}", garbage.text());
    let mut old = first.clone();
    old[4] = 2;
    let crx2 = put(&fixture, "/repository/extensions/x.crx", &writer, &old).await;
    assert_eq!(crx2.status, StatusCode::BAD_REQUEST);
    assert!(crx2.text().contains("CRX2"), "{}", crx2.text());
    let versionless = crx(
        b"acme key",
        r#"{"manifest_version": 3, "name": "Acme"}"#,
        &[],
    );
    let refused = put(
        &fixture,
        "/repository/extensions/x.crx",
        &writer,
        &versionless,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(refused.text().contains("no version"), "{}", refused.text());
    let anonymous = fixture
        .send_body(
            Method::PUT,
            "/repository/extensions/x.crx",
            Auth::None,
            Body::from(first.clone()),
            &[],
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let newer = crx(b"acme key", &manifest("Acme", "1.10.0"), &[]);
    let anywhere = put(
        &fixture,
        "/repository/extensions/upload/acme.crx",
        &writer,
        &newer,
    )
    .await;
    assert_eq!(anywhere.status, StatusCode::CREATED, "{}", anywhere.text());
    let localized = crx(
        b"other key",
        r#"{"manifest_version": 2, "name": "__MSG_extName__", "version": "0.9", "default_locale": "de"}"#,
        &[(
            "_locales/de/messages.json",
            br#"{"extName": {"message": "Anderer Name"}}"#,
        )],
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/extensions/other.crx",
            &writer,
            &localized
        )
        .await
        .status,
        StatusCode::CREATED
    );

    let query = check(
        &[
            (&acme, Some("1.0.0")),
            (&other, Some("0.9")),
            (&unknown, None),
        ],
        Some(PRODVERSION),
    );
    let apps = answer(
        &get(
            &fixture,
            &format!("/repository/extensions/updates.xml?{query}"),
        )
        .await,
    );
    assert_eq!(apps[&acme].version.as_deref(), Some("1.10.0"));
    assert!(apps[&acme].offered().is_some());
    assert!(apps[&acme].codebase.as_deref().unwrap().ends_with(&format!(
        "/repository/extensions{}",
        package_path(&acme, "1.10.0")
    )));
    assert_eq!(apps[&acme].sha256.as_deref(), Some(sha256(&newer).as_str()));
    assert_eq!(apps[&acme].size, Some(newer.len() as u64));
    assert_eq!(apps[&other].check_status, "noupdate");
    assert_eq!(apps[&unknown].status, "error-unknownApplication");

    let at_root = get(&fixture, &format!("/repository/extensions?{query}")).await;
    assert_eq!(answer(&at_root)[&acme].version.as_deref(), Some("1.10.0"));
    let at_slash = get(&fixture, &format!("/repository/extensions/?{query}")).await;
    assert_eq!(answer(&at_slash)[&acme].version.as_deref(), Some("1.10.0"));
    let empty = get(
        &fixture,
        "/repository/extensions/updates.xml?response=updatecheck",
    )
    .await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);

    let codebase = apps[&acme].codebase.clone().unwrap();
    let path = &codebase[codebase.find("/repository/").unwrap()..];
    let downloaded = get(&fixture, path).await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(downloaded.body, newer);
    assert_eq!(
        header_of(&downloaded, "content-type"),
        Some("application/x-chrome-extension")
    );

    let latest = get(
        &fixture,
        &format!("/repository/extensions/{acme}/latest.crx"),
    )
    .await;
    assert_eq!(latest.status, StatusCode::FOUND);
    assert!(header_of(&latest, "location").unwrap().ends_with(path));
    let redirected = get(
        &fixture,
        &format!("/repository/extensions/updates.xml?response=redirect&prodversion={PRODVERSION}&acceptformat=crx3&x=id%3D{acme}%26uc"),
    )
    .await;
    assert_eq!(redirected.status, StatusCode::FOUND);
    assert!(header_of(&redirected, "location").unwrap().ends_with(path));
    let nothing = get(
        &fixture,
        &format!("/repository/extensions/{unknown}/latest.crx"),
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);

    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=extensions",
    )
    .await;
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
    let mut expected = vec![
        (acme.clone(), "1.0.0".to_owned()),
        (acme.clone(), "1.10.0".to_owned()),
        (other.clone(), "0.9".to_owned()),
    ];
    expected.sort();
    assert_eq!(names, expected);
    assert_eq!(items[0]["format"], "chrome");

    let by_name = get(
        &fixture,
        "/service/rest/v1/search?repository=extensions&chrome.name=Anderer%20Name",
    )
    .await;
    let found = by_name.json()["items"].as_array().unwrap().clone();
    assert_eq!(found.len(), 1, "{}", by_name.text());
    assert_eq!(found[0]["name"], other.as_str());
    let by_manifest = get(
        &fixture,
        "/service/rest/v1/search?repository=extensions&chrome.manifestVersion=3",
    )
    .await;
    assert_eq!(
        by_manifest.json()["items"].as_array().unwrap().len(),
        2,
        "{}",
        by_manifest.text()
    );
    let by_keyword = get(
        &fixture,
        "/service/rest/v1/search?repository=extensions&q=Anderer",
    )
    .await;
    assert_eq!(by_keyword.json()["items"][0]["name"], other.as_str());

    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/repository/extensions{}", package_path(&acme, "1.10.0")),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let after = answer(
        &get(
            &fixture,
            &format!("/repository/extensions/updates.xml?{query}"),
        )
        .await,
    );
    assert_eq!(after[&acme].check_status, "noupdate");
}

#[tokio::test]
async fn the_upload_form_and_the_browse_tree_cover_extensions() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("extensions", "ALLOW")).await;
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let package = crx(b"form key", &manifest("Form", "2.0"), &[]);
    let sent = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=extensions",
            Auth::Admin,
            Body::from(multipart("chrome.asset", &package)),
            &[("content-type", content_type.as_str())],
        )
        .await;
    assert_eq!(sent.status, StatusCode::NO_CONTENT, "{}", sent.text());
    let again = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=extensions",
            Auth::Admin,
            Body::from(multipart("chrome.asset", &package)),
            &[("content-type", content_type.as_str())],
        )
        .await;
    assert_eq!(again.status, StatusCode::NO_CONTENT, "{}", again.text());
    let refused = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=extensions",
            Auth::Admin,
            Body::from(multipart("chrome.asset", b"not a package")),
            &[("content-type", content_type.as_str())],
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let form = id(b"form key");
    let tree = get(
        &fixture,
        "/service/rest/internal/ui/browse/extensions?path=/",
    )
    .await;
    assert_eq!(tree.status, StatusCode::OK, "{}", tree.text());
    let names: Vec<String> = tree.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, std::slice::from_ref(&form));
    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=extensions",
    )
    .await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    let component = items[0]["id"].as_str().unwrap().to_owned();
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{component}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let gone = get(
        &fixture,
        &format!("/repository/extensions{}", package_path(&form, "2.0")),
    )
    .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn content_selectors_decide_which_extensions_a_reader_sees() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("extensions", "ALLOW")).await;
    let writer = token(&fixture).await;
    let public = id(b"public key");
    let secret = id(b"secret key");
    for key in [b"public key".as_slice(), b"secret key".as_slice()] {
        let reply = put(
            &fixture,
            "/repository/extensions/x.crx",
            &writer,
            &crx(key, &manifest("X", "1.0"), &[]),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "public-extensions", "description": "", "expression": format!("format == \"chrome\" and path =^ \"/{public}/\"")}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "public-extensions-read", "description": "", "actions": ["READ"], "format": "chrome", "repository": "extensions", "contentSelector": "public-extensions"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "extension-readers", "name": "extension-readers", "description": "", "privileges": ["public-extensions-read"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-pass", &["extension-readers"])
        .await;
    let reader = |path: String| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::GET,
                    &path,
                    Auth::Basic("reader", "reader-pass"),
                    None,
                )
                .await
        }
    };
    let query = check(&[(&public, None), (&secret, None)], None);
    let apps = answer(&reader(format!("/repository/extensions/updates.xml?{query}")).await);
    assert_eq!(apps[&public].version.as_deref(), Some("1.0"));
    assert_eq!(apps[&secret].status, "error-unknownApplication");
    assert_eq!(
        reader(format!("/repository/extensions/{secret}/latest.crx"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        reader(format!(
            "/repository/extensions{}",
            package_path(&secret, "1.0")
        ))
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        reader(format!(
            "/repository/extensions{}",
            package_path(&public, "1.0")
        ))
        .await
        .status,
        StatusCode::OK
    );
}

#[derive(Clone)]
struct WebStore {
    address: String,
    seen: Arc<Mutex<Vec<String>>>,
    files: Arc<AtomicUsize>,
    down: Arc<AtomicBool>,
    known: Arc<Vec<u8>>,
    tampered: Arc<Vec<u8>>,
}

const KNOWN_KEY: &[u8] = b"web store key";
const TAMPERED_KEY: &[u8] = b"tampered key";
const INNER_KEY: &[u8] = b"internal address key";

impl WebStore {
    fn checks(&self) -> Vec<String> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|seen| seen.starts_with("service/update2/crx?"))
            .cloned()
            .collect()
    }
}

fn app(id: &str, codebase: &str, hash: &str, size: usize, version: &str) -> String {
    format!(
        "<app appid=\"{id}\" cohort=\"1::\" cohortname=\"\" status=\"ok\"><updatecheck codebase=\"{codebase}\" fp=\"1.{hash}\" hash_sha256=\"{hash}\" protected=\"0\" size=\"{size}\" status=\"ok\" version=\"{version}\"/></app>"
    )
}

async fn web_store(
    State(store): State<WebStore>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let query = query.unwrap_or_default();
    store.seen.lock().unwrap().push(format!("{path}?{query}"));
    if store.down.load(Ordering::SeqCst) {
        return (StatusCode::SERVICE_UNAVAILABLE, "down").into_response();
    }
    match path.as_str() {
        "service/update2/crx" => {
            let mut apps = String::new();
            for (name, value) in form_urlencoded::parse(query.as_bytes()) {
                if name != "x" {
                    continue;
                }
                let wanted: BTreeMap<String, String> = form_urlencoded::parse(value.as_bytes())
                    .map(|(field, content)| (field.into_owned(), content.into_owned()))
                    .collect();
                let asked = wanted.get("id").cloned().unwrap_or_default();
                if asked == id(KNOWN_KEY) {
                    apps.push_str(&app(
                        &asked,
                        &format!("{}/redirect/known.crx", store.address),
                        &sha256(&store.known),
                        store.known.len(),
                        "2.0.0",
                    ));
                } else if asked == id(TAMPERED_KEY) {
                    apps.push_str(&app(
                        &asked,
                        &format!("{}/files/tampered.crx", store.address),
                        &"0".repeat(64),
                        store.tampered.len(),
                        "1.0",
                    ));
                } else if asked == id(INNER_KEY) {
                    apps.push_str(&app(
                        &asked,
                        "http://169.254.169.254/latest.crx",
                        &"0".repeat(64),
                        1,
                        "1.0",
                    ));
                } else {
                    apps.push_str(&format!(
                        "<app appid=\"{asked}\" status=\"error-unknownApplication\"/>"
                    ));
                }
            }
            (
                [(header::CONTENT_TYPE, "text/xml; charset=UTF-8")],
                format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><gupdate xmlns=\"http://www.google.com/update2/response\" protocol=\"2.0\" server=\"prod\"><daystart elapsed_days=\"7236\" elapsed_seconds=\"100\"/>{apps}</gupdate>"
                ),
            )
                .into_response()
        }
        "redirect/known.crx" => (
            StatusCode::FOUND,
            [(
                header::LOCATION,
                format!("{}/files/known.crx", store.address),
            )],
        )
            .into_response(),
        "files/known.crx" => {
            store.files.fetch_add(1, Ordering::SeqCst);
            (
                [(header::CONTENT_TYPE, "application/x-chrome-extension")],
                store.known.as_ref().clone(),
            )
                .into_response()
        }
        "files/tampered.crx" => (
            [(header::CONTENT_TYPE, "application/x-chrome-extension")],
            store.tampered.as_ref().clone(),
        )
            .into_response(),
        _ => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn serve_web_store(fixture: &Fixture) -> (WebStore, String) {
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let state = WebStore {
        address: address.clone(),
        seen: Arc::default(),
        files: Arc::default(),
        down: Arc::default(),
        known: Arc::new(crx(
            KNOWN_KEY,
            &manifest("Web Store Extension", "2.0.0"),
            &[],
        )),
        tampered: Arc::new(crx(TAMPERED_KEY, &manifest("Tampered", "1.0"), &[])),
    };
    let app = Router::new()
        .route("/{*path}", routing::get(web_store))
        .with_state(state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (state, format!("{address}/service/update2/crx"))
}

fn local_path(codebase: &str) -> String {
    codebase[codebase.find("/repository/").unwrap()..].to_owned()
}

#[tokio::test]
async fn proxies_rewrite_update_answers_and_cache_packages() {
    let fixture = ready().await;
    let (store, remote) = serve_web_store(&fixture).await;
    create(&fixture, "proxy", proxy("webstore", &remote, 1440)).await;
    create(&fixture, "proxy", proxy("live", &remote, 0)).await;
    let known = id(KNOWN_KEY);
    let unknown = id(b"not in the store");

    let query = check(
        &[(&known, Some("1.0.0")), (&unknown, None)],
        Some(PRODVERSION),
    );
    let first = answer(
        &get(
            &fixture,
            &format!("/repository/webstore/updates.xml?{query}"),
        )
        .await,
    );
    let codebase = first[&known].codebase.clone().unwrap();
    assert!(
        codebase.ends_with(&format!(
            "/repository/webstore{}",
            package_path(&known, "2.0.0")
        )),
        "{codebase}"
    );
    assert_eq!(
        first[&known].sha256.as_deref(),
        Some(sha256(&store.known).as_str())
    );
    assert_eq!(first[&known].size, Some(store.known.len() as u64));
    assert_eq!(first[&unknown].status, "error-unknownApplication");
    let asked = store.checks();
    assert_eq!(asked.len(), 1);
    let params: Vec<(String, String)> =
        form_urlencoded::parse(asked[0].split_once('?').unwrap().1.as_bytes())
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect();
    assert!(
        params.contains(&("prodversion".to_owned(), PRODVERSION.to_owned())),
        "{params:?}"
    );
    assert!(params.contains(&("acceptformat".to_owned(), "crx3".to_owned())));
    assert!(params.contains(&("x".to_owned(), format!("id={known}&v=0.0.0.0&uc"))));
    assert!(params.contains(&("x".to_owned(), format!("id={unknown}&v=0.0.0.0&uc"))));

    let second = answer(&get(&fixture, &format!("/repository/webstore?{query}")).await);
    assert_eq!(second[&known].codebase.as_deref(), Some(codebase.as_str()));
    assert_eq!(second[&unknown].status, "error-unknownApplication");
    assert_eq!(store.checks().len(), 1);
    let current = check(&[(&known, Some("2.0.0"))], Some(PRODVERSION));
    let noupdate = answer(
        &get(
            &fixture,
            &format!("/repository/webstore/updates.xml?{current}"),
        )
        .await,
    );
    assert_eq!(noupdate[&known].check_status, "noupdate");

    let downloaded = get(&fixture, &local_path(&codebase)).await;
    assert_eq!(downloaded.status, StatusCode::OK, "{}", downloaded.text());
    assert_eq!(downloaded.body, *store.known);
    assert_eq!(store.files.load(Ordering::SeqCst), 1);
    let cached = get(&fixture, &local_path(&codebase)).await;
    assert_eq!(cached.body, *store.known);
    assert_eq!(store.files.load(Ordering::SeqCst), 1);

    let redirected = get(
        &fixture,
        &format!("/repository/webstore/updates.xml?response=redirect&acceptformat=crx3&x=id%3D{known}%26uc"),
    )
    .await;
    assert_eq!(redirected.status, StatusCode::FOUND);
    assert_eq!(
        header_of(&redirected, "location").map(local_path),
        Some(local_path(&codebase))
    );
    let latest = get(
        &fixture,
        &format!("/repository/webstore/{known}/latest.crx"),
    )
    .await;
    assert_eq!(latest.status, StatusCode::FOUND);

    let components = get(&fixture, "/service/rest/v1/components?repository=webstore").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1, "{}", components.text());
    assert_eq!(items[0]["name"], known.as_str());
    assert_eq!(items[0]["version"], "2.0.0");
    let tree = get(&fixture, "/service/rest/internal/ui/browse/webstore?path=/").await;
    let names: Vec<String> = tree.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, std::slice::from_ref(&known));

    let tampered = id(TAMPERED_KEY);
    let inner = id(INNER_KEY);
    let more = check(&[(&tampered, None), (&inner, None)], None);
    let apps = answer(
        &get(
            &fixture,
            &format!("/repository/webstore/updates.xml?{more}"),
        )
        .await,
    );
    let defaulted = store.checks().pop().unwrap();
    assert!(
        defaulted.contains(&format!(
            "prodversion={}",
            astrum_format_chrome::DEFAULT_PRODVERSION
        )),
        "{defaulted}"
    );
    let broken = get(
        &fixture,
        &local_path(apps[&tampered].codebase.as_deref().unwrap()),
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_GATEWAY);
    assert!(
        broken.text().contains("does not match the hash_sha256"),
        "{}",
        broken.text()
    );
    let guarded = get(
        &fixture,
        &local_path(apps[&inner].codebase.as_deref().unwrap()),
    )
    .await;
    assert_eq!(guarded.status, StatusCode::BAD_GATEWAY);
    assert!(guarded.text().contains("SSRF"), "{}", guarded.text());
    let unknown_file = get(
        &fixture,
        &format!("/repository/webstore{}", package_path(&unknown, "1.0")),
    )
    .await;
    assert_eq!(unknown_file.status, StatusCode::NOT_FOUND);

    let writer = token(&fixture).await;
    let upload = put(
        &fixture,
        "/repository/webstore/x.crx",
        &writer,
        &store.known,
    )
    .await;
    assert_eq!(upload.status, StatusCode::BAD_REQUEST);

    let removed = fixture
        .call(Method::DELETE, &local_path(&codebase), Auth::Admin, None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let refetched = get(&fixture, &local_path(&codebase)).await;
    assert_eq!(refetched.body, *store.known);
    assert_eq!(store.files.load(Ordering::SeqCst), 2);

    let live = check(&[(&known, None)], Some(PRODVERSION));
    let before = store.checks().len();
    answer(&get(&fixture, &format!("/repository/live/updates.xml?{live}")).await);
    answer(&get(&fixture, &format!("/repository/live/updates.xml?{live}")).await);
    assert_eq!(store.checks().len(), before + 2);
    store.down.store(true, Ordering::SeqCst);
    let stale = answer(&get(&fixture, &format!("/repository/live/updates.xml?{live}")).await);
    assert_eq!(stale[&known].version.as_deref(), Some("2.0.0"));
    let nothing = check(&[(&id(b"never seen"), None)], Some(PRODVERSION));
    let failed = get(&fixture, &format!("/repository/live/updates.xml?{nothing}")).await;
    assert_eq!(failed.status, StatusCode::BAD_GATEWAY, "{}", failed.text());
}

#[tokio::test]
async fn groups_ask_their_members_in_order() {
    let fixture = ready().await;
    let (store, remote) = serve_web_store(&fixture).await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(&fixture, "proxy", proxy("webstore", &remote, 1440)).await;
    create(&fixture, "group", group("all", &["own", "webstore"])).await;
    let writer = token(&fixture).await;
    let own_id = id(b"own key");
    let known = id(KNOWN_KEY);
    let own = crx(b"own key", &manifest("Own", "3.1"), &[]);
    assert_eq!(
        put(&fixture, "/repository/own/own.crx", &writer, &own)
            .await
            .status,
        StatusCode::CREATED
    );

    let query = check(
        &[(&own_id, None), (&known, None), (&id(b"nobody"), None)],
        Some(PRODVERSION),
    );
    let apps = answer(&get(&fixture, &format!("/repository/all/updates.xml?{query}")).await);
    let own_codebase = apps[&own_id].codebase.clone().unwrap();
    let known_codebase = apps[&known].codebase.clone().unwrap();
    assert!(own_codebase.ends_with(&format!("/repository/all{}", package_path(&own_id, "3.1"))));
    assert!(known_codebase.ends_with(&format!("/repository/all{}", package_path(&known, "2.0.0"))));
    assert_eq!(apps[&id(b"nobody")].status, "error-unknownApplication");
    let asked = store.checks();
    assert_eq!(asked.len(), 1);
    assert!(!asked[0].contains(&own_id), "{}", asked[0]);

    assert_eq!(get(&fixture, &local_path(&own_codebase)).await.body, own);
    assert_eq!(
        get(&fixture, &local_path(&known_codebase)).await.body,
        *store.known
    );
    let latest = get(&fixture, &format!("/repository/all/{known}/latest.crx")).await;
    assert_eq!(latest.status, StatusCode::FOUND);
    assert_eq!(
        header_of(&latest, "location").map(local_path),
        Some(local_path(&known_codebase))
    );
    let write = put(&fixture, "/repository/all/x.crx", &writer, &own).await;
    assert_eq!(write.status, StatusCode::BAD_REQUEST);
    let delete = fixture
        .call(
            Method::DELETE,
            &local_path(&own_codebase),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(delete.status, StatusCode::METHOD_NOT_ALLOWED);
}
