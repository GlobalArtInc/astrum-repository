mod support;

use std::sync::Arc;

use astrum_content::transfer::{Conflict, PROMOTED, Status, Terms};
use astrum_content::{ContentError, Ctx};
use astrum_format_docker::transfer::{self as promotion, Options, TRANSFER};
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

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn hosted(name: &str, policy: &str, patterns: &[&str], exceptions: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {
            "blobStoreName": "default",
            "strictContentTypeValidation": true,
            "writePolicy": policy,
            "latestPolicy": true,
            "immutableTags": {"patterns": patterns, "exceptions": exceptions},
        },
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
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
        fixture
            .state
            .set_image_transfers(Arc::new(astrum_format_docker::transfer::Images));
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

    async fn promote_image(&self, source: &str, target: &str, tag: &str, options: Value) -> Value {
        let mut body = json!({
            "source": source,
            "target": target,
            "items": [{"image": "app", "tag": tag}],
        });
        body.as_object_mut()
            .unwrap()
            .extend(options.as_object().unwrap().clone());
        let reply = self
            .rest(Method::POST, "/service/rest/v1/promotions", Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        reply.json()["results"][0].clone()
    }

    async fn status(&self, method: Method, path: &str) -> StatusCode {
        self.send(method, path, &[], Vec::new()).await.status
    }

    async fn rest(&self, method: Method, path: &str, body: Option<Value>) -> Reply {
        self.fixture.call(method, path, Auth::Admin, body).await
    }

    async fn create(&self, body: Value) {
        let reply = self
            .rest(
                Method::POST,
                "/service/rest/v1/repositories/docker/hosted",
                Some(body),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }

    async fn reconfigure(&self, name: &str, body: Value) {
        let reply = self
            .rest(
                Method::PUT,
                &format!("/service/rest/v1/repositories/docker/hosted/{name}"),
                Some(body),
            )
            .await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    }

    async fn push_blob(&self, prefix: &str, bytes: &[u8]) -> String {
        let digest = sha256(bytes);
        let reply = self
            .send(
                Method::POST,
                &format!("{prefix}/blobs/uploads/?digest={digest}"),
                &[("content-type", "application/octet-stream")],
                bytes.to_vec(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        digest
    }

    async fn try_push(
        &self,
        prefix: &str,
        reference: &str,
        layer: &[u8],
    ) -> (Reply, String, String) {
        let config = self.push_blob(prefix, b"{\"os\":\"linux\"}").await;
        let layer = self.push_blob(prefix, layer).await;
        let manifest = json!({
            "schemaVersion": 2,
            "mediaType": OCI_MANIFEST,
            "config": {"mediaType": OCI_CONFIG, "digest": config, "size": 14},
            "layers": [{"mediaType": LAYER, "digest": layer, "size": 1}],
        });
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let reply = self
            .send(
                Method::PUT,
                &format!("{prefix}/manifests/{reference}"),
                &[("content-type", OCI_MANIFEST)],
                bytes.clone(),
            )
            .await;
        (reply, sha256(&bytes), layer)
    }

    async fn push(&self, prefix: &str, reference: &str, layer: &[u8]) -> (String, String) {
        let (reply, digest, layer) = self.try_push(prefix, reference, layer).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        (digest, layer)
    }

    async fn pulls(&self, path: &str) -> Option<String> {
        let reply = self.send(Method::GET, path, &[], Vec::new()).await;
        match reply.status {
            StatusCode::OK => Some(
                reply
                    .headers
                    .get("docker-content-digest")
                    .map(|value| value.to_str().unwrap().to_owned())
                    .unwrap_or_default(),
            ),
            StatusCode::NOT_FOUND => None,
            other => panic!("{path}: {other} {}", reply.text()),
        }
    }

    async fn count(&self, sql: &str) -> i64 {
        self.fixture
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
}

fn denied(reply: &Reply, message: &str) {
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    let error = &reply.json()["errors"][0];
    assert_eq!(error["code"], "DENIED");
    assert_eq!(error["message"], message);
}

async fn trusted(registry: &Registry, name: &str) -> Ctx {
    let ctx = support::repository_ctx(&registry.fixture, name).await;
    let repository = ctx.repository.clone();
    ctx.member(repository)
}

async fn promote(
    registry: &Registry,
    source: &str,
    target: &str,
    reference: &str,
    conflict: Conflict,
) -> Result<astrum_content::transfer::Outcome, ContentError> {
    let from = trusted(registry, source).await;
    let to = trusted(registry, target).await;
    let prepared = promotion::prepare(&from, "app", reference, &Options::default()).await?;
    TRANSFER
        .receive(&to, &prepared, Terms::new(conflict, PROMOTED))
        .await
}

#[tokio::test]
async fn a_release_tag_takes_the_same_digest_again_and_refuses_another() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let (first, _) = registry.push("/v2/rel/team/app", "v1", b"one").await;
    let (again, _) = registry.push("/v2/rel/team/app", "v1", b"one").await;
    assert_eq!(first, again);
    let (reply, _, _) = registry.try_push("/v2/rel/team/app", "v1", b"two").await;
    denied(
        &reply,
        "tag v1 of team/app is immutable in repository rel (rule v*)",
    );
    assert_eq!(
        registry
            .pulls("/v2/rel/team/app/manifests/v1")
            .await
            .as_deref(),
        Some(first.as_str())
    );
    let (_, _) = registry.push("/v2/rel/team/app", "dev", b"one").await;
    let (moved, _) = registry.push("/v2/rel/team/app", "dev", b"two").await;
    assert_ne!(moved, first);
}

#[tokio::test]
async fn an_excepted_tag_moves_and_the_latest_policy_cannot_free_an_immutable_one() {
    let registry = Registry::new().await;
    registry
        .create(hosted("once", "ALLOW_ONCE", &["*"], &["latest"]))
        .await;
    registry
        .create(hosted("strict", "ALLOW_ONCE", &["*"], &[]))
        .await;
    registry.create(hosted("dev", "ALLOW", &[], &[])).await;
    for target in ["once", "strict"] {
        registry
            .push(&format!("/v2/{target}/app"), "latest", b"one")
            .await;
    }
    let (moved, _) = registry.push("/v2/once/app", "latest", b"two").await;
    assert_eq!(
        registry
            .pulls("/v2/once/app/manifests/latest")
            .await
            .as_deref(),
        Some(moved.as_str())
    );
    let (reply, _, _) = registry.try_push("/v2/strict/app", "latest", b"two").await;
    denied(
        &reply,
        "tag latest of app is immutable in repository strict (rule *)",
    );

    let (first, _) = registry.push("/v2/dev/app", "latest", b"three").await;
    let moved = promote(&registry, "dev", "once", "latest", Conflict::Replace)
        .await
        .unwrap();
    assert_eq!(moved.status, Status::Copied);
    let refused = promote(&registry, "dev", "strict", "latest", Conflict::Replace).await;
    assert!(
        matches!(refused, Err(ContentError::Protected(_))),
        "{refused:?}"
    );
    assert_eq!(
        registry
            .pulls("/v2/once/app/manifests/latest")
            .await
            .as_deref(),
        Some(first.as_str())
    );
    assert_ne!(
        registry
            .pulls("/v2/strict/app/manifests/latest")
            .await
            .as_deref(),
        Some(first.as_str())
    );
}

#[tokio::test]
async fn an_immutable_tag_is_not_deleted_by_tag_or_by_digest_but_a_mutable_one_is() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let (digest, layer) = registry.push("/v2/rel/app", "v1", b"one").await;
    registry.push("/v2/rel/app", "dev", b"one").await;
    let (other, _) = registry.push("/v2/rel/app", "scratch", b"two").await;

    let by_tag = registry
        .send(Method::DELETE, "/v2/rel/app/manifests/v1", &[], Vec::new())
        .await;
    denied(
        &by_tag,
        "tag v1 of app is immutable in repository rel (rule v*)",
    );
    let by_digest = registry
        .send(
            Method::DELETE,
            &format!("/v2/rel/app/manifests/{digest}"),
            &[],
            Vec::new(),
        )
        .await;
    denied(
        &by_digest,
        "tag v1 of app is immutable in repository rel (rule v*)",
    );
    for path in [
        "/v2/rel/app/manifests/v1".to_owned(),
        "/v2/rel/app/manifests/dev".to_owned(),
        format!("/v2/rel/app/manifests/{digest}"),
        format!("/v2/rel/app/blobs/{layer}"),
    ] {
        assert_eq!(
            registry.status(Method::HEAD, &path).await,
            StatusCode::OK,
            "{path}"
        );
    }

    assert_eq!(
        registry
            .status(Method::DELETE, "/v2/rel/app/manifests/scratch")
            .await,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        registry
            .status(Method::DELETE, &format!("/v2/rel/app/manifests/{other}"))
            .await,
        StatusCode::ACCEPTED
    );
    assert_eq!(registry.pulls("/v2/rel/app/manifests/scratch").await, None);
}

#[tokio::test]
async fn a_cosign_signature_tag_moves_under_a_rule_that_matches_everything() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["*"], &[])).await;
    let (digest, _) = registry.push("/v2/rel/app", "v1", b"one").await;
    let signature = format!("{}.sig", digest.replace(':', "-"));
    registry
        .push("/v2/rel/app", &signature, b"signed once")
        .await;
    let (resigned, _) = registry
        .push("/v2/rel/app", &signature, b"signed twice")
        .await;
    assert_eq!(
        registry
            .pulls(&format!("/v2/rel/app/manifests/{signature}"))
            .await
            .as_deref(),
        Some(resigned.as_str())
    );
    assert_eq!(
        registry
            .status(
                Method::DELETE,
                &format!("/v2/rel/app/manifests/{signature}")
            )
            .await,
        StatusCode::ACCEPTED
    );
}

