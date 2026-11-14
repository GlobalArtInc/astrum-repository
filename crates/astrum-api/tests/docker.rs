mod support;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;

use astrum_content::transfer::{Conflict, LocalSource, Outcome, PROMOTED, Status, Terms, Transfer};
use astrum_content::{ContentError, Ctx, store};
use astrum_format_docker::transfer::{self as promotion, Options, TRANSFER};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use astrum_security::{Action, ContentSelector, SecurityModel, Selector, SelectorGrant, Subject};
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256, Sha512};
use support::two_factor::number;
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const PROMOTIONS: &str = "/service/rest/v1/promotions";
const USER_PASSWORD: &str = "password-1234";
const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const OCI_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn sha512(bytes: &[u8]) -> String {
    format!("sha512:{}", hex::encode(Sha512::digest(bytes)))
}

fn basic(user: &str, password: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{user}:{password}")))
}

impl Registry {
    fn enable_promotion(&self) {
        self.fixture
            .state
            .set_image_transfers(Arc::new(astrum_format_docker::transfer::Images));
    }

    async fn promote(&self, auth: Auth<'_>, body: Value) -> Reply {
        self.fixture
            .call(Method::POST, PROMOTIONS, auth, Some(body))
            .await
    }

    async fn user(&self, user: &str, grants: &[(&str, &str)]) {
        let privileges: Vec<String> = grants
            .iter()
            .map(|(repository, action)| format!("as-repository-view-docker-{repository}-{action}"))
            .collect();
        let role = self
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/security/roles",
                Auth::Admin,
                Some(json!({"id": user, "name": user, "description": "", "privileges": privileges, "roles": []})),
            )
            .await;
        assert_eq!(role.status, StatusCode::OK, "{}", role.text());
        self.fixture.add_user(user, USER_PASSWORD, &[user]).await;
    }

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
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(authorization) = authorization {
            request = request.header(header::AUTHORIZATION, authorization);
        }
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

    async fn admin(&self, method: Method, path: &str) -> Reply {
        self.send(method, path, Some(&self.admin), &[], Vec::new())
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

    async fn hosted(&self, name: &str, policy: &str) {
        self.create("hosted", hosted(name, policy)).await;
    }

    async fn activate_token_realm(&self) {
        astrum_db::setting::put(
            self.fixture.state.db().conn(),
            astrum_security::model::REALMS_KEY,
            &["AstrumAuthenticatingRealm", "DockerToken"],
        )
        .await
        .unwrap();
        self.fixture.state.security().changed().await.unwrap();
    }

    async fn push_blob(&self, prefix: &str, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let reply = self
            .send(
                Method::POST,
                &format!("{prefix}/blobs/uploads/?digest={digest}"),
                Some(&self.admin),
                &[("content-type", "application/octet-stream")],
                bytes.to_vec(),
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
            &[("content-type", media_type)],
            serde_json::to_vec(manifest).unwrap(),
        )
        .await
    }

    async fn push_image(&self, prefix: &str, tag: &str, layer: &[u8]) -> (String, Vec<u8>) {
        let config = self.push_blob(prefix, b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(prefix, layer).await;
        let manifest = image_manifest(&config, 14, &[(&layer, LAYER)], None, None);
        let reply = self.push_manifest(prefix, tag, &manifest).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        let bytes = serde_json::to_vec(&manifest).unwrap();
        (sha256(&bytes), bytes)
    }
}

fn hosted(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy, "latestPolicy": true},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    })
}

fn image_manifest(
    config: &str,
    config_size: usize,
    layers: &[(&str, &str)],
    artifact_type: Option<&str>,
    subject: Option<(&str, usize)>,
) -> Value {
    let mut manifest = json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": if artifact_type.is_some() { EMPTY } else { OCI_CONFIG }, "digest": config, "size": config_size},
        "layers": layers.iter().map(|(digest, media_type)| json!({"mediaType": media_type, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    });
    if let Some(artifact_type) = artifact_type {
        manifest["artifactType"] = artifact_type.into();
        manifest["annotations"] = json!({"org.example.kind": artifact_type});
    }
    if let Some((digest, size)) = subject {
        manifest["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": digest, "size": size});
    }
    manifest
}

#[tokio::test]
async fn the_base_endpoint_challenges_and_the_token_endpoint_issues_tokens() {
    let registry = Registry::new().await;
    let anonymous = registry
        .send(Method::GET, "/v2/", None, &[], Vec::new())
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous.headers["docker-distribution-api-version"],
        "registry/2.0"
    );
    assert_eq!(
        anonymous.headers[header::WWW_AUTHENTICATE],
        "Basic realm=\"Astrum Repository\""
    );
    assert_eq!(anonymous.json()["errors"][0]["code"], "UNAUTHORIZED");
    let inactive = registry.admin(Method::GET, "/v2/token").await;
    assert_eq!(inactive.status, StatusCode::NOT_FOUND);
    assert_eq!(inactive.json()["errors"][0]["code"], "UNSUPPORTED");
    let admin = registry.admin(Method::GET, "/v2/").await;
    assert_eq!(
        (admin.status, admin.text()),
        (StatusCode::OK, "{}".to_owned())
    );

    registry.activate_token_realm().await;
    let challenged = registry
        .send(
            Method::GET,
            "/v2/",
            None,
            &[("host", "registry.test:5000")],
            Vec::new(),
        )
        .await;
    assert_eq!(
        challenged.headers[header::WWW_AUTHENTICATE],
        "Bearer realm=\"http://registry.test:5000/v2/token\",service=\"registry.test:5000\""
    );
    let issued = registry
        .admin(
            Method::GET,
            "/v2/token?service=registry.test&scope=repository:hosted/app:pull,push",
        )
        .await;
    assert_eq!(issued.status, StatusCode::OK, "{}", issued.text());
    let body = issued.json();
    let token = body["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("DockerToken."));
    assert_eq!(body["access_token"], token.as_str());
    assert_eq!(body["expires_in"], 1800);
    assert!(body["issued_at"].as_str().is_some());
    let bearer = format!("Bearer {token}");
    let with_token = registry
        .send(Method::GET, "/v2/", Some(&bearer), &[], Vec::new())
        .await;
    assert_eq!(with_token.status, StatusCode::OK);

    let wrong = registry
        .send(
            Method::GET,
            "/v2/token",
            Some(&basic("admin", "wrong")),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let nobody = registry
        .send(Method::GET, "/v2/token?scope=x", None, &[], Vec::new())
        .await;
    assert_eq!(nobody.status, StatusCode::UNAUTHORIZED);

    let form = format!(
        "grant_type=password&username=admin&password={}&service=x&scope=repository:a:pull",
        registry.fixture.admin_password
    );
    let oauth = registry
        .send(
            Method::POST,
            "/v2/token",
            None,
            &[("content-type", "application/x-www-form-urlencoded")],
            form.into_bytes(),
        )
        .await;
    assert_eq!(oauth.status, StatusCode::OK, "{}", oauth.text());
    assert!(
        oauth.json()["access_token"]
            .as_str()
            .unwrap()
            .starts_with("DockerToken.")
    );
    let refused = registry
        .send(
            Method::POST,
            "/v2/token",
            None,
            &[("content-type", "application/x-www-form-urlencoded")],
            b"grant_type=password&username=admin&password=nope".to_vec(),
        )
        .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);

    registry
        .fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    registry.fixture.state.security().changed().await.unwrap();
    let anonymous_token = registry
        .send(Method::GET, "/v2/token?scope=x", None, &[], Vec::new())
        .await;
    assert_eq!(anonymous_token.status, StatusCode::OK);
    let anonymous_bearer = format!(
        "Bearer {}",
        anonymous_token.json()["token"].as_str().unwrap()
    );
    let ping = registry
        .send(
            Method::GET,
            "/v2/",
            Some(&anonymous_bearer),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(ping.status, StatusCode::OK);
}

#[tokio::test]
async fn images_are_pushed_and_pulled_through_path_routing() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    let prefix = "/v2/hosted/team/app";

    let session = registry
        .admin(Method::POST, &format!("{prefix}/blobs/uploads/"))
        .await;
    assert_eq!(session.status, StatusCode::ACCEPTED);
    let location = session.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(location.starts_with("/v2/hosted/team/app/blobs/uploads/"));
    assert_eq!(session.headers[header::RANGE], "0-0");
    let layer = b"layer-bytes-1".to_vec();
    let first = registry
        .send(
            Method::PATCH,
            &location,
            Some(&registry.admin),
            &[("content-range", "0-5")],
            layer[..6].to_vec(),
        )
        .await;
    assert_eq!(first.status, StatusCode::ACCEPTED, "{}", first.text());
    assert_eq!(first.headers[header::RANGE], "0-5");
    let again = registry
        .send(
            Method::PATCH,
            &location,
            Some(&registry.admin),
            &[("content-range", "0-5")],
            layer[..6].to_vec(),
        )
        .await;
    assert_eq!(again.status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(again.headers[header::LOCATION], location.as_str());
    assert_eq!(again.headers[header::RANGE], "0-5");
    let status = registry.admin(Method::GET, &location).await;
    assert_eq!(status.status, StatusCode::NO_CONTENT);
    assert_eq!(status.headers[header::RANGE], "0-5");
    let layer_digest = sha256(&layer);
    let closed = registry
        .send(
            Method::PUT,
            &format!("{location}?digest={layer_digest}"),
            Some(&registry.admin),
            &[("content-range", "6-12")],
            layer[6..].to_vec(),
        )
        .await;
    assert_eq!(closed.status, StatusCode::CREATED, "{}", closed.text());
    assert_eq!(
        closed.headers[header::LOCATION],
        format!("{prefix}/blobs/{layer_digest}").as_str()
    );
    assert_eq!(
        closed.headers["docker-content-digest"],
        layer_digest.as_str()
    );
    let gone = registry.admin(Method::GET, &location).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_eq!(gone.json()["errors"][0]["code"], "BLOB_UPLOAD_UNKNOWN");

    let config = registry.push_blob(prefix, b"{\"os\":\"linux\"}").await;
    let manifest = image_manifest(&config, 14, &[(&layer_digest, LAYER)], None, None);
    let bytes = serde_json::to_vec(&manifest).unwrap();
    let digest = sha256(&bytes);
    let pushed = registry.push_manifest(prefix, "1.0", &manifest).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    assert_eq!(
        pushed.headers[header::LOCATION],
        format!("{prefix}/manifests/{digest}").as_str()
    );
    assert_eq!(pushed.headers["docker-content-digest"], digest.as_str());

    for reference in ["1.0", digest.as_str()] {
        let pulled = registry
            .admin(Method::GET, &format!("{prefix}/manifests/{reference}"))
            .await;
        assert_eq!(pulled.status, StatusCode::OK);
        assert_eq!(pulled.body, bytes);
        assert_eq!(pulled.headers[header::CONTENT_TYPE], OCI_MANIFEST);
        assert_eq!(pulled.headers["docker-content-digest"], digest.as_str());
        assert_eq!(
            pulled.headers[header::ETAG],
            format!("\"{digest}\"").as_str()
        );
    }
    let head = registry
        .admin(Method::HEAD, &format!("{prefix}/manifests/1.0"))
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
    assert_eq!(
        head.headers[header::CONTENT_LENGTH],
        bytes.len().to_string().as_str()
    );

    let blob = registry
        .send(
            Method::GET,
            &format!("{prefix}/blobs/{layer_digest}"),
            Some(&registry.admin),
            &[("range", "bytes=6-")],
            Vec::new(),
        )
        .await;
    assert_eq!(blob.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(blob.body, b"bytes-1");
    assert_eq!(blob.headers["docker-content-digest"], layer_digest.as_str());
    let backwards = registry
        .send(
            Method::GET,
            &format!("{prefix}/blobs/{layer_digest}"),
            Some(&registry.admin),
            &[("range", "bytes=6-2")],
            Vec::new(),
        )
        .await;
    assert_eq!(backwards.status, StatusCode::RANGE_NOT_SATISFIABLE);

    registry.push_manifest(prefix, "0.9", &manifest).await;
    let tags = registry
        .admin(Method::GET, &format!("{prefix}/tags/list"))
        .await;
    assert_eq!(
        tags.json(),
        json!({"name": "hosted/team/app", "tags": ["0.9", "1.0"]})
    );
    let page = registry
        .admin(Method::GET, &format!("{prefix}/tags/list?n=1"))
        .await;
    assert_eq!(page.json()["tags"], json!(["0.9"]));
    assert_eq!(
        page.headers[header::LINK],
        "</v2/hosted/team/app/tags/list?n=1&last=0.9>; rel=\"next\""
    );
    let catalog = registry.admin(Method::GET, "/v2/_catalog").await;
    assert_eq!(catalog.json(), json!({"repositories": ["hosted/team/app"]}));
    let by_content_route = registry
        .admin(Method::GET, "/repository/hosted/v2/team/app/tags/list")
        .await;
    assert_eq!(
        by_content_route.json(),
        json!({"name": "team/app", "tags": ["0.9", "1.0"]})
    );

    let components = registry
        .fixture
        .call(
            Method::GET,
            "/service/rest/v1/components?repository=hosted",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let items = components["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["group"], "");
    assert_eq!(items[0]["name"], "team/app");
    assert_eq!(items[0]["assets"][0]["docker"]["digest"], digest.as_str());
    assert_eq!(items[0]["assets"][0]["docker"]["artifactKind"], "image");
    let found = registry
        .fixture
        .call(
            Method::GET,
            &format!(
                "/service/rest/v1/search/assets?docker.contentDigest={digest}&docker.imageTag=1.0"
            ),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(found["items"].as_array().unwrap().len(), 1);
    assert_eq!(found["items"][0]["path"], "/v2/team/app/manifests/1.0");

    let unknown = registry
        .admin(Method::GET, "/v2/nosuch/team/app/tags/list")
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.json()["errors"][0]["code"], "NAME_UNKNOWN");
    let missing = registry
        .admin(Method::GET, &format!("{prefix}/manifests/2.0"))
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.json()["errors"][0]["code"], "MANIFEST_UNKNOWN");
    let invalid = registry
        .admin(Method::GET, "/v2/hosted/Team/App/tags/list")
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid.json()["errors"][0]["code"], "NAME_INVALID");
}

#[tokio::test]
async fn anonymous_callers_are_challenged_with_a_scope() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    registry.activate_token_realm().await;
    let pull = registry
        .send(
            Method::GET,
            "/v2/hosted/app/manifests/1.0",
            None,
            &[("host", "r.test")],
            Vec::new(),
        )
        .await;
    assert_eq!(pull.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        pull.headers[header::WWW_AUTHENTICATE],
        "Bearer realm=\"http://r.test/v2/token\",service=\"r.test\",scope=\"repository:hosted/app:pull\""
    );
    let push = registry
        .send(
            Method::POST,
            "/v2/hosted/app/blobs/uploads/",
            None,
            &[("host", "r.test")],
            Vec::new(),
        )
        .await;
    assert_eq!(push.status, StatusCode::UNAUTHORIZED);
    assert!(
        push.headers[header::WWW_AUTHENTICATE]
            .to_str()
            .unwrap()
            .ends_with("scope=\"repository:hosted/app:pull,push\"")
    );
    registry
        .fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let denied = registry
        .send(
            Method::POST,
            "/v2/hosted/app/blobs/uploads/",
            Some(&basic("reader", "reader-password-1")),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN, "{}", denied.text());
    assert_eq!(denied.json()["errors"][0]["code"], "DENIED");
}

#[tokio::test]
async fn write_policies_guard_tags() {
    let registry = Registry::new().await;
    registry.hosted("once", "ALLOW_ONCE").await;
    let prefix = "/v2/once/app";
    registry.push_image(prefix, "1.0", b"one").await;
    registry.push_image(prefix, "1.0", b"one").await;
    let config = registry.push_blob(prefix, b"{\"os\":\"linux\"}").await;
    let other = registry.push_blob(prefix, b"two").await;
    let changed = image_manifest(&config, 14, &[(&other, LAYER)], None, None);
    let refused = registry.push_manifest(prefix, "1.0", &changed).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.json()["errors"][0]["code"], "DENIED");
    assert_eq!(
        registry
            .push_manifest(prefix, "latest", &changed)
            .await
            .status,
        StatusCode::CREATED
    );
    let first = image_manifest(
        &config,
        14,
        &[(&other, LAYER), (&config, LAYER)],
        None,
        None,
    );
    assert_eq!(
        registry
            .push_manifest(prefix, "latest", &first)
            .await
            .status,
        StatusCode::CREATED
    );

    registry.hosted("frozen", "DENY").await;
    let denied = registry
        .admin(Method::POST, "/v2/frozen/app/blobs/uploads/")
        .await;
    assert_eq!(denied.status, StatusCode::BAD_REQUEST);
    assert_eq!(denied.json()["errors"][0]["code"], "DENIED");
}

