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
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256, Sha512};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

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
        self.fixture.state.security().reload().await.unwrap();
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
    registry.fixture.state.security().reload().await.unwrap();
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
    registry
        .fixture
        .state
        .security()
        .issue_format_token(
            astrum_format_docker::REALM,
            "admin",
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
