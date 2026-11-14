mod support;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use astrum_tasks::cleanup::Criteria;
use astrum_tasks::cleanup::query::Matcher;
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};
use tokio::task::JoinHandle;
use tower::ServiceExt;

const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const DOCKER_CONFIG: &str = "application/vnd.docker.container.image.v1+json";
const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const COSIGN: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
const BUNDLE: &str = "application/vnd.dev.sigstore.bundle.v0.3+json";
const IN_TOTO: &str = "application/vnd.in-toto+json";
const ANNOTATION: &str = "dev.cosignproject.cosign/signature";
const KEYS: &str = "/service/rest/v1/security/keys";
const POLICIES: &str = "/service/rest/v1/security/policies";
const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";

struct Spec<'a> {
    key: &'a str,
    layers: Vec<(&'a str, Vec<u8>)>,
    artifact_type: Option<&'a str>,
    subject: Option<&'a str>,
    docker: bool,
    foreign: bool,
}

fn genuine(media_type: &str, marker: &str) -> Vec<u8> {
    let document = match media_type {
        SIMPLE_SIGNING => {
            json!({"critical": {"identity": {"docker-reference": marker}, "image": {"docker-manifest-digest": "sha256:aa"}, "type": "cosign container image signature"}})
        }
        IN_TOTO => {
            json!({"_type": "https://in-toto.io/Statement/v1", "subject": [{"name": marker}], "predicateType": "https://slsa.dev/provenance/v1", "predicate": {}})
        }
        "application/vnd.dsse.envelope.v1+json" => {
            json!({"payloadType": IN_TOTO, "payload": "e30=", "signatures": [{"sig": marker}]})
        }
        "application/spdx+json" | "text/spdx+json" => {
            json!({"spdxVersion": "SPDX-2.3", "name": marker})
        }
        "application/vnd.cyclonedx+json" => {
            json!({"bomFormat": "CycloneDX", "specVersion": "1.5", "serialNumber": marker})
        }
        "application/vnd.syft+json" => json!({"artifacts": [], "schema": {"version": marker}}),
        "application/vnd.cncf.notary.signature" | "application/jose+json" | "application/cose" => {
            json!({"payload": "e30", "protected": "e30", "header": {"x": marker}, "signature": "AA"})
        }
        other if other.starts_with("application/vnd.dev.sigstore.bundle") => {
            json!({"mediaType": other, "verificationMaterial": {"x": marker}})
        }
        _ => json!({"media": media_type, "marker": marker}),
    };
    serde_json::to_vec(&document).unwrap()
}

fn tar_header(name: &str, field: fn(&str) -> String) -> Vec<u8> {
    let mut block = vec![b'a'; 512];
    block[..name.len()].copy_from_slice(name.as_bytes());
    let mut put =
        |at: usize, text: &str| block[at..at + text.len()].copy_from_slice(text.as_bytes());
    put(100, "0000755 ");
    put(108, "0000000 ");
    put(116, "0000000 ");
    put(124, "00000001000 ");
    put(136, "00000000000 ");
    put(148, "        ");
    block[156] = b'0';
    let sum: u32 = block.iter().map(|byte| u32::from(*byte)).sum();
    block[148..156].copy_from_slice(field(&format!("{sum:o}")).as_bytes());
    block
}

fn exact(sum: &str) -> String {
    format!("{sum:0>7} ")
}

fn polyglot_with(prefix: &str, offset: usize, field: fn(&str) -> String) -> Vec<u8> {
    let mut blob = Vec::new();
    if offset == 0 {
        blob.extend(tar_header(prefix, field));
        blob.extend(vec![b'b'; 512]);
    } else {
        let mut first = vec![b'x'; 512];
        first[..prefix.len()].copy_from_slice(prefix.as_bytes());
        blob.extend(first);
        blob.extend(tar_header("", field));
    }
    blob.extend(b"\"}");
    blob
}

fn polyglot(prefix: &str, offset: usize) -> Vec<u8> {
    polyglot_with(prefix, offset, exact)
}

fn tar_gz(name: &str) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    let content = b"layer content";
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, name, &content[..])
        .unwrap();
    let archive = builder.into_inner().unwrap();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&archive).unwrap();
    encoder.finish().unwrap()
}

struct Signer {
    pair: Ed25519KeyPair,
    pem: String,
}

impl Signer {
    fn new() -> Self {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let mut der = vec![
            0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
        ];
        der.extend_from_slice(pair.public_key().as_ref());
        let pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            STANDARD.encode(der)
        );
        Signer { pair, pem }
    }

    fn sign(&self, message: &[u8]) -> String {
        STANDARD.encode(self.pair.sign(message).as_ref())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Via {
    Tag,
    Referrer,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn payload(digest: &str) -> Vec<u8> {
    json!({"critical": {"identity": {"docker-reference": "example.test/app"}, "image": {"docker-manifest-digest": digest}, "type": "cosign container image signature"}, "optional": null})
        .to_string()
        .into_bytes()
}

fn attestation_bundle(signer: &Signer, subject: &str) -> Vec<u8> {
    let statement = json!({"_type": "https://in-toto.io/Statement/v1", "subject": [{"name": "app", "digest": {"sha256": &subject[7..]}}], "predicateType": "https://slsa.dev/provenance/v1", "predicate": {}})
        .to_string()
        .into_bytes();
    let mut signed =
        format!("DSSEv1 {} {IN_TOTO} {} ", IN_TOTO.len(), statement.len()).into_bytes();
    signed.extend_from_slice(&statement);
    json!({"mediaType": BUNDLE, "verificationMaterial": {}, "dsseEnvelope": {"payload": STANDARD.encode(&statement), "payloadType": IN_TOTO, "signatures": [{"sig": signer.sign(&signed)}]}})
        .to_string()
        .into_bytes()
}

fn hosted_body(name: &str, policy: Option<&str>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    body
}

fn image_manifest(config: &str, layer: &str) -> Value {
    json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": OCI_CONFIG, "digest": config, "size": 14},
        "layers": [{"mediaType": LAYER, "digest": layer, "size": 1}],
    })
}