#[tokio::test]
async fn bad_manifests_and_digests_are_refused() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    let prefix = "/v2/hosted/app";
    let config = registry.push_blob(prefix, b"{}").await;
    let missing = sha256(b"never pushed");
    let manifest = image_manifest(&config, 2, &[(&missing, LAYER)], None, None);
    let unknown = registry.push_manifest(prefix, "1", &manifest).await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown.json()["errors"][0]["code"], "MANIFEST_BLOB_UNKNOWN");

    let fine = image_manifest(&config, 2, &[], None, None);
    let wrong = registry
        .push_manifest(prefix, &sha256(b"something else"), &fine)
        .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    assert_eq!(wrong.json()["errors"][0]["code"], "DIGEST_INVALID");
    let malformed = registry
        .push_manifest(prefix, "sha256:baddigeststring", &fine)
        .await;
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);

    let mut huge = fine.clone();
    huge["annotations"] = json!({"big": "x".repeat(5 * 1024 * 1024)});
    let too_large = registry.push_manifest(prefix, "big", &huge).await;
    assert_eq!(too_large.status, StatusCode::PAYLOAD_TOO_LARGE);

    let digest = sha256(b"abc");
    let bad_upload = registry
        .send(
            Method::POST,
            &format!("{prefix}/blobs/uploads/?digest={digest}"),
            Some(&registry.admin),
            &[],
            b"abd".to_vec(),
        )
        .await;
    assert_eq!(bad_upload.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad_upload.json()["errors"][0]["code"], "DIGEST_INVALID");
    let absent = registry
        .admin(Method::HEAD, &format!("{prefix}/blobs/{digest}"))
        .await;
    assert_eq!(absent.status, StatusCode::NOT_FOUND);

    let long = vec![7u8; 300];
    let long_digest = sha512(&long);
    let stored = registry
        .send(
            Method::POST,
            &format!("{prefix}/blobs/uploads/?digest={long_digest}"),
            Some(&registry.admin),
            &[],
            long.clone(),
        )
        .await;
    assert_eq!(stored.status, StatusCode::CREATED);
    let fetched = registry
        .admin(Method::GET, &format!("{prefix}/blobs/{long_digest}"))
        .await;
    assert_eq!(fetched.body, long);
    assert_eq!(
        fetched.headers["docker-content-digest"],
        long_digest.as_str()
    );
    let by512 = serde_json::to_vec(&fine).unwrap();
    let pushed = registry.push_manifest(prefix, &sha512(&by512), &fine).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let pulled = registry
        .admin(
            Method::GET,
            &format!("{prefix}/manifests/{}", sha512(&by512)),
        )
        .await;
    assert_eq!(pulled.body, by512);
}

#[tokio::test]
async fn referrers_are_listed_and_deleted_with_their_subject() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    let prefix = "/v2/hosted/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"base").await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let sbom_layer = registry.push_blob(prefix, b"{\"spdx\":1}").await;
    let sbom = image_manifest(
        &empty,
        2,
        &[(&sbom_layer, "application/spdx+json")],
        Some("application/spdx+json"),
        Some((&subject, subject_bytes.len())),
    );
    let sbom_bytes = serde_json::to_vec(&sbom).unwrap();
    let sbom_digest = sha256(&sbom_bytes);
    let pushed = registry.push_manifest(prefix, &sbom_digest, &sbom).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    assert_eq!(pushed.headers["oci-subject"], subject.as_str());
    let signature_layer = registry.push_blob(prefix, b"sig").await;
    let signature = image_manifest(
        &empty,
        2,
        &[(&signature_layer, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let signature_digest = sha256(&serde_json::to_vec(&signature).unwrap());
    let tagged = registry
        .push_manifest(
            prefix,
            &format!("{signature_digest}?tag=signed&tag=sig-2"),
            &signature,
        )
        .await;
    assert_eq!(tagged.status, StatusCode::CREATED, "{}", tagged.text());
    assert_eq!(tagged.headers["oci-tag"], "signed, sig-2");

    let listed = registry
        .admin(Method::GET, &format!("{prefix}/referrers/{subject}"))
        .await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.headers[header::CONTENT_TYPE], OCI_INDEX);
    let index = listed.json();
    assert_eq!(index["schemaVersion"], 2);
    let manifests = index["manifests"].as_array().unwrap();
    assert_eq!(manifests.len(), 2);
    let entry = manifests
        .iter()
        .find(|entry| entry["digest"] == sbom_digest.as_str())
        .unwrap();
    assert_eq!(entry["artifactType"], "application/spdx+json");
    assert_eq!(entry["size"], sbom_bytes.len());
    assert_eq!(entry["mediaType"], OCI_MANIFEST);
    assert_eq!(
        entry["annotations"]["org.example.kind"],
        "application/spdx+json"
    );
    let filtered = registry
        .admin(
            Method::GET,
            &format!(
                "{prefix}/referrers/{subject}?artifactType=application/vnd.cncf.notary.signature"
            ),
        )
        .await;
    assert_eq!(filtered.headers["oci-filters-applied"], "artifactType");
    assert_eq!(filtered.json()["manifests"].as_array().unwrap().len(), 1);
    let none = registry
        .admin(Method::GET, &format!("{prefix}/referrers/{}", sha256(b"x")))
        .await;
    assert_eq!(none.status, StatusCode::OK);
    assert_eq!(none.json()["manifests"], json!([]));
    let bad = registry
        .admin(Method::GET, &format!("{prefix}/referrers/sha256:nope"))
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let kinds = registry
        .fixture
        .call(
            Method::GET,
            "/service/rest/v1/search/assets?docker.artifactKind=sbom",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(kinds["items"].as_array().unwrap().len(), 1);

    let schema_index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [{"mediaType": OCI_MANIFEST, "digest": sha256(b"legacy"), "size": 7, "artifactType": "application/vnd.example.legacy"}],
    });
    let schema_tag = subject.replace(':', "-");
    let kept = registry
        .push_manifest(prefix, &schema_tag, &schema_index)
        .await;
    assert_eq!(kept.status, StatusCode::CREATED, "{}", kept.text());
    let merged = registry
        .admin(Method::GET, &format!("{prefix}/referrers/{subject}"))
        .await
        .json();
    assert_eq!(merged["manifests"].as_array().unwrap().len(), 3);

    let deleted = registry
        .admin(Method::DELETE, &format!("{prefix}/manifests/{subject}"))
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED);
    for reference in [
        "1.0",
        subject.as_str(),
        sbom_digest.as_str(),
        signature_digest.as_str(),
        "signed",
        schema_tag.as_str(),
    ] {
        let gone = registry
            .admin(Method::HEAD, &format!("{prefix}/manifests/{reference}"))
            .await;
        assert_eq!(gone.status, StatusCode::NOT_FOUND, "{reference}");
    }
    let again = registry
        .admin(Method::DELETE, &format!("{prefix}/manifests/{subject}"))
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let layer = registry
        .admin(Method::HEAD, &format!("{prefix}/blobs/{sbom_layer}"))
        .await;
    assert_eq!(layer.status, StatusCode::OK);
    let blob_deleted = registry
        .admin(Method::DELETE, &format!("{prefix}/blobs/{sbom_layer}"))
        .await;
    assert_eq!(blob_deleted.status, StatusCode::ACCEPTED);
    let tag_only = registry.push_image(prefix, "2.0", b"two").await;
    let untagged = registry
        .admin(Method::DELETE, &format!("{prefix}/manifests/2.0"))
        .await;
    assert_eq!(untagged.status, StatusCode::ACCEPTED);
    let still = registry
        .admin(Method::HEAD, &format!("{prefix}/manifests/{}", tag_only.0))
        .await;
    assert_eq!(still.status, StatusCode::OK);
}