#[tokio::test]
async fn removing_the_rule_frees_the_tags_and_a_repository_with_the_rule_can_be_deleted() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    registry.create(hosted("gone", "ALLOW", &["v*"], &[])).await;
    registry.push("/v2/rel/app", "v1", b"one").await;
    registry.push("/v2/gone/app", "v1", b"one").await;
    let (reply, _, _) = registry.try_push("/v2/rel/app", "v1", b"two").await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    registry
        .reconfigure("rel", hosted("rel", "ALLOW", &[], &[]))
        .await;
    let (moved, _) = registry.push("/v2/rel/app", "v1", b"two").await;
    assert_eq!(
        registry.pulls("/v2/rel/app/manifests/v1").await.as_deref(),
        Some(moved.as_str())
    );
    assert_eq!(
        registry
            .status(Method::DELETE, "/v2/rel/app/manifests/v1")
            .await,
        StatusCode::ACCEPTED
    );

    let removed = registry
        .rest(Method::DELETE, "/service/rest/v1/repositories/gone", None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert_eq!(
        registry
            .count("SELECT count(*) AS n FROM asset WHERE repository = 'gone'")
            .await,
        0
    );
}

#[tokio::test]
async fn a_promotion_that_would_replace_an_immutable_tag_is_refused_and_writes_nothing() {
    let registry = Registry::new().await;
    registry.create(hosted("dev", "ALLOW", &[], &[])).await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let (first, _) = registry.push("/v2/dev/app", "v1", b"one").await;
    promote(&registry, "dev", "rel", "v1", Conflict::Fail)
        .await
        .unwrap();
    promote(&registry, "dev", "rel", "v1", Conflict::Fail)
        .await
        .unwrap();
    registry.push("/v2/dev/app", "v1", b"two").await;
    let assets = registry
        .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
        .await;
    let blobs = registry
        .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel' AND kind = 'blob'")
        .await;

    let refused = promote(&registry, "dev", "rel", "v1", Conflict::Replace).await;
    assert!(
        matches!(refused, Err(ContentError::Protected(_))),
        "{refused:?}"
    );
    let refused = promote(&registry, "dev", "rel", "v1", Conflict::Fail).await;
    assert!(
        matches!(refused, Err(ContentError::Conflict(_))),
        "{refused:?}"
    );
    assert_eq!(
        registry
            .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
            .await,
        assets
    );
    assert_eq!(
        registry
            .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel' AND kind = 'blob'")
            .await,
        blobs
    );
    assert_eq!(
        registry.count("SELECT count(*) AS n FROM blob_hold").await,
        0
    );
    assert_eq!(
        registry.pulls("/v2/rel/app/manifests/v1").await.as_deref(),
        Some(first.as_str())
    );
}