fn index(children: &[&str]) -> Value {
    json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": children.iter().map(|digest| json!({"mediaType": OCI_MANIFEST, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    })
}

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
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
        body: Vec<u8>,
        content_type: Option<&str>,
    ) -> Reply {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, &self.admin);
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

    async fn rest(&self, method: Method, path: &str, body: Option<Value>) -> Reply {
        self.fixture.call(method, path, Auth::Admin, body).await
    }

    async fn repository(&self, kind: &str, body: Value) {
        let reply = self
            .rest(
                Method::POST,
                &format!("/service/rest/v1/repositories/docker/{kind}"),
                Some(body),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn key(&self, name: &str, signer: &Signer) {
        let reply = self
            .rest(
                Method::POST,
                KEYS,
                Some(json!({"name": name, "publicKey": signer.pem})),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn policy(&self, name: &str, mode: &str, keys: &[&str]) {
        let body = json!({"name": name, "signatures": {"mode": mode, "keys": keys}});
        let reply = self.rest(Method::POST, POLICIES, Some(body)).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn change_policy(&self, name: &str, mode: &str, keys: &[&str]) {
        let body = json!({"signatures": {"mode": mode, "keys": keys}});
        let path = format!("{POLICIES}/{name}");
        let reply = self.rest(Method::PUT, &path, Some(body)).await;
        assert!(reply.status.is_success(), "{}", reply.text());
        tokio::time::sleep(Duration::from_millis(1100)).await;
    }

    async fn push_blob(&self, prefix: &str, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let reply = self
            .send(
                Method::POST,
                &format!("{prefix}/blobs/uploads/?digest={digest}"),
                bytes.to_vec(),
                Some("application/octet-stream"),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn push_manifest(&self, prefix: &str, reference: &str, manifest: &Value) -> String {
        let bytes = serde_json::to_vec(manifest).unwrap();
        let media_type = manifest["mediaType"].as_str().unwrap_or(OCI_MANIFEST);
        let reply = self
            .send(
                Method::PUT,
                &format!("{prefix}/manifests/{reference}"),
                bytes.clone(),
                Some(media_type),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&bytes)
    }

    async fn push_artifact(
        &self,
        prefix: &str,
        layer_key: &str,
        layers: &[&str],
        artifact_type: &str,
        subject: Option<&str>,
    ) -> String {
        self.push_salted(prefix, layer_key, layers, artifact_type, subject, "")
            .await
    }

    async fn push_salted(
        &self,
        prefix: &str,
        layer_key: &str,
        layers: &[&str],
        artifact_type: &str,
        subject: Option<&str>,
        salt: &str,
    ) -> String {
        let layers = layers
            .iter()
            .enumerate()
            .map(|(index, media_type)| {
                let marker = format!("{salt}{index}{subject:?}");
                (*media_type, genuine(media_type, &marker))
            })
            .collect();
        let spec = Spec {
            key: layer_key,
            layers,
            artifact_type: Some(artifact_type),
            subject,
            docker: false,
            foreign: false,
        };
        self.push_spec(prefix, spec).await.0
    }

    async fn push_spec(&self, prefix: &str, spec: Spec<'_>) -> (String, Vec<String>) {
        let config_type = if spec.docker {
            DOCKER_CONFIG
        } else {
            OCI_CONFIG
        };
        let config = self.push_blob(prefix, b"{\"os\":\"linux\"}").await;
        let mut described = Vec::new();
        let mut blobs = Vec::new();
        let mut pushed: std::collections::HashMap<Vec<u8>, String> = Default::default();
        for (media_type, bytes) in &spec.layers {
            let digest = match pushed.get(bytes) {
                Some(known) => known.clone(),
                None => {
                    let fresh = self.push_blob(prefix, bytes).await;
                    pushed.insert(bytes.clone(), fresh.clone());
                    fresh
                }
            };
            let mut layer = json!({"mediaType": media_type, "digest": digest, "size": bytes.len()});
            if spec.foreign {
                layer["urls"] = json!(["https://layers.example.test/blob"]);
            }
            described.push(layer);
            blobs.push(digest);
        }
        let mut manifest = json!({
            "schemaVersion": 2,
            "mediaType": if spec.docker { DOCKER_MANIFEST } else { OCI_MANIFEST },
            "config": {"mediaType": config_type, "digest": config, "size": 14},
        });
        manifest[spec.key] = Value::Array(described);
        if let Some(artifact_type) = spec.artifact_type {
            manifest["artifactType"] = artifact_type.into();
        }
        if let Some(subject) = spec.subject {
            manifest["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": subject, "size": 2});
        }
        let digest = sha256(&serde_json::to_vec(&manifest).unwrap());
        (self.push_manifest(prefix, &digest, &manifest).await, blobs)
    }

    async fn push_image(&self, prefix: &str, tag: &str, layer: &[u8]) -> String {
        let config = self.push_blob(prefix, b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(prefix, layer).await;
        self.push_manifest(prefix, tag, &image_manifest(&config, &layer))
            .await
    }

    async fn push_signature(
        &self,
        prefix: &str,
        subject: &str,
        payload: &[u8],
        signature: &str,
        via: Via,
    ) -> String {
        let config = self.push_blob(prefix, b"{}").await;
        let layer = self.push_blob(prefix, payload).await;
        let mut manifest = json!({
            "schemaVersion": 2,
            "mediaType": OCI_MANIFEST,
            "config": {"mediaType": if via == Via::Tag { OCI_CONFIG } else { EMPTY }, "digest": config, "size": 2},
            "layers": [{"mediaType": SIMPLE_SIGNING, "digest": layer, "size": payload.len(), "annotations": {ANNOTATION: signature}}],
        });
        let reference = match via {
            Via::Tag => format!("sha256-{}.sig", &subject[7..]),
            Via::Referrer => {
                manifest["artifactType"] = COSIGN.into();
                manifest["subject"] =
                    json!({"mediaType": OCI_MANIFEST, "digest": subject, "size": 2});
                sha256(&serde_json::to_vec(&manifest).unwrap())
            }
        };
        self.push_manifest(prefix, &reference, &manifest).await
    }

    async fn sign(&self, prefix: &str, subject: &str, signer: &Signer, via: Via) -> String {
        let payload = payload(subject);
        let signature = signer.sign(&payload);
        self.push_signature(prefix, subject, &payload, &signature, via)
            .await
    }

    async fn pull(&self, prefix: &str, reference: &str) -> Reply {
        self.send(
            Method::GET,
            &format!("{prefix}/manifests/{reference}"),
            Vec::new(),
            None,
        )
        .await
    }

    async fn recorded(&self, repository: &str, digest: &str) -> Option<(String, Option<String>)> {
        let row = self
            .fixture
            .state
            .db()
            .conn()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT state, signer FROM oci_signature_check WHERE repository = $1 AND digest = $2",
                vec![repository.into(), digest.into()],
            ))
            .await
            .unwrap()?;
        Some((
            row.try_get("", "state").unwrap(),
            row.try_get("", "signer").unwrap(),
        ))
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

fn refusal(reply: &Reply) -> String {
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["errors"][0]["code"], "DENIED");
    body["errors"][0]["message"].as_str().unwrap().to_owned()
}

async fn signed_only(registry: &Registry, mode: &str) -> Signer {
    let signer = Signer::new();
    registry.key("release", &signer).await;
    registry.policy("signed-only", mode, &["release"]).await;
    registry
        .repository("hosted", hosted_body("hosted", Some("signed-only")))
        .await;
    signer
}

#[tokio::test]
async fn an_unsigned_image_is_refused_in_block_mode_and_only_recorded_in_report_mode() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"layer").await;
    for reference in ["1.0", digest.as_str()] {
        let message = refusal(&registry.pull(prefix, reference).await);
        assert_eq!(
            message,
            format!(
                "signature: {} is not signed by a trusted key of policy signed-only",
                &digest[..19]
            )
        );
    }
    let head = registry
        .send(
            Method::HEAD,
            &format!("{prefix}/manifests/1.0"),
            Vec::new(),
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::FORBIDDEN);
    assert_eq!(
        registry.recorded("hosted", &digest).await,
        Some(("unsigned".to_owned(), None))
    );

    registry
        .change_policy("signed-only", "report", &["release"])
        .await;
    assert_eq!(registry.pull(prefix, "1.0").await.status, StatusCode::OK);
    assert_eq!(
        registry.recorded("hosted", &digest).await,
        Some(("unsigned".to_owned(), None))
    );
    registry
        .change_policy("signed-only", "off", &["release"])
        .await;
    assert_eq!(registry.pull(prefix, &digest).await.status, StatusCode::OK);
}

#[tokio::test]
async fn a_tag_signature_and_a_referrer_signature_by_a_trusted_key_both_sign_an_image() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let by_tag = registry.push_image(prefix, "tag", b"signed by tag").await;
    let by_referrer = registry
        .push_image(prefix, "referrer", b"signed by referrer")
        .await;
    let unsigned = registry.push_image(prefix, "bare", b"unsigned").await;
    registry.sign(prefix, &by_tag, &signer, Via::Tag).await;
    let artifact = registry
        .sign(prefix, &by_referrer, &signer, Via::Referrer)
        .await;
    for (reference, digest) in [("tag", &by_tag), ("referrer", &by_referrer)] {
        for wanted in [reference, digest.as_str()] {
            let pulled = registry.pull(prefix, wanted).await;
            assert_eq!(pulled.status, StatusCode::OK, "{wanted}: {}", pulled.text());
        }
        let row = registry.recorded("hosted", digest).await;
        assert_eq!(row, Some(("signed".to_owned(), Some("release".to_owned()))));
    }
    refusal(&registry.pull(prefix, "bare").await);
    let sig_tag = format!("sha256-{}.sig", &by_tag[7..]);
    assert_eq!(registry.pull(prefix, &sig_tag).await.status, StatusCode::OK);
    assert_eq!(
        registry.pull(prefix, &artifact).await.status,
        StatusCode::OK
    );
    assert_eq!(
        registry.recorded("hosted", &unsigned).await.unwrap().0,
        "unsigned"
    );
}

#[tokio::test]
async fn signatures_of_other_keys_for_other_digests_and_tampered_ones_do_not_sign() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let stranger = Signer::new();
    registry.key("stranger", &stranger).await;
    let prefix = "/v2/hosted/app";
    let foreign = registry.push_image(prefix, "foreign", b"foreign key").await;
    let wrong = registry.push_image(prefix, "wrong", b"wrong digest").await;
    let tampered = registry.push_image(prefix, "tampered", b"tampered").await;
    registry.sign(prefix, &foreign, &stranger, Via::Tag).await;
    let other = payload(&sha256(b"another image"));
    registry
        .push_signature(prefix, &wrong, &other, &signer.sign(&other), Via::Tag)
        .await;
    let genuine = payload(&tampered);
    let mut forged = genuine.clone();
    forged[12] ^= 1;
    registry
        .push_signature(prefix, &tampered, &forged, &signer.sign(&genuine), Via::Tag)
        .await;
    for (tag, digest) in [
        ("foreign", &foreign),
        ("wrong", &wrong),
        ("tampered", &tampered),
    ] {
        let message = refusal(&registry.pull(prefix, tag).await);
        assert!(
            message.contains("is not signed by a trusted key"),
            "{message}"
        );
        assert_eq!(
            registry.recorded("hosted", digest).await.unwrap().0,
            "invalid",
            "{tag}"
        );
    }
}

#[tokio::test]
async fn an_attestation_signed_by_a_trusted_key_does_not_make_an_image_signed() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"attested only").await;
    let config = registry.push_blob(prefix, b"{}").await;
    let bundle = attestation_bundle(&signer, &digest);
    let layer = registry.push_blob(prefix, &bundle).await;
    let manifest = json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "artifactType": BUNDLE,
        "config": {"mediaType": EMPTY, "digest": config, "size": 2},
        "layers": [{"mediaType": BUNDLE, "digest": layer, "size": bundle.len()}],
        "subject": {"mediaType": OCI_MANIFEST, "digest": digest, "size": 2},
    });
    registry
        .push_manifest(
            prefix,
            &sha256(&serde_json::to_vec(&manifest).unwrap()),
            &manifest,
        )
        .await;
    let message = refusal(&registry.pull(prefix, "1.0").await);
    assert!(
        message.contains("only an attestation signed by release was found"),
        "{message}"
    );
    assert_eq!(
        registry.recorded("hosted", &digest).await.unwrap().0,
        "unsigned"
    );
}