#[tokio::test]
async fn blobs_are_mounted_from_other_repositories() {
    let registry = Registry::new().await;
    registry.hosted("one", "ALLOW").await;
    registry.hosted("two", "ALLOW").await;
    let digest = registry.push_blob("/v2/two/source", b"shared layer").await;
    let mounted = registry
        .admin(
            Method::POST,
            &format!("/v2/one/target/blobs/uploads/?mount={digest}&from=two/source"),
        )
        .await;
    assert_eq!(mounted.status, StatusCode::CREATED, "{}", mounted.text());
    assert_eq!(
        mounted.headers[header::LOCATION],
        format!("/v2/one/target/blobs/{digest}").as_str()
    );
    let copy = registry
        .admin(Method::GET, &format!("/v2/one/target/blobs/{digest}"))
        .await;
    assert_eq!(copy.body, b"shared layer");

    let other = registry.push_blob("/v2/two/source", b"another").await;
    let anonymous = registry
        .admin(
            Method::POST,
            &format!("/v2/one/target/blobs/uploads/?mount={other}"),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::CREATED);

    let missing = registry
        .admin(
            Method::POST,
            &format!(
                "/v2/one/target/blobs/uploads/?mount={}&from=two/source",
                sha256(b"nothing")
            ),
        )
        .await;
    assert_eq!(missing.status, StatusCode::ACCEPTED);
    let location = missing.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let cancelled = registry.admin(Method::DELETE, &location).await;
    assert_eq!(cancelled.status, StatusCode::NO_CONTENT);
    assert_eq!(
        registry.admin(Method::GET, &location).await.status,
        StatusCode::NOT_FOUND
    );
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
async fn proxies_follow_the_token_flow_and_groups_merge_members() {
    let registry = Registry::new().await;
    allow_loopback(&registry).await;
    registry.activate_token_realm().await;
    registry.hosted("upstream", "ALLOW").await;
    registry.hosted("local", "ALLOW").await;
    let (digest, bytes) = registry
        .push_image("/v2/upstream/library/app", "1.0", b"remote layer")
        .await;
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
        .admin(Method::GET, "/v2/mirror/app/manifests/1.0")
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    assert_eq!(pulled.body, bytes);
    assert_eq!(pulled.headers["docker-content-digest"], digest.as_str());
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    let layer = manifest["layers"][0]["digest"].as_str().unwrap().to_owned();
    let head = registry
        .admin(Method::HEAD, &format!("/v2/mirror/app/blobs/{layer}"))
        .await;
    assert_eq!(head.status, StatusCode::OK);
    let blob = registry
        .admin(Method::GET, &format!("/v2/mirror/app/blobs/{layer}"))
        .await;
    assert_eq!(blob.body, b"remote layer");
    let tags = registry
        .admin(Method::GET, "/v2/mirror/app/tags/list")
        .await;
    assert_eq!(tags.json()["tags"], json!(["1.0"]));
    let cached = registry
        .fixture
        .call(
            Method::GET,
            "/service/rest/v1/assets?repository=mirror",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let paths: Vec<&str> = cached["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"/v2/app/manifests/1.0"));
    assert!(paths.contains(&format!("/v2/-/blobs/{layer}").as_str()));
    assert!(
        cached["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["registryUrl"] == "localhost/mirror"),
        "{cached}"
    );
    let pushed = registry
        .admin(Method::POST, "/v2/mirror/app/blobs/uploads/")
        .await;
    assert_eq!(pushed.status, StatusCode::METHOD_NOT_ALLOWED);

    registry
        .push_image("/v2/local/app", "2.0", b"local layer")
        .await;
    registry
        .create(
            "group",
            json!({
                "name": "all",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["local", "mirror"], "writableMember": "local"},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
            }),
        )
        .await;
    let merged = registry.admin(Method::GET, "/v2/all/app/tags/list").await;
    assert_eq!(
        merged.json(),
        json!({"name": "all/app", "tags": ["1.0", "2.0"]})
    );
    let through = registry
        .admin(Method::GET, "/v2/all/app/manifests/1.0")
        .await;
    assert_eq!(through.body, bytes);
    let written = registry.push_blob("/v2/all/app", b"via group").await;
    let landed = registry
        .admin(Method::HEAD, &format!("/v2/local/app/blobs/{written}"))
        .await;
    assert_eq!(landed.status, StatusCode::OK);
    let config = registry
        .push_blob("/v2/all/app", b"{\"os\":\"linux\"}")
        .await;
    let reusing = image_manifest(&config, 14, &[(&layer, LAYER)], None, None);
    let reused = registry.push_manifest("/v2/all/app", "3.0", &reusing).await;
    assert_eq!(reused.status, StatusCode::CREATED, "{}", reused.text());
    let copied = registry
        .admin(Method::HEAD, &format!("/v2/local/app/blobs/{layer}"))
        .await;
    assert_eq!(copied.status, StatusCode::OK);
    let catalog = registry.admin(Method::GET, "/v2/_catalog").await.json();
    let names: Vec<&str> = catalog["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    assert!(names.contains(&"all/app"));
    assert!(names.contains(&"mirror/app"));
    assert!(names.contains(&"upstream/library/app"));
    let refused = registry
        .admin(Method::DELETE, &format!("/v2/all/app/manifests/{digest}"))
        .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[derive(Clone)]
struct Remote {
    address: SocketAddr,
    manifest: Vec<u8>,
    layer: Vec<u8>,
    hits: Arc<std::sync::atomic::AtomicUsize>,
}

async fn remote_token(headers: axum::http::HeaderMap) -> axum::response::Response {
    use axum::response::IntoResponse;
    let expected = basic("mirror-user", "mirror-secret");
    match headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        Some(given) if given == expected => {
            axum::Json(json!({"token": "remote-token", "expires_in": 300})).into_response()
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn remote_registry(
    axum::extract::State(remote): axum::extract::State<Remote>,
    axum::extract::Path(path): axum::extract::Path<String>,
    method: Method,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    remote
        .hits
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let authorized = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        == Some("Bearer remote-token");
    if !authorized {
        let challenge = format!(
            "Bearer realm=\"http://{}/token\",service=\"remote.test\",scope=\"repository:library/tool:pull\"",
            remote.address
        );
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, challenge)],
        )
            .into_response();
    }
    let manifest_digest = sha256(&remote.manifest);
    let layer_digest = sha256(&remote.layer);
    let body = if path == "library/tool/manifests/1.0"
        || path == format!("library/tool/manifests/{manifest_digest}")
    {
        Some((OCI_MANIFEST, remote.manifest.clone(), manifest_digest))
    } else if path == format!("library/tool/blobs/{layer_digest}") {
        Some((
            "application/octet-stream",
            remote.layer.clone(),
            layer_digest,
        ))
    } else if path == "library/tool/tags/list" {
        Some((
            "application/json",
            serde_json::to_vec(&json!({"name": "library/tool", "tags": ["1.0", "1.1"]})).unwrap(),
            String::new(),
        ))
    } else {
        None
    };
    match body {
        Some((content_type, bytes, digest)) => {
            let length = bytes.len().to_string();
            let body = if method == Method::HEAD {
                Vec::new()
            } else {
                bytes
            };
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type.to_owned()),
                    (header::CONTENT_LENGTH, length),
                    (
                        header::HeaderName::from_static("docker-content-digest"),
                        digest,
                    ),
                ],
                body,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start_remote(layer: &[u8]) -> Remote {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = br#"{"os":"linux"}"#;
    let manifest = serde_json::to_vec(&image_manifest(
        &sha256(config),
        config.len(),
        &[(&sha256(layer), LAYER)],
        None,
        None,
    ))
    .unwrap();
    let remote = Remote {
        address,
        manifest,
        layer: layer.to_vec(),
        hits: Arc::default(),
    };
    let app = Router::new()
        .route("/token", axum::routing::get(remote_token))
        .route("/v2/{*path}", axum::routing::any(remote_registry))
        .with_state(remote.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    remote
}

#[tokio::test]
async fn hub_proxies_ask_the_token_service_and_add_the_library_prefix() {
    let registry = Registry::new().await;
    allow_loopback(&registry).await;
    let remote = start_remote(b"tool layer").await;
    registry
        .create(
            "proxy",
            json!({
                "name": "hub",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": format!("http://{}", remote.address), "contentMaxAge": 1440, "metadataMaxAge": 0},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false, "authentication": {"type": "username", "username": "mirror-user", "password": "mirror-secret"}},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "HUB"}
            }),
        )
        .await;
    let pulled = registry
        .admin(Method::GET, "/v2/hub/tool/manifests/1.0")
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    assert_eq!(pulled.body, remote.manifest);
    let layer = registry
        .admin(
            Method::GET,
            &format!("/v2/hub/tool/blobs/{}", sha256(b"tool layer")),
        )
        .await;
    assert_eq!(layer.body, b"tool layer");
    let tags = registry.admin(Method::GET, "/v2/hub/tool/tags/list").await;
    assert_eq!(
        tags.json(),
        json!({"name": "hub/tool", "tags": ["1.0", "1.1"]})
    );
    let before = remote.hits.load(std::sync::atomic::Ordering::SeqCst);
    let again = registry
        .admin(Method::GET, "/v2/hub/tool/manifests/1.0")
        .await;
    assert_eq!(again.body, remote.manifest);
    let after = remote.hits.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        after - before,
        1,
        "a stale tag is revalidated with one HEAD"
    );
    let by_digest = registry
        .admin(
            Method::GET,
            &format!("/v2/hub/tool/manifests/{}", sha256(&remote.manifest)),
        )
        .await;
    assert_eq!(by_digest.status, StatusCode::OK);
    assert_eq!(
        remote.hits.load(std::sync::atomic::Ordering::SeqCst),
        after,
        "manifests by digest come from the cache"
    );
    let missing = registry
        .admin(Method::GET, "/v2/hub/tool/manifests/9.9")
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn abandoned_uploads_and_expired_tokens_are_swept() {
    use sea_orm::ConnectionTrait;
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    let started = registry
        .admin(Method::POST, "/v2/hosted/app/blobs/uploads/")
        .await;
    let location = started.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let chunk = registry
        .send(
            Method::PATCH,
            &location,
            Some(&registry.admin),
            &[],
            b"half a layer".to_vec(),
        )
        .await;
    assert_eq!(chunk.status, StatusCode::ACCEPTED);
    let fresh = registry
        .admin(Method::POST, "/v2/hosted/app/blobs/uploads/")
        .await;
    let kept = fresh.headers[header::LOCATION].to_str().unwrap().to_owned();
    let db = registry.fixture.state.db().conn();
    let id = location.rsplit('/').next().unwrap();
    db.execute_unprepared(&format!(
        "UPDATE docker_upload SET updated_at = now() - interval '25 hours' WHERE id = '{id}'"
    ))
    .await
    .unwrap();
    let security = registry.fixture.state.security();
    let admin = security
        .model()
        .local_subject("admin", astrum_format_docker::REALM)
        .unwrap();
    security
        .issue_format_token(
            astrum_format_docker::REALM,
            &admin,
            Some(chrono::Duration::seconds(-5)),
        )
        .await
        .unwrap();
    let swept = astrum_format_docker::expire_uploads(registry.fixture.state.content())
        .await
        .unwrap();
    assert!(swept <= 1);
    assert_eq!(
        registry.admin(Method::GET, &location).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        registry.admin(Method::GET, &kept).await.status,
        StatusCode::NO_CONTENT
    );
    let tokens = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT count(*) AS n FROM format_token WHERE realm = 'DockerToken'",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(tokens, 0);
}

#[tokio::test]
async fn agents_list_the_tags_of_an_image_over_mcp() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    registry
        .push_image("/v2/hosted/team/app", "1.0", b"layer-one")
        .await;
    registry
        .push_image("/v2/hosted/team/app", "2.0", b"layer-two")
        .await;
    registry
        .push_image("/v2/hosted/team/other", "1.0", b"layer-three")
        .await;
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": "list_docker_tags", "arguments": {"repository": "hosted", "image": "team/app"}},
    });
    let reply = registry
        .send(
            Method::POST,
            "/mcp",
            Some(&registry.admin),
            &[
                ("content-type", "application/json"),
                ("host", "astrum.example"),
            ],
            serde_json::to_vec(&body).unwrap(),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let listed = &reply.json()["result"]["structuredContent"];
    assert_eq!(listed["count"], 2, "{listed}");
    let mut tags: Vec<&str> = listed["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tag| tag["tag"].as_str().unwrap())
        .collect();
    tags.sort();
    assert_eq!(tags, ["1.0", "2.0"]);
    let newer = listed["tags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tag| tag["tag"] == "2.0")
        .unwrap();
    assert_eq!(newer["pull"], "astrum.example/hosted/team/app:2.0");
    assert!(newer["manifestBytes"].as_u64().unwrap() > 0, "{newer}");
}

#[tokio::test]
async fn tags_show_the_size_of_their_image() {
    let registry = Registry::new().await;
    registry.hosted("hosted", "ALLOW").await;
    let prefix = "/v2/hosted/team/app";
    let (amd, _) = registry.push_image(prefix, "amd", b"layer-amd").await;
    let (arm, _) = registry.push_image(prefix, "arm", b"layer-arm").await;
    let index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": amd, "size": 10, "platform": {"os": "linux", "architecture": "amd64"}},
            {"mediaType": OCI_MANIFEST, "digest": arm, "size": 10, "platform": {"os": "linux", "architecture": "arm64", "variant": "v8"}},
            {"mediaType": OCI_MANIFEST, "digest": amd, "size": 10, "platform": {"os": "unknown", "architecture": "unknown"}, "annotations": {"vnd.docker.reference.type": "attestation-manifest"}},
        ],
    });
    let pushed = registry.push_manifest(prefix, "multi", &index).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());

    let details = |reference: &str| {
        format!(
            "/service/rest/internal/ui/browse/hosted/asset?path=/v2/team/app/manifests/{reference}"
        )
    };
    let single = registry.admin(Method::GET, &details("amd")).await;
    assert_eq!(single.status, StatusCode::OK, "{}", single.text());
    assert_eq!(single.json()["image"], json!({"size": 15, "layers": 1}));

    let multi = registry.admin(Method::GET, &details("multi")).await.json();
    let platforms = multi["image"]["platforms"].as_array().unwrap();
    assert_eq!(platforms.len(), 2, "{multi}");
    assert_eq!(platforms[0]["platform"], "linux/amd64");
    assert_eq!(platforms[0]["size"], 15);
    assert_eq!(platforms[1]["platform"], "linux/arm64/v8");
    assert_eq!(platforms[1]["layers"], 1);

    let blob = registry
        .admin(
            Method::GET,
            "/service/rest/internal/ui/browse/hosted/asset?path=/v2/-/blobs/x",
        )
        .await;
    assert_ne!(blob.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn a_directory_account_gets_no_docker_token() {
    let registry = Registry::new().await;
    registry.activate_token_realm().await;
    let _directory = registry.fixture.directory().await;
    registry
        .fixture
        .add_user("bob", "bob-local-password-1", &[])
        .await;
    let by_basic = registry
        .send(
            Method::GET,
            "/v2/token?scope=x",
            Some(&basic("bob", "bob-secret")),
            &[],
            Vec::new(),
        )
        .await;
    let by_grant = registry
        .send(
            Method::POST,
            "/v2/token",
            None,
            &[("content-type", "application/x-www-form-urlencoded")],
            b"grant_type=password&username=alice&password=alice-secret".to_vec(),
        )
        .await;
    assert_eq!(
        by_basic.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        by_basic.text()
    );
    assert!(
        !by_basic.text().contains("directory"),
        "{}",
        by_basic.text()
    );
    assert_eq!(
        by_grant.status,
        StatusCode::FORBIDDEN,
        "{}",
        by_grant.text()
    );
    let error = &by_grant.json()["errors"][0];
    assert_eq!(error["code"], "DENIED");
    assert_eq!(
        error["message"],
        "this account signs in through a directory; use an access token instead"
    );
    assert_eq!(
        number(&registry.fixture, "SELECT count(*) FROM format_token").await,
        0
    );
}

#[tokio::test]
async fn a_customer_pulls_listed_images_after_docker_login_and_is_refused_the_rest() {
    let registry = Registry::new().await;
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &[
            "AstrumAuthenticatingRealm",
            "AccessTokenRealm",
            "DockerToken",
        ],
    )
    .await
    .unwrap();
    registry.fixture.state.security().changed().await.unwrap();
    registry.hosted("onprem", "ALLOW").await;
    registry.hosted("private", "ALLOW").await;
    let (_, listed_manifest) = registry
        .push_image("/v2/onprem/app", "1.0", b"listed")
        .await;
    registry
        .push_image("/v2/private/app", "1.0", b"unlisted")
        .await;
    let (_, token) = registry.fixture.customer_token("acme", &["onprem"]).await;
    let login = basic("acme", &token);

    let exchanged = registry
        .send(
            Method::GET,
            "/v2/token?scope=repository:onprem/app:pull",
            Some(&login),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(exchanged.status, StatusCode::OK, "{}", exchanged.text());
    assert_eq!(exchanged.json()["token"], token.as_str());
    let bearer = format!("Bearer {token}");

    let pulled = registry
        .send(
            Method::GET,
            "/v2/onprem/app/manifests/1.0",
            Some(&bearer),
            &[("accept", OCI_MANIFEST)],
            Vec::new(),
        )
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    assert_eq!(pulled.body, listed_manifest);
    let pulled_by_basic = registry
        .send(
            Method::GET,
            "/v2/onprem/app/manifests/1.0",
            Some(&login),
            &[("accept", OCI_MANIFEST)],
            Vec::new(),
        )
        .await;
    assert_eq!(pulled_by_basic.status, StatusCode::OK);

    let refused = registry
        .send(
            Method::GET,
            "/v2/private/app/manifests/1.0",
            Some(&bearer),
            &[("accept", OCI_MANIFEST)],
            Vec::new(),
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    let pushed = registry
        .send(
            Method::POST,
            "/v2/onprem/app/blobs/uploads/?digest=sha256:00",
            Some(&bearer),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(pushed.status, StatusCode::FORBIDDEN, "{}", pushed.text());

    let catalog = registry
        .send(Method::GET, "/v2/_catalog", Some(&bearer), &[], Vec::new())
        .await;
    assert_eq!(catalog.status, StatusCode::OK, "{}", catalog.text());
    assert_eq!(catalog.json()["repositories"], json!(["onprem/app"]));

    for scheme in ["", "Token "] {
        let replayed = registry
            .send(
                Method::GET,
                "/v2/token",
                Some(&format!("{scheme}{token}")),
                &[],
                Vec::new(),
            )
            .await;
        assert_eq!(
            replayed.status,
            StatusCode::FORBIDDEN,
            "{scheme:?}: {}",
            replayed.text()
        );
        assert!(!replayed.text().contains("DockerToken."));
    }

    let form = |user: &str| {
        format!(
            "grant_type=password&username={user}&password={token}&scope=repository:onprem/app:pull"
        )
        .into_bytes()
    };
    let urlencoded = [("content-type", "application/x-www-form-urlencoded")];
    let oauth = registry
        .send(Method::POST, "/v2/token", None, &urlencoded, form("acme"))
        .await;
    assert_eq!(oauth.status, StatusCode::OK, "{}", oauth.text());
    let stranger = registry
        .send(
            Method::POST,
            "/v2/token",
            None,
            &urlencoded,
            form("stranger"),
        )
        .await;
    assert_eq!(stranger.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_blob_pulled_with_a_customer_token_counts_one_download_for_the_token() {
    let registry = Registry::new().await;
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &[
            "AstrumAuthenticatingRealm",
            "AccessTokenRealm",
            "DockerToken",
        ],
    )
    .await
    .unwrap();
    registry.fixture.state.security().changed().await.unwrap();
    registry.hosted("onprem", "ALLOW").await;
    registry
        .push_image("/v2/onprem/app", "1.0", b"listed")
        .await;
    let (id, token) = registry.fixture.customer_token("acme", &["onprem"]).await;
    let pulled = registry
        .send(
            Method::GET,
            &format!("/v2/onprem/app/blobs/{}", sha256(b"listed")),
            Some(&format!("Bearer {token}")),
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    registry
        .fixture
        .state
        .content()
        .flush_downloads()
        .await
        .unwrap();
    let rows = astrum_db::entity::token_download_day::Entity::find()
        .all(registry.fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token_id, id);
    assert_eq!(rows[0].customer.as_deref(), Some("acme"));
    assert_eq!(rows[0].repository, "onprem");
    assert_eq!((rows[0].downloads, rows[0].bytes), (1, 6));
}

async fn trusted(registry: &Registry, name: &str) -> Ctx {
    let ctx = support::repository_ctx(&registry.fixture, name).await;
    let repository = ctx.repository.clone();
    ctx.member(repository)
}

fn copy() -> Terms {
    Terms::new(Conflict::Fail, PROMOTED)
}

async fn promote(
    registry: &Registry,
    source: &str,
    target: &str,
    reference: &str,
    options: &Options,
    terms: Terms,
) -> Result<Outcome, ContentError> {
    let from = trusted(registry, source).await;
    let to = trusted(registry, target).await;
    let prepared = promotion::prepare(&from, "app", reference, options).await?;
    TRANSFER.receive(&to, &prepared, terms).await
}

async fn rows(registry: &Registry, sql: &str) -> i64 {
    registry
        .fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

fn files_under(path: &std::path::Path) -> usize {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| match entry.path().is_dir() {
                    true => files_under(&entry.path()),
                    false => 1,
                })
                .sum()
        })
        .unwrap_or(0)
}

async fn pair(registry: &Registry) {
    registry.hosted("dev", "ALLOW").await;
    registry.hosted("release", "ALLOW").await;
}

async fn pulled(registry: &Registry, path: &str) -> Reply {
    let reply = registry.admin(Method::GET, path).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply
}

async fn collect_garbage(registry: &Registry, repository: &str) -> String {
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!("UPDATE asset SET blob_updated_at = now() - interval '2 days' WHERE repository = '{repository}'"),
        ))
        .await
        .unwrap();
    registry
        .fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_docker::gc::Gc));
    let created = registry
        .fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "repository.docker.gc", "name": "GC", "properties": {"repositoryName": repository, "deployOffset": 24}})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let started = registry
        .fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/tasks/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(started.status, StatusCode::NO_CONTENT);
    registry.fixture.state.tasks().run_pending().await.unwrap();
    let runs = registry
        .fixture
        .call(
            Method::GET,
            &format!("/service/rest/internal/ui/tasks/{id}/runs"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    runs[0]["message"].as_str().unwrap().to_owned()
}

fn tags(names: &[&str]) -> Options {
    Options {
        target_tags: names.iter().map(|tag| (*tag).to_owned()).collect(),
        ..Options::default()
    }
}

async fn heads(registry: &Registry, repository: &str, references: &[&str], digest: &str) {
    for reference in references {
        let manifest = pulled(
            registry,
            &format!("/v2/{repository}/app/manifests/{reference}"),
        )
        .await;
        assert_eq!(
            manifest.headers["docker-content-digest"], digest,
            "{reference}"
        );
    }
}

#[tokio::test]
async fn an_image_is_promoted_and_pulled_from_the_target() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let (digest, bytes) = registry
        .push_image("/v2/dev/app", "1.0", b"layer one")
        .await;
    let outcome = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert!(outcome.bytes_copied > 0);
    assert_eq!(outcome.bytes_reused, 0);
    for reference in ["1.0", digest.as_str()] {
        let manifest = pulled(&registry, &format!("/v2/release/app/manifests/{reference}")).await;
        assert_eq!(manifest.headers["docker-content-digest"], digest.as_str());
        assert_eq!(manifest.body, bytes);
    }
    let layer = pulled(
        &registry,
        &format!("/v2/release/app/blobs/{}", sha256(b"layer one")),
    )
    .await;
    assert_eq!(layer.body, b"layer one");
    let again = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(again.status, Status::Unchanged);
    assert_eq!(again.bytes_copied, 0);

    registry
        .push_image("/v2/dev/app", "2.0", b"layer two")
        .await;
    let shared = promote(
        &registry,
        "dev",
        "release",
        "2.0",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(shared.status, Status::Copied);
    assert_eq!(shared.bytes_reused, 14);
}

#[tokio::test]
async fn an_index_is_promoted_with_both_children() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (amd, _) = registry.push_image(prefix, "amd64", b"amd layer").await;
    let (arm, _) = registry.push_image(prefix, "arm64", b"arm layer").await;
    let index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": amd, "size": 1, "platform": {"os": "linux", "architecture": "amd64"}},
            {"mediaType": OCI_MANIFEST, "digest": arm, "size": 1, "platform": {"os": "linux", "architecture": "arm64"}},
        ],
    });
    let pushed = registry.push_manifest(prefix, "multi", &index).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let index_bytes = serde_json::to_vec(&index).unwrap();
    let outcome = promote(
        &registry,
        "dev",
        "release",
        "multi",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.assets.len(), 7);
    let served = pulled(&registry, "/v2/release/app/manifests/multi").await;
    assert_eq!(
        served.headers["docker-content-digest"],
        sha256(&index_bytes).as_str()
    );
    assert_eq!(served.body, index_bytes);
    for child in [&amd, &arm] {
        pulled(&registry, &format!("/v2/release/app/manifests/{child}")).await;
    }
    let untouched = registry
        .admin(Method::HEAD, "/v2/release/app/manifests/amd64")
        .await;
    assert_eq!(untouched.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn referrers_travel_with_their_subject_unless_switched_off() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry.hosted("bare", "ALLOW").await;
    let prefix = "/v2/dev/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"signed layer").await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let signature_layer = registry.push_blob(prefix, b"signature").await;
    let signature = image_manifest(
        &empty,
        2,
        &[(&signature_layer, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let signature_digest = sha256(&serde_json::to_vec(&signature).unwrap());
    let pushed = registry
        .push_manifest(prefix, &signature_digest, &signature)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    let listed = pulled(&registry, &format!("/v2/release/app/referrers/{subject}"))
        .await
        .json();
    assert_eq!(listed["manifests"][0]["digest"], signature_digest.as_str());
    let layer = pulled(
        &registry,
        &format!("/v2/release/app/blobs/{signature_layer}"),
    )
    .await;
    assert_eq!(layer.body, b"signature");
    let options = Options {
        referrers: false,
        ..Options::default()
    };
    promote(&registry, "dev", "bare", "1.0", &options, copy())
        .await
        .unwrap();
    let none = pulled(&registry, &format!("/v2/bare/app/referrers/{subject}"))
        .await
        .json();
    assert_eq!(none["manifests"], json!([]));
    let missing = registry
        .admin(
            Method::HEAD,
            &format!("/v2/bare/app/manifests/{signature_digest}"),
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn target_tags_retag_and_a_digest_item_gets_no_tag() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let (digest, _) = registry.push_image("/v2/dev/app", "1.0", b"retagged").await;
    let retag = |tags: &[&str]| Options {
        target_tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        ..Options::default()
    };
    let refused = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &retag(&["bad tag"]),
        copy(),
    )
    .await;
    assert!(
        matches!(refused, Err(ContentError::BadRequest(_))),
        "{refused:?}"
    );
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
    promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &retag(&["stable"]),
        copy(),
    )
    .await
    .unwrap();
    let stable = pulled(&registry, "/v2/release/app/manifests/stable").await;
    assert_eq!(stable.headers["docker-content-digest"], digest.as_str());
    let old = registry
        .admin(Method::HEAD, "/v2/release/app/manifests/1.0")
        .await;
    assert_eq!(old.status, StatusCode::NOT_FOUND);

    registry.hosted("digests", "ALLOW").await;
    promote(
        &registry,
        "dev",
        "digests",
        &digest,
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    pulled(&registry, &format!("/v2/digests/app/manifests/{digest}")).await;
    let tagless = registry
        .admin(Method::HEAD, "/v2/digests/app/manifests/1.0")
        .await;
    assert_eq!(tagless.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_tag_with_another_digest_is_a_conflict_the_caller_resolves() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let (fresh, _) = registry.push_image("/v2/dev/app", "1.0", b"fresh").await;
    let (old, _) = registry.push_image("/v2/release/app", "1.0", b"old").await;
    let failed = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy(),
    )
    .await;
    assert!(
        matches!(failed, Err(ContentError::Conflict(_))),
        "{failed:?}"
    );
    let head = pulled(&registry, "/v2/release/app/manifests/1.0").await;
    assert_eq!(head.headers["docker-content-digest"], old.as_str());
    let skipped = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        Terms::new(Conflict::Skip, PROMOTED),
    )
    .await
    .unwrap();
    assert_eq!(skipped.status, Status::Skipped);
    let head = pulled(&registry, "/v2/release/app/manifests/1.0").await;
    assert_eq!(head.headers["docker-content-digest"], old.as_str());
    let replaced = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        Terms::new(Conflict::Replace, PROMOTED),
    )
    .await
    .unwrap();
    assert_eq!(replaced.status, Status::Copied);
    let head = pulled(&registry, "/v2/release/app/manifests/1.0").await;
    assert_eq!(head.headers["docker-content-digest"], fresh.as_str());
}

#[tokio::test]
async fn a_blob_missing_in_the_source_writes_nothing() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"doomed layer")
        .await;
    let blobs = registry.fixture.data.path().join("blobs");
    let before = files_under(&blobs);
    let from = trusted(&registry, "dev").await;
    let to = trusted(&registry, "release").await;
    let prepared = promotion::prepare(&from, "app", "1.0", &Options::default())
        .await
        .unwrap();
    let layer = sha256(b"doomed layer");
    let deleted = registry
        .admin(Method::DELETE, &format!("/v2/dev/app/blobs/{layer}"))
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let blobs_before = files_under(&blobs);
    assert!(blobs_before < before);
    let failed = TRANSFER.receive(&to, &prepared, copy()).await;
    assert!(
        matches!(failed, Err(ContentError::NotFound(_))),
        "{failed:?}"
    );
    let early = promotion::prepare(&from, "app", "1.0", &Options::default()).await;
    assert!(matches!(early, Err(ContentError::NotFound(_))));
    for sql in [
        "SELECT count(*) AS n FROM asset WHERE repository = 'release'",
        "SELECT count(*) AS n FROM blob_hold",
    ] {
        assert_eq!(rows(&registry, sql).await, 0, "{sql}");
    }
    assert_eq!(files_under(&blobs), blobs_before);
    let anonymous = support::repository_ctx(&registry.fixture, "dev").await;
    let refused = promotion::prepare(&anonymous, "app", "1.0", &Options::default()).await;
    assert!(matches!(refused, Err(ContentError::Unauthorized)));
}

