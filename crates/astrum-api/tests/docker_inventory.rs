mod support;

use std::sync::Arc;

use astrum_scan::{matcher, verdict, worker};
use astrum_security::SecurityModel;
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
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const IN_TOTO: &str = "application/vnd.in-toto+json";
const SPDX: &str = "application/spdx+json";
const CYCLONEDX: &str = "application/vnd.cyclonedx+json";
const PREFIX: &str = "/v2/hosted/app";
const ADVISORIES: &str = "\
INSERT INTO advisory (id, aliases, severity, modified_at, document) VALUES \
  ('GHSA-A', '{CVE-A}', 'high', now(), '{}'), \
  ('GHSA-D', '{CVE-D}', 'high', now(), '{}'), \
  ('GHSA-N', '{CVE-N}', 'high', now(), '{}'); \
INSERT INTO advisory_affected (advisory_id, ecosystem, name, versions) VALUES \
  ('GHSA-A', 'Alpine:v3.19', 'musl', '{1.2.4-r2}'), \
  ('GHSA-D', 'Debian:12', 'glibc', '{2.36-9}'), \
  ('GHSA-N', 'npm', 'lodash', '{4.17.20}')";

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn spdx(purls: &[&str]) -> String {
    let packages: Vec<Value> = purls
        .iter()
        .enumerate()
        .map(|(index, purl)| {
            json!({"name": format!("p{index}"), "versionInfo": "1",
                   "externalRefs": [{"referenceType": "purl", "referenceLocator": purl}]})
        })
        .collect();
    json!({"spdxVersion": "SPDX-2.3", "packages": packages}).to_string()
}

fn statement(purls: &[&str]) -> String {
    let document: Value = serde_json::from_str(&spdx(purls)).unwrap();
    json!({"_type": "https://in-toto.io/Statement/v0.1", "predicateType": "https://spdx.dev/Document",
           "subject": [], "predicate": document})
    .to_string()
}

fn manifest(
    config: &str,
    layer: (&str, &str),
    artifact_type: Option<&str>,
    subject: Option<&str>,
) -> Value {
    let mut body = json!({
        "schemaVersion": 2, "mediaType": OCI_MANIFEST,
        "config": {"mediaType": if artifact_type.is_some() { EMPTY } else { OCI_CONFIG }, "digest": config, "size": 2},
        "layers": [{"mediaType": layer.1, "digest": layer.0, "size": 1}],
    });
    if let Some(artifact_type) = artifact_type {
        body["artifactType"] = artifact_type.into();
    }
    if let Some(subject) = subject {
        body["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": subject, "size": 2});
    }
    body
}

fn index(children: &[(&str, Option<(&str, &str)>)]) -> Value {
    let entries: Vec<Value> = children
        .iter()
        .map(|(digest, attests)| {
            let mut entry = json!({"mediaType": OCI_MANIFEST, "digest": digest, "size": 1});
            if let Some((kind, image)) = attests {
                entry["annotations"] = json!({
                    "vnd.docker.reference.type": kind, "vnd.docker.reference.digest": image});
            }
            entry
        })
        .collect();
    json!({"schemaVersion": 2, "mediaType": OCI_INDEX, "manifests": entries})
}

