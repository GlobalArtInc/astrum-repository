mod support;

use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "helmboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_helm::Helm::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/helm/{kind}"),
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

fn proxy(name: &str, remote: &str, metadata_max_age: i64) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {
            "blocked": false,
            "autoBlock": false,
            "authentication": {"type": "username", "username": "mirror", "password": "mirror-secret"}
        }
    })
}

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members}
    })
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

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn chart_yaml(name: &str, version: &str) -> String {
    format!(
        "apiVersion: v2\nname: {name}\ndescription: The {name} test chart\ntype: application\nversion: {version}\nappVersion: \"1.16.0\"\nkeywords:\n  - web\n  - probe\nmaintainers:\n  - name: Jane Doe\n    email: jane@example.org\nhome: https://example.org/{name}\n"
    )
}

fn package(name: &str, version: &str) -> Vec<u8> {
    package_with(name, &chart_yaml(name, version))
}

fn package_with(top: &str, chart: &str) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in [
        (format!("{top}/Chart.yaml"), chart.as_bytes().to_vec()),
        (format!("{top}/values.yaml"), b"replicaCount: 1\n".to_vec()),
        (
            format!("{top}/templates/configmap.yaml"),
            b"apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {{ .Release.Name }}\n".to_vec(),
        ),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path, data.as_slice())
            .unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

fn provenance(name: &str, version: &str, data: &[u8]) -> String {
    format!(
        "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA512\n\napiVersion: v2\ndescription: The {name} test chart\nkeywords:\n- - web\nname: {name}\nversion: {version}\n\n...\nfiles:\n  {name}-{version}.tgz: sha256:{}\n-----BEGIN PGP SIGNATURE-----\n\nwsBcBAEBCgAQBQJqzTW/\n-----END PGP SIGNATURE-----\n",
        sha256(data)
    )
}