#[tokio::test]
async fn a_dry_run_reports_the_plan_and_writes_nothing() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry.push_image("/v2/dev/app", "1.0", b"planned").await;
    let blobs = registry.fixture.data.path().join("blobs");
    let before = files_under(&blobs);
    let plan = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy().dry_run(),
    )
    .await
    .unwrap();
    assert_eq!(plan.status, Status::Copied);
    assert_eq!(plan.assets.len(), 4);
    assert!(plan.bytes_copied > 0);
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
    assert_eq!(
        rows(&registry, "SELECT count(*) AS n FROM blob_hold").await,
        0
    );
    assert_eq!(files_under(&blobs), before);
    let retagged = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&["a", "b"]),
        copy().dry_run(),
    )
    .await
    .unwrap();
    assert_eq!(retagged.assets.len(), 5);
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
    assert_eq!(files_under(&blobs), before);
}

#[tokio::test]
async fn garbage_collection_right_after_a_promotion_keeps_everything() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (amd, amd_bytes) = registry.push_image(prefix, "amd64", b"amd layer").await;
    let index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [{"mediaType": OCI_MANIFEST, "digest": amd, "size": amd_bytes.len()}],
    });
    registry.push_manifest(prefix, "multi", &index).await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let layer = registry.push_blob(prefix, b"signature").await;
    let signature = image_manifest(
        &empty,
        2,
        &[(&layer, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&amd, amd_bytes.len())),
    );
    let signature_digest = sha256(&serde_json::to_vec(&signature).unwrap());
    registry
        .push_manifest(prefix, &signature_digest, &signature)
        .await;
    promote(
        &registry,
        "dev",
        "release",
        "multi",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    let message = collect_garbage(&registry, "release").await;
    assert!(
        message.starts_with("deleted 0 untagged manifests and 0 unused layers"),
        "{message}"
    );
    for reference in [signature_digest.as_str(), amd.as_str(), "multi"] {
        pulled(&registry, &format!("/v2/release/app/manifests/{reference}")).await;
    }
    pulled(&registry, &format!("/v2/release/app/blobs/{layer}")).await;
}