#[tokio::test]
async fn garbage_collection_keeps_the_image_of_an_immutable_tag() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let (digest, layer) = registry.push("/v2/rel/app", "v1", b"kept").await;
    let (orphan, _) = registry.push("/v2/rel/app", "scratch", b"orphan").await;
    registry
        .status(Method::DELETE, "/v2/rel/app/manifests/scratch")
        .await;
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            "UPDATE asset SET blob_updated_at = now() - interval '2 days' WHERE repository = 'rel'",
        ))
        .await
        .unwrap();
    registry
        .fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_docker::gc::Gc));
    let created = registry
        .rest(
            Method::POST,
            "/service/rest/v1/tasks",
            Some(json!({"type": "repository.docker.gc", "name": "GC", "properties": {"repositoryName": "rel", "deployOffset": 24}})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let started = registry
        .rest(
            Method::POST,
            &format!("/service/rest/v1/tasks/{id}/run"),
            None,
        )
        .await;
    assert_eq!(started.status, StatusCode::NO_CONTENT);
    registry.fixture.state.tasks().run_pending().await.unwrap();
    let runs = registry
        .rest(
            Method::GET,
            &format!("/service/rest/internal/ui/tasks/{id}/runs"),
            None,
        )
        .await
        .json();
    let message = runs[0]["message"].as_str().unwrap();
    assert!(
        message.starts_with("deleted 1 untagged manifest and 1 unused layer"),
        "{message}"
    );
    assert_eq!(
        registry
            .status(Method::HEAD, &format!("/v2/rel/app/manifests/{orphan}"))
            .await,
        StatusCode::NOT_FOUND
    );
    for path in [
        "/v2/rel/app/manifests/v1".to_owned(),
        format!("/v2/rel/app/manifests/{digest}"),
        format!("/v2/rel/app/blobs/{layer}"),
    ] {
        assert_eq!(
            registry.status(Method::GET, &path).await,
            StatusCode::OK,
            "{path}"
        );
    }
}

#[tokio::test]
async fn a_push_naming_several_tags_is_refused_before_any_of_them_is_written() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let prefix = "/v2/rel/app";
    let (first, _) = registry.push(prefix, "v1", b"one").await;
    let config = registry.push_blob(prefix, b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(prefix, b"two").await;
    let bytes = serde_json::to_vec(&json!({
        "schemaVersion": 2,
        "mediaType": OCI_MANIFEST,
        "config": {"mediaType": OCI_CONFIG, "digest": config, "size": 14},
        "layers": [{"mediaType": LAYER, "digest": layer, "size": 1}],
    }))
    .unwrap();
    let digest = sha256(&bytes);
    let reply = registry
        .send(
            Method::PUT,
            &format!("{prefix}/manifests/{digest}?tag=free&tag=v1"),
            &[("content-type", OCI_MANIFEST)],
            bytes,
        )
        .await;
    denied(
        &reply,
        "tag v1 of app is immutable in repository rel (rule v*)",
    );
    assert_eq!(
        registry
            .pulls(&format!("{prefix}/manifests/{digest}"))
            .await,
        None
    );
    assert_eq!(
        registry.pulls(&format!("{prefix}/manifests/free")).await,
        None
    );
    assert_eq!(
        registry
            .pulls(&format!("{prefix}/manifests/v1"))
            .await
            .as_deref(),
        Some(first.as_str())
    );
}

#[tokio::test]
async fn immutable_tags_hold_for_images_with_a_slash_or_manifests_in_their_name() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    for image in ["team/app", "a/manifests/b"] {
        let prefix = format!("/v2/rel/{image}");
        let message = format!("tag v1 of {image} is immutable in repository rel (rule v*)");
        let (digest, _) = registry.push(&prefix, "v1", b"one").await;

        let by_tag = registry
            .send(
                Method::DELETE,
                &format!("{prefix}/manifests/v1"),
                &[],
                Vec::new(),
            )
            .await;
        denied(&by_tag, &message);
        let by_digest = registry
            .send(
                Method::DELETE,
                &format!("{prefix}/manifests/{digest}"),
                &[],
                Vec::new(),
            )
            .await;
        denied(&by_digest, &message);
        let (reply, _, _) = registry.try_push(&prefix, "v1", b"two").await;
        denied(&reply, &message);

        assert_eq!(
            registry.pulls(&format!("{prefix}/manifests/v1")).await,
            Some(digest.clone())
        );
        assert_eq!(
            registry
                .pulls(&format!("{prefix}/manifests/{digest}"))
                .await,
            Some(digest)
        );
    }
}