const WASM: &str = "application/vnd.wasm.content.layer.v1+wasm";
const HELM: &str = "application/vnd.cncf.helm.chart.content.v1.tar+gzip";
const FLUX: &str = "application/vnd.cncf.flux.content.v1.tar+gzip";

#[tokio::test]
async fn image_layers_under_another_spelling_of_the_layers_key_are_not_exempt() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    for key in ["Layers", "LAYERS", "layer\u{17f}"] {
        let digest = registry
            .push_artifact(prefix, key, &[LAYER], COSIGN, Some(&subject))
            .await;
        let message = refusal(&registry.pull(prefix, &digest).await);
        assert!(message.contains("is not signed"), "{key}: {message}");
    }
}

#[tokio::test]
async fn artifacts_that_could_run_are_not_exempt_whatever_they_declare() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    for media_type in [WASM, HELM, FLUX, LAYER] {
        for declared in [Some(subject.as_str()), None] {
            let digest = registry
                .push_artifact(prefix, "layers", &[media_type], COSIGN, declared)
                .await;
            refusal(&registry.pull(prefix, &digest).await);
        }
    }
    let empty = registry
        .push_artifact(prefix, "layers", &[], COSIGN, Some(&subject))
        .await;
    refusal(&registry.pull(prefix, &empty).await);
    let mixed = registry
        .push_artifact(
            prefix,
            "layers",
            &[SIMPLE_SIGNING, LAYER],
            COSIGN,
            Some(&subject),
        )
        .await;
    refusal(&registry.pull(prefix, &mixed).await);
}

