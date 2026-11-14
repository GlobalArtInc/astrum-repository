mod support;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const COSIGN: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
const DSSE: &str = "application/vnd.dsse.envelope.v1+json";
const SPDX: &str = "application/spdx+json";
const CYCLONEDX: &str = "application/vnd.cyclonedx+json";
const API: &str = "/service/rest/v1/docker";
const PREFIX: &str = "/v2/hosted/app";
const LIMIT: usize = 32 * 1024 * 1024;
const COSIGN_ATTESTATION: &[u8] =
    include_bytes!("../../astrum-sbom/tests/fixtures/cosign-dsse-spdx.json");

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn basic(user: &str, password: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{user}:{password}")))
}

fn hosted_body(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    })
}

fn spdx_document(packages: &[Value]) -> String {
    json!({"spdxVersion": "SPDX-2.3", "packages": packages}).to_string()
}

fn manifest(
    config: &str,
    layers: &[(&str, &str)],
    artifact_type: Option<&str>,
    subject: Option<&str>,
) -> Value {
    let mut manifest = json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": if artifact_type.is_some() { EMPTY } else { OCI_CONFIG }, "digest": config, "size": 2},
        "layers": layers.iter().map(|(digest, media_type)| json!({"mediaType": media_type, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    });
    if let Some(artifact_type) = artifact_type {
        manifest["artifactType"] = artifact_type.into();
    }
    if let Some(subject) = subject {
        manifest["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": subject, "size": 2});
    }
    manifest
}

impl Registry {
    async fn new() -> Self {
        Self::with_routes(astrum_format_docker::routes()).await
    }

    async fn with_sbom_limit(permits: usize) -> Self {
        Self::with_routes(astrum_format_docker::routes_with_sbom_limit(permits)).await
    }