#[tokio::test]
async fn a_stored_rule_that_does_not_compile_freezes_the_existing_tags_only() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    registry.create(hosted("gone", "ALLOW", &["v*"], &[])).await;
    let (first, _) = registry.push("/v2/rel/app", "v1", b"one").await;
    registry.push("/v2/gone/app", "v1", b"one").await;
    registry
        .fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            r#"UPDATE repository SET attributes = jsonb_set(attributes, '{storage,immutableTags,patterns}', '["["]') WHERE name IN ('rel', 'gone')"#,
        ))
        .await
        .unwrap();

    let (refused, _, _) = registry.try_push("/v2/rel/app", "v1", b"two").await;
    let by_tag = registry
        .send(Method::DELETE, "/v2/rel/app/manifests/v1", &[], Vec::new())
        .await;
    for reply in [&refused, &by_tag] {
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        let error = &reply.json()["errors"][0];
        assert_eq!(error["code"], "DENIED");
        assert!(
            error["message"]
                .as_str()
                .unwrap()
                .contains("immutableTags.patterns[0]"),
            "{error}"
        );
    }
    assert_eq!(
        registry.pulls("/v2/rel/app/manifests/v1").await.as_deref(),
        Some(first.as_str())
    );
    registry.push("/v2/rel/app", "dev", b"two").await;

    let removed = registry
        .rest(Method::DELETE, "/service/rest/v1/repositories/gone", None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());

    registry
        .reconfigure("rel", hosted("rel", "ALLOW", &[], &[]))
        .await;
    let (fixed, _) = registry.push("/v2/rel/app", "v1", b"two").await;
    assert_ne!(fixed, first);
}

