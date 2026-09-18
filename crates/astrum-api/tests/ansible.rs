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
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "ansibleboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_ansible::Ansible::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/ansible/{kind}"),
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

fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, path, *data).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

fn collection(namespace: &str, name: &str, version: &str, dependencies: Value) -> Vec<u8> {
    let manifest = json!({
        "collection_info": {
            "namespace": namespace,
            "name": name,
            "version": version,
            "authors": ["Jane Doe"],
            "readme": "README.md",
            "tags": ["demo", "tools"],
            "description": format!("The {name} collection"),
            "license": ["MIT"],
            "license_file": null,
            "dependencies": dependencies,
            "repository": "https://example.org/repo",
            "documentation": null,
            "homepage": null,
            "issues": null
        },
        "file_manifest_file": {"name": "FILES.json", "ftype": "file", "chksum_type": "sha256", "chksum_sha256": "0", "format": 1},
        "format": 1
    })
    .to_string();
    tar_gz(&[
        ("MANIFEST.json", manifest.as_bytes()),
        ("FILES.json", b"{\"files\": [], \"format\": 1}"),
        ("plugins/modules/hello.py", b"print('hello')\n"),
        ("meta/runtime.yml", b"requires_ansible: '>=2.15.0'\n"),
    ])
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn form(sha: Option<&str>, file_field: &str, file_name: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some(sha) = sha {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"sha256\"\r\n\r\n{sha}\r\n"
        )
        .unwrap();
    }
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{file_field}\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(data);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

/// The form `ansible-galaxy collection publish` sends: Python's MIME classes write the archive
/// base64-encoded in lines of 76 characters.
fn galaxy_form(sha: Option<&str>, file_name: &str, data: &[u8]) -> Vec<u8> {
    use base64::Engine;
    let mut body = Vec::new();
    if let Some(sha) = sha {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Type: text/plain\r\nContent-Disposition: form-data; name=\"sha256\"\r\n\r\n{sha}\r\n"
        )
        .unwrap();
    }
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Type: application/octet-stream\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\r\n"
    )
    .unwrap();
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    for line in encoded.as_bytes().chunks(76) {
        body.extend_from_slice(line);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn publish_plain(
    fixture: &Fixture,
    repository: &str,
    authorization: &str,
    file_name: &str,
    data: &[u8],
) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/v3/artifacts/collections/"),
            Auth::None,
            Body::from(form(Some(&sha256(data)), "file", file_name, data)),
            &[
                ("content-type", content_type.as_str()),
                ("authorization", authorization),
            ],
        )
        .await
}

async fn publish(
    fixture: &Fixture,
    repository: &str,
    authorization: Option<&str>,
    file_name: &str,
    data: &[u8],
    sha: Option<&str>,
) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let mut headers = vec![("content-type", content_type.as_str())];
    if let Some(authorization) = authorization {
        headers.push(("authorization", authorization));
    }
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/v3/artifacts/collections/"),
            Auth::None,
            Body::from(galaxy_form(sha, file_name, data)),
            &headers,
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn task(fixture: &Fixture, reply: &Reply) -> Value {
    assert_eq!(reply.status, StatusCode::ACCEPTED, "{}", reply.text());
    let task = reply.json()["task"].as_str().unwrap().to_owned();
    assert!(
        task.starts_with("/repository/") && task.contains("/api/v3/imports/collections/"),
        "{task}"
    );
    let found = get(fixture, &task).await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.text());
    found.json()
}

