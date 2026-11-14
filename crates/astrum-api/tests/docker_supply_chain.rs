mod support;

use std::net::SocketAddr;
use std::sync::Arc;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
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
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const COSIGN: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
const BUNDLE: &str = "application/vnd.dev.sigstore.bundle.v0.3+json";
const NOTARY: &str = "application/vnd.cncf.notary.signature";
const IN_TOTO: &str = "application/vnd.in-toto+json";
const SPDX: &str = "application/spdx+json";
const CYCLONEDX: &str = "application/vnd.cyclonedx+json";
const SUPPLY: &str = "/service/rest/v1/docker";

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

fn hosted_body(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy, "latestPolicy": true},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    })
}

fn image_manifest(
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

fn index(children: &[&str]) -> Value {
    json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": children.iter().map(|digest| json!({"mediaType": OCI_MANIFEST, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    })
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
        let admin = basic("admin", &fixture.admin_password);
        Registry {
            fixture,
            app,
            admin,
        }
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
        let status = response.status();
        let headers = response.headers().clone();
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

    async fn admin(&self, method: Method, path: &str) -> Reply {
        self.send(method, path, Some(&self.admin), Vec::new(), None)
            .await
    }

    async fn create(&self, kind: &str, body: Value) {
        let reply = self
            .fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/repositories/docker/{kind}"),
                Auth::Admin,
                Some(body),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn push_blob(&self, prefix: &str, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let reply = self
            .send(
                Method::POST,
                &format!("{prefix}/blobs/uploads/?digest={digest}"),
                Some(&self.admin),
                bytes.to_vec(),
                Some("application/octet-stream"),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn push_manifest(&self, prefix: &str, reference: &str, manifest: &Value) -> Reply {
        let media_type = manifest["mediaType"].as_str().unwrap_or(OCI_MANIFEST);
        self.send(
            Method::PUT,
            &format!("{prefix}/manifests/{reference}"),
            Some(&self.admin),
            serde_json::to_vec(manifest).unwrap(),
            Some(media_type),
        )
        .await
    }

    async fn push_image(&self, prefix: &str, tag: &str, layer: &[u8]) -> String {
        let config = self.push_blob(prefix, b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(prefix, layer).await;
        let manifest = image_manifest(&config, &[(&layer, LAYER)], None, None);
        let reply = self.push_manifest(prefix, tag, &manifest).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&serde_json::to_vec(&manifest).unwrap())
    }

    async fn push_artifact(
        &self,
        prefix: &str,
        subject: &str,
        reference: Option<&str>,
        artifact_type: Option<&str>,
        layer_type: &str,
        payload: &str,
    ) -> String {
        let config = self.push_blob(prefix, b"{}").await;
        let layer = self.push_blob(prefix, payload.as_bytes()).await;
        let subject = reference.is_none().then_some(subject);
        let manifest = image_manifest(&config, &[(&layer, layer_type)], artifact_type, subject);
        let digest = sha256(&serde_json::to_vec(&manifest).unwrap());
        let reference = reference.unwrap_or(&digest);
        let reply = self.push_manifest(prefix, reference, &manifest).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn rows(&self, repository: &str, parent: &str) -> Vec<String> {
        let rows = self
            .fixture
            .state
            .db()
            .conn()
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT child FROM docker_manifest_child WHERE repository = $1 AND parent = $2 ORDER BY child",
                vec![repository.into(), parent.into()],
            ))
            .await
            .unwrap();
        rows.iter()
            .map(|row| row.try_get::<String>("", "child").unwrap())
            .collect()
    }

    async fn row_count(&self, repository: &str) -> i64 {
        self.fixture
            .state
            .db()
            .conn()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT count(*) AS n FROM docker_manifest_child WHERE repository = $1",
                vec![repository.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "n")
            .unwrap()
    }

    async fn execute(&self, sql: &str) {
        self.fixture
            .state
            .db()
            .conn()
            .execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
    }
}

fn sorted(mut digests: Vec<String>) -> Vec<String> {
    digests.sort();
    digests
}

fn pairs(items: &Value, field: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["digest"].as_str().unwrap().to_owned(),
                item[field].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    pairs.sort();
    pairs
}

fn digests(items: &Value) -> Vec<String> {
    sorted(
        items
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["digest"].as_str().unwrap().to_owned())
            .collect(),
    )
}

async fn allow_loopback(registry: &Registry) {
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
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

async fn serve_over_tcp(registry: &Registry) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = registry.app.clone();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

#[tokio::test]
async fn index_children_are_recorded_for_their_index_only() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("hosted", "ALLOW"))
        .await;
    let prefix = "/v2/hosted/app";
    let first = registry.push_image(prefix, "amd64", b"amd64 layer").await;
    let second = registry.push_image(prefix, "arm64", b"arm64 layer").await;
    let multi = index(&[&first, &second]);
    let pushed = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let parent = sha256(&serde_json::to_vec(&multi).unwrap());
    let expected = sorted(vec![first.clone(), second.clone()]);
    assert_eq!(registry.rows("hosted", &parent).await, expected);

    let single = index(&[&first]);
    let pushed = registry.push_manifest(prefix, "single", &single).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let other = sha256(&serde_json::to_vec(&single).unwrap());
    assert_eq!(registry.rows("hosted", &other).await, vec![first.clone()]);
    assert_eq!(registry.rows("hosted", &parent).await, expected);

    let again = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(again.status, StatusCode::CREATED, "{}", again.text());
    assert_eq!(registry.rows("hosted", &parent).await, expected);
}

#[tokio::test]
async fn non_index_manifests_write_no_child_rows() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("hosted", "ALLOW"))
        .await;
    registry.push_image("/v2/hosted/app", "1.0", b"layer").await;
    assert_eq!(registry.row_count("hosted").await, 0);
}