async fn rest_ids(registry: &Registry, path: &str) -> (String, String) {
    let components = registry
        .rest(
            Method::GET,
            "/service/rest/v1/components?repository=rel",
            None,
        )
        .await
        .json();
    for component in components["items"].as_array().unwrap() {
        for asset in component["assets"].as_array().unwrap() {
            if asset["path"] == path {
                return (
                    asset["id"].as_str().unwrap().to_owned(),
                    component["id"].as_str().unwrap().to_owned(),
                );
            }
        }
    }
    panic!("no asset {path} in rel");
}

#[tokio::test]
async fn the_rest_delete_of_an_immutable_tag_answers_409_and_a_mutable_one_is_deleted() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    registry.push("/v2/rel/app", "v1", b"one").await;
    registry.push("/v2/rel/other", "dev", b"two").await;
    registry.push("/v2/rel/third", "dev", b"three").await;
    let assets = registry
        .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
        .await;

    let (asset, component) = rest_ids(&registry, "/v2/app/manifests/v1").await;
    for path in [
        format!("/service/rest/v1/assets/{asset}"),
        format!("/service/rest/v1/components/{component}"),
    ] {
        let reply = registry.rest(Method::DELETE, &path, None).await;
        assert_eq!(reply.status, StatusCode::CONFLICT, "{}", reply.text());
        assert_eq!(
            reply.json()["message"],
            "tag v1 of app is immutable in repository rel (rule v*)"
        );
    }
    assert_eq!(
        registry
            .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
            .await,
        assets
    );

    let (asset, _) = rest_ids(&registry, "/v2/other/manifests/dev").await;
    let deleted = registry
        .rest(
            Method::DELETE,
            &format!("/service/rest/v1/assets/{asset}"),
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let (_, component) = rest_ids(&registry, "/v2/third/manifests/dev").await;
    let deleted = registry
        .rest(
            Method::DELETE,
            &format!("/service/rest/v1/components/{component}"),
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
}

#[tokio::test]
async fn a_folder_delete_keeps_immutable_tags_and_answers_409_when_only_they_are_left() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    registry.push("/v2/rel/app", "v1", b"one").await;
    registry.push("/v2/rel/app", "dev", b"two").await;
    registry.push("/v2/rel/solo", "v1", b"three").await;

    let mixed = registry
        .rest(
            Method::DELETE,
            "/service/rest/internal/ui/browse/rel?path=app",
            None,
        )
        .await;
    assert_eq!(mixed.status, StatusCode::OK, "{}", mixed.text());
    assert_eq!(
        mixed.json(),
        json!({"deletedAssets": 1, "deletedComponents": 1, "skippedAssets": 0, "protectedAssets": 1})
    );
    let audited = registry
        .count("SELECT count(*) AS n FROM audit_event WHERE domain = 'asset' AND context = '/v2/app/manifests/v1'")
        .await;
    assert_eq!(audited, 0);
    assert!(registry.pulls("/v2/rel/app/manifests/v1").await.is_some());

    let solo = registry
        .rest(
            Method::DELETE,
            "/service/rest/internal/ui/browse/rel?path=solo",
            None,
        )
        .await;
    assert_eq!(solo.status, StatusCode::CONFLICT, "{}", solo.text());
    assert_eq!(
        solo.json()["message"],
        "1 protected asset not deleted in /solo/; first reason: tag v1 of solo is immutable in repository rel (rule v*)"
    );
    assert!(registry.pulls("/v2/rel/solo/manifests/v1").await.is_some());
}