#[tokio::test]
async fn several_target_tags_arrive_in_one_transfer() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry.hosted("pair", "ALLOW").await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"tagged layer")
        .await;
    let three = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&["stable", "v1", "v1.0", "stable"]),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(three.status, Status::Copied);
    assert_eq!(three.component_ids.len(), 3);
    assert_eq!(three.assets.len(), 6);
    heads(
        &registry,
        "release",
        &["stable", "v1", "v1.0", digest.as_str()],
        &digest,
    )
    .await;
    let untouched = registry
        .admin(Method::HEAD, "/v2/release/app/manifests/1.0")
        .await;
    assert_eq!(untouched.status, StatusCode::NOT_FOUND);
    let two = promote(&registry, "dev", "pair", "1.0", &tags(&["a", "b"]), copy())
        .await
        .unwrap();
    assert_eq!(two.component_ids.len(), 2);
    heads(&registry, "pair", &["a", "b"], &digest).await;
    let again = promote(&registry, "dev", "pair", "1.0", &tags(&["a", "b"]), copy())
        .await
        .unwrap();
    assert_eq!(again.status, Status::Unchanged);
    let extended = promote(
        &registry,
        "dev",
        "pair",
        "1.0",
        &tags(&["a", "b", "c"]),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(extended.status, Status::Copied);
    heads(&registry, "pair", &["a", "b", "c"], &digest).await;
}

#[tokio::test]
async fn tag_based_and_oci_referrers_travel_with_the_image_unless_switched_off() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry.hosted("lean", "ALLOW").await;
    let prefix = "/v2/dev/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"signed layer").await;
    let cosign_config = registry.push_blob(prefix, b"{\"cosign\":true}").await;
    let cosign_layer = registry.push_blob(prefix, b"cosign payload").await;
    let cosign = image_manifest(
        &cosign_config,
        15,
        &[(
            &cosign_layer,
            "application/vnd.dev.cosign.simplesigning.v1+json",
        )],
        None,
        None,
    );
    let cosign_tag = format!("{}.sig", subject.replace(':', "-"));
    let pushed = registry.push_manifest(prefix, &cosign_tag, &cosign).await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let cosign_digest = sha256(&serde_json::to_vec(&cosign).unwrap());
    let empty = registry.push_blob(prefix, b"{}").await;
    let notary_layer = registry.push_blob(prefix, b"notary payload").await;
    let notary = image_manifest(
        &empty,
        2,
        &[(&notary_layer, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let notary_digest = sha256(&serde_json::to_vec(&notary).unwrap());
    let pushed = registry
        .push_manifest(prefix, &notary_digest, &notary)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());

    let outcome = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &Options::default(),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.component_ids.len(), 2);
    heads(
        &registry,
        "release",
        &[cosign_tag.as_str(), cosign_digest.as_str()],
        &cosign_digest,
    )
    .await;
    let listed = pulled(&registry, &format!("/v2/release/app/referrers/{subject}"))
        .await
        .json();
    assert_eq!(listed["manifests"][0]["digest"], notary_digest.as_str());
    for layer in [&cosign_layer, &notary_layer] {
        pulled(&registry, &format!("/v2/release/app/blobs/{layer}")).await;
    }

    let options = Options {
        referrers: false,
        ..Options::default()
    };
    promote(&registry, "dev", "lean", "1.0", &options, copy())
        .await
        .unwrap();
    for path in [
        format!("/v2/lean/app/manifests/{cosign_tag}"),
        format!("/v2/lean/app/manifests/{cosign_digest}"),
        format!("/v2/lean/app/manifests/{notary_digest}"),
        format!("/v2/lean/app/blobs/{cosign_layer}"),
    ] {
        let missing = registry.admin(Method::HEAD, &path).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }
    let none = pulled(&registry, &format!("/v2/lean/app/referrers/{subject}"))
        .await
        .json();
    assert_eq!(none["manifests"], json!([]));
}

#[tokio::test]
async fn latest_moves_under_allow_once_only_with_the_latest_policy() {
    let registry = Registry::new().await;
    registry.hosted("dev", "ALLOW").await;
    registry.hosted("once", "ALLOW_ONCE").await;
    let mut fixed = hosted("fixed", "ALLOW_ONCE");
    fixed["storage"]["latestPolicy"] = json!(false);
    registry.create("hosted", fixed).await;
    let replace = || Terms::new(Conflict::Replace, PROMOTED);
    let (first, _) = registry.push_image("/v2/dev/app", "latest", b"one").await;
    for target in ["once", "fixed"] {
        let moved = promote(
            &registry,
            "dev",
            target,
            "latest",
            &Options::default(),
            replace(),
        )
        .await
        .unwrap();
        assert_eq!(moved.status, Status::Copied);
    }
    let (second, _) = registry.push_image("/v2/dev/app", "latest", b"two").await;
    assert_ne!(first, second);

    let failed = promote(
        &registry,
        "dev",
        "once",
        "latest",
        &Options::default(),
        copy(),
    )
    .await;
    assert!(
        matches!(failed, Err(ContentError::Conflict(_))),
        "{failed:?}"
    );
    heads(&registry, "once", &["latest"], &first).await;
    let moved = promote(
        &registry,
        "dev",
        "once",
        "latest",
        &Options::default(),
        replace(),
    )
    .await
    .unwrap();
    assert_eq!(moved.status, Status::Copied);
    heads(&registry, "once", &["latest"], &second).await;

    let refused = promote(
        &registry,
        "dev",
        "fixed",
        "latest",
        &Options::default(),
        replace(),
    )
    .await;
    assert!(
        matches!(refused, Err(ContentError::Conflict(_))),
        "{refused:?}"
    );
    heads(&registry, "fixed", &["latest"], &first).await;

    let (v1, _) = registry.push_image("/v2/dev/app", "v1", b"three").await;
    promote(&registry, "dev", "once", "v1", &Options::default(), copy())
        .await
        .unwrap();
    registry.push_image("/v2/dev/app", "v1", b"four").await;
    for terms in [copy(), replace()] {
        let refused = promote(&registry, "dev", "once", "v1", &Options::default(), terms).await;
        assert!(
            matches!(refused, Err(ContentError::Conflict(_))),
            "{refused:?}"
        );
    }
    heads(&registry, "once", &["v1"], &v1).await;
}

