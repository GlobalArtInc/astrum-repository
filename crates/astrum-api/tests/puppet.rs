mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use base64::Engine;
use md5::Md5;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "puppetboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_puppet::Puppet::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/puppet/{kind}"),
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

fn tar_gz(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in files {
        let mut header = tar::Header::new_ustar();
        if path.ends_with('/') {
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_mode(0o755);
        } else {
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
        }
        header.set_mtime(1_790_000_000);
        header.set_cksum();
        builder
            .append_data(&mut header, path, data.as_slice())
            .unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

fn metadata(name: &str, version: &str, dependencies: Value) -> Value {
    json!({
        "name": name,
        "version": version,
        "author": "Jane Doe",
        "summary": format!("The {name} test module"),
        "license": "Apache-2.0",
        "source": "https://example.org/repo",
        "project_page": "https://example.org/project",
        "issues_url": "https://example.org/issues",
        "dependencies": dependencies,
        "operatingsystem_support": [{"operatingsystem": "Debian", "operatingsystemrelease": ["12"]}],
        "requirements": [{"name": "puppet", "version_requirement": ">= 7.0.0 < 9.0.0"}],
        "tags": ["demo", "tools"],
        "pdk-version": "3.4.0"
    })
}

fn built(top: &str, metadata: &Value) -> Vec<u8> {
    tar_gz(&[
        (format!("{top}/"), Vec::new()),
        (
            format!("{top}/metadata.json"),
            serde_json::to_vec_pretty(metadata).unwrap(),
        ),
        (format!("{top}/README.md"), b"# Test module\n".to_vec()),
        (format!("{top}/manifests/"), Vec::new()),
        (
            format!("{top}/manifests/init.pp"),
            b"class tools {}\n".to_vec(),
        ),
        (
            format!("{top}/tasks/hello.json"),
            b"{\"description\": \"Says hello\"}".to_vec(),
        ),
    ])
}

fn module(name: &str, version: &str, dependencies: Value) -> Vec<u8> {
    built(
        &format!("{}-{version}", name.replace('/', "-")),
        &metadata(name, version, dependencies),
    )
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn md5(data: &[u8]) -> String {
    hex::encode(Md5::digest(data))
}

fn form(file_field: &str, file_name: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{file_field}\"; filename=\"{file_name}\"\r\nContent-Type: application/gzip\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(data);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn publish(
    fixture: &Fixture,
    repository: &str,
    authorization: Option<&str>,
    data: &[u8],
) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let mut headers = vec![("content-type", content_type.as_str())];
    if let Some(authorization) = authorization {
        headers.push(("authorization", authorization));
    }
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/v3/releases"),
            Auth::None,
            Body::from(form("file", "pkg.tar.gz", data)),
            &headers,
        )
        .await
}

async fn publish_json(
    fixture: &Fixture,
    repository: &str,
    authorization: &str,
    data: &[u8],
) -> Reply {
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    let wrapped: String = encoded
        .as_bytes()
        .chunks(60)
        .map(|line| format!("{}\n", std::str::from_utf8(line).unwrap()))
        .collect();
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/v3/releases"),
            Auth::None,
            Body::from(json!({"file": wrapped}).to_string()),
            &[
                ("content-type", "application/json"),
                ("authorization", authorization),
            ],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn versions(document: &Value) -> Vec<String> {
    document
        .as_array()
        .unwrap()
        .iter()
        .map(|release| release["version"].as_str().unwrap().to_owned())
        .collect()
}

fn basic(user: &str, password: &str) -> String {
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"))
    )
}

#[tokio::test]
async fn hosted_modules_publish_and_answer_the_forge_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("forge", "ALLOW_ONCE")).await;
    let (_, token) = fixture
        .state
        .security()
        .create_token("admin", "forge", Scope::Write, None)
        .await
        .unwrap();
    let bearer = format!("Bearer {token}");

    let page = fixture
        .call(Method::GET, "/repository/forge/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.text()
            .contains("puppet module install --module_repository")
    );

    let first = module("acme-tools", "1.0.0", json!([]));
    let published = publish(&fixture, "forge", Some(&bearer), &first).await;
    assert_eq!(
        published.status,
        StatusCode::CREATED,
        "{}",
        published.text()
    );
    assert_eq!(
        published.json(),
        json!({
            "uri": "/v3/releases/acme-tools-1.0.0",
            "slug": "acme-tools-1.0.0",
            "file_uri": "/v3/files/acme-tools-1.0.0.tar.gz",
        })
    );

    let again = publish(&fixture, "forge", Some(&bearer), &first).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json()["message"], "409 Conflict");
    assert!(
        again.json()["errors"][0]
            .as_str()
            .unwrap()
            .contains("already exists")
    );
    let anonymous = publish(&fixture, "forge", None, &first).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["message"], "401 Unauthorized");
    let unknown = publish(
        &fixture,
        "forge",
        Some("Bearer astrum_0000000000000000000000000000000000000000"),
        &first,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    let misnamed = publish(
        &fixture,
        "forge",
        Some(&bearer),
        &built("tools-1.0.1", &metadata("acme-tools", "1.0.1", json!([]))),
    )
    .await;
    assert_eq!(misnamed.status, StatusCode::BAD_REQUEST);
    assert!(
        misnamed.json()["errors"][0]
            .as_str()
            .unwrap()
            .contains("top directory tools-1.0.1/ does not match"),
        "{}",
        misnamed.text()
    );
    let bad_version = publish(
        &fixture,
        "forge",
        Some(&bearer),
        &module("acme-tools", "1.0", json!([])),
    )
    .await;
    assert_eq!(bad_version.status, StatusCode::BAD_REQUEST);
    assert!(bad_version.text().contains("not a semantic version"));
    let not_a_tarball = publish(&fixture, "forge", Some(&bearer), b"not a tarball").await;
    assert_eq!(not_a_tarball.status, StatusCode::BAD_REQUEST);
    let no_file = fixture
        .send_body(
            Method::POST,
            "/repository/forge/v3/releases",
            Auth::None,
            Body::from(form("other", "x.tar.gz", &first)),
            &[
                (
                    "content-type",
                    &format!("multipart/form-data; boundary={BOUNDARY}"),
                ),
                ("authorization", &bearer),
            ],
        )
        .await;
    assert_eq!(no_file.status, StatusCode::BAD_REQUEST);
    assert!(no_file.text().contains("no file part"));

    let second = module(
        "acme/tools",
        "1.1.0",
        json!([{"name": "puppetlabs/stdlib", "version_requirement": ">= 9.0.0 < 10.0.0"}]),
    );
    let reply = publish_json(&fixture, "forge", &bearer, &second).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let candidate = module("acme-tools", "2.0.0-rc.1", json!([]));
    let reply = publish(
        &fixture,
        "forge",
        Some(&basic("admin", &fixture.admin_password)),
        &candidate,
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    let found = get(&fixture, "/repository/forge/v3/modules/acme-tools").await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.text());
    let found = found.json();
    assert_eq!(found["slug"], "acme-tools");
    assert_eq!(found["owner"]["slug"], "acme");
    assert_eq!(found["current_release"]["version"], "1.1.0");
    assert_eq!(
        found["current_release"]["file_uri"],
        "/v3/files/acme-tools-1.1.0.tar.gz"
    );
    assert_eq!(
        found["current_release"]["metadata"]["dependencies"][0]["name"],
        "puppetlabs/stdlib"
    );
    assert_eq!(
        versions(&found["releases"]),
        ["2.0.0-rc.1", "1.1.0", "1.0.0"]
    );
    assert_eq!(found["homepage_url"], "https://example.org/project");
    assert!(found["created_at"].as_str().unwrap().ends_with(" +0000"));
    let same = get(&fixture, "/repository/forge/v3/modules/ACME/tools").await;
    assert_eq!(same.json()["slug"], "acme-tools");

    let listed = get(
        &fixture,
        "/repository/forge/v3/releases?module=acme-tools&limit=2&exclude_fields=readme%20changelog%20license%20uri%20module",
    )
    .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let listed = listed.json();
    assert_eq!(listed["pagination"]["total"], 3);
    assert_eq!(versions(&listed["results"]), ["2.0.0-rc.1", "1.1.0"]);
    assert!(listed["results"][0].get("module").is_none());
    assert!(listed["results"][0].get("uri").is_none());
    assert_eq!(listed["results"][1]["file_md5"], md5(&second));
    let next = listed["pagination"]["next"].as_str().unwrap();
    assert_eq!(
        next,
        "/v3/releases?module=acme-tools&exclude_fields=readme%20changelog%20license%20uri%20module&limit=2&offset=2"
    );
    let rest = get(&fixture, &format!("/repository/forge{next}"))
        .await
        .json();
    assert_eq!(versions(&rest["results"]), ["1.0.0"]);
    assert_eq!(rest["pagination"]["next"], Value::Null);
    assert_eq!(
        rest["pagination"]["previous"],
        "/v3/releases?module=acme-tools&exclude_fields=readme%20changelog%20license%20uri%20module&limit=2&offset=0"
    );

    let release = get(&fixture, "/repository/forge/v3/releases/acme-tools-1.1.0").await;
    assert_eq!(release.status, StatusCode::OK, "{}", release.text());
    let release = release.json();
    assert_eq!(release["slug"], "acme-tools-1.1.0");
    assert_eq!(release["module"]["slug"], "acme-tools");
    assert_eq!(release["metadata"]["name"], "acme/tools");
    assert_eq!(release["file_sha256"], sha256(&second));
    assert_eq!(release["file_md5"], md5(&second));
    assert_eq!(release["file_size"], second.len());
    assert_eq!(release["tags"], json!(["demo", "tools"]));
    assert_eq!(release["pdk"], true);

    let download = get(
        &fixture,
        &format!("/repository/forge{}", release["file_uri"].as_str().unwrap()),
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, second);
    assert_eq!(md5(&download.body), release["file_md5"]);
    assert_eq!(download.headers[header::CONTENT_DISPOSITION], "attachment");
    let direct = get(
        &fixture,
        "/repository/forge/acme/tools/1.1.0/acme-tools-1.1.0.tar.gz",
    )
    .await;
    assert_eq!(direct.body, second);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/forge/v3/modules/acme-tools",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());

    let missing = get(&fixture, "/repository/forge/v3/releases/acme-tools-9.0.0").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.json()["message"], "404 Not Found");
    assert_eq!(
        get(&fixture, "/repository/forge/v3/modules/acme-missing")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/forge/v3/files/acme-tools-9.0.0.tar.gz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let none = get(
        &fixture,
        "/repository/forge/v3/releases?module=acme-missing",
    )
    .await
    .json();
    assert_eq!(none["pagination"]["total"], 0);
    assert_eq!(none["results"], json!([]));
    let invalid = get(&fixture, "/repository/forge/v3/releases?module=missing").await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);

    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=forge",
            Auth::Admin,
            Body::from(form(
                "puppet.asset",
                "upload.tar.gz",
                &module("acme-other", "0.1.0", json!([])),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());
    let refused = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=forge",
            Auth::Admin,
            Body::from(form("puppet.asset", "x.tar.gz", b"nope")),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let search = get(&fixture, "/repository/forge/v3/modules?query=other")
        .await
        .json();
    assert_eq!(search["pagination"]["total"], 1, "{search}");
    assert_eq!(search["results"][0]["slug"], "acme-other");
    let tagged = get(&fixture, "/repository/forge/v3/modules?tag=demo&limit=1")
        .await
        .json();
    assert_eq!(tagged["pagination"]["total"], 2);
    assert_eq!(tagged["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        tagged["pagination"]["next"],
        "/v3/modules?tag=demo&limit=1&offset=1"
    );
    let by_owner = get(
        &fixture,
        "/repository/forge/v3/releases?owner=acme&sort_by=release_date",
    )
    .await
    .json();
    assert_eq!(by_owner["pagination"]["total"], 4);
    assert_eq!(by_owner["results"][0]["slug"], "acme-other-0.1.0");

    let components = get(
        &fixture,
        "/service/rest/v1/search?format=puppet&puppet.tag=demo&name=other",
    )
    .await
    .json();
    assert_eq!(
        components["items"].as_array().unwrap().len(),
        1,
        "{components}"
    );
    assert_eq!(components["items"][0]["group"], "acme");
    let by_keyword = get(&fixture, "/service/rest/v1/search?format=puppet&q=module").await;
    assert!(!by_keyword.json()["items"].as_array().unwrap().is_empty());

    let delete = fixture
        .call(
            Method::DELETE,
            "/repository/forge/v3/releases/acme-tools-1.0.0",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(delete.status, StatusCode::NO_CONTENT, "{}", delete.text());
    let after = get(&fixture, "/repository/forge/v3/releases?module=acme-tools")
        .await
        .json();
    assert_eq!(after["pagination"]["total"], 2);
    assert_eq!(versions(&after["results"]), ["2.0.0-rc.1", "1.1.0"]);
    let module_after = get(&fixture, "/repository/forge/v3/modules/acme-tools")
        .await
        .json();
    assert_eq!(versions(&module_after["releases"]), ["2.0.0-rc.1", "1.1.0"]);
    assert_eq!(
        get(
            &fixture,
            "/repository/forge/v3/files/acme-tools-1.0.0.tar.gz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/forge/v3/modules/acme-other",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&fixture, "/repository/forge/v3/modules/acme-other")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let gone = get(&fixture, "/repository/forge/v3/modules?query=other")
        .await
        .json();
    assert_eq!(gone["pagination"]["total"], 0);
}

#[tokio::test]
async fn write_policies_permissions_and_other_repository_kinds() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("open", "ALLOW")).await;
    create(&fixture, "hosted", hosted("closed", "DENY")).await;
    create(&fixture, "group", group("all", &["open"])).await;
    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let admin = basic("admin", &fixture.admin_password);
    let data = module("acme-tools", "1.0.0", json!([]));
    for _ in 0..2 {
        let reply = publish(&fixture, "open", Some(&admin), &data).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    let closed = publish(&fixture, "closed", Some(&admin), &data).await;
    assert_eq!(closed.status, StatusCode::BAD_REQUEST);
    assert!(closed.text().contains("read-only"));
    let reader = basic("reader", "reader-password-1");
    let refused = publish(
        &fixture,
        "open",
        Some(&reader),
        &module("acme-tools", "1.0.1", json!([])),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.json()["message"], "403 Forbidden");
    let read = fixture
        .send(
            Method::GET,
            "/repository/open/v3/modules/acme-tools",
            Auth::None,
            None,
            &[("authorization", &reader)],
        )
        .await;
    assert_eq!(read.status, StatusCode::OK);
    let deleting = fixture
        .send(
            Method::DELETE,
            "/repository/open/v3/releases/acme-tools-1.0.0",
            Auth::None,
            None,
            &[("authorization", &reader)],
        )
        .await;
    assert_eq!(deleting.status, StatusCode::FORBIDDEN);

    let anonymous = fixture
        .call(
            Method::GET,
            "/repository/open/v3/modules/acme-tools",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["message"], "401 Unauthorized");
    let anonymous_file = fixture
        .call(
            Method::GET,
            "/repository/open/v3/files/acme-tools-1.0.0.tar.gz",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous_file.status, StatusCode::UNAUTHORIZED);
    let (_, token) = fixture
        .state
        .security()
        .create_token("admin", "puppet", Scope::Read, None)
        .await
        .unwrap();
    for authorization in [
        token.clone(),
        format!("Bearer {token}"),
        basic("admin", &token),
    ] {
        let read = fixture
            .send(
                Method::GET,
                "/repository/open/v3/releases?module=acme-tools",
                Auth::None,
                None,
                &[("authorization", &authorization)],
            )
            .await;
        assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    }
    let read_only = publish(
        &fixture,
        "open",
        Some(&format!("Bearer {token}")),
        &module("acme-tools", "1.0.3", json!([])),
    )
    .await;
    assert_eq!(read_only.status, StatusCode::FORBIDDEN);
    let opened = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(opened.status, StatusCode::OK, "{}", opened.text());
    let anonymous = fixture
        .call(
            Method::GET,
            "/repository/open/v3/releases?module=acme-tools",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::OK, "{}", anonymous.text());
    assert_eq!(anonymous.json()["pagination"]["total"], 1);
    let anonymous_publish = publish(
        &fixture,
        "open",
        None,
        &module("acme-tools", "1.0.2", json!([])),
    )
    .await;
    assert_eq!(anonymous_publish.status, StatusCode::UNAUTHORIZED);

    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "deployer-role", "name": "deployer-role", "description": "", "privileges": ["as-repository-view-puppet-open-add"], "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user("deployer", "deployer-password-1", &["deployer-role"])
        .await;
    let deployer = basic("deployer", "deployer-password-1");
    let deployed = publish(
        &fixture,
        "open",
        Some(&deployer),
        &module("acme-tools", "3.0.0", json!([])),
    )
    .await;
    assert_eq!(deployed.status, StatusCode::CREATED, "{}", deployed.text());
    let overwrite = publish(&fixture, "open", Some(&deployer), &data).await;
    assert_eq!(overwrite.status, StatusCode::FORBIDDEN);

    let to_group = publish(&fixture, "all", Some(&admin), &data).await;
    assert_eq!(to_group.status, StatusCode::BAD_REQUEST);
    let through_group = get(&fixture, "/repository/all/v3/releases/acme-tools-1.0.0")
        .await
        .json();
    assert_eq!(
        through_group["file_uri"],
        "/v3/files/acme-tools-1.0.0.tar.gz"
    );
    let download = get(&fixture, "/repository/all/v3/files/acme-tools-1.0.0.tar.gz").await;
    assert_eq!(download.body, data);
    let group_delete = fixture
        .call(
            Method::DELETE,
            "/repository/all/v3/releases/acme-tools-1.0.0",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(group_delete.status, StatusCode::METHOD_NOT_ALLOWED);
    let put = fixture
        .call(
            Method::PUT,
            "/repository/open/v3/releases",
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(put.headers.contains_key(header::ALLOW));
}

#[derive(Clone, Default)]
struct Upstream {
    address: Arc<Mutex<Option<SocketAddr>>>,
    archive: Arc<Vec<u8>>,
    release_lists: Arc<AtomicUsize>,
    files: Arc<AtomicUsize>,
}

fn upstream_release(address: SocketAddr, version: &str, archive: &[u8], sha: &str) -> Value {
    let file_uri = match version {
        "9.1.0" => format!("http://{address}/cdn/puppetlabs-stdlib-{version}.tar.gz"),
        _ => format!("/v3/files/puppetlabs-stdlib-{version}.tar.gz"),
    };
    json!({
        "uri": format!("/v3/releases/puppetlabs-stdlib-{version}"),
        "slug": format!("puppetlabs-stdlib-{version}"),
        "module": {
            "uri": "/v3/modules/puppetlabs-stdlib",
            "slug": "puppetlabs-stdlib",
            "name": "stdlib",
            "deprecated_at": null,
            "owner": {"uri": "/v3/users/puppetlabs", "slug": "puppetlabs", "username": "puppetlabs", "gravatar_id": "x"}
        },
        "version": version,
        "metadata": metadata("puppetlabs-stdlib", version, json!([])),
        "tags": ["stdlib"],
        "supported": false,
        "pdk": true,
        "validation_score": 100,
        "file_uri": file_uri,
        "file_size": archive.len(),
        "file_md5": md5(archive),
        "file_sha256": sha,
        "downloads": 10,
        "readme": "# stdlib",
        "created_at": format!("2026-0{}-01 00:00:00 -0700", if version == "9.1.0" { 2 } else { 1 }),
        "updated_at": "2026-02-01 00:00:00 -0700",
        "deleted_at": null,
        "deleted_for": null
    })
}

async fn upstream(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let address = upstream.address.lock().unwrap().unwrap();
    let query = query.unwrap_or_default();
    let archive = upstream.archive.as_ref();
    let good = upstream_release(address, "9.0.0", archive, &sha256(archive));
    let tampered = upstream_release(address, "9.1.0", archive, &"0".repeat(64));
    let json_reply = |value: Value| {
        (
            [(header::CONTENT_TYPE, "application/json")],
            value.to_string(),
        )
            .into_response()
    };
    let stdlib = json!({
        "uri": "/v3/modules/puppetlabs-stdlib",
        "slug": "puppetlabs-stdlib",
        "name": "stdlib",
        "downloads": 20,
        "created_at": "2010-01-01 00:00:00 -0700",
        "updated_at": "2026-02-01 00:00:00 -0700",
        "deprecated_at": null,
        "owner": {"uri": "/v3/users/puppetlabs", "slug": "puppetlabs", "username": "puppetlabs", "gravatar_id": "x"},
        "current_release": tampered,
        "releases": [
            {"uri": "/v3/releases/puppetlabs-stdlib-9.0.0", "slug": "puppetlabs-stdlib-9.0.0", "version": "9.0.0", "file_uri": good["file_uri"], "file_size": archive.len()},
            {"uri": "/v3/releases/puppetlabs-stdlib-9.1.0", "slug": "puppetlabs-stdlib-9.1.0", "version": "9.1.0", "file_uri": tampered["file_uri"], "file_size": archive.len()}
        ],
        "homepage_url": "https://example.org/stdlib",
    });
    match path.as_str() {
        "v3/modules/puppetlabs-stdlib" => json_reply(stdlib),
        "v3/releases" if query.contains("module=puppetlabs-stdlib") => {
            upstream.release_lists.fetch_add(1, Ordering::SeqCst);
            assert!(query.contains("exclude_fields=readme"), "{query}");
            let (results, next) = match query.contains("offset=1") {
                false => (
                    vec![tampered],
                    json!("/v3/releases?module=puppetlabs-stdlib&exclude_fields=readme+changelog&limit=1&offset=1"),
                ),
                true => (vec![good], Value::Null),
            };
            json_reply(json!({
                "pagination": {"limit": 1, "offset": 0, "first": "/v3/releases?module=puppetlabs-stdlib&limit=1&offset=0", "previous": null, "current": "x", "next": next, "total": 2},
                "results": results,
            }))
        }
        "v3/releases" if query.contains("module=") => json_reply(json!({
            "pagination": {"limit": 20, "offset": 0, "first": "/v3/releases?limit=20&offset=0", "previous": null, "current": "/v3/releases?limit=20&offset=0", "next": null, "total": 0},
            "results": [],
        })),
        "v3/modules" if query.contains("query=std") => {
            let second = query.contains("offset=1");
            let concat = json!({
                "uri": "/v3/modules/puppetlabs-concat",
                "slug": "puppetlabs-concat",
                "name": "concat",
                "downloads": 5,
                "owner": {"uri": "/v3/users/puppetlabs", "slug": "puppetlabs", "username": "puppetlabs"},
                "current_release": null,
                "releases": [],
            });
            let (results, next) = match second {
                false => (
                    vec![stdlib],
                    json!("https://forge.example/v3/modules?query=std&limit=1&offset=1"),
                ),
                true => (vec![concat], Value::Null),
            };
            json_reply(json!({
                "pagination": {"limit": 1, "offset": if second { 1 } else { 0 }, "first": "https://forge.example/v3/modules?query=std&limit=1&offset=0", "previous": null, "current": "/v3/modules?query=std&limit=1&offset=0", "next": next, "total": 2},
                "results": results,
            }))
        }
        "v3/releases/puppetlabs-stdlib-9.0.0" => json_reply(good),
        "v3/releases/puppetlabs-stdlib-9.1.0" => json_reply(tampered),
        "v3/files/puppetlabs-stdlib-9.0.0.tar.gz" | "cdn/puppetlabs-stdlib-9.1.0.tar.gz" => {
            upstream.files.fetch_add(1, Ordering::SeqCst);
            archive.clone().into_response()
        }
        "v3/releases" | "v3/modules" => (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "application/json")],
            json!({"message": "400 Bad Request", "errors": ["unsupported filter"]}).to_string(),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            json!({"message": "404 Not Found", "errors": ["The requested resource cannot be found"]}).to_string(),
        )
            .into_response(),
    }
}

#[tokio::test]
async fn proxies_rewrite_remote_documents_and_groups_merge_members() {
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
    let state = Upstream {
        archive: Arc::new(module("puppetlabs-stdlib", "9.0.0", json!([]))),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    *state.address.lock().unwrap() = Some(address);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "forge-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": false},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(
        &fixture,
        "group",
        group("forge-all", &["own", "forge-proxy"]),
    )
    .await;

    let found = get(
        &fixture,
        "/repository/forge-proxy/v3/modules/puppetlabs-stdlib",
    )
    .await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.text());
    let found = found.json();
    assert_eq!(found["current_release"]["version"], "9.1.0");
    assert_eq!(
        found["current_release"]["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.1.0.tar.gz"
    );
    assert_eq!(versions(&found["releases"]), ["9.1.0", "9.0.0"]);
    assert_eq!(
        found["releases"][0]["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.1.0.tar.gz"
    );
    assert_eq!(found["downloads"], 20);

    let listed = get(
        &fixture,
        "/repository/forge-proxy/v3/releases?module=puppetlabs-stdlib&sort_by=version",
    )
    .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let listed = listed.json();
    assert_eq!(listed["pagination"]["total"], 2, "{listed}");
    assert_eq!(versions(&listed["results"]), ["9.1.0", "9.0.0"]);
    assert_eq!(
        listed["results"][0]["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.1.0.tar.gz"
    );
    assert_eq!(
        listed["pagination"]["current"],
        "/v3/releases?module=puppetlabs-stdlib&sort_by=version&limit=20&offset=0"
    );
    get(
        &fixture,
        "/repository/forge-proxy/v3/releases?module=puppetlabs/stdlib",
    )
    .await;
    assert_eq!(state.release_lists.load(Ordering::SeqCst), 2);

    let release = get(
        &fixture,
        "/repository/forge-proxy/v3/releases/puppetlabs-stdlib-9.0.0",
    )
    .await
    .json();
    assert_eq!(
        release["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.0.0.tar.gz"
    );
    assert_eq!(release["readme"], "# stdlib");
    assert_eq!(release["validation_score"], 100);
    let download = get(
        &fixture,
        "/repository/forge-proxy/v3/files/puppetlabs-stdlib-9.0.0.tar.gz",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK, "{}", download.text());
    assert_eq!(download.body, *state.archive);
    assert_eq!(md5(&download.body), release["file_md5"]);
    get(
        &fixture,
        "/repository/forge-proxy/v3/files/puppetlabs-stdlib-9.0.0.tar.gz",
    )
    .await;
    assert_eq!(state.files.load(Ordering::SeqCst), 1);
    let tampered = get(
        &fixture,
        "/repository/forge-proxy/v3/files/puppetlabs-stdlib-9.1.0.tar.gz",
    )
    .await;
    assert_eq!(
        tampered.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        tampered.text()
    );
    assert_eq!(state.files.load(Ordering::SeqCst), 2);
    assert_eq!(
        get(&fixture, "/repository/forge-proxy/v3/modules/acme-none")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let components = get(
        &fixture,
        "/service/rest/v1/search?repository=forge-proxy&format=puppet",
    )
    .await
    .json();
    assert_eq!(
        components["items"].as_array().unwrap().len(),
        1,
        "{components}"
    );
    assert_eq!(components["items"][0]["group"], "puppetlabs");
    assert_eq!(components["items"][0]["version"], "9.0.0");

    let searched = get(
        &fixture,
        "/repository/forge-proxy/v3/modules?query=std&limit=1",
    )
    .await;
    assert_eq!(searched.status, StatusCode::OK, "{}", searched.text());
    let searched = searched.json();
    assert_eq!(searched["results"][0]["slug"], "puppetlabs-stdlib");
    assert_eq!(
        searched["results"][0]["current_release"]["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.1.0.tar.gz"
    );
    assert_eq!(
        searched["pagination"]["next"],
        "/v3/modules?query=std&limit=1&offset=1"
    );
    assert_eq!(
        searched["pagination"]["first"],
        "/v3/modules?query=std&limit=1&offset=0"
    );
    let unsupported = get(&fixture, "/repository/forge-proxy/v3/modules?owner=x").await;
    assert_eq!(unsupported.status, StatusCode::BAD_REQUEST);
    assert_eq!(unsupported.json()["errors"][0], "unsupported filter");

    let own = module("puppetlabs-stdlib", "9.2.0", json!([]));
    let reply = publish(
        &fixture,
        "own",
        Some(&basic("admin", &fixture.admin_password)),
        &own,
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    let merged = get(
        &fixture,
        "/repository/forge-all/v3/releases?module=puppetlabs-stdlib",
    )
    .await
    .json();
    assert_eq!(versions(&merged["results"]), ["9.2.0", "9.1.0", "9.0.0"]);
    assert_eq!(merged["pagination"]["total"], 3);
    for release in merged["results"].as_array().unwrap() {
        assert!(
            release["file_uri"]
                .as_str()
                .unwrap()
                .starts_with("/v3/files/puppetlabs-stdlib-"),
            "{release}"
        );
    }
    let module_doc = get(
        &fixture,
        "/repository/forge-all/v3/modules/puppetlabs-stdlib",
    )
    .await
    .json();
    assert_eq!(module_doc["current_release"]["version"], "9.2.0");
    assert_eq!(
        versions(&module_doc["releases"]),
        ["9.2.0", "9.1.0", "9.0.0"]
    );
    assert_eq!(module_doc["downloads"], 20);
    let through = get(
        &fixture,
        "/repository/forge-all/v3/releases/puppetlabs-stdlib-9.0.0",
    )
    .await
    .json();
    assert_eq!(
        through["file_uri"],
        "/v3/files/puppetlabs-stdlib-9.0.0.tar.gz"
    );
    let from_proxy = get(
        &fixture,
        "/repository/forge-all/v3/files/puppetlabs-stdlib-9.0.0.tar.gz",
    )
    .await;
    assert_eq!(from_proxy.body, *state.archive);
    let from_hosted = get(
        &fixture,
        "/repository/forge-all/v3/files/puppetlabs-stdlib-9.2.0.tar.gz",
    )
    .await;
    assert_eq!(from_hosted.body, own);

    let searched = get(&fixture, "/repository/forge-all/v3/modules?query=std").await;
    assert_eq!(searched.status, StatusCode::OK, "{}", searched.text());
    let searched = searched.json();
    let slugs: Vec<&str> = searched["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|module| module["slug"].as_str().unwrap())
        .collect();
    assert_eq!(
        slugs,
        ["puppetlabs-stdlib", "puppetlabs-concat"],
        "{searched}"
    );
    assert_eq!(searched["pagination"]["total"], 2);
    assert_eq!(
        searched["results"][0]["current_release"]["version"],
        "9.2.0"
    );
    let by_owner = get(
        &fixture,
        "/repository/forge-all/v3/releases?owner=puppetlabs",
    )
    .await;
    assert_eq!(by_owner.status, StatusCode::OK, "{}", by_owner.text());
    assert_eq!(versions(&by_owner.json()["results"]), ["9.2.0"]);
    assert_eq!(
        get(&fixture, "/repository/forge-all/v3/modules/acme-none")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}