#[tokio::test]
async fn signature_sbom_and_attestation_artifacts_stay_fetchable() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    refusal(&registry.pull(prefix, &subject).await);
    for (layer, artifact_type) in [
        (SIMPLE_SIGNING, COSIGN),
        ("application/spdx+json", "application/spdx+json"),
        (
            "application/vnd.cyclonedx+json",
            "application/vnd.cyclonedx+json",
        ),
        (IN_TOTO, IN_TOTO),
        ("application/vnd.dsse.envelope.v1+json", IN_TOTO),
        ("application/vnd.cncf.notary.signature", COSIGN),
        ("application/jose+json", COSIGN),
        (BUNDLE, COSIGN),
    ] {
        let digest = registry
            .push_artifact(prefix, "layers", &[layer], artifact_type, Some(&subject))
            .await;
        let pulled = registry.pull(prefix, &digest).await;
        assert_eq!(pulled.status, StatusCode::OK, "{layer}: {}", pulled.text());
        let again = registry.pull(prefix, &digest).await;
        assert_eq!(again.status, StatusCode::OK, "{layer}: {}", again.text());
        assert_eq!(
            registry
                .recorded("hosted", &format!("content:{digest}"))
                .await,
            Some(("exempt".to_owned(), None))
        );
    }
}

#[tokio::test]
async fn junk_referrers_cannot_push_a_genuine_signature_out_of_the_examined_ones() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let first = registry.push_image(prefix, "first", b"first").await;
    registry.sign(prefix, &first, &signer, Via::Referrer).await;
    for number in 0..25 {
        let salt = number.to_string();
        registry
            .push_salted(
                prefix,
                "layers",
                &["application/octet-stream"],
                COSIGN,
                Some(&first),
                &salt,
            )
            .await;
    }
    let pulled = registry.pull(prefix, "first").await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());

    let second = registry.push_image(prefix, "second", b"second").await;
    for _ in 0..21 {
        registry
            .sign(prefix, &second, &Signer::new(), Via::Referrer)
            .await;
    }
    for number in 0..25 {
        let salt = number.to_string();
        registry
            .push_salted(
                prefix,
                "layers",
                &["application/octet-stream"],
                COSIGN,
                Some(&second),
                &salt,
            )
            .await;
    }
    registry.sign(prefix, &second, &signer, Via::Referrer).await;
    let pulled = registry.pull(prefix, "second").await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    let row = registry.recorded("hosted", &second).await;
    assert_eq!(row, Some(("signed".to_owned(), Some("release".to_owned()))));
}

const LABELS: [&str; 7] = [
    IN_TOTO,
    "application/spdx+json",
    "application/vnd.cyclonedx+json",
    "application/vnd.dsse.envelope.v1+json",
    SIMPLE_SIGNING,
    BUNDLE,
    "application/vnd.cncf.notary.signature",
];

#[tokio::test]
async fn signature_labels_on_layers_that_are_not_json_exempt_nothing() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    for label in LABELS {
        for (docker, declared) in [(false, Some(subject.as_str())), (false, None), (true, None)] {
            let spec = Spec {
                key: "layers",
                layers: vec![(label, tar_gz(label))],
                artifact_type: None,
                subject: declared,
                docker,
                foreign: false,
            };
            let (digest, _) = registry.push_spec(prefix, spec).await;
            let message = refusal(&registry.pull(prefix, &digest).await);
            assert!(message.contains("is not signed"), "{label}: {message}");
        }
    }
    let disguised = Spec {
        key: "layers",
        layers: vec![(EMPTY, tar_gz("empty"))],
        artifact_type: Some(COSIGN),
        subject: Some(&subject),
        docker: false,
        foreign: false,
    };
    refusal(
        &registry
            .pull(prefix, &registry.push_spec(prefix, disguised).await.0)
            .await,
    );
    let mixed = Spec {
        key: "layers",
        layers: vec![(EMPTY, b"{}".to_vec()), (LAYER, tar_gz("mixed"))],
        artifact_type: Some(COSIGN),
        subject: Some(&subject),
        docker: false,
        foreign: false,
    };
    refusal(
        &registry
            .pull(prefix, &registry.push_spec(prefix, mixed).await.0)
            .await,
    );
}

