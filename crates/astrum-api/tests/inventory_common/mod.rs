#![allow(dead_code)]

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
use tower::ServiceExt;

use crate::support::{Auth, Fixture, Reply};

pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub const EMPTY: &str = "application/vnd.oci.empty.v1+json";
pub const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
pub const IN_TOTO: &str = "application/vnd.in-toto+json";
pub const SPDX: &str = "application/spdx+json";
pub const CYCLONEDX: &str = "application/vnd.cyclonedx+json";
pub const MUSL: &str = "pkg:apk/alpine/musl@1.2.4-r2?distro=alpine-3.19.1";
pub const LODASH: &str = "pkg:npm/lodash@4.17.20";
const ADVISORIES: &str = "\
INSERT INTO advisory (id, aliases, severity, modified_at, document) VALUES \
  ('GHSA-A', '{CVE-A}', 'high', now(), '{}'), \
  ('GHSA-D', '{CVE-D}', 'high', now(), '{}'), \
  ('GHSA-N', '{CVE-N}', 'high', now(), '{}'), \
  ('GHSA-J', '{CVE-J}', 'high', now(), '{}'); \
INSERT INTO advisory_affected (advisory_id, ecosystem, name, versions) VALUES \
  ('GHSA-A', 'Alpine:v3.19', 'musl', '{1.2.4-r2}'), \
  ('GHSA-D', 'Debian:12', 'glibc', '{2.36-9}'), \
  ('GHSA-N', 'npm', 'lodash', '{4.17.20}'), \
  ('GHSA-J', 'NuGet', 'newtonsoft.json', '{13.0.1}')";

pub struct Registry {
    pub fixture: Arc<Fixture>,
    app: Router,
    admin: String,
    repository: String,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn marker_layer(marker: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(marker.len() as u64);
    header.set_mode(0o644);
    builder.append_data(&mut header, "marker", marker).unwrap();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&builder.into_inner().unwrap()).unwrap();
    encoder.finish().unwrap()
}

pub fn spdx(purls: &[&str]) -> String {
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

pub fn statement(purls: &[&str]) -> String {
    let document: Value = serde_json::from_str(&spdx(purls)).unwrap();
    json!({"_type": "https://in-toto.io/Statement/v0.1", "predicateType": "https://spdx.dev/Document",
           "subject": [], "predicate": document})
    .to_string()
}

pub fn manifest(
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

pub fn index(children: &[(&str, Option<(&str, &str)>)]) -> Value {
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
    pub async fn new() -> Self {
        let fixture = Arc::new(Fixture::new().await);
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
            repository: "hosted".to_owned(),
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
        registry.execute(ADVISORIES).await;
        registry.create_repository("hosted").await;
        registry
    }

    pub async fn create_repository(&self, name: &str) {
        self.create_repository_with(name, Some("strict")).await;
    }

    pub async fn create_repository_with(&self, name: &str, policy: Option<&str>) {
        let mut repository = json!({
            "name": name, "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true},
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
        });
        if let Some(policy) = policy {
            repository["policy"] = json!({"name": policy});
        }
        let made = self
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/repositories/docker/hosted",
                Auth::Admin,
                Some(repository),
            )
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    }

    pub async fn repository(&self, name: &str) -> Registry {
        self.create_repository(name).await;
        Registry {
            fixture: self.fixture.clone(),
            app: self.app.clone(),
            admin: self.admin.clone(),
            repository: name.to_owned(),
        }
    }

    pub async fn repository_without_policy(&self, name: &str) -> Registry {
        self.create_repository_with(name, None).await;
        Registry {
            fixture: self.fixture.clone(),
            app: self.app.clone(),
            admin: self.admin.clone(),
            repository: name.to_owned(),
        }
    }

    pub async fn send(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Reply {
        self.send_with(method, path, body, content_type, &[]).await
    }

    pub async fn send_with(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
        headers: &[(&str, &str)],
    ) -> Reply {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, &self.admin)
            .header(header::CONTENT_TYPE, content_type);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let request = request.body(Body::from(body)).unwrap();
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

    pub async fn push_blob(&self, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let path = format!("/v2/{}/app/blobs/uploads/?digest={digest}", self.repository);
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

    pub async fn push_manifest(&self, reference: &str, body: &Value) -> String {
        let bytes = serde_json::to_vec(body).unwrap();
        let media_type = body["mediaType"].as_str().unwrap().to_owned();
        let path = format!("/v2/{}/app/manifests/{reference}", self.repository);
        let reply = self
            .send(Method::PUT, &path, bytes.clone(), &media_type)
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&bytes)
    }

    pub async fn image(&self, tag: Option<&str>, layer: &[u8]) -> String {
        let config = self.push_blob(b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(&marker_layer(layer)).await;
        let body = manifest(&config, (&layer, LAYER), None, None);
        let reference = tag.map_or_else(
            || sha256(&serde_json::to_vec(&body).unwrap()),
            str::to_owned,
        );
        self.push_manifest(&reference, &body).await
    }

    pub async fn document(
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

    pub async fn execute(&self, sql: &str) {
        self.fixture
            .state
            .db()
            .conn()
            .execute_unprepared(sql)
            .await
            .unwrap();
    }

    pub async fn rows(&self, sql: &str) -> Vec<Vec<String>> {
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

    pub async fn settle(&self) {
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

    pub async fn scan_of(&self, tag: &str) -> Vec<String> {
        let sql = format!(
            "SELECT cs.state, COALESCE(cs.digest, '-') AS digest, COALESCE(i.key, '-') AS key, \
                    COALESCE(i.source, '-') AS source, \
                    (SELECT count(*) FROM sca_inventory_package ip WHERE ip.inventory_id = i.id)::text AS packages \
             FROM component c JOIN component_scan cs ON cs.component_id = c.id \
             LEFT JOIN sca_inventory i ON i.id = cs.inventory_id \
             WHERE c.name = 'app' AND c.version = '{tag}' AND c.repository = '{}'",
            self.repository
        );
        self.rows(&sql).await.remove(0)
    }

    pub async fn verdicts(&self) -> Vec<Vec<String>> {
        self.rows(&format!(
            "SELECT CASE WHEN subject LIKE 'oci:%' THEN 'oci' ELSE 'c' END AS subject, \
                    state, blocked::text AS blocked, high::text AS high \
             FROM policy_verdict WHERE repository = '{}' ORDER BY subject",
            self.repository
        ))
        .await
    }
}

pub fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