#[tokio::test]
async fn deleting_an_index_removes_its_child_rows() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("hosted", "ALLOW"))
        .await;
    let prefix = "/v2/hosted/app";
    let first = registry.push_image(prefix, "amd64", b"amd64 layer").await;
    let multi = index(&[&first]);
    let pushed = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let parent = sha256(&serde_json::to_vec(&multi).unwrap());
    assert_eq!(registry.rows("hosted", &parent).await, vec![first.clone()]);

    let deleted = registry
        .admin(Method::DELETE, &format!("{prefix}/manifests/{parent}"))
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED, "{}", deleted.text());
    assert_eq!(registry.rows("hosted", &parent).await, Vec::<String>::new());
    let child = registry
        .admin(Method::GET, &format!("{prefix}/manifests/{first}"))
        .await;
    assert_eq!(child.status, StatusCode::OK);
}

#[tokio::test]
async fn proxies_record_the_children_of_a_cached_index() {
    let registry = Registry::new().await;
    allow_loopback(&registry).await;
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "DockerToken"],
    )
    .await
    .unwrap();
    registry.fixture.state.security().changed().await.unwrap();
    registry
        .create("hosted", hosted_body("upstream", "ALLOW"))
        .await;
    let prefix = "/v2/upstream/library/app";
    let first = registry.push_image(prefix, "amd64", b"amd64 layer").await;
    let second = registry.push_image(prefix, "arm64", b"arm64 layer").await;
    let multi = index(&[&first, &second]);
    let pushed = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let address = serve_over_tcp(&registry).await;
    registry
        .create(
            "proxy",
            json!({
                "name": "mirror",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": format!("http://{address}/repository/upstream"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false, "authentication": {"type": "username", "username": "admin", "password": registry.fixture.admin_password}},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "HUB"}
            }),
        )
        .await;
    let pulled = registry
        .admin(Method::GET, "/v2/mirror/app/manifests/multi")
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    let parent = sha256(&serde_json::to_vec(&multi).unwrap());
    assert_eq!(
        registry.rows("mirror", &parent).await,
        sorted(vec![first, second])
    );
}