#[tokio::test]
async fn a_failure_on_the_second_tag_leaves_nothing_behind() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"all or nothing")
        .await;
    let blobs = registry.fixture.data.path().join("blobs");
    let before = files_under(&blobs);
    let conn = registry.fixture.state.db().conn();
    for sql in [
        "CREATE FUNCTION fail_second_tag() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.path LIKE '%/manifests/second' THEN RAISE EXCEPTION 'injected failure'; END IF; RETURN NEW; END $$",
        "CREATE TRIGGER fail_second_tag BEFORE INSERT ON asset FOR EACH ROW EXECUTE FUNCTION fail_second_tag()",
    ] {
        conn.execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
    }
    let failed = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&["first", "second"]),
        copy(),
    )
    .await;
    assert!(failed.is_err(), "{failed:?}");
    for sql in [
        "SELECT count(*) AS n FROM asset WHERE repository = 'release'",
        "SELECT count(*) AS n FROM component WHERE repository = 'release'",
        "SELECT count(*) AS n FROM blob_hold",
    ] {
        assert_eq!(rows(&registry, sql).await, 0, "{sql}");
    }
    assert_eq!(files_under(&blobs), before);
    conn.execute(Statement::from_string(
        DbBackend::Postgres,
        "DROP TRIGGER fail_second_tag ON asset",
    ))
    .await
    .unwrap();
    let done = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&["first", "second"]),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(done.status, Status::Copied);
}

#[tokio::test]
async fn an_index_with_a_child_missing_in_the_source_writes_nothing() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (amd, amd_bytes) = registry.push_image(prefix, "amd64", b"amd only").await;
    let (arm, arm_bytes) = registry.push_image(prefix, "arm64", b"arm only").await;
    let index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": amd, "size": amd_bytes.len()},
            {"mediaType": OCI_MANIFEST, "digest": arm, "size": arm_bytes.len()},
        ],
    });
    registry.push_manifest(prefix, "multi", &index).await;
    let from = trusted(&registry, "dev").await;
    let to = trusted(&registry, "release").await;
    let prepared = promotion::prepare(&from, "app", "multi", &Options::default())
        .await
        .unwrap();
    let index_digest = sha256(&serde_json::to_vec(&index).unwrap());
    let paths: Vec<&str> = prepared
        .bundle()
        .assets
        .iter()
        .map(|asset| asset.path.as_str())
        .collect();
    let position = |path: String| {
        paths
            .iter()
            .position(|found| *found == path)
            .unwrap_or_else(|| panic!("{path} is not in {paths:?}"))
    };
    let at = |reference: &str| position(format!("/v2/app/manifests/{reference}"));
    assert!(at(&amd) < at(&index_digest), "{paths:?}");
    assert!(at(&arm) < at(&index_digest), "{paths:?}");
    assert!(at(&index_digest) < at("multi"), "{paths:?}");
    assert_eq!(at("multi"), paths.len() - 1, "{paths:?}");
    let blobs = registry.fixture.data.path().join("blobs");
    let deleted = registry
        .admin(Method::DELETE, &format!("/v2/dev/app/manifests/{arm}"))
        .await;
    assert_eq!(deleted.status, StatusCode::ACCEPTED);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let before = files_under(&blobs);
    let failed = TRANSFER.receive(&to, &prepared, copy()).await;
    assert!(
        matches!(failed, Err(ContentError::NotFound(_))),
        "{failed:?}"
    );
    let early = promotion::prepare(&from, "app", "multi", &Options::default()).await;
    assert!(matches!(early, Err(ContentError::NotFound(_))));
    for sql in [
        "SELECT count(*) AS n FROM asset WHERE repository = 'release'",
        "SELECT count(*) AS n FROM blob_hold",
    ] {
        assert_eq!(rows(&registry, sql).await, 0, "{sql}");
    }
    assert_eq!(files_under(&blobs), before);
}

#[tokio::test]
async fn garbage_collection_keeps_a_multi_tag_promotion_with_its_tag_referrer() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, _) = registry.push_image(prefix, "1.0", b"collected").await;
    let config = registry.push_blob(prefix, b"{\"cosign\":true}").await;
    let layer = registry.push_blob(prefix, b"cosign collected").await;
    let cosign = image_manifest(&config, 15, &[(&layer, LAYER)], None, None);
    let cosign_tag = format!("{}.sig", subject.replace(':', "-"));
    registry.push_manifest(prefix, &cosign_tag, &cosign).await;
    let cosign_digest = sha256(&serde_json::to_vec(&cosign).unwrap());
    promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&["stable", "v1"]),
        copy(),
    )
    .await
    .unwrap();
    let message = collect_garbage(&registry, "release").await;
    assert!(
        message.starts_with("deleted 0 untagged manifests and 0 unused layers"),
        "{message}"
    );
    heads(
        &registry,
        "release",
        &["stable", "v1", subject.as_str()],
        &subject,
    )
    .await;
    heads(
        &registry,
        "release",
        &[cosign_tag.as_str(), cosign_digest.as_str()],
        &cosign_digest,
    )
    .await;
    pulled(&registry, &format!("/v2/release/app/blobs/{layer}")).await;
}

#[tokio::test]
async fn unusual_manifest_media_types_are_promoted_unchanged() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let config = registry.push_blob(prefix, b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(prefix, b"docker layer").await;
    let docker = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
        "config": {"mediaType": "application/vnd.docker.container.image.v1+json", "digest": config, "size": 14},
        "layers": [{"mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip", "digest": layer, "size": 12}],
    });
    let docker_digest = sha256(&serde_json::to_vec(&docker).unwrap());
    registry.push_manifest(prefix, "docker", &docker).await;
    let list = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
        "manifests": [{"mediaType": "application/vnd.docker.distribution.manifest.v2+json", "digest": docker_digest, "size": 1, "platform": {"os": "linux", "architecture": "amd64"}}],
    });
    registry.push_manifest(prefix, "list", &list).await;
    let oci_index = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "artifactType": "application/vnd.example.bundle",
        "manifests": [{"mediaType": "application/vnd.docker.distribution.manifest.v2+json", "digest": docker_digest, "size": 1}],
    });
    registry.push_manifest(prefix, "index", &oci_index).await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let payload = registry.push_blob(prefix, b"example payload").await;
    let artifact = image_manifest(
        &empty,
        2,
        &[(&payload, "application/x-example")],
        Some("application/vnd.example.thing"),
        None,
    );
    registry.push_manifest(prefix, "artifact", &artifact).await;
    for tag in ["docker", "list", "index", "artifact"] {
        let source = pulled(&registry, &format!("/v2/dev/app/manifests/{tag}")).await;
        promote(
            &registry,
            "dev",
            "release",
            tag,
            &Options::default(),
            copy(),
        )
        .await
        .unwrap();
        let target = pulled(&registry, &format!("/v2/release/app/manifests/{tag}")).await;
        assert_eq!(target.body, source.body, "{tag}");
        assert_eq!(
            target.headers["content-type"],
            source.headers["content-type"]
        );
        assert_eq!(
            target.headers["docker-content-digest"],
            source.headers["docker-content-digest"]
        );
    }
}

#[tokio::test]
async fn a_bundle_that_did_not_come_from_prepare_is_refused() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"forged layer")
        .await;
    let from = trusted(&registry, "dev").await;
    let to = trusted(&registry, "release").await;
    let prepared = promotion::prepare(&from, "app", "1.0", &Options::default())
        .await
        .unwrap();
    let mut thin = prepared.bundle().clone();
    thin.assets.retain(|asset| asset.kind == "manifest");
    let source = LocalSource::new(from.clone());
    for bundle in [prepared.bundle().clone(), thin] {
        let refused = Transfer::receive(&TRANSFER, &to, &bundle, &source, copy()).await;
        match refused {
            Err(ContentError::BadRequest(message)) => {
                assert!(message.contains("must come from prepare"), "{message}")
            }
            other => panic!("{other:?}"),
        }
    }
    for sql in [
        "SELECT count(*) AS n FROM asset WHERE repository = 'release'",
        "SELECT count(*) AS n FROM blob_hold",
    ] {
        assert_eq!(rows(&registry, sql).await, 0, "{sql}");
    }
}

#[tokio::test]
async fn tags_that_look_like_file_names_are_promoted() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let (zip, _) = registry
        .push_image("/v2/dev/app", "build.zip", b"layer zip")
        .await;
    let (png, _) = registry
        .push_image("/v2/dev/app", "logo.png", b"layer png")
        .await;
    for (reference, digest) in [("build.zip", &zip), ("logo.png", &png)] {
        let outcome = promote(
            &registry,
            "dev",
            "release",
            reference,
            &Options::default(),
            copy(),
        )
        .await
        .unwrap();
        assert_eq!(outcome.status, Status::Copied, "{reference}");
        heads(&registry, "release", &[reference], digest).await;
    }
    let retagged = promote(
        &registry,
        "dev",
        "release",
        "build.zip",
        &tags(&["archive.tar.gz", "latest"]),
        copy(),
    )
    .await
    .unwrap();
    assert_eq!(retagged.status, Status::Copied);
    heads(&registry, "release", &["archive.tar.gz", "latest"], &zip).await;
}

#[tokio::test]
async fn more_than_a_hundred_target_tags_are_refused_while_the_image_is_read() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"many names")
        .await;
    let from = trusted(&registry, "dev").await;
    let names: Vec<String> = (0..101).map(|number| format!("t{number}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let refused = promotion::prepare(&from, "app", "1.0", &tags(&refs)).await;
    match refused {
        Err(ContentError::BadRequest(message)) => {
            assert!(message.contains("101 tags"), "{message}");
            assert!(message.contains("100 allowed"), "{message}");
        }
        other => panic!("{:?}", other.map(|_| ())),
    }
    let without_referrers = Options {
        referrers: false,
        ..tags(&refs)
    };
    let refused = promotion::prepare(&from, "app", "1.0", &without_referrers).await;
    match refused {
        Err(ContentError::BadRequest(message)) => {
            assert!(message.contains("101 tags"), "{message}");
        }
        other => panic!("{:?}", other.map(|_| ())),
    }
    let refs: Vec<&str> = refs.into_iter().take(100).collect();
    let fits = promote(
        &registry,
        "dev",
        "release",
        "1.0",
        &tags(&refs),
        copy().dry_run(),
    )
    .await
    .unwrap();
    assert_eq!(fits.status, Status::Copied);
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_flood_of_referrers_is_refused_before_they_are_read() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"flooded").await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let payload = registry.push_blob(prefix, b"flood payload").await;
    let signature = image_manifest(
        &empty,
        2,
        &[(&payload, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let signature_digest = sha256(&serde_json::to_vec(&signature).unwrap());
    let pushed = registry
        .push_manifest(prefix, &signature_digest, &signature)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let from = trusted(&registry, "dev").await;
    for (first, last) in [(1, 10_000), (10_001, 11_000)] {
        registry
            .fixture
            .state
            .db()
            .conn()
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!(
                    "INSERT INTO asset (repository, format, path, kind, component_id, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, attributes) \
                     SELECT repository, format, '/v2/app/manifests/sha256:' || lpad(n::text, 64, '0'), kind, component_id, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, \
                     jsonb_set(attributes, '{{docker,digest}}', to_jsonb('sha256:' || lpad(n::text, 64, '0'))) \
                     FROM asset, generate_series({first}, {last}) AS n WHERE repository = 'dev' AND path = '/v2/app/manifests/{signature_digest}'"
                ),
            ))
            .await
            .unwrap();
        let started = std::time::Instant::now();
        let refused = promotion::prepare(&from, "app", "1.0", &Options::default()).await;
        let elapsed = started.elapsed();
        match refused {
            Err(ContentError::BadRequest(message)) => {
                assert!(message.contains("more than the 10000"), "{message}")
            }
            other => panic!("{:?}", other.map(|_| ())),
        }
        assert!(elapsed < std::time::Duration::from_secs(5), "{elapsed:?}");
    }
    let without = Options {
        referrers: false,
        ..Options::default()
    };
    promotion::prepare(&from, "app", "1.0", &without)
        .await
        .unwrap();
}

#[tokio::test]
async fn referrers_with_unreadable_digests_still_count_toward_the_limit() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"counted").await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let payload = registry.push_blob(prefix, b"counted payload").await;
    let signature = image_manifest(
        &empty,
        2,
        &[(&payload, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let signature_digest = sha256(&serde_json::to_vec(&signature).unwrap());
    let pushed = registry
        .push_manifest(prefix, &signature_digest, &signature)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "INSERT INTO asset (repository, format, path, kind, component_id, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, attributes) \
                 SELECT repository, format, '/v2/app/manifests/sha256:' || lpad(n::text, 64, '0'), kind, component_id, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, \
                 jsonb_set(attributes, '{{docker,digest}}', to_jsonb('unreadable-' || n::text)) \
                 FROM asset, generate_series(1, 10000) AS n WHERE repository = 'dev' AND path = '/v2/app/manifests/{signature_digest}'"
            ),
        ))
        .await
        .unwrap();
    let from = trusted(&registry, "dev").await;
    let refused = promotion::prepare(&from, "app", "1.0", &Options::default()).await;
    match refused {
        Err(ContentError::BadRequest(message)) => {
            assert!(message.contains("more than the 10000"), "{message}")
        }
        other => panic!("{:?}", other.map(|_| ())),
    }
}

