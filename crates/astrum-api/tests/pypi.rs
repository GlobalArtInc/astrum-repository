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

const PIP_ACCEPT: &str = "application/vnd.pypi.simple.v1+json, application/vnd.pypi.simple.v1+html; q=0.1, text/html; q=0.01";
const BOUNDARY: &str = "pypiboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pypi::PyPi));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/pypi/{kind}"),
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

fn proxy(name: &str, remote: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
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

fn metadata_text(name: &str, version: &str) -> String {
    format!(
        "Metadata-Version: 2.1\nName: {name}\nVersion: {version}\nSummary: The {name} test package\nKeywords: testing,astrum\nClassifier: Programming Language :: Python :: 3\nRequires-Python: >=3.8\n\nA longer description of {name}.\n"
    )
}

fn wheel(name: &str, version: &str) -> Vec<u8> {
    let module = name.replace(['-', '.'], "_").to_lowercase();
    let info = format!("{module}-{version}.dist-info");
    let files: Vec<(String, Vec<u8>)> = vec![
        (format!("{module}/__init__.py"), b"VALUE = 1\n".to_vec()),
        (
            format!("{info}/METADATA"),
            metadata_text(name, version).into_bytes(),
        ),
        (
            format!("{info}/WHEEL"),
            b"Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n".to_vec(),
        ),
    ];
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (path, data) in &files {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        let packed = encoder.finish().unwrap();
        let mut crc = flate2::Crc::new();
        crc.update(data);
        let offset = out.len() as u32;
        let common = |target: &mut Vec<u8>| {
            target.extend_from_slice(&8u16.to_le_bytes());
            target.extend_from_slice(&[0, 0, 0x21, 0]);
            target.extend_from_slice(&crc.sum().to_le_bytes());
            target.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            target.extend_from_slice(&(data.len() as u32).to_le_bytes());
            target.extend_from_slice(&(path.len() as u16).to_le_bytes());
        };
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0]);
        common(&mut out);
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&packed);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&[20, 3, 20, 0, 0, 0]);
        common(&mut central);
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(path.as_bytes());
    }
    let start = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

