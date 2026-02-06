mod support;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash as _, Hasher};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_maven::metadata::{self, Metadata};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use md5::Md5;
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use support::{Auth, Fixture, Reply};

const JAR: &[u8] = b"PK\x03\x04\x14\x00\x08\x00\x08\x00jar-content";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_maven::Maven));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/maven/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, versions: &str, layout: &str, write: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": write},
        "maven": {"versionPolicy": versions, "layoutPolicy": layout, "contentDisposition": "ATTACHMENT"}
    })
}

fn proxy(name: &str, remote: &str, versions: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 0},
        "negativeCache": {"enabled": false, "timeToLive": 1},
        "httpClient": {"blocked": false, "autoBlock": false},
        "maven": {"versionPolicy": versions, "layoutPolicy": "STRICT", "contentDisposition": "INLINE"}
    })
}

fn pom(group: &str, artifact: &str, version: &str, packaging: &str) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>{group}</groupId>\n  <artifactId>{artifact}</artifactId>\n  <version>{version}</version>\n  <packaging>{packaging}</packaging>\n</project>\n"
    )
    .into_bytes()
}

async fn put(fixture: &Fixture, path: &str, body: impl Into<Vec<u8>>, auth: Auth<'_>) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            auth,
            Body::from(body.into()),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn delete(fixture: &Fixture, path: &str) -> StatusCode {
    fixture
        .call(Method::DELETE, path, Auth::Admin, None)
        .await
        .status
}

async fn metadata_of(fixture: &Fixture, path: &str) -> Metadata {
    let reply = get(fixture, path).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    assert_eq!(reply.headers[header::CONTENT_TYPE], "application/xml");
    metadata::parse(&reply.body).unwrap()
}

fn versions(metadata: &Metadata) -> Vec<&str> {
    metadata
        .versioning
        .as_ref()
        .map(|versioning| versioning.versions.iter().map(String::as_str).collect())
        .unwrap_or_default()
}

fn sha1(bytes: &[u8]) -> String {
    hex::encode(Sha1::digest(bytes))
}

async fn deploy(fixture: &Fixture, repository: &str, group: &str, artifact: &str, version: &str) {
    let directory = format!(
        "/repository/{repository}/{}/{artifact}/{version}",
        group.replace('.', "/")
    );
    let jar = put(
        fixture,
        &format!("{directory}/{artifact}-{version}.jar"),
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(jar.status, StatusCode::CREATED, "{}", jar.text());
    let pom = put(
        fixture,
        &format!("{directory}/{artifact}-{version}.pom"),
        pom(group, artifact, version, "jar"),
        Auth::Admin,
    )
    .await;
    assert_eq!(pom.status, StatusCode::CREATED, "{}", pom.text());
}

#[tokio::test]
async fn hosted_deploys_keep_metadata_and_checksums() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW_ONCE"),
    )
    .await;
    let lib = "/repository/releases/org/example/lib";
    assert_eq!(
        get(&fixture, &format!("{lib}/maven-metadata.xml"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    for version in ["1.0", "1.1", "1.10"] {
        deploy(&fixture, "releases", "org.example", "lib", version).await;
    }
    let jar_path = format!("{lib}/1.0/lib-1.0.jar");
    assert_eq!(
        put(
            &fixture,
            &format!("{jar_path}.sha1"),
            sha1(JAR),
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let refused = put(
        &fixture,
        &format!("{jar_path}.md5"),
        "deadbeef",
        Auth::Admin,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(
        refused.text().contains("Not a Maven2 digest"),
        "{}",
        refused.text()
    );
    let client_metadata = b"<metadata><groupId>org.example</groupId><artifactId>lib</artifactId><versioning><release>9.9</release><versions><version>9.9</version></versions></versioning></metadata>";
    assert_eq!(
        put(
            &fixture,
            &format!("{lib}/maven-metadata.xml"),
            &client_metadata[..],
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let invalid = put(
        &fixture,
        &format!("{lib}/maven-metadata.xml"),
        "garbage <not xml",
        Auth::Admin,
    )
    .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert!(invalid.text().contains("is not a valid maven-metadata.xml"));

    let reply = get(&fixture, &format!("{lib}/maven-metadata.xml")).await;
    assert_eq!(reply.status, StatusCode::OK);
    let generated = metadata::parse(&reply.body).unwrap();
    assert_eq!(generated.group_id.as_deref(), Some("org.example"));
    assert_eq!(generated.artifact_id.as_deref(), Some("lib"));
    assert_eq!(versions(&generated), ["1.0", "1.1", "1.10"]);
    let versioning = generated.versioning.as_ref().unwrap();
    assert_eq!(versioning.latest.as_deref(), Some("1.10"));
    assert_eq!(versioning.release.as_deref(), Some("1.10"));
    assert_eq!(versioning.last_updated.as_ref().unwrap().len(), 14);
    let etag = reply.headers[header::ETAG].to_str().unwrap().to_owned();
    assert_eq!(etag, format!("\"{}\"", sha1(&reply.body)));
    let cached = fixture
        .send(
            Method::GET,
            &format!("{lib}/maven-metadata.xml"),
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(cached.status, StatusCode::NOT_MODIFIED);
    let checksum = get(&fixture, &format!("{lib}/maven-metadata.xml.sha1")).await;
    assert_eq!(checksum.status, StatusCode::OK);
    assert_eq!(checksum.text(), sha1(&reply.body));
    assert_eq!(
        get(&fixture, &format!("{lib}/maven-metadata.xml.md5"))
            .await
            .text(),
        hex::encode(Md5::digest(&reply.body))
    );

    assert_eq!(
        get(&fixture, &format!("{jar_path}.sha1")).await.text(),
        sha1(JAR)
    );
    assert_eq!(
        get(&fixture, &format!("{jar_path}.sha256")).await.text(),
        hex::encode(Sha256::digest(JAR))
    );
    assert_eq!(
        get(&fixture, &format!("{jar_path}.sha512"))
            .await
            .text()
            .len(),
        128
    );
    let pom_md5 = get(&fixture, &format!("{lib}/1.1/lib-1.1.pom.md5")).await;
    assert_eq!(pom_md5.status, StatusCode::OK);
    assert_eq!(
        pom_md5.text(),
        hex::encode(Md5::digest(pom("org.example", "lib", "1.1", "jar")))
    );
    assert_eq!(
        put(
            &fixture,
            &format!("{jar_path}.sha1"),
            "0".repeat(40),
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(&fixture, &format!("{jar_path}.sha1")).await.text(),
        sha1(JAR)
    );
    assert_eq!(
        get(&fixture, &format!("{lib}/1.0/lib-1.0-sources.jar.sha1"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let again = put(&fixture, &jar_path, JAR, Auth::Admin).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(
        put(
            &fixture,
            &format!("{lib}/maven-metadata.xml"),
            &client_metadata[..],
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );

    let jar = get(&fixture, &jar_path).await;
    assert_eq!(jar.body, JAR);
    assert_eq!(jar.headers[header::CONTENT_DISPOSITION], "attachment");
    assert_eq!(
        jar.headers[header::CONTENT_TYPE],
        "application/java-archive"
    );
    let pom_reply = get(&fixture, &format!("{lib}/1.0/lib-1.0.pom")).await;
    assert_eq!(pom_reply.headers[header::CONTENT_TYPE], "application/xml");
    assert_eq!(
        put(
            &fixture,
            &format!("{lib}/1.0/lib-1.0.module"),
            "{\"formatVersion\": \"1.1\"}",
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(&fixture, &format!("{lib}/1.0/lib-1.0.module"))
            .await
            .headers[header::CONTENT_TYPE],
        "application/json"
    );
    let head = fixture
        .call(
            Method::HEAD,
            &format!("{lib}/maven-metadata.xml"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());

    assert_eq!(
        delete(&fixture, &format!("{lib}/1.10/lib-1.10.jar")).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        versions(&metadata_of(&fixture, &format!("{lib}/maven-metadata.xml")).await),
        ["1.0", "1.1", "1.10"]
    );
    assert_eq!(
        delete(&fixture, &format!("{lib}/1.10/lib-1.10.pom")).await,
        StatusCode::NO_CONTENT
    );
    let after = metadata_of(&fixture, &format!("{lib}/maven-metadata.xml")).await;
    assert_eq!(versions(&after), ["1.0", "1.1"]);
    assert_eq!(after.versioning.unwrap().latest.as_deref(), Some("1.1"));
    assert_eq!(
        delete(&fixture, &format!("{lib}/1.10/lib-1.10.pom")).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&fixture, &format!("{jar_path}.sha1")).await,
        StatusCode::NO_CONTENT
    );

    let post = fixture
        .call(Method::POST, &jar_path, Auth::Admin, None)
        .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(post.headers[header::ALLOW], "GET,HEAD,PUT,DELETE");
    let directory = get(&fixture, &format!("{lib}/")).await;
    assert_eq!(directory.status, StatusCode::OK);
    assert!(
        directory
            .text()
            .contains("/service/rest/repository/browse/releases/org/example/lib/")
    );
    let catalog = get(&fixture, "/repository/releases/archetype-catalog.xml").await;
    assert_eq!(catalog.status, StatusCode::OK);
    assert!(catalog.text().contains("<archetype-catalog"));
    assert_eq!(
        put(
            &fixture,
            "/repository/releases/archetype-catalog.xml",
            "x",
            Auth::Admin
        )
        .await
        .status,
        StatusCode::METHOD_NOT_ALLOWED
    );
}

#[tokio::test]
async fn version_layout_and_write_policies_are_enforced() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "hosted",
        hosted("snapshots", "SNAPSHOT", "STRICT", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "hosted",
        hosted("mixed", "MIXED", "PERMISSIVE", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "hosted",
        hosted("frozen", "MIXED", "STRICT", "DENY"),
    )
    .await;

    let snapshot = put(
        &fixture,
        "/repository/releases/org/x/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(snapshot.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        snapshot.text(),
        "Repository version policy: RELEASE does not allow version: 1.0-SNAPSHOT"
    );
    let stamped = put(
        &fixture,
        "/repository/releases/org/x/lib/1.0-SNAPSHOT/lib-1.0-20261012.101010-3.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(stamped.status, StatusCode::BAD_REQUEST);
    let release = put(
        &fixture,
        "/repository/snapshots/org/x/lib/1.0/lib-1.0.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(release.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        release.text(),
        "Repository version policy: SNAPSHOT does not allow version: 1.0"
    );
    let snapshot_metadata = put(
        &fixture,
        "/repository/releases/org/x/lib/1.0-SNAPSHOT/maven-metadata.xml",
        "<metadata/>",
        Auth::Admin,
    )
    .await;
    assert_eq!(snapshot_metadata.status, StatusCode::BAD_REQUEST);
    assert!(
        snapshot_metadata
            .text()
            .contains("does not allow metadata in path")
    );
    let read = get(
        &fixture,
        "/repository/releases/org/x/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.jar",
    )
    .await;
    assert_eq!(read.status, StatusCode::NOT_FOUND);
    assert!(
        read.text()
            .starts_with("Repository version policy: RELEASE")
    );

    for strict in [
        "/repository/releases/foo.txt",
        "/repository/releases/.index/repository-index.properties",
        "/repository/releases/org/x/lib/1.0/other-1.0.jar",
        "/repository/releases/org/x/lib/1.0/lib-1.0",
    ] {
        let reply = put(&fixture, strict, "text", Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{strict}");
        assert!(
            reply
                .text()
                .starts_with("Invalid path for a Maven 2 repository")
        );
    }
    assert_eq!(
        put(&fixture, "/repository/mixed/foo.txt", "text", Auth::Admin)
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(&fixture, "/repository/mixed/foo.txt").await.text(),
        "text"
    );
    deploy(&fixture, "mixed", "org.x", "lib", "1.0").await;
    let mixed_snapshot = put(
        &fixture,
        "/repository/mixed/org/x/lib/2.0-SNAPSHOT/lib-2.0-SNAPSHOT.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(mixed_snapshot.status, StatusCode::CREATED);
    assert_eq!(
        versions(&metadata_of(&fixture, "/repository/mixed/org/x/lib/maven-metadata.xml").await),
        ["1.0", "2.0-SNAPSHOT"]
    );
    let listed = fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=mixed",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let other = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["path"] == "/foo.txt")
        .unwrap();
    assert_eq!(other["maven2"], json!({}));

    let mismatch = put(
        &fixture,
        "/repository/releases/org/x/lib/1.0/lib-1.0.jar",
        "plain text, not a jar",
        Auth::Admin,
    )
    .await;
    assert_eq!(mismatch.status, StatusCode::BAD_REQUEST);
    assert!(mismatch.text().starts_with("Detected content type"));

    for path in [
        "/repository/frozen/org/x/lib/1.0/lib-1.0.jar",
        "/repository/frozen/org/x/lib/1.0/lib-1.0.jar.sha1",
        "/repository/frozen/org/x/lib/maven-metadata.xml",
    ] {
        let reply = put(&fixture, path, sha1(JAR), Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(reply.text().contains("read-only"));
    }
}

#[tokio::test]
async fn snapshot_deploys_build_snapshot_metadata() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("snapshots", "SNAPSHOT", "STRICT", "ALLOW"),
    )
    .await;
    let directory = "/repository/snapshots/org/example/app/1.0-SNAPSHOT";
    assert_eq!(
        get(&fixture, &format!("{directory}/maven-metadata.xml"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    for (file, body) in [
        ("app-1.0-20261012.101010-1.jar", JAR.to_vec()),
        (
            "app-1.0-20261012.101010-1.pom",
            pom("org.example", "app", "1.0-SNAPSHOT", "jar"),
        ),
        ("app-1.0-20261012.101010-1-sources.jar", JAR.to_vec()),
    ] {
        let reply = put(&fixture, &format!("{directory}/{file}"), body, Auth::Admin).await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{file}: {}",
            reply.text()
        );
    }
    let first = metadata_of(&fixture, &format!("{directory}/maven-metadata.xml")).await;
    assert_eq!(first.version.as_deref(), Some("1.0-SNAPSHOT"));
    assert_eq!(first.artifact_id.as_deref(), Some("app"));
    let versioning = first.versioning.unwrap();
    let snapshot = versioning.snapshot.unwrap();
    assert_eq!(snapshot.timestamp.as_deref(), Some("20261012.101010"));
    assert_eq!(snapshot.build_number, Some(1));
    assert_eq!(versioning.snapshot_versions.len(), 3);
    assert!(
        versioning
            .snapshot_versions
            .iter()
            .all(|entry| entry.value == "1.0-20261012.101010-1")
    );

    for (file, body) in [
        ("app-1.0-20261013.080000-2.jar", JAR.to_vec()),
        (
            "app-1.0-20261013.080000-2.pom",
            pom("org.example", "app", "1.0-SNAPSHOT", "jar"),
        ),
    ] {
        let reply = put(&fixture, &format!("{directory}/{file}"), body, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{file}");
    }
    let second = metadata_of(&fixture, &format!("{directory}/maven-metadata.xml")).await;
    let versioning = second.versioning.unwrap();
    assert_eq!(versioning.snapshot.unwrap().build_number, Some(2));
    let value = |classifier: Option<&str>, extension: &str| {
        versioning
            .snapshot_versions
            .iter()
            .find(|entry| entry.classifier.as_deref() == classifier && entry.extension == extension)
            .map(|entry| entry.value.clone())
    };
    assert_eq!(value(None, "jar").as_deref(), Some("1.0-20261013.080000-2"));
    assert_eq!(value(None, "pom").as_deref(), Some("1.0-20261013.080000-2"));
    assert_eq!(
        value(Some("sources"), "jar").as_deref(),
        Some("1.0-20261012.101010-1")
    );
    let artifact_level = metadata_of(
        &fixture,
        "/repository/snapshots/org/example/app/maven-metadata.xml",
    )
    .await;
    assert_eq!(versions(&artifact_level), ["1.0-SNAPSHOT"]);
    let artifact_versioning = artifact_level.versioning.unwrap();
    assert_eq!(artifact_versioning.latest.as_deref(), Some("1.0-SNAPSHOT"));
    assert_eq!(artifact_versioning.release, None);

    let builds = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=snapshots&maven.baseVersion=1.0-SNAPSHOT&sort=version",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let found: Vec<&str> = builds["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["version"].as_str().unwrap())
        .collect();
    assert_eq!(found.len(), 2, "{builds}");
    assert!(found.contains(&"1.0-20261012.101010-1"));
    assert!(found.contains(&"1.0-20261013.080000-2"));

    for file in [
        "app-1.0-20261013.080000-2.jar",
        "app-1.0-20261013.080000-2.pom",
    ] {
        assert_eq!(
            delete(&fixture, &format!("{directory}/{file}")).await,
            StatusCode::NO_CONTENT
        );
    }
    let back = metadata_of(&fixture, &format!("{directory}/maven-metadata.xml")).await;
    assert_eq!(
        back.versioning.unwrap().snapshot.unwrap().build_number,
        Some(1)
    );

    let plain = put(
        &fixture,
        "/repository/snapshots/org/example/app/2.0-SNAPSHOT/app-2.0-SNAPSHOT.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(plain.status, StatusCode::CREATED);
    let plain_metadata = metadata_of(
        &fixture,
        "/repository/snapshots/org/example/app/2.0-SNAPSHOT/maven-metadata.xml",
    )
    .await;
    let versioning = plain_metadata.versioning.unwrap();
    assert_eq!(versioning.snapshot.unwrap().timestamp, None);
    assert_eq!(versioning.snapshot_versions[0].value, "2.0-SNAPSHOT");
    assert_eq!(
        versions(
            &metadata_of(
                &fixture,
                "/repository/snapshots/org/example/app/maven-metadata.xml"
            )
            .await
        ),
        ["1.0-SNAPSHOT", "2.0-SNAPSHOT"]
    );
}

async fn search(fixture: &Fixture, query: &str) -> Value {
    let reply = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/{query}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{query}: {}", reply.text());
    reply.json()
}

#[tokio::test]
async fn deleting_through_the_api_updates_metadata() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW"),
    )
    .await;
    deploy(&fixture, "releases", "com.acme", "tool", "1.0").await;
    deploy(&fixture, "releases", "com.acme", "tool", "2.0").await;
    let metadata_path = "/repository/releases/com/acme/tool/maven-metadata.xml";
    assert_eq!(
        versions(&metadata_of(&fixture, metadata_path).await),
        ["1.0", "2.0"]
    );

    let found = search(
        &fixture,
        "search?repository=releases&maven.artifactId=tool&version=2.0",
    )
    .await;
    let id = found["items"][0]["id"].as_str().unwrap().to_owned();
    assert_eq!(
        delete(&fixture, &format!("/service/rest/v1/components/{id}")).await,
        StatusCode::NO_CONTENT
    );
    let after = metadata_of(&fixture, metadata_path).await;
    assert_eq!(versions(&after), ["1.0"]);
    assert_eq!(after.versioning.unwrap().release.as_deref(), Some("1.0"));

    let assets = search(&fixture, "search/assets?repository=releases&version=1.0").await;
    for asset in assets["items"].as_array().unwrap() {
        let id = asset["id"].as_str().unwrap();
        assert_eq!(
            delete(&fixture, &format!("/service/rest/v1/assets/{id}")).await,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        get(&fixture, metadata_path).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, &format!("{metadata_path}.sha1")).await.status,
        StatusCode::NOT_FOUND
    );
}

type Hits = Arc<Mutex<Vec<(String, Option<String>)>>>;

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Hits,
    credentials: Option<String>,
}

impl Upstream {
    fn set(&self, path: &str, body: impl Into<Vec<u8>>) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_owned(), body.into());
    }

    fn hits(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(hit, _)| hit == path)
            .count()
    }

    fn authorizations(&self) -> Vec<Option<String>> {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .map(|(_, authorization)| authorization.clone())
            .collect()
    }
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    upstream
        .hits
        .lock()
        .unwrap()
        .push((path.clone(), authorization.clone()));
    if let Some(expected) = &upstream.credentials
        && authorization.as_ref() != Some(expected)
    {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"upstream\"")],
        )
            .into_response();
    }
    let Some(body) = upstream.files.lock().unwrap().get(&path).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    let etag = format!("\"{:x}\"", hasher.finish());
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        == Some(etag.as_str())
    {
        return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
    }
    (
        StatusCode::OK,
        [
            (header::ETAG, etag),
            (header::CONTENT_TYPE, "application/octet-stream".to_owned()),
        ],
        body,
    )
        .into_response()
}

async fn start_upstream(upstream: Upstream) -> SocketAddr {
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
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

fn remote_metadata(versions: &[&str], last_updated: &str) -> String {
    let listed: String = versions
        .iter()
        .map(|version| format!("<version>{version}</version>"))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata><groupId>org.example</groupId><artifactId>lib</artifactId><versioning><latest>{}</latest><versions>{listed}</versions><lastUpdated>{last_updated}</lastUpdated></versioning></metadata>",
        versions.last().unwrap()
    )
}

#[tokio::test]
async fn proxies_cache_artifacts_and_refresh_metadata() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let upstream = Upstream::default();
    upstream.set("org/example/lib/0.9/lib-0.9.jar", JAR);
    upstream.set(
        "org/example/lib/0.9/lib-0.9.pom",
        pom("org.example", "lib", "0.9", "jar"),
    );
    upstream.set(
        "org/example/lib/maven-metadata.xml",
        remote_metadata(&["0.9"], "20250101000000"),
    );
    upstream.set("org/example/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.jar", JAR);
    upstream.set("robots.txt", "no");
    let address = start_upstream(upstream.clone()).await;
    create(
        &fixture,
        "proxy",
        proxy("central", &format!("http://{address}/"), "RELEASE"),
    )
    .await;
    let base = "/repository/central/org/example/lib";

    let jar = get(&fixture, &format!("{base}/0.9/lib-0.9.jar")).await;
    assert_eq!(jar.status, StatusCode::OK, "{}", jar.text());
    assert_eq!(jar.body, JAR);
    assert_eq!(jar.headers[header::CONTENT_DISPOSITION], "inline");
    assert_eq!(
        get(&fixture, &format!("{base}/0.9/lib-0.9.jar")).await.body,
        JAR
    );
    assert_eq!(upstream.hits("org/example/lib/0.9/lib-0.9.jar"), 1);
    assert_eq!(
        get(&fixture, &format!("{base}/0.9/lib-0.9.jar.sha1"))
            .await
            .text(),
        sha1(JAR)
    );
    assert_eq!(upstream.hits("org/example/lib/0.9/lib-0.9.jar"), 1);
    assert_eq!(upstream.hits("org/example/lib/0.9/lib-0.9.jar.sha1"), 0);
    let pom_sha1 = get(&fixture, &format!("{base}/0.9/lib-0.9.pom.sha1")).await;
    assert_eq!(
        pom_sha1.text(),
        sha1(&pom("org.example", "lib", "0.9", "jar"))
    );

    let first = get(&fixture, &format!("{base}/maven-metadata.xml")).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(versions(&metadata::parse(&first.body).unwrap()), ["0.9"]);
    upstream.set(
        "org/example/lib/maven-metadata.xml",
        remote_metadata(&["0.9", "1.0"], "20260101000000"),
    );
    let refreshed = get(&fixture, &format!("{base}/maven-metadata.xml")).await;
    assert_eq!(
        versions(&metadata::parse(&refreshed.body).unwrap()),
        ["0.9", "1.0"]
    );
    assert_eq!(
        get(&fixture, &format!("{base}/maven-metadata.xml.sha1"))
            .await
            .text(),
        sha1(&refreshed.body)
    );
    assert!(upstream.hits("org/example/lib/maven-metadata.xml") >= 2);

    let snapshot = get(
        &fixture,
        &format!("{base}/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.jar"),
    )
    .await;
    assert_eq!(snapshot.status, StatusCode::NOT_FOUND);
    assert!(snapshot.text().contains("RELEASE does not allow"));
    assert_eq!(
        upstream.hits("org/example/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.jar"),
        0
    );
    assert_eq!(
        get(&fixture, "/repository/central/robots.txt").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(upstream.hits("robots.txt"), 0);
    assert_eq!(
        get(&fixture, &format!("{base}/0.8/lib-0.8.jar"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let refused = put(
        &fixture,
        &format!("{base}/2.0/lib-2.0.jar"),
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(refused.headers[header::ALLOW], "GET,HEAD,DELETE");

    let found = search(
        &fixture,
        "search?repository=central&maven.groupId=org.example",
    )
    .await;
    assert_eq!(found["items"].as_array().unwrap().len(), 1);
    assert_eq!(found["items"][0]["version"], "0.9");
    let kinds: Vec<&str> = found["items"][0]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["maven2"]["extension"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.len(), 2);

    assert_eq!(
        delete(&fixture, &format!("{base}/maven-metadata.xml")).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete(&fixture, &format!("{base}/0.9/lib-0.9.jar")).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&fixture, &format!("{base}/0.9/lib-0.9.jar")).await.body,
        JAR
    );
    assert_eq!(upstream.hits("org/example/lib/0.9/lib-0.9.jar"), 2);
}

#[tokio::test]
async fn proxies_authenticate_preemptively_only_when_asked() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let expected = format!("Basic {}", STANDARD.encode("mirror-user:mirror-pass"));
    let upstream = Upstream {
        credentials: Some(expected.clone()),
        ..Default::default()
    };
    upstream.set("org/example/lib/0.9/lib-0.9.jar", JAR);
    let address = start_upstream(upstream.clone()).await;
    for (name, preemptive) in [("eager", true), ("lazy", false)] {
        let mut body = proxy(name, &format!("http://{address}/"), "RELEASE");
        body["httpClient"]["authentication"] = json!({
            "type": "username", "username": "mirror-user", "password": "mirror-pass", "preemptive": preemptive
        });
        create(&fixture, "proxy", body).await;
    }
    let eager = get(
        &fixture,
        "/repository/eager/org/example/lib/0.9/lib-0.9.jar",
    )
    .await;
    assert_eq!(eager.status, StatusCode::OK, "{}", eager.text());
    assert_eq!(upstream.authorizations(), [Some(expected.clone())]);
    let lazy = get(&fixture, "/repository/lazy/org/example/lib/0.9/lib-0.9.jar").await;
    assert_eq!(lazy.status, StatusCode::OK, "{}", lazy.text());
    assert_eq!(
        upstream.authorizations(),
        [Some(expected.clone()), None, Some(expected)]
    );
}

#[tokio::test]
async fn groups_merge_metadata_across_members() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "hosted",
        hosted("snapshots", "SNAPSHOT", "STRICT", "ALLOW"),
    )
    .await;
    let upstream = Upstream::default();
    upstream.set(
        "org/example/lib/maven-metadata.xml",
        remote_metadata(&["0.9", "1.0"], "20300101000000"),
    );
    upstream.set(
        "org/example/lib/0.9/lib-0.9.jar",
        b"PK\x03\x04remote".to_vec(),
    );
    upstream.set(
        "org/example/lib/1.0/lib-1.0.jar",
        b"PK\x03\x04shadowed".to_vec(),
    );
    let address = start_upstream(upstream.clone()).await;
    create(
        &fixture,
        "proxy",
        proxy("central", &format!("http://{address}/"), "RELEASE"),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "public",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["releases", "snapshots", "central"]}
        }),
    )
    .await;
    deploy(&fixture, "releases", "org.example", "lib", "1.0").await;
    let snapshot = put(
        &fixture,
        "/repository/snapshots/org/example/lib/2.0-SNAPSHOT/lib-2.0-20261012.101010-1.jar",
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(snapshot.status, StatusCode::CREATED);

    let base = "/repository/public/org/example/lib";
    let merged_reply = get(&fixture, &format!("{base}/maven-metadata.xml")).await;
    assert_eq!(merged_reply.status, StatusCode::OK);
    let merged = metadata::parse(&merged_reply.body).unwrap();
    assert_eq!(versions(&merged), ["0.9", "1.0", "2.0-SNAPSHOT"]);
    let versioning = merged.versioning.unwrap();
    assert_eq!(versioning.latest.as_deref(), Some("2.0-SNAPSHOT"));
    assert_eq!(versioning.release.as_deref(), Some("1.0"));
    assert_eq!(versioning.last_updated.as_deref(), Some("20300101000000"));
    assert_eq!(
        get(&fixture, &format!("{base}/maven-metadata.xml.sha1"))
            .await
            .text(),
        sha1(&merged_reply.body)
    );
    assert_eq!(
        get(&fixture, &format!("{base}/maven-metadata.xml.md5"))
            .await
            .text(),
        hex::encode(Md5::digest(&merged_reply.body))
    );

    assert_eq!(
        get(&fixture, &format!("{base}/1.0/lib-1.0.jar")).await.body,
        JAR
    );
    assert_eq!(upstream.hits("org/example/lib/1.0/lib-1.0.jar"), 0);
    assert_eq!(
        get(&fixture, &format!("{base}/0.9/lib-0.9.jar")).await.body,
        b"PK\x03\x04remote"
    );
    assert_eq!(
        get(&fixture, &format!("{base}/0.9/lib-0.9.jar.sha1"))
            .await
            .text(),
        sha1(b"PK\x03\x04remote")
    );
    let snapshot_metadata =
        metadata_of(&fixture, &format!("{base}/2.0-SNAPSHOT/maven-metadata.xml")).await;
    assert_eq!(
        snapshot_metadata
            .versioning
            .unwrap()
            .snapshot
            .unwrap()
            .build_number,
        Some(1)
    );
    assert_eq!(
        get(&fixture, &format!("{base}/3.0/lib-3.0.jar"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/public/org/nothing/maven-metadata.xml"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let catalog = get(&fixture, "/repository/public/archetype-catalog.xml").await;
    assert_eq!(catalog.status, StatusCode::OK);
    let refused = put(
        &fixture,
        &format!("{base}/1.0/lib-1.0.jar"),
        JAR,
        Auth::Admin,
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(refused.headers[header::ALLOW], "GET,HEAD");
}

type Part<'a> = (&'a str, Option<&'a str>, &'a [u8]);

fn multipart(parts: &[Part<'_>]) -> (String, Vec<u8>) {
    let boundary = "MAVENBOUNDARY";
    let mut body = Vec::new();
    for (name, filename, content) in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match filename {
            Some(filename) => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes(),
            ),
            None => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
            ),
        }
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

async fn upload(fixture: &Fixture, repository: &str, parts: &[Part<'_>]) -> Reply {
    let (content_type, body) = multipart(parts);
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(body),
            &[("content-type", &content_type)],
        )
        .await
}

#[tokio::test]
async fn uploads_through_the_components_api() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW_ONCE"),
    )
    .await;
    create(
        &fixture,
        "hosted",
        hosted("snapshots", "SNAPSHOT", "STRICT", "ALLOW"),
    )
    .await;
    let reply = upload(
        &fixture,
        "releases",
        &[
            ("maven2.groupId", None, b"com.acme"),
            ("maven2.artifactId", None, b"tool"),
            ("maven2.version", None, b"2.1"),
            ("maven2.generate-pom", None, b"true"),
            ("maven2.packaging", None, b"jar"),
            ("maven2.asset1", Some("tool.jar"), JAR),
            ("maven2.asset1.extension", None, b"jar"),
            ("maven2.asset2", Some("tool-sources.jar"), JAR),
            ("maven2.asset2.extension", None, b"jar"),
            ("maven2.asset2.classifier", None, b"sources"),
        ],
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let base = "/repository/releases/com/acme/tool";
    assert_eq!(
        get(&fixture, &format!("{base}/2.1/tool-2.1.jar"))
            .await
            .body,
        JAR
    );
    assert_eq!(
        get(&fixture, &format!("{base}/2.1/tool-2.1-sources.jar"))
            .await
            .status,
        StatusCode::OK
    );
    let generated = get(&fixture, &format!("{base}/2.1/tool-2.1.pom")).await;
    assert_eq!(generated.status, StatusCode::OK);
    let parsed = astrum_format_maven::pom::parse(&generated.body).unwrap();
    assert_eq!(parsed.group_id.as_deref(), Some("com.acme"));
    assert_eq!(parsed.packaging, "jar");
    assert_eq!(
        get(&fixture, &format!("{base}/2.1/tool-2.1.jar.sha1"))
            .await
            .text(),
        sha1(JAR)
    );
    assert_eq!(
        versions(&metadata_of(&fixture, &format!("{base}/maven-metadata.xml")).await),
        ["2.1"]
    );

    let again = upload(
        &fixture,
        "releases",
        &[
            ("maven2.groupId", None, b"com.acme"),
            ("maven2.artifactId", None, b"tool"),
            ("maven2.version", None, b"2.1"),
            ("maven2.asset1", Some("tool.jar"), JAR),
            ("maven2.asset1.extension", None, b"jar"),
        ],
    )
    .await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert!(
        again.text().contains("redeploy is not allowed"),
        "{}",
        again.text()
    );

    let from_pom = upload(
        &fixture,
        "releases",
        &[
            ("maven2.groupId", None, b"ignored.group"),
            ("maven2.asset1", Some("lib.jar"), JAR),
            ("maven2.asset1.extension", None, b"jar"),
            (
                "maven2.asset2",
                Some("lib.pom"),
                &pom("net.pom", "frompom", "3.0", "jar"),
            ),
            ("maven2.asset2.extension", None, b"pom"),
        ],
    )
    .await;
    assert_eq!(
        from_pom.status,
        StatusCode::NO_CONTENT,
        "{}",
        from_pom.text()
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/releases/net/pom/frompom/3.0/frompom-3.0.jar"
        )
        .await
        .status,
        StatusCode::OK
    );

    let cases: Vec<(Vec<Part<'_>>, &str)> = vec![
        (
            vec![
                ("maven2.groupId", None, b"com.acme"),
                ("maven2.artifactId", None, b"tool"),
                ("maven2.version", None, b"3.0-SNAPSHOT"),
                ("maven2.asset1", Some("tool.jar"), JAR),
                ("maven2.asset1.extension", None, b"jar"),
            ],
            "Version policy mismatch, cannot upload SNAPSHOT content to RELEASE repositories for file 'com/acme/tool/3.0-SNAPSHOT/tool-3.0-SNAPSHOT.jar'",
        ),
        (
            vec![
                ("maven2.groupId", None, b"com.acme"),
                ("maven2.artifactId", None, b"tool"),
                ("maven2.version", None, b"2.3"),
                ("maven2.asset1", Some("tool.jar"), JAR),
            ],
            "Missing required asset field 'Extension' on '1'",
        ),
        (
            vec![
                ("maven2.asset1", Some("tool.jar"), JAR),
                ("maven2.asset1.extension", None, b"jar"),
            ],
            "Missing required component field 'Group ID'; Missing required component field 'Artifact ID'; Missing required component field 'Version'",
        ),
        (
            vec![
                ("maven2.groupId", None, b"com.acme"),
                ("maven2.artifactId", None, b"dup"),
                ("maven2.version", None, b"1.0"),
                ("maven2.asset1", Some("a.jar"), JAR),
                ("maven2.asset1.extension", None, b"jar"),
                ("maven2.asset2", Some("b.jar"), JAR),
                ("maven2.asset2.extension", None, b"jar"),
            ],
            "The assets 1 and 2 have identical coordinates",
        ),
        (
            vec![
                ("maven2.groupId", None, b"com.acme"),
                ("maven2.artifactId", None, b"a b"),
                ("maven2.version", None, b"1.0"),
                ("maven2.asset1", Some("a.jar"), JAR),
                ("maven2.asset1.extension", None, b"jar"),
            ],
            "The coordinates do not make a valid Maven path",
        ),
    ];
    for (parts, message) in cases {
        let reply = upload(&fixture, "releases", &parts).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{message}");
        assert!(
            reply.text().contains(message),
            "{} / {message}",
            reply.text()
        );
    }
    let snapshots = upload(
        &fixture,
        "snapshots",
        &[
            ("maven2.groupId", None, b"com.acme"),
            ("maven2.artifactId", None, b"tool"),
            ("maven2.version", None, b"1.0-SNAPSHOT"),
            ("maven2.asset1", Some("tool.jar"), JAR),
            ("maven2.asset1.extension", None, b"jar"),
        ],
    )
    .await;
    assert_eq!(snapshots.status, StatusCode::BAD_REQUEST);
    assert!(
        snapshots
            .text()
            .contains("Upload to snapshot repositories not supported")
    );

    let spec = fixture
        .call(
            Method::GET,
            "/service/rest/v1/formats/maven2/upload-specs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let field = |name: &str, kind: &str, optional: bool, group: Value| {
        json!({"name": name, "type": kind, "description": null, "optional": optional,
            "group": group, "groupHelpText": null, "options": null})
    };
    let coordinates = json!("Component coordinates");
    assert_eq!(
        spec,
        json!({
            "format": "maven2",
            "multipleUpload": true,
            "componentFields": [
                field("groupId", "STRING", false, coordinates.clone()),
                field("artifactId", "STRING", false, coordinates.clone()),
                field("version", "STRING", false, coordinates.clone()),
                field("generate-pom", "BOOLEAN", true, coordinates.clone()),
                field("packaging", "STRING", true, coordinates),
            ],
            "assetFields": [
                field("classifier", "STRING", true, Value::Null),
                field("extension", "STRING", false, Value::Null),
                field("asset", "FILE", false, Value::Null),
            ]
        })
    );
}

#[tokio::test]
async fn search_finds_components_by_maven_keys() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "MIXED", "STRICT", "ALLOW"),
    )
    .await;
    deploy(&fixture, "releases", "org.example", "lib", "1.0").await;
    deploy(&fixture, "releases", "com.acme", "tool", "2.1").await;
    assert_eq!(
        put(
            &fixture,
            "/repository/releases/org/example/lib/1.0/lib-1.0-sources.jar",
            JAR,
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/releases/org/example/lib/1.1-SNAPSHOT/lib-1.1-20261012.101010-1.jar",
            JAR,
            Auth::Admin
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let names = |found: &Value| -> Vec<String> {
        found["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                format!(
                    "{}:{}:{}",
                    item["group"].as_str().unwrap(),
                    item["name"].as_str().unwrap(),
                    item["version"].as_str().unwrap()
                )
            })
            .collect()
    };
    assert_eq!(
        names(
            &search(
                &fixture,
                "search?maven.groupId=org.example&maven.artifactId=lib&sort=version"
            )
            .await
        ),
        [
            "org.example:lib:1.1-20261012.101010-1",
            "org.example:lib:1.0"
        ]
    );
    assert_eq!(
        names(
            &search(
                &fixture,
                "search?maven.groupId=org.example&maven.artifactId=lib&sort=version&direction=asc"
            )
            .await
        ),
        [
            "org.example:lib:1.0",
            "org.example:lib:1.1-20261012.101010-1"
        ]
    );
    assert_eq!(
        names(&search(&fixture, "search?maven.baseVersion=1.1-SNAPSHOT").await),
        ["org.example:lib:1.1-20261012.101010-1"]
    );
    assert_eq!(
        names(&search(&fixture, "search?maven.groupId=com.*").await),
        ["com.acme:tool:2.1"]
    );
    let sources = search(&fixture, "search/assets?maven.classifier=sources").await;
    let items = sources["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["path"], "/org/example/lib/1.0/lib-1.0-sources.jar");
    assert_eq!(
        items[0]["maven2"],
        json!({"groupId": "org.example", "artifactId": "lib", "version": "1.0",
            "baseVersion": "1.0", "classifier": "sources", "extension": "jar"})
    );
    let poms = search(
        &fixture,
        "search/assets?maven.extension=pom&repository=releases",
    )
    .await;
    assert_eq!(poms["items"].as_array().unwrap().len(), 2);
    let plain = search(
        &fixture,
        "search/assets?maven.artifactId=lib&maven.extension=jar&maven.classifier=",
    )
    .await;
    let paths: Vec<&str> = plain["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "/org/example/lib/1.0/lib-1.0.jar",
            "/org/example/lib/1.1-SNAPSHOT/lib-1.1-20261012.101010-1.jar"
        ]
    );
    let download = |query: &str| {
        format!(
            "/service/rest/v1/search/assets/download?maven.artifactId=lib&maven.extension=jar&maven.classifier=&{query}"
        )
    };
    let ambiguous = fixture
        .call(
            Method::GET,
            &download("repository=releases"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(ambiguous.status, StatusCode::BAD_REQUEST);
    let latest = fixture
        .call(
            Method::GET,
            &download("repository=releases&sort=version"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(latest.status, StatusCode::FOUND, "{}", latest.text());
    assert!(
        latest.headers[header::LOCATION]
            .to_str()
            .unwrap()
            .ends_with(
                "/repository/releases/org/example/lib/1.1-SNAPSHOT/lib-1.1-20261012.101010-1.jar"
            )
    );
    let release = fixture
        .call(
            Method::GET,
            &download("repository=releases&maven.baseVersion=1.0"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(release.status, StatusCode::FOUND, "{}", release.text());
    let component = search(&fixture, "search?maven.artifactId=tool").await;
    assert_eq!(component["items"][0]["group"], "com.acme");
    assert_eq!(component["items"][0]["format"], "maven2");
    assert_eq!(component["items"][0]["assets"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn content_permissions_cover_artifacts_metadata_and_checksums() {
    let fixture = ready().await;
    create(
        &fixture,
        "hosted",
        hosted("releases", "RELEASE", "STRICT", "ALLOW"),
    )
    .await;
    deploy(&fixture, "releases", "org.example", "lib", "1.0").await;
    let jar = "/repository/releases/org/example/lib/1.0/lib-1.0.jar";
    let anonymous = fixture.call(Method::GET, jar, Auth::None, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));

    let role = |id: &str, privileges: &[&str]| json!({"id": id, "name": id, "description": "", "privileges": privileges, "roles": []});
    for body in [
        role(
            "maven-readers",
            &[
                "as-repository-view-maven2-releases-read",
                "as-repository-view-maven2-releases-browse",
            ],
        ),
        role(
            "maven-deployers",
            &[
                "as-repository-view-maven2-releases-read",
                "as-repository-view-maven2-releases-add",
            ],
        ),
    ] {
        let reply = fixture
            .call(
                Method::POST,
                "/service/rest/v1/security/roles",
                Auth::Admin,
                Some(body),
            )
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-pass", &["maven-readers"])
        .await;
    fixture
        .add_user("deployer", "deployer-pass", &["maven-deployers"])
        .await;
    fixture.add_user("nobody", "nobody-pass", &[]).await;
    let reader = || Auth::Basic("reader", "reader-pass");
    let deployer = || Auth::Basic("deployer", "deployer-pass");

    assert_eq!(
        fixture.call(Method::GET, jar, reader(), None).await.status,
        StatusCode::OK
    );
    for path in [
        format!("{jar}.sha1"),
        "/repository/releases/org/example/lib/maven-metadata.xml".to_owned(),
    ] {
        assert_eq!(
            fixture
                .call(Method::GET, &path, reader(), None)
                .await
                .status,
            StatusCode::OK,
            "{path}"
        );
        assert_eq!(
            fixture
                .call(
                    Method::GET,
                    &path,
                    Auth::Basic("nobody", "nobody-pass"),
                    None
                )
                .await
                .status,
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }
    let new_jar = "/repository/releases/org/example/lib/2.0/lib-2.0.jar";
    assert_eq!(
        put(&fixture, new_jar, JAR, reader()).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(&fixture, new_jar, JAR, deployer()).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, &format!("{new_jar}.sha1"), sha1(JAR), deployer())
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/releases/org/example/lib/maven-metadata.xml",
            "<metadata/>",
            deployer()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, new_jar, JAR, deployer()).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .call(Method::DELETE, new_jar, deployer(), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let listing = fixture
        .call(Method::GET, "/repository/releases/org/", Auth::None, None)
        .await;
    assert_eq!(listing.status, StatusCode::UNAUTHORIZED);
}