async fn limited(registry: &Registry, name: &str, expression: &str) -> Ctx {
    let repository = registry
        .fixture
        .state
        .content()
        .repository(name)
        .await
        .unwrap()
        .unwrap();
    let mut model = SecurityModel::default();
    model.selectors.insert(
        "picked".to_owned(),
        ContentSelector {
            name: "picked".to_owned(),
            description: String::new(),
            selector: Selector::parse(expression).unwrap(),
        },
    );
    let subject = Subject::new(
        "jane",
        "default",
        "",
        BTreeSet::new(),
        vec![],
        vec![SelectorGrant {
            selector: "picked".to_owned(),
            format: "docker".to_owned(),
            repository: "*".to_owned(),
            actions: vec![Action::Read],
        }],
    );
    registry
        .fixture
        .state
        .content()
        .context(repository, Arc::new(model), Some(subject), "", "")
}

#[tokio::test]
async fn a_caller_who_may_not_read_the_blobs_cannot_promote_the_image() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"guarded layer")
        .await;
    let to = trusted(&registry, "release").await;
    let manifests_only = limited(&registry, "dev", "path =^ \"/v2/app/manifests/\"").await;
    let refused = promotion::prepare(&manifests_only, "app", "1.0", &Options::default()).await;
    assert!(matches!(refused, Err(ContentError::Forbidden)));
    for sql in [
        "SELECT count(*) AS n FROM asset WHERE repository = 'release'",
        "SELECT count(*) AS n FROM blob_hold",
    ] {
        assert_eq!(rows(&registry, sql).await, 0, "{sql}");
    }
    let everything = limited(&registry, "dev", "path =^ \"/v2/app/\"").await;
    let prepared = promotion::prepare(&everything, "app", "1.0", &Options::default())
        .await
        .unwrap();
    let outcome = TRANSFER.receive(&to, &prepared, copy()).await.unwrap();
    assert_eq!(outcome.status, Status::Copied);
}

#[tokio::test]
async fn a_stale_tag_index_loses_its_missing_children_but_the_main_index_may_not() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, _) = registry.push_image(prefix, "1.0", b"stale subject").await;
    let (kept, kept_bytes) = registry.push_image(prefix, "kept", b"kept child").await;
    let (gone, gone_bytes) = registry.push_image(prefix, "gone", b"gone child").await;
    let fallback = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": kept, "size": kept_bytes.len()},
            {"mediaType": OCI_MANIFEST, "digest": gone, "size": gone_bytes.len()},
        ],
    });
    let fallback_tag = subject.replace(':', "-");
    let pushed = registry
        .push_manifest(prefix, &fallback_tag, &fallback)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let main = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": kept, "size": kept_bytes.len()},
            {"mediaType": OCI_MANIFEST, "digest": gone, "size": gone_bytes.len()},
        ],
    });
    registry.push_manifest(prefix, "multi", &main).await;
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "DELETE FROM asset WHERE repository = 'dev' AND path = '/v2/app/manifests/{gone}'"
            ),
        ))
        .await
        .unwrap();
    let from = trusted(&registry, "dev").await;
    let to = trusted(&registry, "release").await;
    let prepared = promotion::prepare(&from, "app", "1.0", &Options::default())
        .await
        .unwrap();
    assert_eq!(prepared.skipped_children(), 1);
    let outcome = TRANSFER.receive(&to, &prepared, copy()).await.unwrap();
    assert_eq!(outcome.status, Status::Copied);
    heads(&registry, "release", &["1.0", subject.as_str()], &subject).await;
    pulled(
        &registry,
        &format!("/v2/release/app/manifests/{fallback_tag}"),
    )
    .await;
    pulled(&registry, &format!("/v2/release/app/manifests/{kept}")).await;
    let missing = registry
        .admin(Method::HEAD, &format!("/v2/release/app/manifests/{gone}"))
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let strict = promotion::prepare(&from, "app", "multi", &Options::default()).await;
    assert!(matches!(strict, Err(ContentError::NotFound(_))));
}

#[tokio::test]
async fn a_strict_path_to_a_child_a_tag_index_skipped_is_still_refused() {
    let registry = Registry::new().await;
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, _) = registry
        .push_image(prefix, "1.0", b"revisited subject")
        .await;
    let (kept, kept_bytes) = registry.push_image(prefix, "kept", b"revisited kept").await;
    let (gone, gone_bytes) = registry.push_image(prefix, "gone", b"revisited gone").await;
    let fallback = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": kept, "size": kept_bytes.len()},
            {"mediaType": OCI_MANIFEST, "digest": gone, "size": gone_bytes.len()},
        ],
    });
    let fallback_bytes = serde_json::to_vec(&fallback).unwrap();
    let fallback_digest = sha256(&fallback_bytes);
    let pushed = registry
        .push_manifest(prefix, &subject.replace(':', "-"), &fallback)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let strict = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "subject": {"mediaType": OCI_INDEX, "digest": fallback_digest, "size": fallback_bytes.len()},
        "manifests": [{"mediaType": OCI_MANIFEST, "digest": gone, "size": gone_bytes.len()}],
    });
    let strict_digest = sha256(&serde_json::to_vec(&strict).unwrap());
    let pushed = registry
        .push_manifest(prefix, &strict_digest, &strict)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "DELETE FROM asset WHERE repository = 'dev' AND path = '/v2/app/manifests/{gone}'"
            ),
        ))
        .await
        .unwrap();
    let from = trusted(&registry, "dev").await;
    let refused = promotion::prepare(&from, "app", "1.0", &Options::default()).await;
    assert!(matches!(refused, Err(ContentError::NotFound(_))));
}

async fn notices(registry: &Registry, filter: &str) -> i64 {
    rows(
        registry,
        &format!(
            "SELECT count(*) AS n FROM notification_change WHERE repository = 'release' AND {filter}"
        ),
    )
    .await
}

fn images(request: Value) -> Value {
    let mut body = json!({"source": "dev", "target": "release"});
    body["items"] = request;
    body
}

fn rest_statuses(reply: &Reply) -> Vec<String> {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["status"].as_str().unwrap().to_owned())
        .collect()
}

async fn missing(registry: &Registry, path: &str) -> bool {
    registry.admin(Method::HEAD, path).await.status == StatusCode::NOT_FOUND
}

async fn audited(registry: &Registry) -> Vec<astrum_db::entity::audit_event::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    astrum_db::entity::audit_event::Entity::find()
        .filter(astrum_db::entity::audit_event::Column::Domain.eq("repository.content"))
        .all(registry.fixture.state.db().conn())
        .await
        .unwrap()
}

#[tokio::test]
async fn rest_copies_an_image_to_two_new_tags_with_its_referrer() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, subject_bytes) = registry.push_image(prefix, "1.0", b"rest layer").await;
    let empty = registry.push_blob(prefix, b"{}").await;
    let notary_layer = registry.push_blob(prefix, b"notary payload").await;
    let notary = image_manifest(
        &empty,
        2,
        &[(&notary_layer, "application/jose+json")],
        Some("application/vnd.cncf.notary.signature"),
        Some((&subject, subject_bytes.len())),
    );
    let notary_digest = sha256(&serde_json::to_vec(&notary).unwrap());
    let pushed = registry
        .push_manifest(prefix, &notary_digest, &notary)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    let reply = registry
        .promote(
            Auth::Admin,
            images(json!([{"image": "app", "tag": "1.0", "targetTags": ["stable", "2.0"]}])),
        )
        .await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    let result = &reply.json()["results"][0];
    assert!(result.get("warning").is_none());
    assert_eq!(result["componentIds"].as_array().unwrap().len(), 2);
    heads(&registry, "release", &["stable", "2.0"], &subject).await;
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
    let listed = pulled(&registry, &format!("/v2/release/app/referrers/{subject}"))
        .await
        .json();
    assert_eq!(listed["manifests"][0]["digest"], notary_digest.as_str());
    let rows = audited(&registry).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, "promoted");
    assert_eq!(rows[0].attributes["via"], "rest");
    assert_eq!(rows[0].attributes["component"], ":app:stable");
    let again = registry
        .promote(
            Auth::Admin,
            images(json!([{"image": "app", "tag": "1.0", "targetTags": ["stable", "2.0"]}])),
        )
        .await;
    assert_eq!(rest_statuses(&again), ["unchanged"]);
}

#[tokio::test]
async fn a_promoted_image_raises_promoted_notices_for_its_tags_only() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute_unprepared("INSERT INTO notification_channel (id, name, type) VALUES (gen_random_uuid(), 'watch', 'webhook')")
        .await
        .unwrap();
    registry
        .push_image("/v2/dev/app", "1.0", b"noticed layer")
        .await;
    let reply = registry
        .promote(
            Auth::Admin,
            images(json!([{"image": "app", "tag": "1.0", "targetTags": ["stable", "2.0"]}])),
        )
        .await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    assert_eq!(notices(&registry, "action <> 'promoted'").await, 0);
    assert_eq!(notices(&registry, "name <> 'app'").await, 0);
    for tag in ["stable", "2.0"] {
        assert!(notices(&registry, &format!("version = '{tag}'")).await >= 1);
    }
    assert_eq!(
        notices(&registry, "version NOT IN ('stable', '2.0')").await,
        0,
        "blobs and the digest manifest raise no notice of their own"
    );
}

#[tokio::test]
async fn a_docker_dry_run_reports_the_plan_and_writes_nothing() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"planned layer")
        .await;
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["dryRun"] = json!(true);
    let reply = registry.promote(Auth::Admin, body).await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    let result = &reply.json()["results"][0];
    assert_eq!(reply.json()["dryRun"], true);
    assert!(result["bytesCopied"].as_u64().unwrap() > 0);
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
    assert!(missing(&registry, &format!("/v2/release/app/manifests/{digest}")).await);
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
    assert!(audited(&registry).await.is_empty());
}

#[tokio::test]
async fn a_docker_move_by_tag_deletes_only_the_tag() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"moved layer")
        .await;
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["move"] = json!(true);
    let reply = registry.promote(Auth::Admin, body).await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    assert_eq!(reply.json()["results"][0]["sourceKept"], false);
    assert!(missing(&registry, "/v2/dev/app/manifests/1.0").await);
    pulled(&registry, &format!("/v2/dev/app/manifests/{digest}")).await;
    heads(&registry, "release", &["1.0"], &digest).await;
    let rows = audited(&registry).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, "moved");
}

#[tokio::test]
async fn a_docker_move_of_an_unchanged_image_still_deletes_the_source_tag() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"half moved layer")
        .await;
    let copied = registry
        .promote(Auth::Admin, images(json!([{"image": "app", "tag": "1.0"}])))
        .await;
    assert_eq!(rest_statuses(&copied), ["copied"]);
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["move"] = json!(true);
    let reply = registry.promote(Auth::Admin, body).await;
    assert_eq!(rest_statuses(&reply), ["unchanged"]);
    assert_eq!(reply.json()["results"][0]["sourceKept"], false);
    assert!(missing(&registry, "/v2/dev/app/manifests/1.0").await);
    heads(&registry, "release", &["1.0"], &digest).await;
}

#[tokio::test]
async fn a_docker_move_by_digest_is_refused_and_nothing_is_written() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"digest layer")
        .await;
    let mut body = images(json!([{"image": "app", "digest": digest}]));
    body["move"] = json!(true);
    let reply = registry.promote(Auth::Admin, body).await;
    assert_eq!(rest_statuses(&reply), ["failed"]);
    assert_eq!(reply.json()["results"][0]["sourceKept"], true);
    assert!(missing(&registry, &format!("/v2/release/app/manifests/{digest}")).await);
    pulled(&registry, "/v2/dev/app/manifests/1.0").await;
}

#[tokio::test]
async fn an_untagged_digest_copy_carries_a_warning() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"untagged layer")
        .await;
    let reply = registry
        .promote(
            Auth::Admin,
            images(json!([{"image": "app", "digest": digest}])),
        )
        .await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    assert!(
        reply.json()["results"][0]["warning"]
            .as_str()
            .unwrap()
            .contains("garbage collection")
    );
    pulled(&registry, &format!("/v2/release/app/manifests/{digest}")).await;
}