#[tokio::test]
async fn foreign_oversized_and_missing_layer_blobs_are_not_exempt() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    let json_layer = |what: &str| {
        (
            SIMPLE_SIGNING,
            json!({"what": what}).to_string().into_bytes(),
        )
    };
    let spec = |layers: Vec<(&'static str, Vec<u8>)>, foreign: bool| Spec {
        key: "layers",
        layers,
        artifact_type: Some(COSIGN),
        subject: Some(&subject),
        docker: false,
        foreign,
    };
    let (foreign, _) = registry
        .push_spec(prefix, spec(vec![json_layer("foreign")], true))
        .await;
    let big = json!({"pad": "x".repeat(1100 * 1024)})
        .to_string()
        .into_bytes();
    let (oversized, _) = registry
        .push_spec(prefix, spec(vec![(SIMPLE_SIGNING, big)], false))
        .await;
    let (lost, blobs) = registry
        .push_spec(prefix, spec(vec![json_layer("lost")], false))
        .await;
    registry
        .execute(&format!(
            "DELETE FROM asset WHERE repository = 'hosted' AND path = '/v2/-/blobs/{}'",
            blobs[0]
        ))
        .await;
    for digest in [foreign, oversized, lost] {
        refusal(&registry.pull(prefix, &digest).await);
    }
}

#[tokio::test]
async fn signatures_that_fail_the_cheap_checks_neither_use_a_slot_nor_turn_the_answer_into_an_error()
 {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let genuine = registry.push_image(prefix, "genuine", b"genuine").await;
    let bare = registry.push_image(prefix, "bare", b"bare").await;
    registry
        .sign(prefix, &genuine, &signer, Via::Referrer)
        .await;
    for subject in [&genuine, &bare] {
        for number in 0..25 {
            let other = payload(&sha256(format!("another {number}").as_bytes()));
            registry
                .push_signature(prefix, subject, &other, "AAAA", Via::Referrer)
                .await;
        }
        let spec = Spec {
            key: "layers",
            layers: vec![(
                SIMPLE_SIGNING,
                json!({"never": "stored"}).to_string().into_bytes(),
            )],
            artifact_type: Some(COSIGN),
            subject: Some(subject),
            docker: false,
            foreign: true,
        };
        registry.push_spec(prefix, spec).await;
    }
    let pulled = registry.pull(prefix, "genuine").await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    refusal(&registry.pull(prefix, "bare").await);
    assert_eq!(
        registry.recorded("hosted", &bare).await.unwrap().0,
        "invalid"
    );
}

const POLYGLOTS: [(&str, &str); 11] = [
    (
        SIMPLE_SIGNING,
        r#"{"critical":{"image":{},"type":"t"},"k":""#,
    ),
    (
        BUNDLE,
        r#"{"mediaType":"m","verificationMaterial":{},"k":""#,
    ),
    (
        IN_TOTO,
        r#"{"_type":"https://in-toto.io/Statement/v1","subject":[],"predicateType":"p","k":""#,
    ),
    (
        "application/vnd.dsse.envelope.v1+json",
        r#"{"payloadType":"a","payload":"b","signatures":[],"k":""#,
    ),
    (
        "application/spdx+json",
        r#"{"spdxVersion":"SPDX-2.3","k":""#,
    ),
    ("text/spdx+json", r#"{"spdxVersion":"SPDX-2.3","k":""#),
    (
        "application/vnd.cyclonedx+json",
        r#"{"bomFormat":"CycloneDX","k":""#,
    ),
    ("application/vnd.syft+json", r#"{"schema":{},"k":""#),
    (
        "application/vnd.cncf.notary.signature",
        r#"{"payload":"a","protected":"b","signature":"c","k":""#,
    ),
    (
        "application/jose+json",
        r#"{"payload":"a","protected":"b","signature":"c","k":""#,
    ),
    (
        "application/cose",
        r#"{"payload":"a","protected":"b","signature":"c","k":""#,
    ),
];

#[tokio::test]
async fn a_json_blob_that_is_also_a_tar_archive_is_not_exempt_under_any_label() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    for (label, start) in POLYGLOTS {
        for offset in [0, 512] {
            let spec = Spec {
                key: "layers",
                layers: vec![(label, polyglot(start, offset))],
                artifact_type: Some(COSIGN),
                subject: Some(&subject),
                docker: false,
                foreign: false,
            };
            let (digest, _) = registry.push_spec(prefix, spec).await;
            let message = refusal(&registry.pull(prefix, &digest).await);
            assert!(
                message.contains("is not signed"),
                "{label} {offset}: {message}"
            );
        }
    }
}

#[tokio::test]
async fn lenient_tar_checksum_fields_do_not_make_a_polyglot_exempt() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    let variants: [fn(&str) -> String; 4] = [
        |sum| format!("{sum} 7"),
        |sum| format!("+{sum} "),
        |sum| format!("0o{sum}"),
        |sum| format!("{}_{} ", &sum[..3], &sum[3..]),
    ];
    for (label, start) in [POLYGLOTS[0], POLYGLOTS[4]] {
        for field in variants {
            for offset in [0, 512] {
                let spec = Spec {
                    key: "layers",
                    layers: vec![(label, polyglot_with(start, offset, field))],
                    artifact_type: Some(COSIGN),
                    subject: Some(&subject),
                    docker: false,
                    foreign: false,
                };
                let (digest, _) = registry.push_spec(prefix, spec).await;
                refusal(&registry.pull(prefix, &digest).await);
            }
        }
    }
}

#[tokio::test]
async fn documents_of_the_wrong_shape_and_endless_layer_lists_are_not_exempt() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    let wrong = [
        (IN_TOTO, genuine("application/spdx+json", "x")),
        (SIMPLE_SIGNING, b"{}".to_vec()),
        ("application/spdx+json", genuine(IN_TOTO, "x")),
    ];
    for (label, bytes) in wrong {
        let spec = Spec {
            key: "layers",
            layers: vec![(label, bytes)],
            artifact_type: Some(COSIGN),
            subject: Some(&subject),
            docker: false,
            foreign: false,
        };
        let (digest, _) = registry.push_spec(prefix, spec).await;
        refusal(&registry.pull(prefix, &digest).await);
    }
    let mut layers: Vec<(&str, Vec<u8>)> = (0..2000).map(|_| (EMPTY, b"{}".to_vec())).collect();
    layers.push((IN_TOTO, tar_gz("last")));
    let spec = Spec {
        key: "layers",
        layers,
        artifact_type: Some(COSIGN),
        subject: Some(&subject),
        docker: false,
        foreign: false,
    };
    let (digest, _) = registry.push_spec(prefix, spec).await;
    refusal(&registry.pull(prefix, &digest).await);
}