impl Registry {
    async fn new() -> Self {
        let fixture = Fixture::new().await;
        astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
            .await
            .unwrap();
        fixture
            .state
            .content()
            .register(Arc::new(astrum_format_docker::Docker::default()));
        let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes());
        let admin = format!(
            "Basic {}",
            STANDARD.encode(format!("admin:{}", fixture.admin_password))
        );
        let registry = Registry {
            fixture,
            app,
            admin,
        };
        let policy =
            json!({"name": "strict", "vulnerabilities": {"mode": "block", "severity": "high"}});
        let made = registry
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/security/policies",
                Auth::Admin,
                Some(policy),
            )
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
        let repository = json!({
            "name": "hosted", "online": true, "policy": {"name": "strict"},
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true},
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
        });
        let made = registry
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/repositories/docker/hosted",
                Auth::Admin,
                Some(repository),
            )
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
        registry.execute(ADVISORIES).await;
        registry
    }

    async fn send(&self, method: Method, path: &str, body: Vec<u8>, content_type: &str) -> Reply {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, &self.admin)
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(body))
            .unwrap();
        let response = self.app.clone().oneshot(request).await.unwrap();
        let (status, headers) = (response.status(), response.headers().clone());
        let body = response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        Reply {
            status,
            headers,
            body,
        }
    }

    async fn push_blob(&self, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let path = format!("{PREFIX}/blobs/uploads/?digest={digest}");
        let reply = self
            .send(
                Method::POST,
                &path,
                bytes.to_vec(),
                "application/octet-stream",
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn push_manifest(&self, reference: &str, body: &Value) -> String {
        let bytes = serde_json::to_vec(body).unwrap();
        let media_type = body["mediaType"].as_str().unwrap().to_owned();
        let path = format!("{PREFIX}/manifests/{reference}");
        let reply = self
            .send(Method::PUT, &path, bytes.clone(), &media_type)
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&bytes)
    }

    async fn image(&self, tag: Option<&str>, layer: &[u8]) -> String {
        let config = self.push_blob(b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(layer).await;
        let body = manifest(&config, (&layer, LAYER), None, None);
        let reference = tag.map_or_else(
            || sha256(&serde_json::to_vec(&body).unwrap()),
            str::to_owned,
        );
        self.push_manifest(&reference, &body).await
    }

    async fn document(
        &self,
        subject: &str,
        tag: Option<&str>,
        layer_type: &str,
        payload: &str,
    ) -> String {
        let config = self.push_blob(b"{}").await;
        let layer = self.push_blob(payload.as_bytes()).await;
        let artifact = tag.is_none().then_some(SPDX);
        let subject_of = tag.is_none().then_some(subject);
        let body = manifest(&config, (&layer, layer_type), artifact, subject_of);
        let reference = tag.map_or_else(
            || sha256(&serde_json::to_vec(&body).unwrap()),
            str::to_owned,
        );
        self.push_manifest(&reference, &body).await
    }

    async fn execute(&self, sql: &str) {
        self.fixture
            .state
            .db()
            .conn()
            .execute_unprepared(sql)
            .await
            .unwrap();
    }

    async fn rows(&self, sql: &str) -> Vec<Vec<String>> {
        let rows = self
            .fixture
            .state
            .db()
            .conn()
            .query_all(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
            .await
            .unwrap();
        rows.iter()
            .map(|row| {
                let columns = row.column_names();
                columns
                    .iter()
                    .map(|name| {
                        row.try_get::<String>("", name)
                            .unwrap_or_else(|_| "-".to_owned())
                    })
                    .collect()
            })
            .collect()
    }

    async fn settle(&self) {
        let content = self.fixture.state.content();
        let security = Arc::new(SecurityModel::default());
        let scanning = worker::Config::default();
        let scans = worker::Metrics::default();
        while worker::drain(content, &scanning, &scans, &security).await > 0 {}
        let db = self.fixture.state.db().conn();
        let matching = matcher::Config::default();
        let matches = matcher::Metrics::default();
        while matcher::step(db, &matching, &matches).await {}
        let judging = verdict::Config::default();
        loop {
            let judged = verdict::judge_batch(db, self.fixture.state.policy_cache(), &judging)
                .await
                .unwrap();
            if judged.taken == 0 && !judged.busy {
                break;
            }
        }
    }

    async fn scan_of(&self, tag: &str) -> Vec<String> {
        let sql = format!(
            "SELECT cs.state, COALESCE(cs.digest, '-') AS digest, COALESCE(i.key, '-') AS key, \
                    COALESCE(i.source, '-') AS source, \
                    (SELECT count(*) FROM sca_inventory_package ip WHERE ip.inventory_id = i.id)::text AS packages \
             FROM component c JOIN component_scan cs ON cs.component_id = c.id \
             LEFT JOIN sca_inventory i ON i.id = cs.inventory_id \
             WHERE c.name = 'app' AND c.version = '{tag}'"
        );
        self.rows(&sql).await.remove(0)
    }

    async fn verdicts(&self) -> Vec<Vec<String>> {
        self.rows(
            "SELECT CASE WHEN subject LIKE 'oci:%' THEN 'oci' ELSE 'c' END AS subject, \
                    state, blocked::text AS blocked, high::text AS high \
             FROM policy_verdict ORDER BY subject",
        )
        .await
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

const MUSL: &str = "pkg:apk/alpine/musl@1.2.4-r2?distro=alpine-3.19.1";
const LODASH: &str = "pkg:npm/lodash@4.17.20";

#[tokio::test]
async fn an_image_with_a_referrer_sbom_gets_an_inventory_findings_and_the_same_verdict_by_tag_and_digest()
 {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    registry
        .document(
            &digest,
            None,
            SPDX,
            &spdx(&[MUSL, LODASH, "pkg:npm/fine@1.0.0"]),
        )
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        vec![
            "scanned".to_owned(),
            digest.clone(),
            format!("oci:{digest}"),
            "sbom".to_owned(),
            "3".to_owned()
        ]
    );
    assert_eq!(
        registry.verdicts().await,
        vec![
            strings(&["c", "block", "true", "2"]),
            strings(&["oci", "block", "true", "2"])
        ]
    );
}

#[tokio::test]
async fn an_image_with_only_a_legacy_sbom_tag_is_inventoried_from_it_with_the_source_package() {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    let bom = json!({"bomFormat": "CycloneDX", "specVersion": "1.5", "components": [
        {"name": "libc6", "version": "2.36-9", "purl": "pkg:deb/debian/libc6@2.36-9?distro=debian-12&upstream=glibc"}]})
    .to_string();
    let tag = format!("{}.sbom", digest.replace(':', "-"));
    registry
        .document(&digest, Some(&tag), CYCLONEDX, &bom)
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        vec![
            "scanned".to_owned(),
            digest.clone(),
            format!("oci:{digest}"),
            "sbom".to_owned(),
            "1".to_owned()
        ]
    );
    assert_eq!(
        registry.verdicts().await[0],
        strings(&["c", "block", "true", "1"])
    );
}

#[tokio::test]
async fn a_buildkit_attestation_manifest_inside_an_index_is_the_sbom_of_its_platform_image() {
    let registry = Registry::new().await;
    let platform = registry.image(None, b"layer").await;
    let config = registry.push_blob(b"{}").await;
    let layer = registry.push_blob(statement(&[MUSL]).as_bytes()).await;
    let body = manifest(&config, (&layer, IN_TOTO), None, None);
    let attested = registry
        .push_manifest(&sha256(&serde_json::to_vec(&body).unwrap()), &body)
        .await;
    let listing = index(&[
        (&platform, None),
        (&attested, Some(("attestation-manifest", &platform))),
    ]);
    let top = registry.push_manifest("multi", &listing).await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("multi").await,
        vec![
            "scanned".to_owned(),
            top.clone(),
            format!("oci:{top}"),
            "index".to_owned(),
            "1".to_owned()
        ]
    );
    assert_eq!(
        registry.verdicts().await.last().unwrap(),
        &strings(&["oci", "block", "true", "1"])
    );
}

#[tokio::test]
async fn an_index_is_judged_on_the_union_of_its_children_and_a_missing_child_makes_it_incomplete() {
    let registry = Registry::new().await;
    let first = registry.image(None, b"one").await;
    let second = registry.image(None, b"two").await;
    registry
        .document(&first, None, SPDX, &spdx(&[MUSL, "pkg:npm/shared@1.0.0"]))
        .await;
    registry
        .document(
            &second,
            None,
            SPDX,
            &spdx(&[LODASH, "pkg:npm/shared@1.0.0"]),
        )
        .await;
    let complete = registry
        .push_manifest("multi", &index(&[(&first, None), (&second, None)]))
        .await;
    let absent = sha256(b"never pushed");
    let partial = registry
        .push_manifest("partial", &index(&[(&first, None), (&absent, None)]))
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("multi").await,
        vec![
            "scanned".to_owned(),
            complete.clone(),
            format!("oci:{complete}"),
            "index".to_owned(),
            "3".to_owned()
        ]
    );
    let incomplete = registry.scan_of("partial").await;
    assert_eq!(incomplete[0], "incomplete");
    assert_eq!(incomplete[1], partial);
    assert_eq!(incomplete[4], "2");
    let union = registry
        .rows(&format!(
            "SELECT high::text AS high FROM policy_verdict WHERE subject = 'oci:{complete}'"
        ))
        .await;
    assert_eq!(union, vec![strings(&["2"])]);
}

