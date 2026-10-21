mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use serde_json::{Value, json};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const JSON: &str = "application/json";
const BOUNDARY: &str = "vagrant-form-boundary-0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_vagrant::Vagrant));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/vagrant/{kind}"),
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

fn package(provider: &str, marker: &str) -> Vec<u8> {
    let mut archive = tar::Builder::new(Vec::new());
    for (name, data) in [
        ("metadata.json", format!("{{\"provider\": \"{provider}\"}}")),
        ("Vagrantfile", format!("# {marker}\n")),
        ("box.img", marker.repeat(64)),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, name, data.as_bytes())
            .unwrap();
    }
    let tar = archive.into_inner().unwrap();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&tar).unwrap();
    encoder.finish().unwrap()
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn sha1(data: &[u8]) -> String {
    hex::encode(Sha1::digest(data))
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
    fixture
        .send(Method::GET, path, auth, None, &[("accept", JSON)])
        .await
}

async fn head(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    fixture
        .send(Method::HEAD, path, auth, None, &[("accept", JSON)])
        .await
}

async fn bearer(fixture: &Fixture, method: Method, path: &str, token: &str) -> Reply {
    let value = format!("Bearer {token}");
    fixture
        .send(
            method,
            path,
            Auth::None,
            None,
            &[("accept", JSON), ("authorization", &value)],
        )
        .await
}