const RELEASE_RULE: &str = "tag v1 of app is immutable in repository rel (rule v*)";

async fn replaced_target(registry: &Registry) -> String {
    registry.create(hosted("dev", "ALLOW", &[], &[])).await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    let (first, _) = registry.push("/v2/dev/app", "v1", b"one").await;
    let copied = registry.promote_image("dev", "rel", "v1", json!({})).await;
    assert_eq!(copied["status"], "copied", "{copied}");
    registry.push("/v2/dev/app", "v1", b"two").await;
    first
}

async fn assert_target_untouched(registry: &Registry, assets: i64, first: &str) {
    assert_eq!(
        registry
            .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
            .await,
        assets
    );
    assert_eq!(
        registry.count("SELECT count(*) AS n FROM blob_hold").await,
        0
    );
    assert_eq!(
        registry.pulls("/v2/rel/app/manifests/v1").await.as_deref(),
        Some(first)
    );
}

#[tokio::test]
async fn rest_reports_a_replace_over_an_immutable_target_tag_as_a_final_conflict() {
    let registry = Registry::new().await;
    let first = replaced_target(&registry).await;
    let assets = registry
        .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
        .await;

    let result = registry
        .promote_image("dev", "rel", "v1", json!({"conflict": "replace"}))
        .await;
    assert_eq!(result["status"], "conflict", "{result}");
    assert_eq!(result["error"], RELEASE_RULE);
    assert_eq!(result["retryable"], false);
    assert_target_untouched(&registry, assets, &first).await;
}