#[tokio::test]
async fn hosted_collections_publish_and_answer_the_v3_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("galaxy", "ALLOW_ONCE")).await;
    let (_, token) = fixture
        .state
        .security()
        .create_token("admin", "galaxy", Scope::Write, None)
        .await
        .unwrap();
    let token_header = format!("Token {token}");

    let root = get(&fixture, "/repository/galaxy/api/").await;
    assert_eq!(root.status, StatusCode::OK);
    assert_eq!(root.json()["available_versions"]["v3"], "v3/");
    let page = fixture
        .call(Method::GET, "/repository/galaxy/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("ansible-galaxy collection install"));

    let first = collection("acme", "tools", "1.0.0", json!({}));
    let published = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.0.0.tar.gz",
        &first,
        Some(&sha256(&first)),
    )
    .await;
    let done = task(&fixture, &published).await;
    assert_eq!(done["state"], "completed", "{done}");
    assert!(done["finished_at"].is_string());
    assert_eq!(done["version"], "1.0.0");
    assert!(
        done["messages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message["level"] == "INFO")
    );

    let again = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.0.0.tar.gz",
        &first,
        None,
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json()["errors"][0]["code"], "conflict");
    let anonymous = publish(
        &fixture,
        "galaxy",
        None,
        "acme-tools-1.0.1.tar.gz",
        &first,
        None,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["errors"][0]["code"], "not_authenticated");
    let unknown = publish(
        &fixture,
        "galaxy",
        Some("Token astrum_0000000000000000000000000000000000000000"),
        "acme-tools-1.0.1.tar.gz",
        &first,
        None,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    let wrong_sum = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.0.0.tar.gz",
        &collection("acme", "tools", "1.0.0", json!({})),
        Some(&"0".repeat(64)),
    )
    .await;
    assert_eq!(wrong_sum.status, StatusCode::CONFLICT);
    let wrong_sum = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.0.5.tar.gz",
        &collection("acme", "tools", "1.0.5", json!({})),
        Some(&"0".repeat(64)),
    )
    .await;
    assert_eq!(
        wrong_sum.status,
        StatusCode::BAD_REQUEST,
        "{}",
        wrong_sum.text()
    );
    let bad_name = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "tools.tar.gz",
        &first,
        None,
    )
    .await;
    assert_eq!(bad_name.status, StatusCode::BAD_REQUEST);
    assert!(
        bad_name.json()["errors"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("<namespace>-<name>-<version>.tar.gz")
    );
    let mismatch = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.1.0.tar.gz",
        &collection("acme", "tools", "1.2.0", json!({})),
        None,
    )
    .await;
    let failed = task(&fixture, &mismatch).await;
    assert_eq!(failed["state"], "failed");
    assert_eq!(failed["error"]["code"], "invalid");
    assert!(
        failed["error"]["description"]
            .as_str()
            .unwrap()
            .contains("does not match MANIFEST.json"),
        "{failed}"
    );
    let not_a_tarball = publish(
        &fixture,
        "galaxy",
        Some(&token_header),
        "acme-tools-1.3.0.tar.gz",
        b"not a tarball",
        None,
    )
    .await;
    assert_eq!(task(&fixture, &not_a_tarball).await["state"], "failed");

    let second = collection(
        "acme",
        "tools",
        "1.1.0",
        json!({"community.general": ">=8.0.0"}),
    );
    let bearer = format!("Bearer {token}");
    for (version, data) in [
        ("1.1.0", second.clone()),
        (
            "2.0.0-beta.1",
            collection("acme", "tools", "2.0.0-beta.1", json!({})),
        ),
    ] {
        let reply = publish(
            &fixture,
            "galaxy",
            Some(&bearer),
            &format!("/tmp/build/acme-tools-{version}.tar.gz"),
            &data,
            Some(&sha256(&data)),
        )
        .await;
        assert_eq!(task(&fixture, &reply).await["state"], "completed");
    }

    let summary = get(
        &fixture,
        "/repository/galaxy/api/v3/collections/acme/tools/",
    )
    .await;
    assert_eq!(summary.status, StatusCode::OK, "{}", summary.text());
    let summary = summary.json();
    assert_eq!(summary["namespace"], "acme");
    assert_eq!(summary["highest_version"]["version"], "1.1.0");
    assert_eq!(
        summary["versions_url"],
        "/repository/galaxy/api/v3/collections/acme/tools/versions/"
    );
    assert!(summary["updated_at"].as_str().unwrap().ends_with('Z'));

    let versions = get(
        &fixture,
        "/repository/galaxy/api/v3/collections/acme/tools/versions/?limit=2",
    )
    .await
    .json();
    assert_eq!(versions["meta"]["count"], 3);
    let listed: Vec<&str> = versions["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["version"].as_str().unwrap())
        .collect();
    assert_eq!(listed, ["2.0.0-beta.1", "1.1.0"]);
    assert_eq!(
        versions["links"]["next"],
        "/repository/galaxy/api/v3/collections/acme/tools/versions/?limit=2&offset=2"
    );
    assert_eq!(versions["data"][1]["requires_ansible"], ">=2.15.0");
    let rest = get(&fixture, versions["links"]["next"].as_str().unwrap())
        .await
        .json();
    assert_eq!(rest["data"][0]["version"], "1.0.0");
    assert_eq!(rest["links"]["next"], Value::Null);

    let version = fixture
        .send(
            Method::GET,
            "/repository/galaxy/api/v3/collections/acme/tools/versions/1.1.0/",
            Auth::None,
            None,
            &[("authorization", &token_header)],
        )
        .await;
    assert_eq!(version.status, StatusCode::OK, "{}", version.text());
    let version = version.json();
    assert_eq!(
        version["download_url"],
        "http://localhost/repository/galaxy/acme/tools/1.1.0/acme-tools-1.1.0.tar.gz"
    );
    assert_eq!(version["artifact"]["sha256"], sha256(&second));
    assert_eq!(version["artifact"]["filename"], "acme-tools-1.1.0.tar.gz");
    assert_eq!(version["namespace"]["name"], "acme");
    assert_eq!(version["collection"]["name"], "tools");
    assert_eq!(
        version["metadata"]["dependencies"]["community.general"],
        ">=8.0.0"
    );
    assert_eq!(version["metadata"]["contents"][0]["name"], "hello");
    assert_eq!(version["manifest"]["collection_info"]["version"], "1.1.0");
    assert_eq!(version["signatures"], json!([]));

    let download = get(
        &fixture,
        "/repository/galaxy/acme/tools/1.1.0/acme-tools-1.1.0.tar.gz",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, second);
    assert_eq!(
        get(
            &fixture,
            "/repository/galaxy/api/v3/collections/acme/tools/versions/9.0.0/"
        )
        .await
        .json()["errors"][0]["code"],
        "not_found"
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/galaxy/api/v3/collections/acme/missing/versions/"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=galaxy",
            Auth::Admin,
            Body::from(form(
                None,
                "ansible.asset",
                "upload.tar.gz",
                &collection("acme", "other", "0.1.0", json!({})),
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
            "/service/rest/v1/components?repository=galaxy",
            Auth::Admin,
            Body::from(form(None, "ansible.asset", "x.tar.gz", b"nope")),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    let all = get(&fixture, "/repository/galaxy/api/v3/collections/")
        .await
        .json();
    assert_eq!(all["meta"]["count"], 2);
    assert_eq!(all["data"][0]["name"], "other");
    assert_eq!(all["data"][1]["highest_version"]["version"], "1.1.0");

    let found = get(
        &fixture,
        "/service/rest/v1/search?format=ansible&ansible.tag=demo&name=other",
    )
    .await
    .json();
    assert_eq!(found["items"].as_array().unwrap().len(), 1, "{found}");
    assert_eq!(found["items"][0]["group"], "acme");
    let by_keyword = get(
        &fixture,
        "/service/rest/v1/search?format=ansible&q=collection",
    )
    .await;
    assert!(!by_keyword.json()["items"].as_array().unwrap().is_empty());

    let delete = fixture
        .call(
            Method::DELETE,
            "/repository/galaxy/api/v3/collections/acme/tools/versions/1.0.0/",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(delete.status, StatusCode::NO_CONTENT, "{}", delete.text());
    let versions = get(
        &fixture,
        "/repository/galaxy/api/v3/collections/acme/tools/versions/",
    )
    .await
    .json();
    assert_eq!(versions["meta"]["count"], 2);
}

#[tokio::test]
async fn write_policies_permissions_and_other_repository_kinds() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("open", "ALLOW")).await;
    create(&fixture, "hosted", hosted("closed", "DENY")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["open"]},
        }),
    )
    .await;
    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let admin = fixture.admin_password.clone();
    let basic = |user: &str, password: &str| {
        use base64::Engine;
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"))
        )
    };
    let data = collection("acme", "tools", "1.0.0", json!({}));
    let plain = publish_plain(
        &fixture,
        "open",
        &basic("admin", &admin),
        "acme-tools-0.9.0.tar.gz",
        &collection("acme", "tools", "0.9.0", json!({})),
    )
    .await;
    assert_eq!(task(&fixture, &plain).await["state"], "completed");
    for _ in 0..2 {
        let reply = publish(
            &fixture,
            "open",
            Some(&basic("admin", &admin)),
            "acme-tools-1.0.0.tar.gz",
            &data,
            None,
        )
        .await;
        assert_eq!(task(&fixture, &reply).await["state"], "completed");
    }
    let closed = publish(
        &fixture,
        "closed",
        Some(&basic("admin", &admin)),
        "acme-tools-1.0.0.tar.gz",
        &data,
        None,
    )
    .await;
    assert_eq!(closed.status, StatusCode::BAD_REQUEST);
    let reader = publish(
        &fixture,
        "open",
        Some(&basic("reader", "reader-password-1")),
        "acme-tools-1.0.1.tar.gz",
        &data,
        None,
    )
    .await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "deployer-role", "name": "deployer-role", "description": "", "privileges": ["as-repository-view-ansible-open-add"], "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user("deployer", "deployer-password-1", &["deployer-role"])
        .await;
    let deployed = publish(
        &fixture,
        "open",
        Some(&basic("deployer", "deployer-password-1")),
        "acme-tools-3.0.0.tar.gz",
        &collection("acme", "tools", "3.0.0", json!({})),
        None,
    )
    .await;
    assert_eq!(deployed.status, StatusCode::ACCEPTED, "{}", deployed.text());
    let polled = fixture
        .send(
            Method::GET,
            deployed.json()["task"].as_str().unwrap(),
            Auth::Basic("deployer", "deployer-password-1"),
            None,
            &[],
        )
        .await;
    assert_eq!(polled.status, StatusCode::OK, "{}", polled.text());
    assert_eq!(polled.json()["state"], "completed");
    let unread = fixture
        .send(
            Method::GET,
            "/repository/open/api/v3/collections/acme/tools/",
            Auth::Basic("deployer", "deployer-password-1"),
            None,
            &[],
        )
        .await;
    assert_eq!(unread.status, StatusCode::FORBIDDEN);
    let to_group = publish(
        &fixture,
        "all",
        Some(&basic("admin", &admin)),
        "acme-tools-1.0.1.tar.gz",
        &data,
        None,
    )
    .await;
    assert_eq!(to_group.status, StatusCode::BAD_REQUEST);
    let through_group = get(
        &fixture,
        "/repository/all/api/v3/collections/acme/tools/versions/1.0.0/",
    )
    .await
    .json();
    assert_eq!(
        through_group["download_url"],
        "http://localhost/repository/all/acme/tools/1.0.0/acme-tools-1.0.0.tar.gz"
    );
    let download = get(
        &fixture,
        "/repository/all/acme/tools/1.0.0/acme-tools-1.0.0.tar.gz",
    )
    .await;
    assert_eq!(download.body, data);
    let get_publish = get(&fixture, "/repository/open/api/v3/artifacts/collections/").await;
    assert_eq!(get_publish.status, StatusCode::METHOD_NOT_ALLOWED);
    let missing_task = get(
        &fixture,
        "/repository/open/api/v3/imports/collections/00000000-0000-0000-0000-000000000000/",
    )
    .await;
    assert_eq!(missing_task.status, StatusCode::NOT_FOUND);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/open/api/v3/collections/acme/tools/",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
}