fn versions(document: &Value) -> Vec<String> {
    document["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| version["version"].as_str().unwrap().to_owned())
        .collect()
}

fn providers(document: &Value, version: &str) -> Vec<Value> {
    document["versions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["version"] == version)
        .unwrap_or_else(|| panic!("no version {version} in {document}"))["providers"]
        .as_array()
        .unwrap()
        .clone()
}

fn error_text(reply: &Reply) -> String {
    reply.json()["errors"][0]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn form(fields: &[(&str, &str)], file: &[u8]) -> Vec<u8> {
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
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"vagrant.asset\"; filename=\"package.box\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

#[tokio::test]
async fn hosted_repositories_take_boxes_and_serve_their_metadata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("boxes", "ALLOW_ONCE")).await;
    let missing = get(&fixture, "/repository/boxes/acme/base", Auth::Admin).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers["content-type"], JSON);
    assert_eq!(error_text(&missing), "box acme/base was not found");

    let amd64 = package("virtualbox", "amd64");
    let arm64 = package("virtualbox", "arm64");
    let libvirt = package("libvirt", "libvirt");
    let uploaded = put(
        &fixture,
        "boxes",
        "/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &amd64,
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::CREATED, "{}", uploaded.text());
    assert_eq!(
        uploaded.json(),
        json!({
            "name": "acme/base",
            "version": "1.0.0",
            "provider": "virtualbox",
            "architecture": "amd64",
            "checksum_type": "sha256",
            "checksum": sha256(&amd64),
            "size": amd64.len(),
            "url": "http://localhost/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
        })
    );
    for (path, body) in [
        ("/acme/base/1.0.0/virtualbox/arm64.box", &arm64),
        ("/acme/base/1.1.0/libvirt/amd64.box", &libvirt),
    ] {
        let reply = put(&fixture, "boxes", path, Auth::Admin, body).await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{path}: {}",
            reply.text()
        );
    }

    let checked = head(&fixture, "/repository/boxes/acme/base", Auth::Admin).await;
    assert_eq!(checked.status, StatusCode::OK);
    assert_eq!(checked.headers["content-type"], JSON);
    assert!(checked.body.is_empty());
    let listing = get(&fixture, "/repository/boxes/acme/base", Auth::Admin).await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(listing.headers["content-type"], JSON);
    assert_eq!(
        checked.headers["content-length"].to_str().unwrap(),
        listing.body.len().to_string()
    );
    let document = listing.json();
    for alias in [
        "/repository/boxes/api/v2/vagrant/acme/base",
        "/repository/boxes/acme/base/metadata.json",
    ] {
        assert_eq!(
            get(&fixture, alias, Auth::Admin).await.json(),
            document,
            "{alias}"
        );
    }
    assert_eq!(document["name"], "acme/base");
    assert_eq!(versions(&document), ["1.1.0", "1.0.0"]);
    assert_eq!(document["versions"][0]["status"], "active");
    assert_eq!(
        providers(&document, "1.0.0"),
        [
            json!({
                "name": "virtualbox",
                "url": "http://localhost/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
                "checksum_type": "sha256",
                "checksum": sha256(&amd64),
                "architecture": "amd64",
                "default_architecture": true,
            }),
            json!({
                "name": "virtualbox",
                "url": "http://localhost/repository/boxes/acme/base/1.0.0/virtualbox/arm64.box",
                "checksum_type": "sha256",
                "checksum": sha256(&arm64),
                "architecture": "arm64",
                "default_architecture": false,
            }),
        ]
    );
    let etag = listing.headers["etag"].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/boxes/acme/base",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let download = get(
        &fixture,
        "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, amd64);
    assert_eq!(download.headers["content-type"], "application/gzip");
    let probe = fixture
        .send(
            Method::HEAD,
            "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
            &[],
        )
        .await;
    assert_eq!(probe.status, StatusCode::OK);
    assert_eq!(
        probe.headers["content-length"].to_str().unwrap(),
        amd64.len().to_string()
    );
    assert_eq!(probe.headers["accept-ranges"], "bytes");
    let resumed = fixture
        .send(
            Method::GET,
            "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
            &[("range", "bytes=10-")],
        )
        .await;
    assert_eq!(resumed.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(resumed.body, amd64[10..]);
    assert_eq!(
        resumed.headers["content-range"].to_str().unwrap(),
        format!("bytes 10-{}/{}", amd64.len() - 1, amd64.len())
    );

    let again = put(
        &fixture,
        "boxes",
        "/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &amd64,
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(error_text(&again).contains("does not allow redeployment"));
    let plain = put(
        &fixture,
        "boxes",
        "/acme/base/1.2.0/virtualbox/amd64.box",
        Auth::Admin,
        b"not a box at all",
    )
    .await;
    assert_eq!(plain.status, StatusCode::BAD_REQUEST);
    assert!(error_text(&plain).contains("not a tar, tar.gz or zip archive"));
    let foreign = put(
        &fixture,
        "boxes",
        "/acme/base/1.2.0/virtualbox/x86_64.box",
        Auth::Admin,
        &amd64,
    )
    .await;
    assert_eq!(foreign.status, StatusCode::BAD_REQUEST);
    assert!(
        error_text(&foreign).contains("amd64, arm64"),
        "{}",
        foreign.text()
    );
    let unversioned = put(
        &fixture,
        "boxes",
        "/acme/base/latest/virtualbox/amd64.box",
        Auth::Admin,
        &amd64,
    )
    .await;
    assert_eq!(unversioned.status, StatusCode::BAD_REQUEST);
    assert!(
        error_text(&unversioned).contains("/<org>/<name>/<version>/<provider>/<architecture>.box")
    );

    let spec = get(
        &fixture,
        "/service/rest/v1/formats/vagrant/upload-specs",
        Auth::Admin,
    )
    .await
    .json();
    let fields: Vec<&str> = spec["componentFields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        [
            "organization",
            "name",
            "version",
            "provider",
            "architecture",
            "description"
        ]
    );
    let documented = package("virtualbox", "documented");
    let form_upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=boxes",
            Auth::Admin,
            Body::from(form(
                &[
                    ("vagrant.organization", "acme"),
                    ("vagrant.name", "base"),
                    ("vagrant.version", "1.2.0"),
                    ("vagrant.provider", "virtualbox"),
                    ("vagrant.architecture", "amd64"),
                    ("vagrant.description", "Ships the docs"),
                ],
                &documented,
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(
        form_upload.status,
        StatusCode::NO_CONTENT,
        "{}",
        form_upload.text()
    );
    let second_provider = put(
        &fixture,
        "boxes",
        "/acme/base/1.2.0/libvirt/amd64.box",
        Auth::Admin,
        &libvirt,
    )
    .await;
    assert_eq!(second_provider.status, StatusCode::CREATED);
    let document = get(&fixture, "/repository/boxes/acme/base", Auth::Admin)
        .await
        .json();
    assert_eq!(versions(&document), ["1.2.0", "1.1.0", "1.0.0"]);
    assert_eq!(document["description"], "Ships the docs");
    assert_eq!(document["versions"][0]["description"], "Ships the docs");
    assert_eq!(providers(&document, "1.2.0").len(), 2);
    let incomplete = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=boxes",
            Auth::Admin,
            Body::from(form(
                &[("vagrant.organization", "acme"), ("vagrant.name", "base")],
                &documented,
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(incomplete.status, StatusCode::BAD_REQUEST);

    let search = get(
        &fixture,
        "/service/rest/v1/search?repository=boxes&vagrant.provider=libvirt",
        Auth::Admin,
    )
    .await
    .json();
    let found: Vec<(&str, &str, &str)> = search["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["group"].as_str().unwrap(),
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(found.len(), 2, "{search}");
    assert!(found.contains(&("acme", "base", "1.1.0")));
    let by_words = get(
        &fixture,
        "/service/rest/v1/search?q=docs&format=vagrant",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(by_words["items"].as_array().unwrap().len(), 1, "{by_words}");
    assert_eq!(by_words["items"][0]["version"], "1.2.0");
    let browse = get(
        &fixture,
        "/service/rest/internal/ui/browse/boxes?path=/acme/base/",
        Auth::Admin,
    )
    .await;
    assert_eq!(browse.status, StatusCode::OK, "{}", browse.text());
    let children: Vec<(String, String)> = browse.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        children,
        [
            ("component".to_owned(), "1.0.0".to_owned()),
            ("component".to_owned(), "1.1.0".to_owned()),
            ("component".to_owned(), "1.2.0".to_owned()),
        ]
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let document = get(&fixture, "/repository/boxes/acme/base", Auth::Admin)
        .await
        .json();
    let left = providers(&document, "1.0.0");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0]["architecture"], "arm64");
    assert_eq!(left[0]["default_architecture"], true);
    let gone = get(
        &fixture,
        "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let twice = fixture
        .call(
            Method::DELETE,
            "/repository/boxes/acme/base/1.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(twice.status, StatusCode::NOT_FOUND);
    for path in [
        "/repository/boxes/acme/base/1.1.0/libvirt/amd64.box",
        "/repository/boxes/acme/base/1.0.0/virtualbox/arm64.box",
    ] {
        let reply = fixture.call(Method::DELETE, path, Auth::Admin, None).await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{path}");
    }
    let document = get(&fixture, "/repository/boxes/acme/base", Auth::Admin)
        .await
        .json();
    assert_eq!(versions(&document), ["1.2.0"]);
}

#[tokio::test]
async fn vagrant_tokens_and_privileges_decide_who_reads_and_uploads() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("private", "ALLOW")).await;
    let secret = package("virtualbox", "secret");
    let uploaded = put(
        &fixture,
        "private",
        "/acme/secret/2.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &secret,
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::CREATED);

    let anonymous = get(&fixture, "/repository/private/acme/secret", Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous.headers["www-authenticate"],
        "Basic realm=\"Astrum Repository\""
    );
    assert!(error_text(&anonymous).contains("VAGRANT_CLOUD_TOKEN"));
    let probe = head(&fixture, "/repository/private/acme/secret", Auth::None).await;
    assert_eq!(probe.status, StatusCode::UNAUTHORIZED);
    let download = get(
        &fixture,
        "/repository/private/acme/secret/2.0.0/virtualbox/amd64.box",
        Auth::None,
    )
    .await;
    assert_eq!(download.status, StatusCode::UNAUTHORIZED);
    let nobody = get(
        &fixture,
        "/repository/private/api/v2/authenticate",
        Auth::None,
    )
    .await;
    assert_eq!(nobody.status, StatusCode::UNAUTHORIZED);
    let anonymous_upload = put(
        &fixture,
        "private",
        "/acme/secret/2.1.0/virtualbox/amd64.box",
        Auth::None,
        &secret,
    )
    .await;
    assert_eq!(anonymous_upload.status, StatusCode::UNAUTHORIZED);

    let (_, token) = fixture
        .state
        .security()
        .create_token("admin", "vagrant", Scope::Read, None)
        .await
        .unwrap();
    let listed = bearer(
        &fixture,
        Method::GET,
        "/repository/private/acme/secret",
        &token,
    )
    .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    assert_eq!(
        providers(&listed.json(), "2.0.0")[0]["url"],
        "http://localhost/repository/private/acme/secret/2.0.0/virtualbox/amd64.box"
    );
    let checked = bearer(
        &fixture,
        Method::HEAD,
        "/repository/private/api/v2/vagrant/acme/secret",
        &token,
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK);
    assert_eq!(checked.headers["content-type"], JSON);
    let fetched = bearer(
        &fixture,
        Method::GET,
        "/repository/private/acme/secret/2.0.0/virtualbox/amd64.box",
        &token,
    )
    .await;
    assert_eq!(fetched.body, secret);
    let who = bearer(
        &fixture,
        Method::GET,
        "/repository/private/api/v2/authenticate",
        &token,
    )
    .await;
    assert_eq!(who.status, StatusCode::OK);
    assert_eq!(who.json(), json!({"user": {"username": "admin"}}));
    let read_only = fixture
        .send_body(
            Method::PUT,
            "/repository/private/acme/secret/2.1.0/virtualbox/amd64.box",
            Auth::None,
            Body::from(secret.clone()),
            &[("authorization", &format!("Bearer {token}"))],
        )
        .await;
    assert_eq!(
        read_only.status,
        StatusCode::FORBIDDEN,
        "{}",
        read_only.text()
    );
    let with_password = fixture
        .send(
            Method::POST,
            "/repository/private/api/v1/authenticate",
            Auth::None,
            Some(json!({"user": {"login": "admin", "password": "secret"}, "token": {}})),
            &[],
        )
        .await;
    assert_eq!(with_password.status, StatusCode::BAD_REQUEST);
    assert!(error_text(&with_password).contains("vagrant cloud auth login --token"));
    let as_password = get(
        &fixture,
        "/repository/private/acme/secret",
        Auth::Basic("admin", &token),
    )
    .await;
    assert_eq!(as_password.status, StatusCode::OK);

    fixture.add_user("reader", "reader-password", &[]).await;
    let stranger = get(
        &fixture,
        "/repository/private/acme/secret",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(stranger.status, StatusCode::FORBIDDEN);
    assert_eq!(stranger.headers["content-type"], JSON);
    let stranger_upload = put(
        &fixture,
        "private",
        "/acme/secret/2.1.0/virtualbox/amd64.box",
        Auth::Basic("reader", "reader-password"),
        &secret,
    )
    .await;
    assert_eq!(stranger_upload.status, StatusCode::FORBIDDEN);
    let stranger_name = get(
        &fixture,
        "/repository/private/api/v2/authenticate",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(stranger_name.json()["user"]["username"], "reader");

    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "box-readers", "name": "box-readers", "description": "", "privileges": ["as-repository-view-vagrant-private-read", "as-repository-view-vagrant-private-browse"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("consumer", "consumer-password", &["box-readers"])
        .await;
    let read = get(
        &fixture,
        "/repository/private/acme/secret",
        Auth::Basic("consumer", "consumer-password"),
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    let box_body = get(
        &fixture,
        "/repository/private/acme/secret/2.0.0/virtualbox/amd64.box",
        Auth::Basic("consumer", "consumer-password"),
    )
    .await;
    assert_eq!(box_body.body, secret);
    let refused = put(
        &fixture,
        "private",
        "/acme/secret/2.1.0/virtualbox/amd64.box",
        Auth::Basic("consumer", "consumer-password"),
        &secret,
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let not_deleted = fixture
        .call(
            Method::DELETE,
            "/repository/private/acme/secret/2.0.0/virtualbox/amd64.box",
            Auth::Basic("consumer", "consumer-password"),
            None,
        )
        .await;
    assert_eq!(not_deleted.status, StatusCode::FORBIDDEN);
    let replaced = put(
        &fixture,
        "private",
        "/acme/secret/2.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &package("virtualbox", "replaced"),
    )
    .await;
    assert_eq!(replaced.status, StatusCode::CREATED, "{}", replaced.text());
    let document = get(&fixture, "/repository/private/acme/secret", Auth::Admin)
        .await
        .json();
    assert_eq!(
        providers(&document, "2.0.0")[0]["checksum"],
        sha256(&package("virtualbox", "replaced"))
    );
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<(String, String)>>>,
    address: Arc<Mutex<String>>,
    remote: Arc<Vec<u8>>,
    legacy: Arc<Vec<u8>>,
}

impl Upstream {
    fn count(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(hit, _)| hit == path)
            .count()
    }
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> Response {
    let accept = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    upstream.hits.lock().unwrap().push((path.clone(), accept));
    let base = upstream.address.lock().unwrap().clone();
    match path.as_str() {
        "acme/remote" => json!({
            "name": "acme/remote",
            "description": "",
            "short_description": "A box from far away",
            "versions": [
                {"version": "2.0.0", "status": "active", "providers": [
                    {"name": "virtualbox", "architecture": "amd64", "default_architecture": true, "checksum_type": "sha256", "checksum": sha256(&upstream.remote), "url": format!("http://{base}/acme/boxes/remote/versions/2.0.0/providers/virtualbox/amd64/vagrant.box")}
                ]},
                {"version": "1.9.0", "status": "active", "providers": [
                    {"name": "virtualbox", "architecture": "amd64", "default_architecture": true, "checksum_type": "sha256", "checksum": "00".repeat(32), "url": format!("http://{base}/storage/remote.box")}
                ]},
                {"version": "0.1.0", "providers": [
                    {"name": "libvirt", "checksum_type": "sha1", "checksum": sha1(&upstream.legacy), "url": "/storage/legacy.box"}
                ]}
            ]
        })
        .to_string()
        .into_response(),
        "acme/base" => json!({
            "name": "acme/base",
            "description": "The remote base box",
            "versions": [
                {"version": "1.0.0", "status": "active", "providers": [
                    {"name": "virtualbox", "architecture": "amd64", "default_architecture": true, "checksum_type": "sha256", "checksum": sha256(&upstream.remote), "url": format!("http://{base}/storage/remote.box")}
                ]},
                {"version": "0.9.0", "status": "active", "providers": [
                    {"name": "virtualbox", "architecture": "amd64", "default_architecture": true, "url": format!("http://{base}/storage/remote.box")}
                ]}
            ]
        })
        .to_string()
        .into_response(),
        "acme/boxes/remote/versions/2.0.0/providers/virtualbox/amd64/vagrant.box" => {
            Redirect::to(&format!("http://{base}/storage/remote.box")).into_response()
        }
        "storage/remote.box" => upstream.remote.as_ref().clone().into_response(),
        "storage/legacy.box" => upstream.legacy.as_ref().clone().into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            json!({"errors": ["box not found"]}).to_string(),
        )
            .into_response(),
    }
}

#[tokio::test]
async fn proxies_rewrite_box_urls_and_groups_merge_their_members() {
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
        remote: Arc::new(package("virtualbox", "remote")),
        legacy: Arc::new(package("libvirt", "legacy")),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    *upstream.address.lock().unwrap() = address.to_string();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "vagrant-cloud",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
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
            "name": "vagrant-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "vagrant-cloud"]},
        }),
    )
    .await;
    let own = package("virtualbox", "own");
    let uploaded = put(
        &fixture,
        "own",
        "/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &own,
    )
    .await;
    assert_eq!(uploaded.status, StatusCode::CREATED);

    let checked = head(
        &fixture,
        "/repository/vagrant-cloud/acme/remote",
        Auth::Admin,
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK, "{}", checked.text());
    assert_eq!(checked.headers["content-type"], JSON);
    let listing = get(
        &fixture,
        "/repository/vagrant-cloud/acme/remote",
        Auth::Admin,
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    let document = listing.json();
    assert_eq!(upstream.count("acme/remote"), 1);
    assert_eq!(
        upstream.hits.lock().unwrap()[0],
        ("acme/remote".to_owned(), JSON.to_owned())
    );
    assert_eq!(document["name"], "acme/remote");
    assert_eq!(document["description"], "A box from far away");
    assert_eq!(versions(&document), ["2.0.0", "1.9.0", "0.1.0"]);
    assert_eq!(
        providers(&document, "2.0.0"),
        [json!({
            "name": "virtualbox",
            "url": "http://localhost/repository/vagrant-cloud/acme/remote/2.0.0/virtualbox/amd64.box",
            "checksum_type": "sha256",
            "checksum": sha256(&upstream.remote),
            "architecture": "amd64",
            "default_architecture": true,
        })]
    );
    assert_eq!(
        providers(&document, "0.1.0"),
        [json!({
            "name": "libvirt",
            "url": "http://localhost/repository/vagrant-cloud/acme/remote/0.1.0/libvirt/unknown.box",
            "checksum_type": "sha1",
            "checksum": sha1(&upstream.legacy),
            "architecture": "unknown",
            "default_architecture": true,
        })]
    );

    for _ in 0..2 {
        let fetched = get(
            &fixture,
            "/repository/vagrant-cloud/acme/remote/2.0.0/virtualbox/amd64.box",
            Auth::Admin,
        )
        .await;
        assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.text());
        assert_eq!(fetched.body, *upstream.remote);
        assert_eq!(fetched.headers["content-type"], "application/gzip");
    }
    assert_eq!(upstream.count("storage/remote.box"), 1);
    assert_eq!(
        upstream.count("acme/boxes/remote/versions/2.0.0/providers/virtualbox/amd64/vagrant.box"),
        1
    );
    let resumed = fixture
        .send(
            Method::GET,
            "/repository/vagrant-cloud/acme/remote/2.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
            &[("range", "bytes=0-9")],
        )
        .await;
    assert_eq!(resumed.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(resumed.body, upstream.remote[..10]);
    let mismatch = get(
        &fixture,
        "/repository/vagrant-cloud/acme/remote/1.9.0/virtualbox/amd64.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(
        mismatch.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        mismatch.text()
    );
    let legacy = get(
        &fixture,
        "/repository/vagrant-cloud/acme/remote/0.1.0/libvirt/unknown.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(legacy.status, StatusCode::OK, "{}", legacy.text());
    assert_eq!(legacy.body, *upstream.legacy);
    for missing in [
        "/repository/vagrant-cloud/acme/remote/2.0.0/libvirt/amd64.box",
        "/repository/vagrant-cloud/acme/nothing",
        "/repository/vagrant-cloud/acme/nothing/1.0.0/virtualbox/amd64.box",
    ] {
        let reply = get(&fixture, missing, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{missing}");
    }
    let cached = get(
        &fixture,
        "/service/rest/v1/search?repository=vagrant-cloud&format=vagrant",
        Auth::Admin,
    )
    .await
    .json();
    let cached_versions: Vec<&str> = cached["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["version"].as_str().unwrap())
        .collect();
    assert_eq!(cached_versions.len(), 2, "{cached}");
    assert!(cached_versions.contains(&"2.0.0") && cached_versions.contains(&"0.1.0"));

    let merged = get(&fixture, "/repository/vagrant-all/acme/base", Auth::Admin).await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    let merged = merged.json();
    assert_eq!(versions(&merged), ["1.0.0", "0.9.0"]);
    assert_eq!(merged["description"], "The remote base box");
    let first = providers(&merged, "1.0.0");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0]["checksum"], sha256(&own));
    assert_eq!(
        first[0]["url"],
        "http://localhost/repository/vagrant-all/acme/base/1.0.0/virtualbox/amd64.box"
    );
    assert!(providers(&merged, "0.9.0")[0].get("checksum").is_none());
    let from_own = get(
        &fixture,
        "/repository/vagrant-all/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(from_own.body, own);
    let from_remote = get(
        &fixture,
        "/repository/vagrant-all/acme/base/0.9.0/virtualbox/amd64.box",
        Auth::Admin,
    )
    .await;
    assert_eq!(from_remote.status, StatusCode::OK, "{}", from_remote.text());
    assert_eq!(from_remote.body, *upstream.remote);
    let through_group = get(
        &fixture,
        "/repository/vagrant-all/api/v2/vagrant/acme/remote",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions(&through_group), ["2.0.0", "1.9.0", "0.1.0"]);

    for (repository, method) in [
        ("vagrant-all", Method::PUT),
        ("vagrant-all", Method::DELETE),
        ("vagrant-cloud", Method::PUT),
    ] {
        let reply = fixture
            .send_body(
                method.clone(),
                &format!("/repository/{repository}/acme/base/3.0.0/virtualbox/amd64.box"),
                Auth::Admin,
                Body::from(own.clone()),
                &[],
            )
            .await;
        assert_eq!(
            reply.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{repository} {method}"
        );
    }

    let dropped = fixture
        .call(
            Method::DELETE,
            "/repository/vagrant-cloud/acme/remote",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(dropped.status, StatusCode::NO_CONTENT);
    let refreshed = get(
        &fixture,
        "/repository/vagrant-cloud/acme/remote",
        Auth::Admin,
    )
    .await;
    assert_eq!(refreshed.status, StatusCode::OK);
    assert_eq!(upstream.count("acme/remote"), 2);
    let evicted = fixture
        .call(
            Method::DELETE,
            "/repository/vagrant-cloud/acme/remote/2.0.0/virtualbox/amd64.box",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(evicted.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn the_rest_api_creates_and_checks_vagrant_repositories() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("vagrant-hosted", "ALLOW_ONCE")).await;
    let stored = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/vagrant/hosted/vagrant-hosted",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.status, StatusCode::OK);
    let document = stored.json();
    assert_eq!(document["format"], "vagrant");
    assert_eq!(document["type"], "hosted");
    assert_eq!(
        document["url"],
        "http://localhost/repository/vagrant-hosted"
    );
    let mut changed = document.clone();
    changed["storage"]["writePolicy"] = json!("DENY");
    let update = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/vagrant/hosted/vagrant-hosted",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT, "{}", update.text());
    let refused = put(
        &fixture,
        "vagrant-hosted",
        "/acme/base/1.0.0/virtualbox/amd64.box",
        Auth::Admin,
        &package("virtualbox", "read-only"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(error_text(&refused).contains("read-only"));

    let page = fixture
        .call(
            Method::GET,
            "/repository/vagrant-hosted/",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.text()
            .contains("VAGRANT_SERVER_URL=http://localhost/repository/vagrant-hosted")
    );

    let proxy = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/vagrant/proxy",
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
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(hosted("files", "ALLOW")),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let group = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/vagrant/group",
            Auth::Admin,
            Some(json!({
                "name": "mixed",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["vagrant-hosted", "files"]},
            })),
        )
        .await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST, "{}", group.text());
}

#[tokio::test]
async fn cleanup_tells_pre_releases_from_releases() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("clean", "ALLOW_ONCE")).await;
    for version in ["1.0.0", "1.1.0.beta1", "2.0.0-rc1", "2.0.0"] {
        let reply = put(
            &fixture,
            "clean",
            &format!("/acme/tidy/{version}/virtualbox/amd64.box"),
            Auth::Admin,
            &package("virtualbox", version),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{version}");
    }
    let preview = |release: &'static str| {
        let fixture = &fixture;
        async move {
            let reply = fixture
                .call(
                    Method::POST,
                    "/service/rest/internal/cleanup-policies/preview/summary",
                    Auth::Admin,
                    Some(json!({"name": "pre", "format": "vagrant", "criteriaReleaseType": release, "repository": "clean"})),
                )
                .await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
            reply.json()["componentCount"].clone()
        }
    };
    assert_eq!(preview("PRERELEASES").await, 2);
    assert_eq!(preview("RELEASES").await, 2);
}