    async fn with_routes(routes: Router<astrum_api::AppState>) -> Self {
        let fixture = Fixture::new().await;
        astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
            .await
            .unwrap();
        fixture
            .state
            .content()
            .register(Arc::new(astrum_format_docker::Docker::default()));
        let app = astrum_api::router_with(fixture.state.clone(), routes);
        let admin = basic("admin", &fixture.admin_password);
        let registry = Registry {
            fixture,
            app,
            admin,
        };
        registry.create("hosted", hosted_body("hosted")).await;
        registry
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        authorization: Option<&str>,
        body: Vec<u8>,
        content_type: Option<&str>,
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(authorization) = authorization {
            request = request.header(header::AUTHORIZATION, authorization);
        }
        if let Some(content_type) = content_type {
            request = request.header(header::CONTENT_TYPE, content_type);
        }
        let response = self
            .app
            .clone()
            .oneshot(request.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let (status, headers) = (response.status(), response.headers().clone());
        let body = response.into_body().collect().await.unwrap().to_bytes();
        Reply {
            status,
            headers,
            body: body.to_vec(),
        }
    }

    async fn get(&self, authorization: Option<&str>, path: &str) -> Reply {
        self.send(Method::GET, path, authorization, Vec::new(), None)
            .await
    }

    async fn read(&self, digest: &str, extra: &str) -> Reply {
        let path = format!("{API}/hosted/sbom?image=app&digest={digest}{extra}");
        self.get(Some(&self.admin), &path).await
    }

    async fn create(&self, kind: &str, body: Value) {
        let path = format!("/service/rest/v1/repositories/docker/{kind}");
        let reply = self
            .fixture
            .call(Method::POST, &path, Auth::Admin, Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn push_blob(&self, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let path = format!("{PREFIX}/blobs/uploads/?digest={digest}");
        let content_type = Some("application/octet-stream");
        let reply = self
            .send(
                Method::POST,
                &path,
                Some(&self.admin),
                bytes.to_vec(),
                content_type,
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn push_manifest(&self, reference: &str, manifest: &Value) -> String {
        let bytes = serde_json::to_vec(manifest).unwrap();
        let path = format!("{PREFIX}/manifests/{reference}");
        let reply = self
            .send(
                Method::PUT,
                &path,
                Some(&self.admin),
                bytes.clone(),
                Some(OCI_MANIFEST),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&bytes)
    }

    async fn push_image(&self) -> String {
        let config = self.push_blob(b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(b"base").await;
        self.push_manifest("1.0", &manifest(&config, &[(&layer, LAYER)], None, None))
            .await
    }

    async fn push_artifact(
        &self,
        subject: &str,
        tag: Option<&str>,
        artifact_type: Option<&str>,
        layer_type: &str,
        payload: &[u8],
    ) -> String {
        let config = self.push_blob(b"{}").await;
        let layer = self.push_blob(payload).await;
        let subject = tag.is_none().then_some(subject);
        let body = manifest(&config, &[(&layer, layer_type)], artifact_type, subject);
        let reference = tag.map_or_else(
            || sha256(&serde_json::to_vec(&body).unwrap()),
            str::to_owned,
        );
        self.push_manifest(&reference, &body).await
    }

    async fn push_layers(&self, subject: &str, layers: &[&str]) -> String {
        let config = self.push_blob(b"{}").await;
        let layers: Vec<(&str, &str)> = layers.iter().map(|digest| (*digest, SPDX)).collect();
        let body = manifest(&config, &layers, Some(SPDX), Some(subject));
        let digest = sha256(&serde_json::to_vec(&body).unwrap());
        self.push_manifest(&digest, &body).await
    }

    async fn push_sbom(&self, subject: &str, document: &str) -> String {
        self.push_artifact(subject, None, Some(SPDX), SPDX, document.as_bytes())
            .await
    }

    async fn execute(&self, sql: &str) {
        let statement = Statement::from_string(DbBackend::Postgres, sql);
        self.fixture
            .state
            .db()
            .conn()
            .execute(statement)
            .await
            .unwrap();
    }
}

fn listed(reply: &Reply) -> Vec<(String, String)> {
    let packages = reply.json()["packages"].as_array().unwrap().clone();
    let text = |package: &Value, field: &str| package[field].as_str().unwrap().to_owned();
    packages
        .iter()
        .map(|package| (text(package, "name"), text(package, "version")))
        .collect()
}

fn pair(name: &str, version: &str) -> (String, String) {
    (name.to_owned(), version.to_owned())
}

#[tokio::test]
async fn the_packages_of_listed_sboms_are_paged_filtered_and_sorted() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let hex = subject.trim_start_matches("sha256:").to_owned();
    let mut packages = Vec::new();
    let mut expected = vec![pair("dup", "1.0"), pair("dup", "1.1")];
    for index in 0..250usize {
        let number = (index * 37) % 250;
        let (name, version) = (format!("pkg-{number:03}"), format!("1.{index}"));
        let licence = if number % 2 == 0 { "MIT" } else { "Apache-2.0" };
        let mut package = json!({"name": name, "versionInfo": version, "licenseConcluded": licence, "licenseDeclared": "NOASSERTION"});
        if number % 5 == 0 {
            let locator = format!("pkg:npm/{name}@{version}?arch=x86");
            package["externalRefs"] =
                json!([{"referenceType": "purl", "referenceLocator": locator}]);
        }
        expected.push((name, version));
        packages.push(package);
    }
    packages.push(json!({"name": "dup", "versionInfo": "1.1"}));
    packages.push(json!({"name": "dup", "versionInfo": "1.0"}));
    expected.sort();
    let referrer = registry
        .push_sbom(&subject, &spdx_document(&packages))
        .await;
    let cyclonedx = json!({"bomFormat": "CycloneDX", "specVersion": "1.5", "components": [
        {"name": "libz", "version": "1.3", "purl": "pkg:deb/debian/libz@1.3?distro=debian-12"}
    ]})
    .to_string();
    let legacy_sbom = registry
        .push_artifact(
            &subject,
            Some(&format!("sha256-{hex}.sbom")),
            None,
            CYCLONEDX,
            cyclonedx.as_bytes(),
        )
        .await;
    let legacy_att = registry
        .push_artifact(
            &subject,
            Some(&format!("sha256-{hex}.att")),
            None,
            DSSE,
            COSIGN_ATTESTATION,
        )
        .await;
    let path = format!("{API}/hosted/supply-chain?image=app&reference=1.0");
    let supply = registry.get(Some(&registry.admin), &path).await.json();
    let mut offered: Vec<String> = ["sboms", "attestations"]
        .iter()
        .flat_map(|kind| supply[kind].as_array().unwrap().clone())
        .map(|entry| entry["digest"].as_str().unwrap().to_owned())
        .collect();
    offered.sort();
    let mut known = vec![referrer.clone(), legacy_sbom.clone(), legacy_att.clone()];
    known.sort();
    assert_eq!(offered, known);

    let first = registry.read(&referrer, "").await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    let body = first.json();
    assert_eq!(body["format"], "spdx");
    assert_eq!(body["total"], 252);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 100);
    let sample = &body["packages"][2];
    assert_eq!(sample["name"], "pkg-000");
    assert_eq!(sample["licenses"], json!(["MIT"]));
    assert_eq!(sample["ecosystem"], "npm");
    assert_eq!(sample["purl"], "pkg:npm/pkg-000@1.0?arch=x86");
    let mut seen = listed(&first);
    assert_eq!(seen.len(), 100);
    for page in 2..=3 {
        seen.extend(listed(
            &registry.read(&referrer, &format!("&page={page}")).await,
        ));
    }
    assert_eq!(seen, expected);
    let beyond = registry.read(&referrer, "&page=4").await;
    assert_eq!(beyond.status, StatusCode::OK);
    assert_eq!(beyond.json()["packages"], json!([]));
    assert_eq!(beyond.json()["total"], 252);

    for (filter, total) in [
        ("PKG-01", 10),
        ("ARCH%3Dx86", 50),
        ("apache", 125),
        ("no-such", 0),
    ] {
        let reply = registry.read(&referrer, &format!("&filter={filter}")).await;
        assert_eq!(reply.json()["total"], total, "{filter}");
    }
    let duplicates = registry.read(&referrer, "&filter=dup").await;
    assert_eq!(
        listed(&duplicates),
        [pair("dup", "1.0"), pair("dup", "1.1")]
    );

    let legacy = registry.read(&legacy_sbom, "").await;
    assert_eq!(legacy.status, StatusCode::OK, "{}", legacy.text());
    assert_eq!(legacy.json()["format"], "cyclonedx");
    assert_eq!(legacy.json()["packages"][0]["ecosystem"], "Debian:12");
    let attested = registry.read(&legacy_att, "&filter=left-pad").await;
    assert_eq!(attested.status, StatusCode::OK, "{}", attested.text());
    assert_eq!(attested.json()["format"], "spdx");
    assert_eq!(attested.json()["total"], 1);

    for extra in [
        "&page=0",
        "&page=abc",
        "&page=-1",
        &format!("&filter={}", "x".repeat(201)),
    ] {
        let reply = registry.read(&referrer, extra).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{extra}");
    }
    let last = registry.read(&referrer, "&page=4294967295").await;
    assert_eq!(last.status, StatusCode::OK, "{}", last.text());
    assert_eq!(last.json()["packages"], json!([]));
    let longest = format!("&filter={}", "x".repeat(200));
    assert_eq!(
        registry.read(&referrer, &longest).await.status,
        StatusCode::OK
    );
    let no_digest = format!("{API}/hosted/sbom?image=app");
    let reply = registry.get(Some(&registry.admin), &no_digest).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let no_image = format!("{API}/hosted/sbom?digest={referrer}");
    let reply = registry.get(Some(&registry.admin), &no_image).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn only_sboms_and_attestations_of_the_image_can_be_read() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let hex = subject.trim_start_matches("sha256:").to_owned();
    let document = spdx_document(&[json!({"name": "a"})]);
    let referrer = registry.push_sbom(&subject, &document).await;
    let signature = registry
        .push_artifact(&subject, None, Some(COSIGN), SIMPLE_SIGNING, b"sig")
        .await;
    let legacy_signature = registry
        .push_artifact(
            &subject,
            Some(&format!("sha256-{hex}.sig")),
            None,
            SIMPLE_SIGNING,
            b"legacy",
        )
        .await;
    let loose = registry
        .push_artifact(
            &subject,
            Some("loose"),
            Some(SPDX),
            SPDX,
            document.as_bytes(),
        )
        .await;
    assert_eq!(registry.read(&referrer, "").await.status, StatusCode::OK);
    let blob = sha256(document.as_bytes());
    for digest in [
        signature,
        legacy_signature,
        loose,
        subject,
        blob,
        sha256(b"unknown"),
        "nonsense".to_owned(),
    ] {
        let reply = registry.read(&digest, "").await;
        assert_eq!(
            reply.status,
            StatusCode::NOT_FOUND,
            "{digest}: {}",
            reply.text()
        );
    }
    let other = format!("{API}/hosted/sbom?image=other&digest={referrer}");
    let reply = registry.get(Some(&registry.admin), &other).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.text());
}

#[tokio::test]
async fn callers_without_read_and_repositories_of_other_formats_are_refused() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let referrer = registry.push_sbom(&subject, &spdx_document(&[])).await;
    registry
        .fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    let raw = registry
        .fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": "files",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(raw.status, StatusCode::CREATED, "{}", raw.text());
    registry
        .fixture
        .add_user("reader", "reader-pass", &[])
        .await;
    let reader = basic("reader", "reader-pass");
    let query = format!("sbom?image=app&digest={referrer}");
    let admin = Some(registry.admin.as_str());
    let cases = [
        (None, "hosted", StatusCode::UNAUTHORIZED),
        (Some(reader.as_str()), "hosted", StatusCode::FORBIDDEN),
        (admin, "files", StatusCode::BAD_REQUEST),
        (admin, "nowhere", StatusCode::NOT_FOUND),
        (admin, "hosted", StatusCode::OK),
    ];
    for (authorization, repository, status) in cases {
        let reply = registry
            .get(authorization, &format!("{API}/{repository}/{query}"))
            .await;
        assert_eq!(reply.status, status, "{repository}: {}", reply.text());
    }
}

#[tokio::test]
async fn corrupt_and_oversized_documents_are_refused_without_echoing_them() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let push = |kind: &'static str, payload: Vec<u8>| {
        let (registry, subject) = (&registry, subject.clone());
        async move {
            registry
                .push_artifact(&subject, None, Some(kind), kind, &payload)
                .await
        }
    };
    let corrupt = push(
        SPDX,
        br#"{"spdxVersion":"SPDX-2.3","packages":secret-marker"#.to_vec(),
    )
    .await;
    let foreign = push(CYCLONEDX, br#"{"hello":"secret-marker"}"#.to_vec()).await;
    let huge = push(SPDX, vec![b' '; LIMIT + 1]).await;
    for (digest, status, category) in [
        (&corrupt, StatusCode::UNPROCESSABLE_ENTITY, "malformed"),
        (&foreign, StatusCode::UNPROCESSABLE_ENTITY, "unsupported"),
        (&huge, StatusCode::PAYLOAD_TOO_LARGE, "too_large"),
    ] {
        let reply = registry.read(digest, "").await;
        assert_eq!(reply.status, status, "{}", reply.text());
        assert_eq!(reply.json()["category"], category);
        assert!(!reply.text().contains("secret-marker"), "{}", reply.text());
    }
    registry
        .execute(&format!(
            "UPDATE asset SET size = 1 WHERE repository = 'hosted' AND kind = 'blob' \
             AND path = '/v2/-/blobs/{}'",
            sha256(&vec![b' '; LIMIT + 1])
        ))
        .await;
    let reply = registry.read(&huge, "").await;
    assert_eq!(
        reply.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        reply.text()
    );
}

#[tokio::test]
async fn a_group_reads_the_sboms_of_its_members() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let document = spdx_document(&[json!({"name": "grouped", "versionInfo": "1"})]);
    let referrer = registry.push_sbom(&subject, &document).await;
    let group = json!({
        "name": "all",
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": ["hosted"]},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    });
    registry.create("group", group).await;
    let path = format!("{API}/all/sbom?image=app&digest={referrer}");
    let reply = registry.get(Some(&registry.admin), &path).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(listed(&reply), [pair("grouped", "1")]);
}

#[tokio::test]
async fn reads_beyond_the_permits_answer_503_and_permits_come_back() {
    let closed = Registry::with_sbom_limit(0).await;
    let subject = closed.push_image().await;
    let referrer = closed.push_sbom(&subject, &spdx_document(&[])).await;
    let busy = closed.read(&referrer, "").await;
    assert_eq!(
        busy.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "{}",
        busy.text()
    );
    assert_eq!(busy.headers["retry-after"], "5");
    let unknown = closed.read(&sha256(b"unknown"), "").await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    let single = Registry::with_sbom_limit(1).await;
    let subject = single.push_image().await;
    let good = single.push_sbom(&subject, &spdx_document(&[])).await;
    let bad = single
        .push_artifact(&subject, None, Some(CYCLONEDX), CYCLONEDX, b"{")
        .await;
    for (digest, status) in [
        (&good, StatusCode::OK),
        (&bad, StatusCode::UNPROCESSABLE_ENTITY),
        (&good, StatusCode::OK),
    ] {
        assert_eq!(single.read(digest, "").await.status, status);
    }
}

#[tokio::test]
async fn a_failing_layer_does_not_hide_a_later_good_one_and_only_four_layers_are_read() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let good = registry
        .push_blob(spdx_document(&[json!({"name": "good"})]).as_bytes())
        .await;
    let corrupt = registry.push_blob(b"{").await;
    let huge = registry.push_blob(&vec![b' '; LIMIT + 1]).await;
    let vanished = registry.push_blob(b"{\"vanished\":true}").await;
    let recovered = registry
        .push_layers(&subject, &[&vanished, &huge, &corrupt, &good])
        .await;
    let too_large = registry.push_layers(&subject, &[&huge, &corrupt]).await;
    let refused = registry.push_layers(&subject, &[&corrupt, &huge]).await;
    let missing = registry.push_layers(&subject, &[&vanished]).await;
    let fifth = registry
        .push_layers(&subject, &[&corrupt, &corrupt, &corrupt, &corrupt, &good])
        .await;
    registry
        .execute(&format!(
            "DELETE FROM asset WHERE repository = 'hosted' AND kind = 'blob' \
             AND path = '/v2/-/blobs/{vanished}'"
        ))
        .await;
    let reply = registry.read(&recovered, "").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(listed(&reply), [pair("good", "")]);
    for (digest, status) in [
        (&too_large, StatusCode::PAYLOAD_TOO_LARGE),
        (&refused, StatusCode::UNPROCESSABLE_ENTITY),
        (&missing, StatusCode::NOT_FOUND),
        (&fifth, StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let reply = registry.read(digest, "").await;
        assert_eq!(reply.status, status, "{}", reply.text());
    }
}

#[tokio::test]
async fn the_layers_of_one_request_share_one_read_budget() {
    let registry = Registry::new().await;
    let subject = registry.push_image().await;
    let mut padded = spdx_document(&[json!({"name": "padded"})]).into_bytes();
    padded.resize(20 * 1024 * 1024, b' ');
    let good = registry.push_blob(&padded).await;
    let blank = registry.push_blob(&vec![b' '; 20 * 1024 * 1024]).await;
    let alone = registry.push_layers(&subject, &[&good]).await;
    let after_blank = registry.push_layers(&subject, &[&blank, &good]).await;
    let reply = registry.read(&alone, "").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let reply = registry.read(&after_blank, "").await;
    assert_eq!(
        reply.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        reply.text()
    );
}