#[derive(Clone, Default)]
struct Upstream {
    address: Arc<Mutex<Option<SocketAddr>>>,
    archive: Arc<Vec<u8>>,
    version_lists: Arc<AtomicUsize>,
}

const PLUGIN: &str = "/api/v3/plugin/ansible/content/published/collections/index";

async fn upstream(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let address = upstream.address.lock().unwrap().unwrap();
    let version = |version: &str, sha: &str| {
        json!({
            "version": version,
            "href": format!("{PLUGIN}/community/demo/versions/{version}/"),
            "created_at": "2026-01-02T03:04:05.000000Z",
            "updated_at": "2026-01-02T03:04:05.000000Z",
            "requires_ansible": ">=2.14",
            "marks": [],
            "artifact": {"filename": format!("community-demo-{version}.tar.gz"), "sha256": sha, "size": 10},
            "collection": {"id": "c0ffee", "name": "demo", "href": format!("{PLUGIN}/community/demo/")},
            "download_url": format!("http://{address}/api/v3/plugin/ansible/content/published/collections/artifacts/community-demo-{version}.tar.gz"),
            "name": "demo",
            "namespace": {"name": "community", "metadata_sha256": "x"},
            "signatures": [],
            "metadata": {"authors": ["Community"], "description": "Remote demo", "tags": ["remote"], "license": ["GPL-3.0-or-later"], "dependencies": {}, "contents": []},
        })
    };
    match path.as_str() {
        "api/" => json!({"available_versions": {"v3": "v3/", "v1": "v1/"}})
            .to_string()
            .into_response(),
        "api/v3/collections/community/demo/" => {
            Redirect::to(&format!("{PLUGIN}/community/demo/")).into_response()
        }
        "api/v3/plugin/ansible/content/published/collections/index/community/demo/" => json!({
            "href": format!("{PLUGIN}/community/demo/"),
            "namespace": "community",
            "name": "demo",
            "deprecated": false,
            "versions_url": format!("{PLUGIN}/community/demo/versions/"),
            "highest_version": {"href": "x", "version": "1.1.0"},
            "created_at": "2026-01-01T00:00:00.000000Z",
            "updated_at": "2026-02-01T00:00:00.000000Z",
            "download_count": 41
        })
        .to_string()
        .into_response(),
        "api/v3/collections/community/demo/versions/" => {
            upstream.version_lists.fetch_add(1, Ordering::SeqCst);
            let second = query.unwrap_or_default().contains("offset=1");
            let (data, next) = match second {
                false => (
                    vec![
                        json!({"version": "1.1.0", "href": "x", "created_at": "2026-02-01T00:00:00Z", "updated_at": "2026-02-01T00:00:00Z", "requires_ansible": ">=2.14", "marks": []}),
                    ],
                    json!("/api/v3/collections/community/demo/versions/?limit=1&offset=1"),
                ),
                true => (
                    vec![
                        json!({"version": "1.0.0", "href": "x", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "requires_ansible": ">=2.14", "marks": []}),
                    ],
                    Value::Null,
                ),
            };
            json!({"meta": {"count": 2}, "links": {"next": next}, "data": data})
                .to_string()
                .into_response()
        }
        "api/v3/collections/community/demo/versions/1.0.0/" => {
            version("1.0.0", &sha256(&upstream.archive))
                .to_string()
                .into_response()
        }
        "api/v3/collections/community/demo/versions/1.1.0/" => version("1.1.0", &"0".repeat(64))
            .to_string()
            .into_response(),
        "api/v3/plugin/ansible/content/published/collections/artifacts/community-demo-1.0.0.tar.gz"
        | "api/v3/plugin/ansible/content/published/collections/artifacts/community-demo-1.1.0.tar.gz" => {
            upstream.archive.as_ref().clone().into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::test]
async fn proxies_cache_documents_and_groups_merge_members() {
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
        archive: Arc::new(collection("community", "demo", "1.0.0", json!({}))),
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
            "name": "galaxy-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "galaxy-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "galaxy-proxy"]},
        }),
    )
    .await;

    let summary = get(
        &fixture,
        "/repository/galaxy-proxy/api/v3/collections/community/demo/",
    )
    .await;
    assert_eq!(summary.status, StatusCode::OK, "{}", summary.text());
    let summary = summary.json();
    assert_eq!(
        summary["href"],
        "/repository/galaxy-proxy/api/v3/collections/community/demo/"
    );
    assert_eq!(summary["highest_version"]["version"], "1.1.0");
    assert_eq!(summary["download_count"], 41);

    let versions = get(
        &fixture,
        "/repository/galaxy-proxy/api/v3/collections/community/demo/versions/?limit=100",
    )
    .await
    .json();
    assert_eq!(versions["meta"]["count"], 2, "{versions}");
    assert_eq!(
        versions["data"][0]["href"],
        "/repository/galaxy-proxy/api/v3/collections/community/demo/versions/1.1.0/"
    );
    get(
        &fixture,
        "/repository/galaxy-proxy/api/v3/collections/community/demo/versions/",
    )
    .await;
    assert_eq!(state.version_lists.load(Ordering::SeqCst), 2);

    let version = get(
        &fixture,
        "/repository/galaxy-proxy/api/v3/collections/community/demo/versions/1.0.0/",
    )
    .await
    .json();
    assert_eq!(
        version["download_url"],
        "http://localhost/repository/galaxy-proxy/community/demo/1.0.0/community-demo-1.0.0.tar.gz"
    );
    assert_eq!(version["collection"]["id"], "c0ffee");
    let download = get(
        &fixture,
        "/repository/galaxy-proxy/community/demo/1.0.0/community-demo-1.0.0.tar.gz",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK, "{}", download.text());
    assert_eq!(download.body, *state.archive);
    let tampered = get(
        &fixture,
        "/repository/galaxy-proxy/community/demo/1.1.0/community-demo-1.1.0.tar.gz",
    )
    .await;
    assert_eq!(tampered.status, StatusCode::BAD_GATEWAY);

    let component = get(
        &fixture,
        "/service/rest/v1/search?repository=galaxy-proxy&format=ansible",
    )
    .await
    .json();
    assert_eq!(
        component["items"].as_array().unwrap().len(),
        1,
        "{component}"
    );
    assert_eq!(component["items"][0]["group"], "community");
    assert_eq!(component["items"][0]["version"], "1.0.0");
    let cached = get(&fixture, "/repository/galaxy-proxy/api/v3/collections/")
        .await
        .json();
    assert_eq!(cached["meta"]["count"], 1);
    assert_eq!(cached["data"][0]["highest_version"]["version"], "1.1.0");

    let own = collection("community", "demo", "1.2.0", json!({}));
    let reply = publish(
        &fixture,
        "own",
        Some(&format!("Basic {}", {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .encode(format!("admin:{}", fixture.admin_password))
        })),
        "community-demo-1.2.0.tar.gz",
        &own,
        None,
    )
    .await;
    assert_eq!(task(&fixture, &reply).await["state"], "completed");
    let merged = get(
        &fixture,
        "/repository/galaxy-all/api/v3/collections/community/demo/versions/",
    )
    .await
    .json();
    let listed: Vec<&str> = merged["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["version"].as_str().unwrap())
        .collect();
    assert_eq!(listed, ["1.2.0", "1.1.0", "1.0.0"]);
    let collection_doc = get(
        &fixture,
        "/repository/galaxy-all/api/v3/collections/community/demo/",
    )
    .await
    .json();
    assert_eq!(collection_doc["highest_version"]["version"], "1.2.0");
    assert_eq!(collection_doc["created_at"], "2026-01-01T00:00:00.000000Z");
    let through = get(
        &fixture,
        "/repository/galaxy-all/api/v3/collections/community/demo/versions/1.0.0/",
    )
    .await
    .json();
    assert_eq!(
        through["download_url"],
        "http://localhost/repository/galaxy-all/community/demo/1.0.0/community-demo-1.0.0.tar.gz"
    );
    let from_proxy = get(
        &fixture,
        "/repository/galaxy-all/community/demo/1.0.0/community-demo-1.0.0.tar.gz",
    )
    .await;
    assert_eq!(from_proxy.body, *state.archive);
    let from_hosted = get(
        &fixture,
        "/repository/galaxy-all/community/demo/1.2.0/community-demo-1.2.0.tar.gz",
    )
    .await;
    assert_eq!(from_hosted.body, own);
    let all = get(&fixture, "/repository/galaxy-all/api/v3/collections/")
        .await
        .json();
    assert_eq!(all["meta"]["count"], 1);
    assert_eq!(all["data"][0]["highest_version"]["version"], "1.2.0");
    let missing = get(
        &fixture,
        "/repository/galaxy-all/api/v3/collections/community/none/versions/",
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}