#[tokio::test]
async fn an_image_without_any_sbom_is_unsupported_and_has_no_inventory() {
    let registry = Registry::new().await;
    registry.image(Some("1.0"), b"layer").await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        vec![
            "unsupported".to_owned(),
            "-".to_owned(),
            "-".to_owned(),
            "-".to_owned(),
            "0".to_owned()
        ]
    );
}

#[tokio::test]
async fn an_sbom_pushed_after_the_image_queues_every_tag_of_it_and_the_next_scan_finds_the_packages()
 {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(b"layer").await;
    registry
        .push_manifest("stable", &manifest(&config, (&layer, LAYER), None, None))
        .await;
    registry.settle().await;
    assert_eq!(registry.scan_of("stable").await[0], "unsupported");
    assert!(
        registry
            .rows("SELECT 1 AS n FROM scan_queue")
            .await
            .is_empty()
    );
    registry.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    let queued = registry
        .rows("SELECT c.version, q.reason FROM scan_queue q JOIN component c ON c.id = q.component_id ORDER BY c.version")
        .await;
    assert_eq!(
        queued,
        vec![strings(&["1.0", "sbom"]), strings(&["stable", "sbom"])]
    );
    registry.settle().await;
    assert_eq!(registry.scan_of("1.0").await[0], "scanned");
    assert_eq!(registry.scan_of("stable").await[0], "scanned");
    assert_eq!(registry.verdicts().await.len(), 3);
}
