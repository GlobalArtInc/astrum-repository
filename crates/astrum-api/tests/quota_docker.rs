mod support;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn digest_of(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
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
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Reply {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, &self.admin);
        for (name, value) in headers {
            request = request.header(*name, *value);
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

    async fn hosted(&self, name: &str, limit: Option<i64>) {
        let mut storage = json!({"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true});
        if let Some(limit) = limit {
            storage["quota"] = json!({"limitBytes": limit, "enforce": true});
        }
        let reply = self
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/repositories/docker/hosted",
                Auth::Admin,
                Some(json!({
                    "name": name,
                    "online": true,
                    "storage": storage,
                    "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
                })),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn push(&self, prefix: &str, bytes: &[u8]) -> Reply {
        let digest = digest_of(bytes);
        self.send(
            Method::POST,
            &format!("{prefix}/blobs/uploads/?digest={digest}"),
            &[("content-type", "application/octet-stream")],
            bytes.to_vec(),
        )
        .await
    }

    async fn used(&self, name: &str) -> i64 {
        astrum_content::usage::current(self.fixture.state.db().conn(), name)
            .await
            .unwrap()
            .unwrap()
            .bytes
    }
}

fn denied(reply: &Reply) {
    assert_eq!(
        reply.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        reply.text()
    );
    assert_eq!(
        reply.json()["errors"][0]["code"],
        "DENIED",
        "{}",
        reply.text()
    );
}

#[tokio::test]
async fn layer_commits_and_new_sessions_over_the_limit_are_denied() {
    let registry = Registry::new().await;
    registry.hosted("reg", Some(400)).await;
    let prefix = "/v2/reg/app";
    assert_eq!(
        registry.push(prefix, &[b'c'; 14]).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        registry.push(prefix, &[b'a'; 200]).await.status,
        StatusCode::CREATED
    );
    assert_eq!(registry.used("reg").await, 214);

    let oversized = registry.push(prefix, &[b'b'; 300]).await;
    denied(&oversized);
    assert!(oversized.text().contains("reg"), "{}", oversized.text());

    let layer = [b'd'; 300];
    let session = registry
        .send(
            Method::POST,
            &format!("{prefix}/blobs/uploads/"),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(session.status, StatusCode::ACCEPTED, "{}", session.text());
    let location = session.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let chunk = registry
        .send(
            Method::PATCH,
            &location,
            &[("content-type", "application/octet-stream")],
            layer.to_vec(),
        )
        .await;
    assert_eq!(chunk.status, StatusCode::ACCEPTED, "{}", chunk.text());
    let commit = registry
        .send(
            Method::PUT,
            &format!("{location}?digest={}", digest_of(&layer)),
            &[],
            Vec::new(),
        )
        .await;
    denied(&commit);
    assert_eq!(registry.used("reg").await, 214);

    registry.hosted("full", Some(14)).await;
    assert_eq!(
        registry.push("/v2/full/app", &[b'c'; 14]).await.status,
        StatusCode::CREATED
    );
    let refused = registry
        .send(Method::POST, "/v2/full/app/blobs/uploads/", &[], Vec::new())
        .await;
    denied(&refused);
}

#[tokio::test]
async fn manifest_puts_over_the_limit_are_denied() {
    let registry = Registry::new().await;
    registry.hosted("reg", Some(300)).await;
    let prefix = "/v2/reg/app";
    let config = [b'c'; 14];
    let layer = [b'a'; 200];
    registry.push(prefix, &config).await;
    registry.push(prefix, &layer).await;
    let manifest: Value = json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": "application/vnd.oci.image.config.v1+json", "digest": digest_of(&config), "size": 14},
        "layers": [{"mediaType": "application/vnd.oci.image.layer.v1.tar+gzip", "digest": digest_of(&layer), "size": 200}],
    });
    let bytes = serde_json::to_vec(&manifest).unwrap();
    assert!(214 + bytes.len() > 300);
    let refused = registry
        .send(
            Method::PUT,
            &format!("{prefix}/manifests/v1"),
            &[("content-type", OCI_MANIFEST)],
            bytes,
        )
        .await;
    denied(&refused);
    let missing = registry
        .send(
            Method::GET,
            &format!("{prefix}/manifests/v1"),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cross_repository_mounts_over_the_limit_are_denied() {
    let registry = Registry::new().await;
    registry.hosted("src", None).await;
    registry.hosted("reg", Some(400)).await;
    let shared = [b's'; 300];
    assert_eq!(
        registry.push("/v2/src/app", &shared).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        registry.push("/v2/reg/app", &[b'a'; 200]).await.status,
        StatusCode::CREATED
    );
    let refused = registry
        .send(
            Method::POST,
            &format!(
                "/v2/reg/app/blobs/uploads/?mount={}&from=src/app",
                digest_of(&shared)
            ),
            &[],
            Vec::new(),
        )
        .await;
    denied(&refused);
    assert_eq!(registry.used("reg").await, 200);
}
