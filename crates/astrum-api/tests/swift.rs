mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "SwiftPMBoundary0123456789abcdef";
const JSON: &str = "application/vnd.swift.registry.v1+json";

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                0xedb8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn zip(files: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, text) in files {
        let offset = out.len() as u32;
        let mut common = vec![20, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        common.extend_from_slice(&crc32(text.as_bytes()).to_le_bytes());
        common.extend_from_slice(&(text.len() as u32).to_le_bytes());
        common.extend_from_slice(&(text.len() as u32).to_le_bytes());
        common.extend_from_slice(&(name.len() as u16).to_le_bytes());
        common.extend_from_slice(&[0, 0]);
        out.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]);
        out.extend_from_slice(&common);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(text.as_bytes());
        central.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02, 20, 0]);
        central.extend_from_slice(&common);
        central.extend_from_slice(&[0; 10]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let start = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

fn source(version: &str) -> Vec<u8> {
    zip(&[
        (
            "LinkedList/Package.swift",
            "// swift-tools-version:5.7\nimport PackageDescription\n",
        ),
        (
            "LinkedList/Package@swift-5.9.swift",
            "// swift-tools-version: 5.9\n",
        ),
        (
            "LinkedList/Sources/LinkedList/List.swift",
            &format!("let version = \"{version}\"\n"),
        ),
    ])
}

fn form(archive: &[u8], metadata: &Value) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"source-archive\"\r\nContent-Type: application/zip\r\nContent-Transfer-Encoding: binary\r\n\r\n").as_bytes());
    body.extend_from_slice(archive);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n{metadata}\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_swift::Swift));
    fixture
}

async fn publish(fixture: &Fixture, repository: &str, version: &str, archive: &[u8]) -> Reply {
    let content_type = format!("multipart/form-data;boundary=\"{BOUNDARY}\"");
    let metadata = json!({
        "description": "One linked list",
        "repositoryURLs": ["https://github.com/mona/LinkedList.git"],
        "author": {"name": "Mona"}
    });
    fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/mona/LinkedList/{version}"),
            Auth::Admin,
            Body::from(form(archive, &metadata)),
            &[("accept", JSON), ("content-type", content_type.as_str())],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str, accept: &str) -> Reply {
    fixture
        .send(Method::GET, path, Auth::Admin, None, &[("accept", accept)])
        .await
}

#[tokio::test]
async fn releases_are_published_listed_and_fetched_through_a_group() {
    let fixture = ready().await;
    for (kind, body) in [
        (
            "hosted",
            json!({"name": "swift-hosted", "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "allow_once"}}),
        ),
        (
            "group",
            json!({"name": "swift-group", "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["swift-hosted"], "writableMember": "swift-hosted"}}),
        ),
    ] {
        let created = fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/repositories/swift/{kind}"),
                Auth::Admin,
                Some(body),
            )
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    }
    let archive = source("1.1.1");
    let published = publish(&fixture, "swift-group", "1.1.1", &archive).await;
    assert_eq!(
        published.status,
        StatusCode::CREATED,
        "{}",
        published.text()
    );
    assert!(
        published.headers["location"]
            .to_str()
            .unwrap()
            .ends_with("/repository/swift-group/mona/LinkedList/1.1.1")
    );
    assert_eq!(published.headers["content-version"], "1");
    let older = publish(&fixture, "swift-hosted", "1.0.0", &source("1.0.0")).await;
    assert_eq!(older.status, StatusCode::CREATED, "{}", older.text());
    let again = publish(&fixture, "swift-hosted", "1.1.1", &archive).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.headers["content-type"], "application/problem+json");
    let broken = publish(&fixture, "swift-hosted", "2.0.0", b"not a zip").await;
    assert_eq!(broken.status, StatusCode::UNPROCESSABLE_ENTITY);

    let releases = get(&fixture, "/repository/swift-group/MONA/linkedlist", JSON).await;
    assert_eq!(releases.status, StatusCode::OK, "{}", releases.text());
    let listed = releases.json();
    let base = listed["releases"]["1.1.1"]["url"].as_str().unwrap();
    assert!(
        base.ends_with("/repository/swift-group/mona/LinkedList/1.1.1"),
        "{base}"
    );
    assert!(listed["releases"]["1.0.0"].is_object());
    assert!(
        releases.headers["link"]
            .to_str()
            .unwrap()
            .contains("rel=\"latest-version\"")
    );

    let release = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList/1.1.1",
        JSON,
    )
    .await;
    let document = release.json();
    assert_eq!(document["id"], "mona.LinkedList");
    assert_eq!(
        document["resources"][0]["checksum"],
        hex::encode(Sha256::digest(&archive))
    );
    assert_eq!(document["metadata"]["author"]["name"], "Mona");
    assert!(
        release.headers["link"]
            .to_str()
            .unwrap()
            .contains("rel=\"predecessor-version\"")
    );

    let manifest = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList/1.1.1/Package.swift",
        "application/vnd.swift.registry.v1+swift",
    )
    .await;
    assert_eq!(manifest.status, StatusCode::OK);
    assert!(manifest.text().starts_with("// swift-tools-version:5.7"));
    assert!(
        manifest.headers["link"]
            .to_str()
            .unwrap()
            .contains("filename=\"Package@swift-5.9.swift\"")
    );
    let variant = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList/1.1.1/Package.swift?swift-version=5.9",
        "application/vnd.swift.registry.v1+swift",
    )
    .await;
    assert!(variant.text().starts_with("// swift-tools-version: 5.9"));
    let other = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList/1.1.1/Package.swift?swift-version=6",
        "application/vnd.swift.registry.v1+swift",
    )
    .await;
    assert_eq!(other.status, StatusCode::SEE_OTHER);

    let zip = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList/1.1.1.zip",
        "application/vnd.swift.registry.v1+zip",
    )
    .await;
    assert_eq!(zip.status, StatusCode::OK);
    assert_eq!(zip.body, archive);
    assert_eq!(
        zip.headers["content-disposition"],
        "attachment; filename=\"LinkedList-1.1.1.zip\""
    );
    assert!(
        zip.headers["digest"]
            .to_str()
            .unwrap()
            .starts_with("sha-256=")
    );

    let identifiers = get(
        &fixture,
        "/repository/swift-group/identifiers?url=https://github.com/mona/LinkedList",
        JSON,
    )
    .await;
    assert_eq!(
        identifiers.json(),
        json!({"identifiers": ["mona.LinkedList"]})
    );
    let unknown = get(
        &fixture,
        "/repository/swift-group/identifiers?url=https://x.org/y",
        JSON,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    let future = get(
        &fixture,
        "/repository/swift-group/mona/LinkedList",
        "application/vnd.swift.registry.v2+json",
    )
    .await;
    assert_eq!(future.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let login = fixture
        .send(
            Method::POST,
            "/repository/swift-hosted/login",
            Auth::Admin,
            None,
            &[("accept", JSON)],
        )
        .await;
    assert_eq!(login.status, StatusCode::OK);
    let stranger = fixture
        .send(
            Method::POST,
            "/repository/swift-hosted/login",
            Auth::None,
            None,
            &[("accept", JSON)],
        )
        .await;
    assert_eq!(stranger.status, StatusCode::UNAUTHORIZED);
    let search = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=swift-hosted&swift.description=One*",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(search["items"].as_array().unwrap().len(), 2, "{search}");
}