#[tokio::test]
async fn a_genuine_signature_survives_up_to_the_examined_ceiling_of_newer_junk() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"image").await;
    registry.sign(prefix, &digest, &signer, Via::Referrer).await;
    let mut pushed = 0;
    for (target, served) in [(25, true), (150, true), (250, false)] {
        while pushed < target {
            registry
                .sign(prefix, &digest, &Signer::new(), Via::Referrer)
                .await;
            pushed += 1;
        }
        let pulled = registry.pull(prefix, "1.0").await;
        match served {
            true => assert_eq!(pulled.status, StatusCode::OK, "{target}: {}", pulled.text()),
            false => {
                refusal(&pulled);
                assert_eq!(
                    registry.recorded("hosted", &digest).await.unwrap().0,
                    "invalid"
                );
            }
        }
    }
}

#[tokio::test]
async fn a_platform_image_is_signed_through_its_signed_index_only_while_the_index_exists() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let amd64 = registry.push_image(prefix, "amd64", b"amd64").await;
    let arm64 = registry.push_image(prefix, "arm64", b"arm64").await;
    let stray = registry.push_image(prefix, "stray", b"stray").await;
    let multi = index(&[&amd64, &arm64]);
    let parent = registry.push_manifest(prefix, "multi", &multi).await;
    assert_eq!(
        registry.pull(prefix, &amd64).await.status,
        StatusCode::FORBIDDEN
    );
    registry.sign(prefix, &parent, &signer, Via::Tag).await;
    for reference in [&amd64, &arm64, &parent] {
        let pulled = registry.pull(prefix, reference).await;
        assert_eq!(
            pulled.status,
            StatusCode::OK,
            "{reference}: {}",
            pulled.text()
        );
    }
    refusal(&registry.pull(prefix, &stray).await);
    let row = registry.recorded("hosted", &amd64).await.unwrap();
    assert_eq!(row, ("signed".to_owned(), Some("release".to_owned())));

    registry
        .execute(&format!(
            "DELETE FROM asset WHERE repository = 'hosted' AND attributes #>> '{{docker,digest}}' = '{parent}'"
        ))
        .await;
    let rows = registry
        .fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM docker_manifest_child WHERE repository = 'hosted'",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(rows, 2);
    refusal(&registry.pull(prefix, &amd64).await);
    assert_eq!(
        registry.recorded("hosted", &amd64).await.unwrap().0,
        "unsigned"
    );
}

#[tokio::test]
async fn keys_signatures_and_policy_changes_are_picked_up_without_stale_answers() {
    let registry = Registry::new().await;
    let release = signed_only(&registry, "block").await;
    let backup = Signer::new();
    registry.key("backup", &backup).await;
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"image").await;
    registry.sign(prefix, &digest, &release, Via::Tag).await;
    assert_eq!(registry.pull(prefix, "1.0").await.status, StatusCode::OK);

    registry
        .change_policy("signed-only", "block", &["backup"])
        .await;
    refusal(&registry.pull(prefix, "1.0").await);
    assert_eq!(
        registry.recorded("hosted", &digest).await.unwrap().0,
        "invalid"
    );

    registry.sign(prefix, &digest, &backup, Via::Referrer).await;
    assert_eq!(registry.pull(prefix, "1.0").await.status, StatusCode::OK);
    let row = registry.recorded("hosted", &digest).await.unwrap();
    assert_eq!(row, ("signed".to_owned(), Some("backup".to_owned())));

    registry
        .change_policy("signed-only", "block", &["release"])
        .await;
    assert_eq!(registry.pull(prefix, "1.0").await.status, StatusCode::OK);
    let row = registry.recorded("hosted", &digest).await.unwrap();
    assert_eq!(row, ("signed".to_owned(), Some("release".to_owned())));
}

#[tokio::test]
async fn an_exception_for_a_digest_or_an_image_lifts_the_refusal() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    let prefix = "/v2/hosted/app";
    let by_digest = registry.push_image(prefix, "digest", b"digest").await;
    let by_name = registry
        .push_image("/v2/hosted/legacy", "1.0", b"name")
        .await;
    refusal(&registry.pull(prefix, "digest").await);
    for body in [
        json!({"policy": "signed-only", "rule": "signature", "digest": by_digest, "reason": "accepted"}),
        json!({"policy": "signed-only", "rule": "signature", "name": "legacy", "reason": "old image"}),
    ] {
        let reply = registry.rest(Method::POST, EXCEPTIONS, Some(body)).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(registry.pull(prefix, "digest").await.status, StatusCode::OK);
    assert_eq!(
        registry.pull("/v2/hosted/legacy", &by_name).await.status,
        StatusCode::OK
    );
    let other = registry.push_image(prefix, "other", b"other").await;
    refusal(&registry.pull(prefix, &other).await);
}