#[tokio::test]
async fn a_dry_run_reports_the_same_conflict_over_an_immutable_target_tag() {
    let registry = Registry::new().await;
    let first = replaced_target(&registry).await;
    let assets = registry
        .count("SELECT count(*) AS n FROM asset WHERE repository = 'rel'")
        .await;

    let result = registry
        .promote_image(
            "dev",
            "rel",
            "v1",
            json!({"conflict": "replace", "dryRun": true}),
        )
        .await;
    assert_eq!(result["status"], "conflict", "{result}");
    assert_eq!(result["error"], RELEASE_RULE);
    assert_eq!(result["retryable"], false);
    assert_target_untouched(&registry, assets, &first).await;
}

#[tokio::test]
async fn rest_replaces_an_excepted_latest_and_refuses_it_without_the_exception() {
    let registry = Registry::new().await;
    registry.create(hosted("dev", "ALLOW", &[], &[])).await;
    registry
        .create(hosted("once", "ALLOW_ONCE", &["*"], &["latest"]))
        .await;
    registry
        .create(hosted("strict", "ALLOW_ONCE", &["*"], &[]))
        .await;
    let (older, _) = registry.push("/v2/strict/app", "latest", b"one").await;
    registry.push("/v2/once/app", "latest", b"one").await;
    let (newer, _) = registry.push("/v2/dev/app", "latest", b"three").await;

    let moved = registry
        .promote_image("dev", "once", "latest", json!({"conflict": "replace"}))
        .await;
    assert_eq!(moved["status"], "copied", "{moved}");
    assert_eq!(
        registry
            .pulls("/v2/once/app/manifests/latest")
            .await
            .as_deref(),
        Some(newer.as_str())
    );
    let refused = registry
        .promote_image("dev", "strict", "latest", json!({"conflict": "replace"}))
        .await;
    assert_eq!(refused["status"], "conflict", "{refused}");
    assert_eq!(
        refused["error"],
        "tag latest of app is immutable in repository strict (rule *)"
    );
    assert_eq!(refused["retryable"], false);
    assert_eq!(
        registry
            .pulls("/v2/strict/app/manifests/latest")
            .await
            .as_deref(),
        Some(older.as_str())
    );
}

#[tokio::test]
async fn a_move_of_an_immutable_source_tag_is_copied_and_keeps_the_source_for_good() {
    let registry = Registry::new().await;
    registry.create(hosted("rel", "ALLOW", &["v*"], &[])).await;
    registry.create(hosted("dst", "ALLOW", &[], &[])).await;
    let (digest, _) = registry.push("/v2/rel/app", "v1", b"one").await;

    let result = registry
        .promote_image("rel", "dst", "v1", json!({"move": true}))
        .await;
    assert_eq!(result["status"], "copied", "{result}");
    assert_eq!(result["sourceKept"], true);
    assert_eq!(result["retryable"], false);
    assert_eq!(
        result["error"],
        format!("the source was kept: {RELEASE_RULE}")
    );
    assert_eq!(
        registry.pulls("/v2/rel/app/manifests/v1").await.as_deref(),
        Some(digest.as_str())
    );
    assert_eq!(
        registry.pulls("/v2/dst/app/manifests/v1").await.as_deref(),
        Some(digest.as_str())
    );
}