#[tokio::test]
async fn docker_rights_are_checked_on_the_caller_and_nothing_is_written_when_missing() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"guarded layer")
        .await;
    registry
        .user("reader", &[("dev", "read"), ("release", "read")])
        .await;
    registry
        .user("adder", &[("dev", "read"), ("release", "add")])
        .await;
    registry.user("blind", &[("release", "add")]).await;
    let body = images(json!([{"image": "app", "tag": "1.0"}]));
    let reader = registry
        .promote(Auth::Basic("reader", USER_PASSWORD), body.clone())
        .await;
    assert_eq!(rest_statuses(&reader), ["forbidden"]);
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
    assert_eq!(
        rows(
            &registry,
            "SELECT count(*) AS n FROM asset WHERE repository = 'release'"
        )
        .await,
        0
    );
    let blind = registry
        .promote(Auth::Basic("blind", USER_PASSWORD), body.clone())
        .await;
    assert_eq!(blind.status, StatusCode::NOT_FOUND);
    let mut moving = body.clone();
    moving["move"] = json!(true);
    let keeper = registry
        .promote(Auth::Basic("adder", USER_PASSWORD), moving)
        .await;
    assert_eq!(rest_statuses(&keeper), ["forbidden"]);
    assert!(
        keeper.json()["results"][0]["error"]
            .as_str()
            .unwrap()
            .contains("delete access")
    );
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
    let adder = registry
        .promote(Auth::Basic("adder", USER_PASSWORD), body)
        .await;
    assert_eq!(rest_statuses(&adder), ["copied"]);
    heads(&registry, "release", &["1.0"], &digest).await;
}

#[tokio::test]
async fn docker_items_are_unsupported_without_a_registered_implementation() {
    let registry = Registry::new().await;
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"lonely layer")
        .await;
    let reply = registry
        .promote(Auth::Admin, images(json!([{"image": "app", "tag": "1.0"}])))
        .await;
    assert_eq!(rest_statuses(&reply), ["unsupported"]);
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
    let listed = registry
        .fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/promotion-targets?source=dev",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(listed.json(), json!([]));
}

#[tokio::test]
async fn docker_promotion_targets_list_the_repositories_the_caller_may_add_to() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    registry.hosted("sealed", "ALLOW").await;
    registry
        .user(
            "adder",
            &[("dev", "read"), ("release", "add"), ("sealed", "read")],
        )
        .await;
    let listed = registry
        .fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/promotion-targets?source=dev",
            Auth::Basic("adder", USER_PASSWORD),
            None,
        )
        .await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
    assert_eq!(listed.json()[0]["name"], "release");
}

#[tokio::test]
async fn rest_reports_the_children_a_tag_index_named_but_the_source_lacks() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let prefix = "/v2/dev/app";
    let (subject, _) = registry.push_image(prefix, "1.0", b"stale subject").await;
    let (kept, kept_bytes) = registry.push_image(prefix, "kept", b"kept child").await;
    let (gone, gone_bytes) = registry.push_image(prefix, "gone", b"gone child").await;
    let fallback = json!({
        "schemaVersion": 2,
        "mediaType": OCI_INDEX,
        "manifests": [
            {"mediaType": OCI_MANIFEST, "digest": kept, "size": kept_bytes.len()},
            {"mediaType": OCI_MANIFEST, "digest": gone, "size": gone_bytes.len()},
        ],
    });
    let pushed = registry
        .push_manifest(prefix, &subject.replace(':', "-"), &fallback)
        .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "DELETE FROM asset WHERE repository = 'dev' AND path = '/v2/app/manifests/{gone}'"
            ),
        ))
        .await
        .unwrap();
    let reply = registry
        .promote(Auth::Admin, images(json!([{"image": "app", "tag": "1.0"}])))
        .await;
    assert_eq!(rest_statuses(&reply), ["copied"]);
    assert_eq!(reply.json()["results"][0]["skippedChildren"], 1);
}

async fn selector_user(
    registry: &Registry,
    user: &str,
    expression: &str,
    selected: &[(&str, &[&str])],
    plain: &[(&str, &str)],
) {
    let selector = format!("{user}-paths");
    let mut privileges: Vec<String> = plain
        .iter()
        .map(|(repository, action)| format!("as-repository-view-docker-{repository}-{action}"))
        .collect();
    let mut steps = vec![(
        "/service/rest/v1/security/content-selectors".to_owned(),
        json!({"name": selector, "description": "", "expression": format!("format == \"docker\" and {expression}")}),
    )];
    for (repository, actions) in selected {
        let name = format!("{user}-{repository}");
        steps.push((
            "/service/rest/v1/security/privileges/repository-content-selector".to_owned(),
            json!({"name": name, "description": "", "actions": actions, "format": "docker", "repository": repository, "contentSelector": selector}),
        ));
        privileges.push(name);
    }
    steps.push((
        "/service/rest/v1/security/roles".to_owned(),
        json!({"id": user, "name": user, "description": "", "privileges": privileges, "roles": []}),
    ));
    for (path, body) in steps {
        let reply = registry
            .fixture
            .call(Method::POST, &path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.text());
    }
    registry
        .fixture
        .add_user(user, USER_PASSWORD, &[user])
        .await;
}

async fn hold_tag_component(
    registry: &Registry,
    repository: &str,
    path: &str,
) -> sea_orm::DatabaseTransaction {
    let found = store::find(registry.fixture.state.db().conn(), repository, path)
        .await
        .unwrap()
        .unwrap();
    support::hold_row(
        &registry.fixture,
        &format!(
            "SELECT id FROM component WHERE id = {} FOR UPDATE",
            found.component_id.unwrap()
        ),
    )
    .await
}

#[tokio::test]
async fn a_selector_on_the_image_path_is_enough_to_promote_that_image_and_no_other() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (digest, _) = registry
        .push_image("/v2/dev/app", "1.0", b"selected layer")
        .await;
    registry
        .push_image("/v2/dev/other", "1.0", b"unselected layer")
        .await;
    selector_user(
        &registry,
        "scoped",
        "path =^ \"/v2/app/\"",
        &[("release", &["ADD", "BROWSE"])],
        &[("dev", "read")],
    )
    .await;
    let allowed = registry
        .promote(
            Auth::Basic("scoped", USER_PASSWORD),
            images(json!([{"image": "app", "tag": "1.0"}])),
        )
        .await;
    assert_eq!(rest_statuses(&allowed), ["copied"]);
    heads(&registry, "release", &["1.0"], &digest).await;
    let refused = registry
        .promote(
            Auth::Basic("scoped", USER_PASSWORD),
            images(json!([{"image": "other", "tag": "1.0"}])),
        )
        .await;
    assert_eq!(rest_statuses(&refused), ["forbidden"]);
    assert!(missing(&registry, "/v2/release/other/manifests/1.0").await);
}

#[tokio::test]
async fn a_caller_without_add_rights_learns_nothing_about_the_target_from_a_dry_run() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    registry.hosted("dev", "ALLOW").await;
    registry.hosted("release", "ALLOW_ONCE").await;
    registry
        .push_image("/v2/dev/app", "1.0", b"new layer")
        .await;
    let (old, _) = registry
        .push_image("/v2/release/app", "1.0", b"old layer")
        .await;
    selector_user(
        &registry,
        "narrow",
        "path =^ \"/v2/other/\"",
        &[("release", &["ADD"])],
        &[("dev", "read")],
    )
    .await;
    let mut seen = Vec::new();
    for dry_run in [true, false] {
        let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
        body["conflict"] = json!("replace");
        body["dryRun"] = json!(dry_run);
        let reply = registry
            .promote(Auth::Basic("narrow", USER_PASSWORD), body)
            .await;
        assert_eq!(rest_statuses(&reply), ["forbidden"], "{dry_run}");
        seen.push(reply.json()["results"][0].clone());
    }
    assert_eq!(seen[0], seen[1]);
    let text = seen[0]["error"].as_str().unwrap();
    assert!(text.contains("add access"), "{text}");
    for leak in ["exist", "redeploy", "different", "1.0"] {
        assert!(!text.contains(leak), "{leak}: {text}");
    }
    heads(&registry, "release", &["1.0"], &old).await;
}

#[tokio::test]
async fn an_image_the_caller_cannot_see_is_reported_like_one_that_does_not_exist() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    registry
        .push_image("/v2/dev/app", "1.0", b"hidden layer")
        .await;
    selector_user(
        &registry,
        "partial",
        "path =^ \"/v2/other/\"",
        &[("dev", &["BROWSE", "READ"])],
        &[("release", "add")],
    )
    .await;
    let reply = registry
        .promote(
            Auth::Basic("partial", USER_PASSWORD),
            images(json!([
                {"image": "app", "tag": "1.0"},
                {"image": "ghost", "tag": "1.0"}
            ])),
        )
        .await;
    assert_eq!(rest_statuses(&reply), ["notFound", "notFound"]);
    let results = reply.json();
    assert_eq!(
        results["results"][0]["error"],
        results["results"][1]["error"]
    );
    selector_user(
        &registry,
        "browsing",
        "path =^ \"/v2/app/\"",
        &[("dev", &["BROWSE"])],
        &[("release", "add")],
    )
    .await;
    let known = registry
        .promote(
            Auth::Basic("browsing", USER_PASSWORD),
            images(json!([{"image": "app", "tag": "1.0"}])),
        )
        .await;
    assert_eq!(rest_statuses(&known), ["forbidden"]);
    assert!(missing(&registry, "/v2/release/app/manifests/1.0").await);
}

#[tokio::test]
async fn a_tag_that_moved_to_another_image_during_the_transfer_is_not_deleted() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (first, _) = registry
        .push_image("/v2/dev/app", "1.0", b"first layer")
        .await;
    let hold = support::hold_row(
        &registry.fixture,
        "SELECT repository FROM repository_usage WHERE repository = 'release' FOR UPDATE",
    )
    .await;
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["move"] = json!(true);
    let (reply, second) = tokio::join!(registry.promote(Auth::Admin, body), async {
        support::wait_for_waiters(&registry.fixture, 1).await;
        let (second, _) = registry
            .push_image("/v2/dev/app", "1.0", b"second layer")
            .await;
        hold.rollback().await.unwrap();
        second
    });
    assert_eq!(rest_statuses(&reply), ["copied"]);
    let result = &reply.json()["results"][0];
    assert_eq!(result["sourceKept"], true);
    assert!(
        result["error"]
            .as_str()
            .unwrap()
            .contains("tag changed while the transfer ran"),
        "{result}"
    );
    heads(&registry, "release", &["1.0"], &first).await;
    heads(&registry, "dev", &["1.0"], &second).await;
}

#[tokio::test]
async fn a_tag_pushed_again_while_the_delete_waits_for_the_lock_is_not_deleted() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (first, _) = registry
        .push_image("/v2/dev/app", "1.0", b"first layer")
        .await;
    let hold = hold_tag_component(&registry, "dev", "/v2/app/manifests/1.0").await;
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["move"] = json!(true);
    let ((second, _), reply, ()) = tokio::join!(
        registry.push_image("/v2/dev/app", "1.0", b"second layer"),
        async {
            support::wait_for_waiters(&registry.fixture, 1).await;
            registry.promote(Auth::Admin, body).await
        },
        async {
            support::wait_for_waiters(&registry.fixture, 2).await;
            hold.rollback().await.unwrap();
        },
    );
    assert_eq!(rest_statuses(&reply), ["copied"]);
    assert_eq!(reply.json()["results"][0]["sourceKept"], true);
    heads(&registry, "release", &["1.0"], &first).await;
    heads(&registry, "dev", &["1.0"], &second).await;
}

#[tokio::test]
async fn a_tag_deleted_and_pushed_again_while_the_delete_waits_for_the_lock_is_not_deleted() {
    let registry = Registry::new().await;
    registry.enable_promotion();
    pair(&registry).await;
    let (first, _) = registry
        .push_image("/v2/dev/app", "1.0", b"first layer")
        .await;
    let hold = hold_tag_component(&registry, "dev", "/v2/app/manifests/1.0").await;
    let mut body = images(json!([{"image": "app", "tag": "1.0"}]));
    body["move"] = json!(true);
    let (deleted, (second, _), reply, ()) = tokio::join!(
        registry.admin(Method::DELETE, "/v2/dev/app/manifests/1.0"),
        async {
            support::wait_for_waiters(&registry.fixture, 1).await;
            registry
                .push_image("/v2/dev/app", "1.0", b"second layer")
                .await
        },
        async {
            support::wait_for_waiters(&registry.fixture, 2).await;
            registry.promote(Auth::Admin, body).await
        },
        async {
            support::wait_for_waiters(&registry.fixture, 3).await;
            hold.rollback().await.unwrap();
        },
    );
    assert!(deleted.status.is_success(), "{}", deleted.text());
    assert_eq!(rest_statuses(&reply), ["copied"]);
    let result = &reply.json()["results"][0];
    assert_eq!(result["sourceKept"], true);
    assert!(
        result["error"].as_str().unwrap().contains("already gone"),
        "{result}"
    );
    heads(&registry, "release", &["1.0"], &first).await;
    heads(&registry, "dev", &["1.0"], &second).await;
}