#[tokio::test]
async fn a_group_never_serves_a_manifest_that_its_member_refuses() {
    let registry = Registry::new().await;
    signed_only(&registry, "block").await;
    registry
        .repository("hosted", hosted_body("plain", None))
        .await;
    registry
        .repository(
            "group",
            json!({
                "name": "combined",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["hosted", "plain"]},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
            }),
        )
        .await;
    let digest = registry
        .push_image("/v2/hosted/app", "1.0", b"unsigned in the member")
        .await;
    registry
        .push_image("/v2/plain/app", "1.0", b"unsigned in the plain one")
        .await;
    let message = refusal(&registry.pull("/v2/combined/app", "1.0").await);
    assert!(message.contains("policy signed-only"), "{message}");
    refusal(&registry.pull("/v2/combined/app", &digest).await);
    let signer = Signer::new();
    registry.key("late", &signer).await;
    registry
        .sign("/v2/hosted/app", &digest, &signer, Via::Tag)
        .await;
    registry
        .change_policy("signed-only", "block", &["late"])
        .await;
    assert_eq!(
        registry.pull("/v2/combined/app", "1.0").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_cleanup_keeps_the_artifacts_of_existing_subjects_and_nothing_else() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "off").await;
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"image").await;
    registry.sign(prefix, &digest, &signer, Via::Tag).await;
    let sig_tag = format!("sha256-{}.sig", &digest[7..]);
    let other = registry.push_image(prefix, "2.0", b"other image").await;
    let fake_tag = format!("sha256-{}.sig", &other[7..]);
    let helm_config = registry.push_blob(prefix, b"{}").await;
    let helm = json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": "application/vnd.cncf.helm.config.v1+json", "digest": helm_config, "size": 2},
        "layers": [],
    });
    registry.push_manifest(prefix, &fake_tag, &helm).await;
    let config = registry.push_blob(prefix, b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(prefix, b"pointing at an image").await;
    let mut pointing = image_manifest(&config, &layer);
    pointing["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": digest, "size": 2});
    registry.push_manifest(prefix, "pointing", &pointing).await;
    let named = format!("sha256-{}.att", &digest[7..]);
    let junk_layer = registry
        .push_blob(prefix, b"an image under an attestation tag")
        .await;
    registry
        .push_manifest(prefix, &named, &image_manifest(&config, &junk_layer))
        .await;
    let matcher = Matcher::new(
        "hosted",
        "docker",
        Criteria {
            last_blob_updated: Some(0),
            ..Criteria::default()
        },
    );
    let versions = |matches: Vec<astrum_tasks::cleanup::query::Match>| {
        let mut names: Vec<String> = matches.into_iter().map(|found| found.version).collect();
        names.sort();
        names
    };
    let db = registry.fixture.state.db().conn();
    let mut expected = vec![
        "1.0".to_owned(),
        "2.0".to_owned(),
        "pointing".to_owned(),
        fake_tag,
        named,
    ];
    expected.sort();
    assert_eq!(versions(matcher.batch(db, 0, 100).await.unwrap()), expected);
    registry
        .execute(&format!(
            "DELETE FROM asset WHERE repository = 'hosted' AND attributes #>> '{{docker,digest}}' = '{digest}'"
        ))
        .await;
    expected.retain(|version| version != "1.0");
    expected.push(sig_tag);
    expected.sort();
    assert_eq!(versions(matcher.batch(db, 0, 100).await.unwrap()), expected);
}

#[tokio::test]
async fn the_listing_shows_the_state_and_the_signer_of_each_signature() {
    let registry = Registry::new().await;
    let signer = signed_only(&registry, "block").await;
    registry
        .repository("hosted", hosted_body("open", None))
        .await;
    let stranger = Signer::new();
    let prefix = "/v2/hosted/app";
    let digest = registry.push_image(prefix, "1.0", b"image").await;
    let legacy = registry.sign(prefix, &digest, &signer, Via::Tag).await;
    let foreign = registry
        .sign(prefix, &digest, &stranger, Via::Referrer)
        .await;
    let reply = registry
        .send(
            Method::GET,
            "/service/rest/v1/docker/hosted/supply-chain?image=app&reference=1.0",
            Vec::new(),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let mut found: Vec<(String, String, Option<String>)> = reply.json()["signatures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["digest"].as_str().unwrap().to_owned(),
                item["state"].as_str().unwrap().to_owned(),
                item["signer"].as_str().map(str::to_owned),
            )
        })
        .collect();
    found.sort();
    let mut expected = vec![
        (legacy, "verified".to_owned(), Some("release".to_owned())),
        (foreign, "invalid".to_owned(), None),
    ];
    expected.sort();
    assert_eq!(found, expected);

    let open = registry.push_image("/v2/open/other", "1.0", b"other").await;
    registry
        .sign("/v2/open/other", &open, &signer, Via::Tag)
        .await;
    let reply = registry
        .send(
            Method::GET,
            "/service/rest/v1/docker/open/supply-chain?image=other&reference=1.0",
            Vec::new(),
            None,
        )
        .await;
    assert_eq!(
        reply.json()["signatures"][0]["state"],
        "unknown",
        "{}",
        reply.text()
    );
}

#[derive(Default)]
struct Upstream {
    down: AtomicBool,
    requests: AtomicUsize,
    blobs: AtomicUsize,
}

fn serve(
    registry: &Registry,
    listener: tokio::net::TcpListener,
    upstream: Arc<Upstream>,
) -> JoinHandle<()> {
    let app = registry.app.clone().layer(middleware::from_fn(
        move |request: Request<Body>, next: Next| {
            let upstream = upstream.clone();
            async move {
                upstream.requests.fetch_add(1, Ordering::SeqCst);
                if request.uri().path().contains("/blobs/") && request.method() == Method::GET {
                    upstream.blobs.fetch_add(1, Ordering::SeqCst);
                }
                match upstream.down.load(Ordering::SeqCst) {
                    true => StatusCode::SERVICE_UNAVAILABLE.into_response(),
                    false => next.run(request).await,
                }
            }
        },
    ));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() })
}

async fn proxy_of_a_signing_upstream(
    registry: &Registry,
    mode: &str,
) -> (Signer, Arc<Upstream>, JoinHandle<()>) {
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
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "DockerToken"],
    )
    .await
    .unwrap();
    registry.fixture.state.security().changed().await.unwrap();
    registry
        .repository("hosted", hosted_body("upstream", None))
        .await;
    let signer = Signer::new();
    registry.key("release", &signer).await;
    registry.policy("signed-only", mode, &["release"]).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Arc::new(Upstream::default());
    let server = serve(registry, listener, upstream.clone());
    registry
        .repository(
            "proxy",
            json!({
                "name": "mirror",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": format!("http://{address}/repository/upstream"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false, "authentication": {"type": "username", "username": "admin", "password": registry.fixture.admin_password}},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "HUB"},
                "policy": {"name": "signed-only"}
            }),
        )
        .await;
    (signer, upstream, server)
}