fn sdist(name: &str, version: &str) -> Vec<u8> {
    let top = format!("{}-{version}", name.replace(['-', '.'], "_").to_lowercase());
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in [
        (
            format!("{top}/setup.py"),
            b"from setuptools import setup\nsetup()\n".to_vec(),
        ),
        (
            format!("{top}/PKG-INFO"),
            metadata_text(name, version).into_bytes(),
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

fn form(fields: &[(&str, &str)], file: Option<(&str, &str, &[u8])>) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        )
        .unwrap();
    }
    if let Some((field, filename, data)) = file {
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

async fn twine(
    fixture: &Fixture,
    repository: &str,
    auth: Auth<'_>,
    name: &str,
    version: &str,
    filename: &str,
    data: &[u8],
) -> Reply {
    let digest = sha256(data);
    let fields = [
        (":action", "file_upload"),
        ("protocol_version", "1"),
        ("metadata_version", "2.1"),
        ("name", name),
        ("version", version),
        ("summary", "Probe package for the pypi format"),
        ("requires_python", ">=3.8"),
        ("keywords", "astrum, probe"),
        ("classifiers", "Programming Language :: Python :: 3"),
        ("classifiers", "License :: OSI Approved :: MIT License"),
        ("description", "# Probe\n\nLong description."),
        ("sha256_digest", digest.as_str()),
    ];
    post_form(
        fixture,
        &format!("/repository/{repository}/"),
        auth,
        form(&fields, Some(("content", filename, data))),
    )
    .await
}

async fn get(fixture: &Fixture, path: &str, accept: Option<&str>) -> Reply {
    let headers: Vec<(&str, &str)> = accept.map(|a| ("accept", a)).into_iter().collect();
    fixture
        .send(Method::GET, path, Auth::Admin, None, &headers)
        .await
}

#[tokio::test]
async fn hosted_uploads_from_twine_are_served_on_simple_pages() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("pypi-hosted", "ALLOW_ONCE")).await;
    let wheel_bytes = wheel("Astrum_Probe.Pkg", "0.1.0");
    let sdist_bytes = sdist("Astrum_Probe.Pkg", "0.1.0");
    let wheel_name = "astrum_probe_pkg-0.1.0-py3-none-any.whl";
    let sdist_name = "astrum_probe_pkg-0.1.0.tar.gz";
    for (filename, data) in [(wheel_name, &wheel_bytes), (sdist_name, &sdist_bytes)] {
        let reply = twine(
            &fixture,
            "pypi-hosted",
            Auth::Admin,
            "Astrum-Probe.Pkg",
            "0.1.0",
            filename,
            data,
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }

    let root = get(&fixture, "/repository/pypi-hosted/simple/", None).await;
    assert_eq!(root.status, StatusCode::OK);
    assert_eq!(
        root.headers[header::CONTENT_TYPE],
        "text/html; charset=utf-8"
    );
    assert_eq!(root.headers[header::VARY], "Accept");
    assert!(
        root.text()
            .contains("<a href=\"astrum-probe-pkg/\">astrum-probe-pkg</a>"),
        "{}",
        root.text()
    );
    let bare_root = get(&fixture, "/repository/pypi-hosted/simple", None).await;
    assert!(
        bare_root
            .text()
            .contains("href=\"simple/astrum-probe-pkg/\"")
    );

    let page = get(
        &fixture,
        "/repository/pypi-hosted/simple/astrum-probe-pkg/",
        None,
    )
    .await;
    assert_eq!(page.status, StatusCode::OK);
    let html = page.text();
    let metadata_sha = sha256(metadata_text("Astrum_Probe.Pkg", "0.1.0").as_bytes());
    assert!(html.contains(&format!(
        "<a href=\"../../packages/astrum-probe-pkg/0.1.0/{wheel_name}#sha256={}\" data-requires-python=\"&gt;=3.8\" data-dist-info-metadata=\"sha256={metadata_sha}\" data-core-metadata=\"sha256={metadata_sha}\">{wheel_name}</a>",
        sha256(&wheel_bytes)
    )), "{html}");
    assert!(html.contains(&format!(
        "<a href=\"../../packages/astrum-probe-pkg/0.1.0/{sdist_name}#sha256={}\" data-requires-python=\"&gt;=3.8\">{sdist_name}</a>",
        sha256(&sdist_bytes)
    )), "{html}");

    let unnormalized = get(
        &fixture,
        "/repository/pypi-hosted/simple/Astrum_Probe.PKG",
        None,
    )
    .await;
    assert_eq!(unnormalized.status, StatusCode::OK);
    assert!(
        unnormalized
            .text()
            .contains("href=\"../packages/astrum-probe-pkg/0.1.0/")
    );

    let json_page = get(
        &fixture,
        "/repository/pypi-hosted/simple/astrum-probe-pkg/",
        Some(PIP_ACCEPT),
    )
    .await;
    assert_eq!(
        json_page.headers[header::CONTENT_TYPE],
        "application/vnd.pypi.simple.v1+json"
    );
    let document = json_page.json();
    assert_eq!(document["meta"]["api-version"], "1.1");
    assert_eq!(document["name"], "astrum-probe-pkg");
    assert_eq!(document["versions"], json!(["0.1.0"]));
    let files = document["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    let wheel_entry = files
        .iter()
        .find(|file| file["filename"] == wheel_name)
        .unwrap();
    assert_eq!(
        wheel_entry["url"],
        format!("../../packages/astrum-probe-pkg/0.1.0/{wheel_name}")
    );
    assert_eq!(wheel_entry["hashes"]["sha256"], sha256(&wheel_bytes));
    assert_eq!(wheel_entry["requires-python"], ">=3.8");
    assert_eq!(wheel_entry["core-metadata"]["sha256"], metadata_sha);
    assert_eq!(wheel_entry["size"], wheel_bytes.len());
    assert_eq!(wheel_entry["yanked"], false);
    assert!(wheel_entry["upload-time"].as_str().unwrap().ends_with('Z'));
    let json_root = get(
        &fixture,
        "/repository/pypi-hosted/simple/",
        Some(PIP_ACCEPT),
    )
    .await;
    assert_eq!(
        json_root.json()["projects"],
        json!([{"name": "astrum-probe-pkg"}])
    );

    let etag = page.headers[header::ETAG].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/pypi-hosted/simple/astrum-probe-pkg/",
            Auth::Admin,
            None,
            &[("if-none-match", etag.as_str())],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    assert_ne!(json_page.headers[header::ETAG], page.headers[header::ETAG]);

    let file = get(
        &fixture,
        &format!("/repository/pypi-hosted/packages/astrum-probe-pkg/0.1.0/{wheel_name}"),
        None,
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, wheel_bytes);
    assert_eq!(file.headers[header::CONTENT_TYPE], "application/zip");
    let sdist_file = get(
        &fixture,
        &format!("/repository/pypi-hosted/packages/astrum-probe-pkg/0.1.0/{sdist_name}"),
        None,
    )
    .await;
    assert_eq!(sdist_file.body, sdist_bytes);
    assert_eq!(
        sdist_file.headers[header::CONTENT_TYPE],
        "application/x-gzip"
    );
    let metadata = get(
        &fixture,
        &format!("/repository/pypi-hosted/packages/astrum-probe-pkg/0.1.0/{wheel_name}.metadata"),
        None,
    )
    .await;
    assert_eq!(metadata.status, StatusCode::OK);
    assert_eq!(metadata.text(), metadata_text("Astrum_Probe.Pkg", "0.1.0"));

    let again = twine(
        &fixture,
        "pypi-hosted",
        Auth::Admin,
        "Astrum-Probe.Pkg",
        "0.1.0",
        wheel_name,
        &wheel_bytes,
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({
                "id": "pypi-adder",
                "name": "pypi-adder",
                "description": "",
                "privileges": ["as-repository-view-pypi-pypi-hosted-add"],
                "roles": []
            })),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture.add_user("adder", "adder-pass", &["pypi-adder"]).await;
    let add_only = twine(
        &fixture,
        "pypi-hosted",
        Auth::Basic("adder", "adder-pass"),
        "Astrum-Probe.Pkg",
        "0.1.0",
        wheel_name,
        &wheel_bytes,
    )
    .await;
    assert_eq!(
        add_only.status,
        StatusCode::CONFLICT,
        "redeploys are refused before edit rights matter"
    );

    for missing in [
        "/repository/pypi-hosted/simple/nothing-here/",
        "/repository/pypi-hosted/packages/astrum-probe-pkg/0.1.0/missing.tar.gz",
        "/repository/pypi-hosted/simple/astrum-probe-pkg/extra",
    ] {
        assert_eq!(
            get(&fixture, missing, None).await.status,
            StatusCode::NOT_FOUND,
            "{missing}"
        );
    }
    let directory = get(&fixture, "/repository/pypi-hosted/", None).await;
    assert_eq!(directory.status, StatusCode::OK);
    assert!(directory.text().contains("/simple/"));
    let delete = fixture
        .call(
            Method::DELETE,
            &format!("/repository/pypi-hosted/packages/astrum-probe-pkg/0.1.0/{wheel_name}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(delete.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(delete.headers[header::ALLOW], "GET,HEAD,POST");
}

#[tokio::test]
async fn twine_uploads_are_checked() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("checked", "ALLOW")).await;
    let data = sdist("checked-pkg", "1.0");

    let wrong = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        form(
            &[
                (":action", "file_upload"),
                ("name", "checked-pkg"),
                ("version", "1.0"),
                ("sha256_digest", &"0".repeat(64)),
            ],
            Some(("content", "checked_pkg-1.0.tar.gz", &data)),
        ),
    )
    .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    assert!(wrong.text().contains(&sha256(&data)), "{}", wrong.text());
    let md5_wrong = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        form(
            &[
                (":action", "file_upload"),
                ("name", "checked-pkg"),
                ("version", "1.0"),
                ("md5_digest", "00000000000000000000000000000000"),
            ],
            Some(("content", "checked_pkg-1.0.tar.gz", &data)),
        ),
    )
    .await;
    assert_eq!(md5_wrong.status, StatusCode::BAD_REQUEST);
    let assets = fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=checked",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(
        assets["items"],
        json!([]),
        "a rejected upload stores nothing"
    );

    let anonymous = twine(
        &fixture,
        "checked",
        Auth::None,
        "checked-pkg",
        "1.0",
        "checked_pkg-1.0.tar.gz",
        &data,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));

    let plain = fixture
        .send_body(
            Method::POST,
            "/repository/checked/",
            Auth::Admin,
            Body::from(":action=file_upload&name=x&version=1"),
            &[("content-type", "application/x-www-form-urlencoded")],
        )
        .await;
    assert_eq!(plain.status, StatusCode::BAD_REQUEST);
    let no_content = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        form(
            &[(":action", "file_upload"), ("name", "x"), ("version", "1")],
            None,
        ),
    )
    .await;
    assert_eq!(no_content.status, StatusCode::BAD_REQUEST);
    let action = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        form(
            &[(":action", "submit"), ("name", "x"), ("version", "1")],
            Some(("content", "x-1.tar.gz", &data)),
        ),
    )
    .await;
    assert_eq!(action.status, StatusCode::BAD_REQUEST);
    for (name, version, filename) in [
        ("", "1.0", "x-1.0.tar.gz"),
        ("bad/name", "1.0", "x-1.0.tar.gz"),
        ("x", "../1.0", "x-1.0.tar.gz"),
        ("x", "1.0", ".."),
    ] {
        let reply = post_form(
            &fixture,
            "/repository/checked/",
            Auth::Admin,
            form(
                &[
                    (":action", "file_upload"),
                    ("name", name),
                    ("version", version),
                ],
                Some(("content", filename, &data)),
            ),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{name} {version} {filename}"
        );
    }

    let late = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        {
            let mut body = form(
                &[(":action", "file_upload"), ("name", "late"), ("version", "1.0")],
                Some(("content", "late-1.0.tar.gz", &data)),
            );
            body.truncate(body.len() - format!("--{BOUNDARY}--\r\n").len());
            write!(
                body,
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"sha256_digest\"\r\n\r\n{}\r\n--{BOUNDARY}--\r\n",
                "1".repeat(64)
            )
            .unwrap();
            body
        },
    )
    .await;
    assert_eq!(late.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        get(
            &fixture,
            "/repository/checked/packages/late/1.0/late-1.0.tar.gz",
            None
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let unnamed = post_form(
        &fixture,
        "/repository/checked/",
        Auth::Admin,
        form(
            &[
                (":action", "file_upload"),
                ("name", "Foo_Bar"),
                ("version", "2.0"),
                ("md5_digest", &hex::encode(md5::Md5::digest(&data))),
            ],
            Some(("content", "other-9.9.tar.gz", &data)),
        ),
    )
    .await;
    assert_eq!(unnamed.status, StatusCode::OK, "{}", unnamed.text());
    assert_eq!(
        get(
            &fixture,
            "/repository/checked/packages/foo-bar/2.0/other-9.9.tar.gz",
            None
        )
        .await
        .body,
        data
    );

    fixture
        .add_user("reader", "reader-pass", &["pypi-reader"])
        .await;
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({
                "id": "pypi-reader",
                "name": "pypi-reader",
                "description": "",
                "privileges": ["as-repository-view-pypi-checked-read"],
                "roles": []
            })),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    let readable = fixture
        .call(
            Method::GET,
            "/repository/checked/simple/foo-bar/",
            Auth::Basic("reader", "reader-pass"),
            None,
        )
        .await;
    assert_eq!(readable.status, StatusCode::OK);
    let refused = twine(
        &fixture,
        "checked",
        Auth::Basic("reader", "reader-pass"),
        "checked-pkg",
        "1.0",
        "checked_pkg-1.0.tar.gz",
        &data,
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn components_and_search_follow_the_reference() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("pypi-components", "ALLOW")).await;
    let spec = fixture
        .call(
            Method::GET,
            "/service/rest/v1/formats/pypi/upload-specs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(
        spec,
        json!({
            "format": "pypi",
            "multipleUpload": false,
            "componentFields": [],
            "assetFields": [{"name": "asset", "type": "FILE", "description": null, "optional": false, "group": null, "groupHelpText": null, "options": null}]
        })
    );

    let data = wheel("Uploaded.Pkg", "2.1.0");
    let boundary_body = form(
        &[],
        Some(("pypi.asset", "uploaded_pkg-2.1.0-py3-none-any.whl", &data)),
    );
    let reply = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=pypi-components",
            Auth::Admin,
            Body::from(boundary_body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let sdist_body = form(
        &[],
        Some((
            "pypi.asset",
            "uploaded_pkg-2.1.0.tar.gz",
            &sdist("Uploaded.Pkg", "2.1.0"),
        )),
    );
    let reply = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=pypi-components",
            Auth::Admin,
            Body::from(sdist_body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let garbage = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=pypi-components",
            Auth::Admin,
            Body::from(form(&[], Some(("pypi.asset", "notes.txt", b"just text")))),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(
        garbage.status,
        StatusCode::BAD_REQUEST,
        "{}",
        garbage.text()
    );

    let components = fixture
        .call(
            Method::GET,
            "/service/rest/v1/components?repository=pypi-components",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let items = components["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{components}");
    let component = &items[0];
    assert_eq!(component["group"], "");
    assert_eq!(component["name"], "uploaded-pkg");
    assert_eq!(component["version"], "2.1.0");
    let paths: Vec<&str> = component["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "/packages/uploaded-pkg/2.1.0/uploaded_pkg-2.1.0-py3-none-any.whl",
            "/packages/uploaded-pkg/2.1.0/uploaded_pkg-2.1.0-py3-none-any.whl.metadata",
            "/packages/uploaded-pkg/2.1.0/uploaded_pkg-2.1.0.tar.gz",
        ]
    );
    let wheel_asset = &component["assets"][0];
    assert_eq!(
        wheel_asset["pypi"],
        json!({"name": "Uploaded.Pkg", "version": "2.1.0", "summary": "The Uploaded.Pkg test package", "requires_python": ">=3.8"})
    );
    assert_eq!(wheel_asset["contentType"], "application/zip");
    assert_eq!(component["assets"][2]["contentType"], "application/x-gzip");

    for (query, expected) in [
        ("pypi.summary=The%20Uploaded.Pkg%20test%20package", 1),
        ("pypi.summary=*uploaded*", 1),
        ("pypi.keywords=*astrum*", 1),
        ("pypi.classifiers=*Python%20::%203*", 1),
        ("pypi.description=*longer%20description*", 1),
        ("pypi.summary=nothing", 0),
    ] {
        let found = fixture
            .call(
                Method::GET,
                &format!("/service/rest/v1/search?repository=pypi-components&{query}"),
                Auth::Admin,
                None,
            )
            .await
            .json();
        assert_eq!(
            found["items"].as_array().unwrap().len(),
            expected,
            "{query}: {found}"
        );
    }
    let page = get(
        &fixture,
        "/repository/pypi-components/simple/uploaded-pkg/",
        None,
    )
    .await;
    assert!(page.text().contains("uploaded_pkg-2.1.0.tar.gz"));
    assert!(!page.text().contains(".metadata\""));
}

type Seen = (String, Option<String>);

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<AtomicUsize>,
    pages: Arc<HashMap<String, (String, String)>>,
    files: Arc<HashMap<String, Vec<u8>>>,
    authorizations: Arc<Mutex<Vec<Seen>>>,
}

impl Upstream {
    fn saw(&self, path: &str, headers: &HeaderMap) {
        self.hits.fetch_add(1, Ordering::SeqCst);
        self.authorizations.lock().unwrap().push((
            path.to_owned(),
            headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
        ));
    }

    fn authorization_for(&self, path: &str) -> Option<Option<String>> {
        self.authorizations
            .lock()
            .unwrap()
            .iter()
            .find(|(seen, _)| seen == path)
            .map(|(_, authorization)| authorization.clone())
    }
}

async fn upstream_page(
    State(upstream): State<Upstream>,
    Path(project): Path<String>,
    headers: HeaderMap,
) -> Response {
    let path = format!("/simple/{project}");
    upstream.saw(&path, &headers);
    let Some((html, json_body)) = upstream.pages.get(project.trim_end_matches('/')) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let wants_json = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.starts_with("application/vnd.pypi.simple.v1+json"));
    if wants_json && !json_body.is_empty() {
        (
            [(header::CONTENT_TYPE, "application/vnd.pypi.simple.v1+json")],
            json_body.clone(),
        )
            .into_response()
    } else {
        ([(header::CONTENT_TYPE, "text/html")], html.clone()).into_response()
    }
}