#[tokio::test]
async fn the_backfill_task_records_stored_indexes_once_and_skips_unreadable_ones() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("images", "ALLOW"))
        .await;
    registry
        .fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_docker::children::Children));
    let prefix = "/v2/images/app";
    let first = registry.push_image(prefix, "amd64", b"amd64 layer").await;
    let second = registry.push_image(prefix, "arm64", b"arm64 layer").await;
    let multi = index(&[&first, &second]);
    let pushed = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let parent = sha256(&serde_json::to_vec(&multi).unwrap());
    let broken = registry.push_image(prefix, "old", b"old layer").await;
    registry
        .execute(&format!(
            "UPDATE asset SET content_type = '{OCI_INDEX}', attributes = jsonb_set(attributes, '{{docker,mediaType}}', '\"{OCI_INDEX}\"') \
             WHERE repository = 'images' AND kind = 'manifest' AND path = '/v2/app/manifests/{broken}'"
        ))
        .await;
    registry.execute("DELETE FROM docker_manifest_child").await;
    assert_eq!(registry.row_count("images").await, 0);

    let created = registry
        .fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "docker.children", "name": "Children", "properties": {"repository": "images"}})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    run_task(&registry, &id).await;
    let run = last_run(&registry, &id).await;
    assert_eq!(run["state"], "done", "{run}");
    assert_eq!(run["counts"]["indexes"], 1, "{run}");
    assert_eq!(run["counts"]["children"], 2, "{run}");
    assert_eq!(
        registry.rows("images", &parent).await,
        sorted(vec![first.clone(), second.clone()])
    );
    let log = registry
        .fixture
        .call(
            Method::GET,
            &format!(
                "/service/rest/internal/ui/tasks/{id}/runs/{}/log",
                run["id"]
            ),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(
        log["lines"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line["message"].as_str().unwrap().contains("is skipped")),
        "{log}"
    );

    run_task(&registry, &id).await;
    let again = last_run(&registry, &id).await;
    assert_eq!(again["counts"]["children"], 2, "{again}");
    assert_eq!(registry.row_count("images").await, 2);
    assert_eq!(
        registry.rows("images", &parent).await,
        sorted(vec![first, second])
    );
}

async fn run_task(registry: &Registry, id: &str) {
    let reply = registry
        .fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/tasks/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    registry.fixture.state.tasks().run_pending().await.unwrap();
}

async fn last_run(registry: &Registry, id: &str) -> Value {
    let runs = registry
        .fixture
        .call(
            Method::GET,
            &format!("/service/rest/internal/ui/tasks/{id}/runs"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(runs.status, StatusCode::OK, "{}", runs.text());
    runs.json()[0].clone()
}

#[tokio::test]
async fn supply_chain_lists_signatures_attestations_and_sboms_of_an_image() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("hosted", "ALLOW"))
        .await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"base").await;
    let hex = subject.trim_start_matches("sha256:").to_owned();
    let first = registry.push_image(prefix, "amd64", b"amd64 layer").await;
    let second = registry.push_image(prefix, "arm64", b"arm64 layer").await;
    let multi = index(&[&first, &second]);
    let pushed = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let parent = sha256(&serde_json::to_vec(&multi).unwrap());

    let legacy_sig = registry
        .push_artifact(
            prefix,
            &subject,
            Some(&format!("sha256-{hex}.sig")),
            None,
            SIMPLE_SIGNING,
            "legacy signature",
        )
        .await;
    let legacy_att = registry
        .push_artifact(
            prefix,
            &subject,
            Some(&format!("sha256-{hex}.att")),
            Some(IN_TOTO),
            IN_TOTO,
            "legacy attestation",
        )
        .await;
    let legacy_sbom = registry
        .push_artifact(
            prefix,
            &subject,
            Some(&format!("sha256-{hex}.sbom")),
            Some(SPDX),
            SPDX,
            "legacy sbom",
        )
        .await;
    let cosign = registry
        .push_artifact(
            prefix,
            &subject,
            None,
            Some(COSIGN),
            SIMPLE_SIGNING,
            "cosign",
        )
        .await;
    let simple = registry
        .push_artifact(
            prefix,
            &subject,
            None,
            None,
            SIMPLE_SIGNING,
            "simple signing",
        )
        .await;
    let bundle = registry
        .push_artifact(prefix, &subject, None, Some(BUNDLE), BUNDLE, "bundle")
        .await;
    let notary = registry
        .push_artifact(prefix, &subject, None, Some(NOTARY), NOTARY, "notary")
        .await;
    let attestation = registry
        .push_artifact(
            prefix,
            &subject,
            None,
            Some(IN_TOTO),
            IN_TOTO,
            "attestation",
        )
        .await;
    let spdx = registry
        .push_artifact(prefix, &subject, None, Some(SPDX), SPDX, "spdx")
        .await;
    let cyclonedx = registry
        .push_artifact(
            prefix,
            &subject,
            None,
            Some(CYCLONEDX),
            CYCLONEDX,
            "cyclonedx",
        )
        .await;

    let path = format!("{SUPPLY}/hosted/supply-chain?image=app&reference=1.0");
    let listed = registry.admin(Method::GET, &path).await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let listing = listed.json();
    assert_eq!(listing["digest"], subject.as_str());
    assert_eq!(listing["children"], json!([]));
    assert_eq!(
        pairs(&listing["signatures"], "scheme"),
        sorted_pairs(&[
            (&legacy_sig, "legacy"),
            (&cosign, "referrer"),
            (&simple, "referrer"),
            (&bundle, "bundle"),
            (&notary, "notation"),
        ])
    );
    assert_eq!(
        digests(&listing["attestations"]),
        sorted(vec![legacy_att, attestation])
    );
    assert_eq!(
        pairs(&listing["sboms"], "format"),
        sorted_pairs(&[
            (&legacy_sbom, "spdx"),
            (&spdx, "spdx"),
            (&cyclonedx, "cyclonedx")
        ])
    );

    let by_digest = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/hosted/supply-chain?image=app&reference={subject}"),
        )
        .await
        .json();
    assert_eq!(by_digest, listing);

    let children = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/hosted/supply-chain?image=app&reference=multi"),
        )
        .await
        .json();
    assert_eq!(children["digest"], parent.as_str());
    assert_eq!(children["children"], json!([first, second]));
}