#[tokio::test]
async fn a_proxy_serves_what_the_upstream_signed_and_refuses_what_it_did_not() {
    let registry = Registry::new().await;
    let (signer, _upstream, _server) = proxy_of_a_signing_upstream(&registry, "block").await;
    let upstream = "/v2/upstream/library/app";
    let signed = registry
        .push_image(upstream, "signed", b"signed upstream")
        .await;
    registry
        .push_image(upstream, "bare", b"unsigned upstream")
        .await;
    registry
        .sign(upstream, &signed, &signer, Via::Referrer)
        .await;
    let pulled = registry.pull("/v2/mirror/app", "signed").await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    let row = registry.recorded("mirror", &signed).await.unwrap();
    assert_eq!(row, ("signed".to_owned(), Some("release".to_owned())));
    refusal(&registry.pull("/v2/mirror/app", "bare").await);
}

#[tokio::test]
async fn an_upstream_that_cannot_be_asked_refuses_in_block_mode_and_is_recorded_in_report_mode() {
    let registry = Registry::new().await;
    let (signer, remote, _server) = proxy_of_a_signing_upstream(&registry, "off").await;
    let upstream = "/v2/upstream/library/app";
    let digest = registry
        .push_image(upstream, "1.0", b"signed upstream")
        .await;
    registry.sign(upstream, &digest, &signer, Via::Tag).await;
    assert_eq!(
        registry.pull("/v2/mirror/app", "1.0").await.status,
        StatusCode::OK
    );
    remote.down.store(true, Ordering::SeqCst);
    registry
        .change_policy("signed-only", "block", &["release"])
        .await;
    let refused = registry.pull("/v2/mirror/app", &digest).await;
    assert_eq!(
        refused.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "{}",
        refused.text()
    );
    assert!(
        refused.text().contains("could not be checked"),
        "{}",
        refused.text()
    );
    assert_eq!(
        registry.recorded("mirror", &digest).await.unwrap().0,
        "error"
    );
    registry
        .change_policy("signed-only", "report", &["release"])
        .await;
    assert_eq!(
        registry.pull("/v2/mirror/app", &digest).await.status,
        StatusCode::OK
    );
    assert_eq!(
        registry.recorded("mirror", &digest).await.unwrap().0,
        "error"
    );
}

#[tokio::test]
async fn a_signature_kind_manifest_that_is_not_exempt_is_checked_once_per_digest() {
    let registry = Registry::new().await;
    let (_signer, upstream, _server) = proxy_of_a_signing_upstream(&registry, "block").await;
    let prefix = "/v2/upstream/library/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    let spec = Spec {
        key: "layers",
        layers: vec![(IN_TOTO, tar_gz("disguised"))],
        artifact_type: Some(COSIGN),
        subject: Some(&subject),
        docker: false,
        foreign: false,
    };
    let (digest, _) = registry.push_spec(prefix, spec).await;
    refusal(&registry.pull("/v2/mirror/app", &digest).await);
    let first = upstream.blobs.load(Ordering::SeqCst);
    assert!(first >= 1);
    refusal(&registry.pull("/v2/mirror/app", &digest).await);
    assert_eq!(upstream.blobs.load(Ordering::SeqCst), first);
    assert_eq!(
        registry
            .recorded("mirror", &format!("content:{digest}"))
            .await,
        Some(("not-exempt".to_owned(), None))
    );
}

#[tokio::test]
async fn one_verification_asks_the_upstream_for_at_most_sixty_blobs() {
    let registry = Registry::new().await;
    let (_signer, upstream, _server) = proxy_of_a_signing_upstream(&registry, "block").await;
    let prefix = "/v2/upstream/library/app";
    let subject = registry.push_image(prefix, "1.0", b"image").await;
    let payload = payload(&subject);
    for number in 0..6 {
        let mut layers: Vec<(&str, Vec<u8>)> = (0..19)
            .map(|layer| (BUNDLE, genuine(BUNDLE, &format!("{number}-{layer}"))))
            .collect();
        layers.push((SIMPLE_SIGNING, payload.clone()));
        let spec = Spec {
            key: "layers",
            layers,
            artifact_type: Some(COSIGN),
            subject: Some(&subject),
            docker: false,
            foreign: false,
        };
        registry.push_spec(prefix, spec).await;
    }
    let pulled = registry.pull("/v2/mirror/app", &subject).await;
    assert_ne!(pulled.status, StatusCode::OK, "{}", pulled.text());
    let fetched = upstream.blobs.load(Ordering::SeqCst);
    assert!((1..=60).contains(&fetched), "{fetched}");
}

#[tokio::test]
async fn concurrent_checks_of_one_digest_share_one_verification() {
    let registry = Registry::new().await;
    let (signer, upstream, _server) = proxy_of_a_signing_upstream(&registry, "off").await;
    let prefix = "/v2/upstream/library/app";
    let mut digests = Vec::new();
    for name in ["single", "twice"] {
        let digest = registry.push_image(prefix, name, name.as_bytes()).await;
        registry.sign(prefix, &digest, &signer, Via::Referrer).await;
        assert_eq!(
            registry.pull("/v2/mirror/app", name).await.status,
            StatusCode::OK
        );
        digests.push(digest);
    }
    registry
        .change_policy("signed-only", "block", &["release"])
        .await;
    let before = upstream.requests.load(Ordering::SeqCst);
    let alone = registry.pull("/v2/mirror/app", &digests[0]).await;
    assert_eq!(alone.status, StatusCode::OK, "{}", alone.text());
    let one = upstream.requests.load(Ordering::SeqCst) - before;
    assert!(one >= 3, "{one}");
    let before = upstream.requests.load(Ordering::SeqCst);
    let (first, second) = tokio::join!(
        registry.pull("/v2/mirror/app", &digests[1]),
        registry.pull("/v2/mirror/app", &digests[1])
    );
    assert_eq!(
        (first.status, second.status),
        (StatusCode::OK, StatusCode::OK)
    );
    assert_eq!(upstream.requests.load(Ordering::SeqCst) - before, one);
}
