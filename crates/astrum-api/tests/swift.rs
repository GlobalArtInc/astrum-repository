mod support;

use std::sync::Arc;

use astrum_content::store;
use astrum_content::transfer::{Bundle, Conflict, LocalSource, Outcome, PROMOTED, Status, Terms};
use astrum_content::{ContentError, Format};
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
    let links = releases.headers["link"].to_str().unwrap();
    assert!(links.contains("rel=\"latest-version\""), "{links}");
    assert!(
        links.ends_with(", <https://github.com/mona/LinkedList.git>; rel=\"canonical\""),
        "{links}"
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

async fn repository(fixture: &Fixture, name: &str, policy: &str) {
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/swift/hosted",
            Auth::Admin,
            Some(json!({"name": name, "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy}})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
}

async fn publish_signed(fixture: &Fixture, repository: &str, version: &str, archive: &[u8]) {
    let mut body = form(
        archive,
        &json!({"description": "One linked list",
        "repositoryURLs": ["https://github.com/mona/LinkedList.git"], "author": {"name": "Mona"}}),
    );
    let closing = format!("--{BOUNDARY}--\r\n");
    body.truncate(body.len() - closing.len());
    for (name, signature) in [
        ("source-archive-signature", "archive-signature"),
        ("metadata-signature", "metadata-signature"),
    ] {
        body.extend_from_slice(format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n{signature}\r\n").as_bytes());
    }
    body.extend_from_slice(closing.as_bytes());
    let content_type = format!("multipart/form-data;boundary=\"{BOUNDARY}\"");
    let published = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/mona/LinkedList/{version}"),
            Auth::Admin,
            Body::from(body),
            &[
                ("accept", JSON),
                ("content-type", content_type.as_str()),
                ("x-swift-package-signature-format", "cms-1.0.0"),
            ],
        )
        .await;
    assert_eq!(
        published.status,
        StatusCode::CREATED,
        "{}",
        published.text()
    );
}

async fn bundle(fixture: &Fixture, source: &str, version: &str) -> Bundle {
    support::bundle_of(
        fixture,
        &astrum_format_swift::Swift,
        source,
        &format!("/mona/LinkedList/{version}.zip"),
    )
    .await
}

async fn promote(
    fixture: &Fixture,
    source: &str,
    target: &str,
    bundle: &Bundle,
) -> Result<Outcome, ContentError> {
    let from = support::repository_ctx(fixture, source).await;
    let to = support::repository_ctx(fixture, target).await;
    astrum_format_swift::Swift
        .transfer()
        .expect("swift takes part in promotion")
        .receive(
            &to,
            bundle,
            &LocalSource::new(from),
            Terms::new(Conflict::Fail, PROMOTED),
        )
        .await
}

async fn release_of(fixture: &Fixture, repository: &str) -> Value {
    get(
        fixture,
        &format!("/repository/{repository}/mona/LinkedList/1.0.0"),
        JSON,
    )
    .await
    .json()
}

async fn swift_attributes(fixture: &Fixture, repository: &str, version: &str) -> Value {
    let found = store::find(
        fixture.state.db().conn(),
        repository,
        &format!("/mona/LinkedList/{version}.zip"),
    )
    .await
    .unwrap()
    .unwrap();
    store::component_by_id(fixture.state.db().conn(), found.component_id.unwrap())
        .await
        .unwrap()
        .unwrap()
        .attributes["swift"]
        .clone()
}

#[tokio::test]
async fn a_signed_release_is_promoted_with_its_metadata_and_signatures() {
    let fixture = ready().await;
    repository(&fixture, "source", "allow").await;
    repository(&fixture, "target", "allow_once").await;
    let archive = source("1.0.0");
    publish_signed(&fixture, "source", "1.0.0", &archive).await;
    let outcome = promote(
        &fixture,
        "source",
        "target",
        &bundle(&fixture, "source", "1.0.0").await,
    )
    .await
    .unwrap();
    assert_eq!(outcome.status, Status::Copied);

    let (from, to) = (
        swift_attributes(&fixture, "source", "1.0.0").await,
        swift_attributes(&fixture, "target", "1.0.0").await,
    );
    assert_eq!(from, to);
    for field in [
        "metadata",
        "publishedAt",
        "description",
        "repositoryURLs",
        "signing",
        "metadataSigning",
    ] {
        assert!(!to[field].is_null(), "{field} is missing: {to}");
    }
    let mut expected = release_of(&fixture, "source").await;
    expected["id"] = json!("mona.LinkedList");
    assert_eq!(release_of(&fixture, "target").await, expected);
    for (path, accept) in [
        ("1.0.0.zip", "application/vnd.swift.registry.v1+zip"),
        (
            "1.0.0/Package.swift",
            "application/vnd.swift.registry.v1+swift",
        ),
        (
            "1.0.0/Package@swift-5.9.swift",
            "application/vnd.swift.registry.v1+swift",
        ),
    ] {
        let found = |repository: &str| format!("/repository/{repository}/mona/LinkedList/{path}");
        let (a, b) = (
            get(&fixture, &found("source"), accept).await,
            get(&fixture, &found("target"), accept).await,
        );
        assert_eq!(b.status, StatusCode::OK, "{path}");
        assert_eq!(a.body, b.body, "{path}");
    }
    let listing = get(&fixture, "/repository/target/mona/LinkedList", JSON).await;
    assert!(listing.json()["releases"]["1.0.0"].is_object());
}

#[tokio::test]
async fn a_crafted_path_a_duplicate_version_and_a_read_only_target_are_refused() {
    let fixture = ready().await;
    repository(&fixture, "source", "allow").await;
    repository(&fixture, "target", "allow_once").await;
    repository(&fixture, "locked", "deny").await;
    publish_signed(&fixture, "source", "1.0.0", &source("1.0.0")).await;
    let genuine = bundle(&fixture, "source", "1.0.0").await;
    let blobs = || async {
        fixture
            .state
            .content()
            .blob_stores()
            .get("default")
            .await
            .unwrap()
            .list(None, 1000)
            .await
            .unwrap()
            .len()
    };
    let before = blobs().await;

    let mut crafted = genuine.clone();
    crafted
        .assets
        .iter_mut()
        .find(|asset| asset.path.ends_with(".zip"))
        .unwrap()
        .path = "/mona/LinkedList/9.9.9.zip".to_owned();
    let error = promote(&fixture, "source", "target", &crafted)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");

    let error = promote(&fixture, "source", "locked", &genuine)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    for target in ["target", "locked"] {
        assert!(
            store::find(
                fixture.state.db().conn(),
                target,
                "/mona/LinkedList/1.0.0.zip"
            )
            .await
            .unwrap()
            .is_none()
        );
    }
    assert_eq!(blobs().await, before);

    promote(&fixture, "source", "target", &genuine)
        .await
        .unwrap();
    let error = promote(&fixture, "source", "target", &genuine)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
}