fn sorted_pairs(items: &[(&String, &str)]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = items
        .iter()
        .map(|(digest, scheme)| ((*digest).clone(), (*scheme).to_owned()))
        .collect();
    pairs.sort();
    pairs
}

#[tokio::test]
async fn supply_chain_refuses_unknown_references_and_callers_without_read() {
    let registry = Registry::new().await;
    registry
        .create("hosted", hosted_body("hosted", "ALLOW"))
        .await;
    registry.push_image("/v2/hosted/app", "1.0", b"base").await;
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
    let path = format!("{SUPPLY}/hosted/supply-chain?image=app&reference=1.0");

    let unknown = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/hosted/supply-chain?image=app&reference=missing"),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND, "{}", unknown.text());
    let anonymous = registry
        .send(Method::GET, &path, None, Vec::new(), None)
        .await;
    assert_eq!(
        anonymous.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        anonymous.text()
    );
    let unreadable = registry
        .send(Method::GET, &path, Some(&reader), Vec::new(), None)
        .await;
    assert_eq!(
        unreadable.status,
        StatusCode::FORBIDDEN,
        "{}",
        unreadable.text()
    );
    let not_docker = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/files/supply-chain?image=app&reference=1.0"),
        )
        .await;
    assert_eq!(
        not_docker.status,
        StatusCode::BAD_REQUEST,
        "{}",
        not_docker.text()
    );
    let missing = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/nowhere/supply-chain?image=app&reference=1.0"),
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND, "{}", missing.text());
}

#[tokio::test]
async fn group_listings_merge_their_members_with_each_entry_once() {
    let registry = Registry::new().await;
    registry.create("hosted", hosted_body("a", "ALLOW")).await;
    registry.create("hosted", hosted_body("b", "ALLOW")).await;
    let subject = registry.push_image("/v2/a/app", "1.0", b"base").await;
    let copied = registry.push_image("/v2/b/app", "1.0", b"base").await;
    assert_eq!(subject, copied);
    let hex = subject.trim_start_matches("sha256:").to_owned();
    let legacy = registry
        .push_artifact(
            "/v2/a/app",
            &subject,
            Some(&format!("sha256-{hex}.sig")),
            None,
            SIMPLE_SIGNING,
            "legacy signature",
        )
        .await;
    let shared = registry
        .push_artifact(
            "/v2/a/app",
            &subject,
            None,
            Some(COSIGN),
            SIMPLE_SIGNING,
            "cosign",
        )
        .await;
    let again = registry
        .push_artifact(
            "/v2/b/app",
            &subject,
            None,
            Some(COSIGN),
            SIMPLE_SIGNING,
            "cosign",
        )
        .await;
    assert_eq!(shared, again);
    registry
        .create(
            "group",
            json!({
                "name": "all",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["a", "b"]},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
            }),
        )
        .await;
    let listed = registry
        .admin(
            Method::GET,
            &format!("{SUPPLY}/all/supply-chain?image=app&reference=1.0"),
        )
        .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let listing = listed.json();
    assert_eq!(listing["digest"], subject.as_str());
    assert_eq!(
        pairs(&listing["signatures"], "scheme"),
        sorted_pairs(&[(&legacy, "legacy"), (&shared, "referrer")])
    );
}