async fn upstream_root(State(upstream): State<Upstream>, headers: HeaderMap) -> Response {
    upstream.saw("/simple/", &headers);
    let mut names: Vec<&String> = upstream.pages.keys().collect();
    names.sort();
    let links: String = names
        .iter()
        .map(|name| format!("<a href=\"/simple/{name}/\">{name}</a>\n"))
        .collect();
    (
        [(header::CONTENT_TYPE, "text/html")],
        format!("<html><body>{links}</body></html>"),
    )
        .into_response()
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    upstream.saw(&format!("/{path}"), &headers);
    match upstream.files.get(&path) {
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
        .route("/simple/", routing::get(upstream_root))
        .route("/simple/{*project}", routing::get(upstream_page))
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

struct Remote {
    index: Upstream,
    index_address: SocketAddr,
    files: Upstream,
    wheel: Vec<u8>,
    sdist: Vec<u8>,
}

async fn remote() -> Remote {
    let wheel_bytes = wheel("demo", "1.0");
    let sdist_bytes = sdist("demo", "0.9");
    let metadata = metadata_text("demo", "1.0");
    let files = Upstream {
        files: Arc::new(
            [
                (
                    "files/ab/demo-1.0-py3-none-any.whl".to_owned(),
                    wheel_bytes.clone(),
                ),
                (
                    "files/ab/demo-1.0-py3-none-any.whl.metadata".to_owned(),
                    metadata.clone().into_bytes(),
                ),
            ]
            .into(),
        ),
        ..Default::default()
    };
    let files_address = serve_upstream(files.clone()).await;
    let json_page = json!({
        "meta": {"api-version": "1.1"},
        "name": "demo",
        "versions": ["0.9", "1.0"],
        "files": [
            {
                "filename": "demo-1.0-py3-none-any.whl",
                "url": format!("http://{files_address}/files/ab/demo-1.0-py3-none-any.whl"),
                "hashes": {"sha256": sha256(&wheel_bytes)},
                "requires-python": ">=3.8",
                "core-metadata": {"sha256": sha256(metadata.as_bytes())},
                "yanked": false,
                "size": wheel_bytes.len()
            },
            {
                "filename": "demo-0.9.tar.gz",
                "url": "../../local/demo-0.9.tar.gz",
                "hashes": {"sha256": sha256(&sdist_bytes)},
                "yanked": "broken build",
                "size": sdist_bytes.len()
            },
            {
                "filename": "demo-0.1.tar.gz",
                "url": "http://169.254.169.254/latest/demo-0.1.tar.gz",
                "hashes": {"sha256": "00"}
            }
        ]
    })
    .to_string();
    let html_page = "<html><body><a href=\"../../other/HTML-ONLY-0.1.tar.gz#sha256=11\">html-only-0.1.tar.gz</a></body></html>".to_owned();
    let index = Upstream {
        pages: Arc::new(
            [
                ("demo".to_owned(), ("<html></html>".to_owned(), json_page)),
                ("html-only".to_owned(), (html_page, String::new())),
            ]
            .into(),
        ),
        files: Arc::new([("local/demo-0.9.tar.gz".to_owned(), sdist_bytes.clone())].into()),
        ..Default::default()
    };
    let index_address = serve_upstream(index.clone()).await;
    Remote {
        index,
        index_address,
        files,
        wheel: wheel_bytes,
        sdist: sdist_bytes,
    }
}

#[tokio::test]
async fn proxies_rewrite_pages_and_fetch_files_from_other_hosts() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let upstream = remote().await;
    create(
        &fixture,
        "proxy",
        proxy("pypi-proxy", &format!("http://{}/", upstream.index_address)),
    )
    .await;

    let page = get(&fixture, "/repository/pypi-proxy/simple/Demo/", None).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text());
    let html = page.text();
    assert!(html.contains(&format!(
        "<a href=\"../../packages/demo/1.0/demo-1.0-py3-none-any.whl#sha256={}\" data-requires-python=\"&gt;=3.8\" data-dist-info-metadata=\"sha256={}\"",
        sha256(&upstream.wheel),
        sha256(metadata_text("demo", "1.0").as_bytes())
    )), "{html}");
    assert!(html.contains(&format!(
        "<a href=\"../../packages/demo/0.9/demo-0.9.tar.gz#sha256={}\" data-yanked=\"broken build\">",
        sha256(&upstream.sdist)
    )), "{html}");
    assert!(!html.contains("169.254"));
    assert!(!html.contains(&upstream.index_address.to_string()));
    let document = get(
        &fixture,
        "/repository/pypi-proxy/simple/demo/",
        Some(PIP_ACCEPT),
    )
    .await
    .json();
    assert_eq!(document["files"][1]["yanked"], "broken build");
    assert_eq!(
        document["files"][0]["url"],
        "../../packages/demo/1.0/demo-1.0-py3-none-any.whl"
    );
    assert_eq!(document["meta"]["api-version"], "1.0");
    assert_eq!(
        upstream.index.hits.load(Ordering::SeqCst),
        1,
        "the page is cached for metadataMaxAge"
    );

    let wheel_reply = get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/1.0/demo-1.0-py3-none-any.whl",
        None,
    )
    .await;
    assert_eq!(wheel_reply.status, StatusCode::OK, "{}", wheel_reply.text());
    assert_eq!(wheel_reply.body, upstream.wheel);
    assert_eq!(wheel_reply.headers[header::CONTENT_TYPE], "application/zip");
    assert!(
        upstream
            .index
            .authorization_for("/simple/demo/")
            .flatten()
            .is_some(),
        "the remote gets the proxy's credentials"
    );
    assert_eq!(
        upstream
            .files
            .authorization_for("/files/ab/demo-1.0-py3-none-any.whl"),
        Some(None),
        "another host never sees them"
    );
    let metadata = get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/1.0/demo-1.0-py3-none-any.whl.metadata",
        None,
    )
    .await;
    assert_eq!(metadata.text(), metadata_text("demo", "1.0"));
    let sdist_reply = get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/0.9/demo-0.9.tar.gz",
        None,
    )
    .await;
    assert_eq!(sdist_reply.body, upstream.sdist);
    assert!(
        upstream
            .index
            .authorization_for("/local/demo-0.9.tar.gz")
            .flatten()
            .is_some()
    );
    let files_hits = upstream.files.hits.load(Ordering::SeqCst);
    get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/1.0/demo-1.0-py3-none-any.whl",
        None,
    )
    .await;
    assert_eq!(upstream.files.hits.load(Ordering::SeqCst), files_hits);

    let assets = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search/assets?repository=pypi-proxy&name=demo&version=1.0",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let wheel_asset = assets["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"] == "/packages/demo/1.0/demo-1.0-py3-none-any.whl")
        .unwrap()
        .clone();
    assert_eq!(wheel_asset["pypi"]["summary"], "The demo test package");
    assert_eq!(wheel_asset["pypi"]["requires_python"], ">=3.8");

    for (path, status) in [
        (
            "/repository/pypi-proxy/packages/demo/0.1/demo-0.1.tar.gz",
            StatusCode::BAD_GATEWAY,
        ),
        (
            "/repository/pypi-proxy/packages/demo/9.9/demo-1.0-py3-none-any.whl",
            StatusCode::NOT_FOUND,
        ),
        (
            "/repository/pypi-proxy/packages/demo/2.0/demo-2.0.tar.gz",
            StatusCode::NOT_FOUND,
        ),
        (
            "/repository/pypi-proxy/packages/demo/0.9/demo-0.9.tar.gz.metadata",
            StatusCode::NOT_FOUND,
        ),
        (
            "/repository/pypi-proxy/simple/absent/",
            StatusCode::NOT_FOUND,
        ),
    ] {
        assert_eq!(get(&fixture, path, None).await.status, status, "{path}");
    }

    let html_only = get(&fixture, "/repository/pypi-proxy/simple/html-only/", None).await;
    assert!(
        html_only
            .text()
            .contains("href=\"../../packages/html-only/0.1/html-only-0.1.tar.gz#sha256=11\""),
        "{}",
        html_only.text()
    );
    let root = get(&fixture, "/repository/pypi-proxy/simple/", Some(PIP_ACCEPT))
        .await
        .json();
    assert_eq!(
        root["projects"],
        json!([{"name": "demo"}, {"name": "html-only"}])
    );
    let upload = twine(
        &fixture,
        "pypi-proxy",
        Auth::Admin,
        "demo",
        "3.0",
        "demo-3.0.tar.gz",
        &upstream.sdist,
    )
    .await;
    assert_eq!(upload.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn proxies_refuse_internal_addresses_by_default() {
    let fixture = ready().await;
    let upstream = remote().await;
    create(
        &fixture,
        "proxy",
        proxy("guarded", &format!("http://{}/", upstream.index_address)),
    )
    .await;
    let page = get(&fixture, "/repository/guarded/simple/demo/", None).await;
    assert_eq!(page.status, StatusCode::BAD_GATEWAY, "{}", page.text());
    assert_eq!(upstream.index.hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn groups_merge_member_pages_and_read_through_members() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let upstream = remote().await;
    create(&fixture, "hosted", hosted("pypi-local", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy(
            "pypi-remote",
            &format!("http://{}/", upstream.index_address),
        ),
    )
    .await;
    create(
        &fixture,
        "group",
        group("pypi-all", &["pypi-local", "pypi-remote"]),
    )
    .await;
    create(&fixture, "group", group("pypi-outer", &["pypi-all"])).await;

    let local_sdist = sdist("Demo", "0.9");
    let local = twine(
        &fixture,
        "pypi-local",
        Auth::Admin,
        "demo",
        "0.9",
        "demo-0.9.tar.gz",
        &local_sdist,
    )
    .await;
    assert_eq!(local.status, StatusCode::OK, "{}", local.text());
    let mine = twine(
        &fixture,
        "pypi-local",
        Auth::Admin,
        "mine",
        "1.0",
        "mine-1.0.tar.gz",
        &sdist("mine", "1.0"),
    )
    .await;
    assert_eq!(mine.status, StatusCode::OK);
    assert_ne!(sha256(&local_sdist), sha256(&upstream.sdist));

    let document = get(
        &fixture,
        "/repository/pypi-outer/simple/demo/",
        Some(PIP_ACCEPT),
    )
    .await
    .json();
    let files = document["files"].as_array().unwrap();
    let names: Vec<&str> = files
        .iter()
        .map(|file| file["filename"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "demo-0.1.tar.gz",
            "demo-0.9.tar.gz",
            "demo-1.0-py3-none-any.whl"
        ]
    );
    let sdist_entry = &files[1];
    assert_eq!(sdist_entry["hashes"]["sha256"], sha256(&local_sdist));
    assert_eq!(sdist_entry["yanked"], false, "the first member wins");

    let from_local = get(
        &fixture,
        "/repository/pypi-outer/packages/demo/0.9/demo-0.9.tar.gz",
        None,
    )
    .await;
    assert_eq!(from_local.body, local_sdist);
    let from_remote = get(
        &fixture,
        "/repository/pypi-outer/packages/demo/1.0/demo-1.0-py3-none-any.whl",
        None,
    )
    .await;
    assert_eq!(from_remote.status, StatusCode::OK, "{}", from_remote.text());
    assert_eq!(from_remote.body, upstream.wheel);

    let html = get(&fixture, "/repository/pypi-all/simple/mine/", None).await;
    assert!(
        html.text()
            .contains("href=\"../../packages/mine/1.0/mine-1.0.tar.gz#sha256=")
    );
    let root = get(&fixture, "/repository/pypi-all/simple/", Some(PIP_ACCEPT))
        .await
        .json();
    assert_eq!(
        root["projects"],
        json!([{"name": "demo"}, {"name": "html-only"}, {"name": "mine"}])
    );
    assert_eq!(
        get(&fixture, "/repository/pypi-all/simple/absent/", None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let upload = twine(
        &fixture,
        "pypi-all",
        Auth::Admin,
        "demo",
        "3.0",
        "demo-3.0.tar.gz",
        &local_sdist,
    )
    .await;
    assert_eq!(upload.status, StatusCode::NOT_FOUND);
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/pypi-all/packages/demo/3.0/demo-3.0.tar.gz",
            Auth::Admin,
            Body::from(local_sdist.clone()),
            &[],
        )
        .await;
    assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(put.headers[header::ALLOW], "GET,HEAD");
}