async fn put(fixture: &Fixture, path: &str, data: Vec<u8>, auth: Auth<'_>) -> Reply {
    fixture
        .send_body(Method::PUT, path, auth, Body::from(data), &[])
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn index_of(reply: &Reply) -> Value {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    serde_norway::from_slice(&reply.body)
        .unwrap_or_else(|error| panic!("{error}: {}", reply.text()))
}

fn versions(index: &Value, name: &str) -> Vec<String> {
    index["entries"][name]
        .as_array()
        .map(|charts| {
            charts
                .iter()
                .map(|chart| chart["version"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn form(parts: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (field, filename, data) in parts {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn post_form(fixture: &Fixture, path: &str, auth: Auth<'_>, body: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::POST,
            path,
            auth,
            Body::from(body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await
}

#[tokio::test]
async fn hosted_charts_go_in_by_put_and_come_out_through_the_index() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW_ONCE")).await;

    let empty = index_of(&get(&fixture, "/repository/charts/index.yaml").await);
    assert_eq!(empty["apiVersion"], "v1");
    assert_eq!(empty["entries"], json!({}));

    let first = package("demo", "0.1.0");
    let second = package("demo", "0.10.0");
    for (path, data) in [
        ("/repository/charts/demo-0.1.0.tgz", first.clone()),
        ("/repository/charts/demo-0.10.0.tgz", second.clone()),
    ] {
        let reply = put(&fixture, path, data, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }

    let reply = get(&fixture, "/repository/charts/index.yaml").await;
    assert_eq!(reply.headers[header::CONTENT_TYPE], "text/x-yaml");
    let index = index_of(&reply);
    assert_eq!(versions(&index, "demo"), ["0.10.0", "0.1.0"]);
    let newest = &index["entries"]["demo"][0];
    assert_eq!(newest["urls"], json!(["demo-0.10.0.tgz"]));
    assert_eq!(newest["digest"], sha256(&second));
    assert_eq!(newest["appVersion"], "1.16.0");
    assert_eq!(newest["apiVersion"], "v2");
    assert_eq!(newest["description"], "The demo test chart");
    assert_eq!(newest["keywords"], json!(["web", "probe"]));
    assert_eq!(
        newest["maintainers"],
        json!([{"email": "jane@example.org", "name": "Jane Doe"}])
    );
    assert!(newest["created"].as_str().unwrap().ends_with('Z'));

    let etag = reply.headers[header::ETAG].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/charts/index.yaml",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    let again = get(&fixture, "/repository/charts/index.yaml").await;
    assert_eq!(again.body, reply.body);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/charts/index.yaml",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());

    let download = get(&fixture, "/repository/charts/demo-0.1.0.tgz").await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.headers[header::CONTENT_TYPE], "application/x-tgz");
    assert_eq!(download.body, first);

    let redeploy = put(
        &fixture,
        "/repository/charts/demo-0.1.0.tgz",
        first.clone(),
        Auth::Admin,
    )
    .await;
    assert_eq!(redeploy.status, StatusCode::CONFLICT, "{}", redeploy.text());

    for (path, data, expected) in [
        (
            "/repository/charts/other-9.9.9.tgz",
            package("demo", "0.2.0"),
            "belongs at demo-0.2.0.tgz",
        ),
        (
            "/repository/charts/sub/demo-0.2.0.tgz",
            package("demo", "0.2.0"),
            "is not a place for charts",
        ),
        (
            "/repository/charts/bad-1.0.0.tgz",
            b"garbage".to_vec(),
            "not a Helm chart archive",
        ),
        (
            "/repository/charts/nameless-1.0.0.tgz",
            package_with("nameless", "apiVersion: v2\nversion: 1.0.0\n"),
            "has no name",
        ),
        (
            "/repository/charts/readme.txt",
            b"hello".to_vec(),
            "is not a place for charts",
        ),
    ] {
        let reply = put(&fixture, path, data, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(reply.text().contains(expected), "{path}: {}", reply.text());
    }
    let assets = get(&fixture, "/service/rest/v1/assets?repository=charts").await;
    let mut paths: Vec<String> = assets.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap().to_owned())
        .collect();
    paths.sort();
    assert_eq!(paths, ["/demo-0.1.0.tgz", "/demo-0.10.0.tgz"]);

    let directory = get(&fixture, "/repository/charts/").await;
    assert_eq!(directory.status, StatusCode::OK);
    assert!(directory.text().contains("helm repo add charts"));
    assert_eq!(
        get(&fixture, "/repository/charts/missing-1.0.0.tgz")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let patch = fixture
        .call(
            Method::PATCH,
            "/repository/charts/demo-0.1.0.tgz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(patch.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(patch.headers[header::ALLOW], "GET,HEAD,PUT,POST,DELETE");

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/charts/demo-0.10.0.tgz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK);
    let index = index_of(&get(&fixture, "/repository/charts/index.yaml").await);
    assert_eq!(versions(&index, "demo"), ["0.1.0"]);
    let missing = fixture
        .call(
            Method::DELETE,
            "/repository/charts/demo-0.10.0.tgz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn charts_become_components_with_their_metadata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW")).await;
    let data = package("demo", "1.2.3");
    put(
        &fixture,
        "/repository/charts/demo-1.2.3.tgz",
        data.clone(),
        Auth::Admin,
    )
    .await;
    let found = get(&fixture, "/service/rest/v1/search?repository=charts").await;
    let items = found.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["format"], "helm");
    assert_eq!(items[0]["group"], "");
    assert_eq!(items[0]["name"], "demo");
    assert_eq!(items[0]["version"], "1.2.3");
    let asset = &items[0]["assets"][0];
    assert_eq!(asset["path"], "/demo-1.2.3.tgz");
    assert_eq!(asset["contentType"], "application/x-tgz");
    assert_eq!(asset["checksum"]["sha256"], sha256(&data));
    assert_eq!(asset["helm"]["appVersion"], "1.16.0");
    assert_eq!(asset["helm"]["home"], "https://example.org/demo");

    for (query, expected) in [
        ("helm.appVersion=1.16.0", 1),
        ("helm.appVersion=2.0", 0),
        ("helm.description=*test%20chart", 1),
        ("helm.keywords=*probe*", 1),
        ("helm.keywords=*nothing*", 0),
        ("format=helm&name=demo", 1),
    ] {
        let reply = get(
            &fixture,
            &format!("/service/rest/v1/search?repository=charts&{query}"),
        )
        .await;
        assert_eq!(
            reply.json()["items"].as_array().unwrap().len(),
            expected,
            "{query}"
        );
    }

    let replaced = package_with(
        "demo",
        "apiVersion: v2\nname: demo\nversion: 1.2.3\ndescription: Replaced\n",
    );
    let reply = put(
        &fixture,
        "/repository/charts/demo-1.2.3.tgz",
        replaced.clone(),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    let index = index_of(&get(&fixture, "/repository/charts/index.yaml").await);
    assert_eq!(index["entries"]["demo"][0]["description"], "Replaced");
    assert_eq!(index["entries"]["demo"][0]["digest"], sha256(&replaced));
    assert!(index["entries"]["demo"][0].get("keywords").is_none());
}

#[tokio::test]
async fn provenance_files_follow_their_charts() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW_ONCE")).await;
    let data = package("demo", "0.4.0");
    let signed = provenance("demo", "0.4.0", &data);
    put(
        &fixture,
        "/repository/charts/demo-0.4.0.tgz",
        data.clone(),
        Auth::Admin,
    )
    .await;
    let reply = put(
        &fixture,
        "/repository/charts/demo-0.4.0.tgz.prov",
        signed.clone().into_bytes(),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let fetched = get(&fixture, "/repository/charts/demo-0.4.0.tgz.prov").await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.text(), signed);

    let wrong = put(
        &fixture,
        "/repository/charts/other-1.0.0.tgz.prov",
        signed.clone().into_bytes(),
        Auth::Admin,
    )
    .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    assert!(wrong.text().contains("belongs at demo-0.4.0.tgz.prov"));
    let broken = put(
        &fixture,
        "/repository/charts/demo-0.5.0.tgz.prov",
        b"not signed".to_vec(),
        Auth::Admin,
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);

    let component = get(&fixture, "/service/rest/v1/search?repository=charts").await;
    let items = component.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1);
    let mut paths: Vec<&str> = items[0]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap())
        .collect();
    paths.sort();
    assert_eq!(paths, ["/demo-0.4.0.tgz", "/demo-0.4.0.tgz.prov"]);
    assert_eq!(
        items[0]["assets"][0]["helm"]["description"],
        "The demo test chart"
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/charts/demo-0.4.0.tgz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK);
    assert_eq!(
        get(&fixture, "/repository/charts/demo-0.4.0.tgz.prov")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let left = get(&fixture, "/service/rest/v1/search?repository=charts").await;
    assert_eq!(left.json()["items"], json!([]));
}

#[tokio::test]
async fn the_chart_push_api_takes_forms_and_plain_bodies() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW_ONCE")).await;
    let data = package("demo", "0.5.0");
    let signed = provenance("demo", "0.5.0", &data);
    let reply = post_form(
        &fixture,
        "/repository/charts/api/charts",
        Auth::Admin,
        form(&[
            ("chart", "demo-0.5.0.tgz", &data),
            ("prov", "demo-0.5.0.tgz.prov", signed.as_bytes()),
        ]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(reply.json(), json!({"saved": true}));
    assert_eq!(
        get(&fixture, "/repository/charts/demo-0.5.0.tgz.prov")
            .await
            .text(),
        signed
    );

    let again = post_form(
        &fixture,
        "/repository/charts/api/charts",
        Auth::Admin,
        form(&[("chart", "demo-0.5.0.tgz", &data)]),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert!(
        again.json()["error"]
            .as_str()
            .unwrap()
            .contains("redeploy is not allowed")
    );

    let plain = fixture
        .send_body(
            Method::POST,
            "/repository/charts/api/charts",
            Auth::Admin,
            Body::from(package("plain", "1.0.0")),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(plain.status, StatusCode::CREATED, "{}", plain.text());

    let other = package("other", "2.0.0");
    let mismatch = post_form(
        &fixture,
        "/repository/charts/api/charts",
        Auth::Admin,
        form(&[
            ("prov", "x.prov", signed.as_bytes()),
            ("chart", "other-2.0.0.tgz", &other),
        ]),
    )
    .await;
    assert_eq!(mismatch.status, StatusCode::BAD_REQUEST);
    assert!(
        mismatch.json()["error"]
            .as_str()
            .unwrap()
            .contains("not the chart other 2.0.0")
    );

    let other_signed = provenance("other", "2.0.0", &other);
    let prov_only = post_form(
        &fixture,
        "/repository/charts/api/prov",
        Auth::Admin,
        form(&[("prov", "other-2.0.0.tgz.prov", other_signed.as_bytes())]),
    )
    .await;
    assert_eq!(
        prov_only.status,
        StatusCode::CREATED,
        "{}",
        prov_only.text()
    );

    let anonymous = post_form(
        &fixture,
        "/repository/charts/api/charts",
        Auth::None,
        form(&[("chart", "demo-0.6.0.tgz", &package("demo", "0.6.0"))]),
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert!(anonymous.json()["error"].is_string());

    let missing = post_form(
        &fixture,
        "/repository/charts/api/charts",
        Auth::Admin,
        form(&[("other", "x", b"x")]),
    )
    .await;
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);

    let index = index_of(&get(&fixture, "/repository/charts/index.yaml").await);
    assert_eq!(versions(&index, "demo"), ["0.5.0"]);
    assert_eq!(versions(&index, "plain"), ["1.0.0"]);
    assert_eq!(versions(&index, "other"), ["2.0.0"]);
}

#[tokio::test]
async fn uploads_through_the_components_api_and_write_policies() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW_ONCE")).await;
    let spec = get(&fixture, "/service/rest/v1/formats/helm/upload-specs").await;
    assert_eq!(
        spec.json(),
        json!({
            "format": "helm",
            "multipleUpload": false,
            "componentFields": [],
            "assetFields": [{"name": "asset", "type": "FILE", "description": null, "optional": false, "group": null, "groupHelpText": null, "options": null}]
        })
    );
    let data = package("uploaded", "3.0.0");
    let upload = |data: Vec<u8>| {
        let fixture = &fixture;
        async move {
            post_form(
                fixture,
                "/service/rest/v1/components?repository=charts",
                Auth::Admin,
                form(&[("helm.asset", "whatever.tgz", &data)]),
            )
            .await
        }
    };
    let reply = upload(data.clone()).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(
        get(&fixture, "/repository/charts/uploaded-3.0.0.tgz")
            .await
            .body,
        data
    );
    let twice = upload(data.clone()).await;
    assert_eq!(twice.status, StatusCode::BAD_REQUEST);
    assert!(
        twice.text().contains("redeploy is not allowed"),
        "{}",
        twice.text()
    );
    assert_eq!(
        upload(b"garbage".to_vec()).await.status,
        StatusCode::BAD_REQUEST
    );

    create(&fixture, "hosted", hosted("frozen", "DENY")).await;
    let denied = put(
        &fixture,
        "/repository/frozen/uploaded-3.0.0.tgz",
        data.clone(),
        Auth::Admin,
    )
    .await;
    assert_eq!(denied.status, StatusCode::BAD_REQUEST);
    assert!(denied.text().contains("read-only"));
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/frozen/index.yaml",
                Auth::None,
                None
            )
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn indexes_list_only_the_charts_a_selector_lets_the_caller_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("charts", "ALLOW")).await;
    for (name, version) in [("public", "1.0.0"), ("secret", "1.0.0")] {
        put(
            &fixture,
            &format!("/repository/charts/{name}-{version}.tgz"),
            package(name, version),
            Auth::Admin,
        )
        .await;
    }
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "public-charts", "description": "", "expression": "format == \"helm\" and (path == \"/index.yaml\" or path =^ \"/public-\")"}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "public-charts-read", "description": "", "actions": ["READ"], "format": "helm", "repository": "charts", "contentSelector": "public-charts"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "chart-readers", "name": "chart-readers", "description": "", "privileges": ["public-charts-read"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-pass", &["chart-readers"])
        .await;
    let reader = || Auth::Basic("reader", "reader-pass");
    let index = fixture
        .call(Method::GET, "/repository/charts/index.yaml", reader(), None)
        .await;
    let index = index_of(&index);
    assert_eq!(versions(&index, "public"), ["1.0.0"]);
    assert!(index["entries"].get("secret").is_none());
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/charts/secret-1.0.0.tgz",
                reader(),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/charts/public-2.0.0.tgz",
            package("public", "2.0.0"),
            reader()
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
}

type Seen = (String, Option<String>);

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<AtomicUsize>,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Upstream {
    fn authorization_for(&self, path: &str) -> Vec<Option<String>> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(seen, _)| seen == path)
            .map(|(_, authorization)| authorization.clone())
            .collect()
    }

    fn count(&self, path: &str) -> usize {
        self.authorization_for(path).len()
    }

    fn set(&self, path: &str, data: Vec<u8>) {
        self.files.lock().unwrap().insert(path.to_owned(), data);
    }
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.hits.fetch_add(1, Ordering::SeqCst);
    upstream.seen.lock().unwrap().push((
        format!("/{path}"),
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
    ));
    match upstream.files.lock().unwrap().get(&path) {
        Some(data) => (
            [(header::CONTENT_TYPE, "application/octet-stream")],
            data.clone(),
        )
            .into_response(),
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

struct Remote {
    charts: Upstream,
    files: Upstream,
    address: SocketAddr,
    relative: Vec<u8>,
    elsewhere: Vec<u8>,
}

async fn remote() -> Remote {
    let charts = Upstream::default();
    let files = Upstream::default();
    let files_address = serve_upstream(files.clone()).await;
    let address = serve_upstream(charts.clone()).await;
    let relative = package("demo", "1.0.0");
    let elsewhere = package("demo", "2.0.0");
    charts.set("stable/charts/demo-1.0.0.tgz", relative.clone());
    charts.set(
        "stable/charts/demo-1.0.0.tgz.prov",
        provenance("demo", "1.0.0", &relative).into_bytes(),
    );
    files.set("releases/demo-2.0.0.tgz", elsewhere.clone());
    let index = format!(
        "apiVersion: v1\nentries:\n  demo:\n  - apiVersion: v2\n    name: demo\n    version: 1.0.0\n    description: Relative\n    appVersion: \"1.0\"\n    digest: {}\n    created: \"2026-10-01T10:00:00Z\"\n    urls:\n    - charts/demo-1.0.0.tgz\n  - apiVersion: v2\n    name: demo\n    version: 2.0.0\n    description: Elsewhere\n    keywords: [far]\n    digest: {}\n    created: \"2026-10-02T10:00:00Z\"\n    urls:\n    - http://{files_address}/releases/demo-2.0.0.tgz\ngenerated: \"2026-10-02T10:00:00Z\"\n",
        sha256(&relative),
        sha256(&elsewhere)
    );
    charts.set("stable/index.yaml", index.into_bytes());
    Remote {
        charts,
        files,
        address,
        relative,
        elsewhere,
    }
}

#[tokio::test]
async fn proxies_rewrite_chart_urls_and_cache_what_they_fetch() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let remote = remote().await;
    create(
        &fixture,
        "proxy",
        proxy(
            "upstream-charts",
            &format!("http://{}/stable", remote.address),
            1440,
        ),
    )
    .await;

    let index = index_of(&get(&fixture, "/repository/upstream-charts/index.yaml").await);
    assert_eq!(versions(&index, "demo"), ["2.0.0", "1.0.0"]);
    assert_eq!(
        index["entries"]["demo"][0]["urls"],
        json!(["demo-2.0.0.tgz"])
    );
    assert_eq!(
        index["entries"]["demo"][1]["urls"],
        json!(["demo-1.0.0.tgz"])
    );
    assert_eq!(
        index["entries"]["demo"][1]["digest"],
        sha256(&remote.relative)
    );
    assert_eq!(index["generated"], "2026-10-02T10:00:00Z");
    get(&fixture, "/repository/upstream-charts/index.yaml").await;
    assert_eq!(remote.charts.count("/stable/index.yaml"), 1);

    let near = get(&fixture, "/repository/upstream-charts/demo-1.0.0.tgz").await;
    assert_eq!(near.status, StatusCode::OK, "{}", near.text());
    assert_eq!(near.body, remote.relative);
    assert_eq!(near.headers[header::CONTENT_TYPE], "application/x-tgz");
    let far = get(&fixture, "/repository/upstream-charts/demo-2.0.0.tgz").await;
    assert_eq!(far.status, StatusCode::OK, "{}", far.text());
    assert_eq!(far.body, remote.elsewhere);
    let signed = get(&fixture, "/repository/upstream-charts/demo-1.0.0.tgz.prov").await;
    assert_eq!(signed.status, StatusCode::OK);
    assert!(signed.text().contains("demo-1.0.0.tgz: sha256:"));

    let expected = format!("Basic {}", base64_credentials("mirror", "mirror-secret"));
    assert_eq!(
        remote
            .charts
            .authorization_for("/stable/charts/demo-1.0.0.tgz"),
        [Some(expected.clone())]
    );
    assert_eq!(
        remote.charts.authorization_for("/stable/index.yaml"),
        [Some(expected)]
    );
    assert_eq!(
        remote.files.authorization_for("/releases/demo-2.0.0.tgz"),
        [None]
    );

    get(&fixture, "/repository/upstream-charts/demo-2.0.0.tgz").await;
    assert_eq!(remote.files.count("/releases/demo-2.0.0.tgz"), 1);
    assert_eq!(
        get(&fixture, "/repository/upstream-charts/demo-3.0.0.tgz")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let found = get(
        &fixture,
        "/service/rest/v1/search?repository=upstream-charts&sort=version",
    )
    .await;
    let items = found.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 2);
    let far_component = items
        .iter()
        .find(|item| item["version"] == "2.0.0")
        .unwrap();
    assert_eq!(
        far_component["assets"][0]["helm"]["description"],
        "Elsewhere"
    );
    assert_eq!(
        far_component["assets"][0]["helm"]["keywords"],
        json!(["far"])
    );
    assert!(far_component["assets"][0]["helm"].get("urls").is_none());

    let put_refused = put(
        &fixture,
        "/repository/upstream-charts/demo-9.0.0.tgz",
        package("demo", "9.0.0"),
        Auth::Admin,
    )
    .await;
    assert_eq!(put_refused.status, StatusCode::METHOD_NOT_ALLOWED);

    let dropped = fixture
        .call(
            Method::DELETE,
            "/repository/upstream-charts/index.yaml",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(dropped.status, StatusCode::OK);
    get(&fixture, "/repository/upstream-charts/index.yaml").await;
    assert_eq!(remote.charts.count("/stable/index.yaml"), 2);
}

#[tokio::test]
async fn proxies_revalidate_the_index_after_its_max_age() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let remote = remote().await;
    create(
        &fixture,
        "proxy",
        proxy(
            "fresh-charts",
            &format!("http://{}/stable/", remote.address),
            0,
        ),
    )
    .await;
    let first = index_of(&get(&fixture, "/repository/fresh-charts/index.yaml").await);
    assert_eq!(versions(&first, "demo").len(), 2);
    remote.charts.set(
        "stable/index.yaml",
        b"apiVersion: v1\nentries:\n  demo:\n  - name: demo\n    version: 3.0.0\n    urls: [charts/demo-3.0.0.tgz]\n".to_vec(),
    );
    let second = index_of(&get(&fixture, "/repository/fresh-charts/index.yaml").await);
    assert_eq!(versions(&second, "demo"), ["3.0.0"]);
    assert_eq!(remote.charts.count("/stable/index.yaml"), 2);

    remote
        .charts
        .set("stable/index.yaml", b"entries: [broken".to_vec());
    let broken = get(&fixture, "/repository/fresh-charts/index.yaml").await;
    assert_eq!(broken.status, StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn groups_merge_indexes_and_the_first_member_wins() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let remote = remote().await;
    create(&fixture, "hosted", hosted("local-charts", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy(
            "remote-charts",
            &format!("http://{}/stable", remote.address),
            1440,
        ),
    )
    .await;
    create(
        &fixture,
        "group",
        group("all-charts", &["local-charts", "remote-charts"]),
    )
    .await;
    let local = package_with(
        "demo",
        "apiVersion: v2\nname: demo\nversion: 1.0.0\ndescription: Local\n",
    );
    put(
        &fixture,
        "/repository/local-charts/demo-1.0.0.tgz",
        local.clone(),
        Auth::Admin,
    )
    .await;
    put(
        &fixture,
        "/repository/local-charts/tool-0.1.0.tgz",
        package("tool", "0.1.0"),
        Auth::Admin,
    )
    .await;

    let index = index_of(&get(&fixture, "/repository/all-charts/index.yaml").await);
    assert_eq!(versions(&index, "demo"), ["2.0.0", "1.0.0"]);
    assert_eq!(versions(&index, "tool"), ["0.1.0"]);
    let one = &index["entries"]["demo"][1];
    assert_eq!(one["description"], "Local");
    assert_eq!(one["digest"], sha256(&local));
    assert_eq!(one["urls"], json!(["demo-1.0.0.tgz"]));
    assert_eq!(
        index["entries"]["demo"][0]["urls"],
        json!(["demo-2.0.0.tgz"])
    );

    let from_local = get(&fixture, "/repository/all-charts/demo-1.0.0.tgz").await;
    assert_eq!(from_local.body, local);
    let from_remote = get(&fixture, "/repository/all-charts/demo-2.0.0.tgz").await;
    assert_eq!(from_remote.status, StatusCode::OK);
    assert_eq!(from_remote.body, remote.elsewhere);
    assert_eq!(remote.charts.count("/stable/charts/demo-1.0.0.tgz"), 0);

    let refused = put(
        &fixture,
        "/repository/all-charts/demo-5.0.0.tgz",
        package("demo", "5.0.0"),
        Auth::Admin,
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(refused.headers[header::ALLOW], "GET,HEAD");
    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/all-charts/demo-1.0.0.tgz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::METHOD_NOT_ALLOWED);
}

fn base64_credentials(user: &str, password: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"))
}
