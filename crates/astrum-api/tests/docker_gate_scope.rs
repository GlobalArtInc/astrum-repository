mod docker_gate_common;
mod inventory_common;
mod support;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderName, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use docker_gate_common::*;
use inventory_common::*;
use serde_json::json;
use support::Auth;

#[tokio::test]
async fn an_index_with_a_blocked_union_refuses_itself_and_its_children_and_a_clean_one_serves() {
    let registry = Registry::new().await;
    let first = registry.image(None, b"one").await;
    let second = registry.image(None, b"two").await;
    registry.document(&first, None, SPDX, &spdx(&[MUSL])).await;
    registry
        .document(&second, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    let blocked = registry
        .push_manifest("multi", &index(&[(&first, None), (&second, None)]))
        .await;
    let third = registry.image(None, b"three").await;
    registry
        .document(&third, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    let clean = registry
        .push_manifest("tidy", &index(&[(&third, None)]))
        .await;
    registry.settle().await;

    denied(&pull(&registry, "multi").await, "GHSA-A");
    denied(&pull(&registry, &blocked).await, "GHSA-A");
    for child in [&first, &second] {
        denied(&pull(&registry, child).await, "part of a blocked index");
    }
    for reference in ["tidy", clean.as_str(), third.as_str()] {
        let served = pull(&registry, reference).await;
        assert_eq!(
            served.status,
            StatusCode::OK,
            "{reference}: {}",
            served.text()
        );
    }

    let stray = registry.image(None, b"stray").await;
    unscanned_block(&registry).await;
    denied(&pull(&registry, &first).await, "part of a blocked index");
    denied(
        &pull(&registry, &third).await,
        "no scan data for this image digest",
    );
    rejudge(&registry).await;
    for reference in ["tidy", clean.as_str(), third.as_str()] {
        let served = pull(&registry, reference).await;
        assert_eq!(
            served.status,
            StatusCode::OK,
            "{reference}: {}",
            served.text()
        );
    }
    denied(
        &pull(&registry, &stray).await,
        "no scan data for this image digest",
    );
    denied(&pull(&registry, "multi").await, "GHSA-A");
    denied(&pull(&registry, &first).await, "part of a blocked index");
}

#[tokio::test]
async fn an_incomplete_inventory_counts_as_unscanned() {
    let registry = Registry::new().await;
    let present = registry.image(None, b"one").await;
    registry
        .document(&present, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    let absent = sha256(b"never pushed");
    registry
        .push_manifest("partial", &index(&[(&present, None), (&absent, None)]))
        .await;
    registry.settle().await;
    assert_eq!(registry.scan_of("partial").await[0], "incomplete");
    assert_eq!(pull(&registry, "partial").await.status, StatusCode::OK);

    unscanned_block(&registry).await;
    rejudge(&registry).await;
    denied(&pull(&registry, "partial").await, "scan: incomplete");
    denied(&pull(&registry, &present).await, "part of a blocked index");
}

#[tokio::test]
async fn signatures_attestations_sboms_and_the_legacy_signature_tag_of_a_blocked_image_are_served()
{
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    unscanned_block(&registry).await;
    rejudge(&registry).await;

    let bom = registry.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    let config = registry.push_blob(b"{}").await;
    let statement = registry.push_blob(statement(&[MUSL]).as_bytes()).await;
    let attested = manifest(&config, (&statement, IN_TOTO), Some(IN_TOTO), Some(&digest));
    let attestation = registry
        .push_manifest(&sha256(&serde_json::to_vec(&attested).unwrap()), &attested)
        .await;
    let dsse = manifest(
        &config,
        (&statement, "application/vnd.dsse.envelope.v1+json"),
        Some("application/vnd.dsse.envelope.v1+json"),
        Some(&digest),
    );
    let envelope = registry
        .push_manifest(&sha256(&serde_json::to_vec(&dsse).unwrap()), &dsse)
        .await;
    let notation = manifest(
        &config,
        (&statement, "application/jose+json"),
        Some("application/vnd.cncf.notary.signature"),
        None,
    );
    let signature = registry
        .push_manifest(&sha256(&serde_json::to_vec(&notation).unwrap()), &notation)
        .await;
    let legacy = format!("{}.sig", digest.replace(':', "-"));
    let cosign = manifest(
        &config,
        (
            &statement,
            "application/vnd.dev.cosign.simplesigning.v1+json",
        ),
        None,
        None,
    );
    registry.push_manifest(&legacy, &cosign).await;

    for reference in [&bom, &attestation, &envelope, &signature, &legacy] {
        let served = pull(&registry, reference).await;
        assert_eq!(
            served.status,
            StatusCode::OK,
            "{reference}: {}",
            served.text()
        );
    }
    denied(&pull(&registry, "1.0").await, "GHSA-A");
}

#[tokio::test]
async fn a_runnable_image_is_judged_whatever_marks_it_carries() {
    let registry = Registry::new().await;
    let target = blocked_image(&registry).await;
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(&marker_layer(b"crafted")).await;
    let with_subject = manifest(&config, (&layer, LAYER), None, Some(&target));
    let claims_signature = manifest(
        &config,
        (&layer, LAYER),
        Some("application/vnd.cncf.notary.signature"),
        None,
    );
    let plain = manifest(&config, (&layer, LAYER), None, None);
    let mut hidden = manifest(&config, (&layer, LAYER), None, Some(&target));
    hidden.as_object_mut().unwrap().remove("layers");
    hidden["Layers"] = json!([{"MediaType": LAYER, "Digest": layer, "Size": 1}]);
    let mut bodies = vec![with_subject, claims_signature, hidden];
    for media_type in [
        "application/vnd.docker.image.rootfs.diff.tar",
        "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
        "application/vnd.oci.image.layer.v1.tar+zstd",
    ] {
        bodies.push(manifest(&config, (&layer, media_type), None, Some(&target)));
    }
    let mut crafted = Vec::new();
    for body in &bodies {
        let digest = sha256(&serde_json::to_vec(body).unwrap());
        crafted.push(registry.push_manifest(&digest, body).await);
    }
    let legacy = format!("sha256-{}.sig", "ab".repeat(32));
    crafted.push(registry.push_manifest(&legacy, &plain).await);
    for digest in &crafted {
        seed_blocked(&registry, "hosted", digest).await;
    }

    for digest in &crafted {
        denied(&pull(&registry, digest).await, "GHSA-A");
    }
    denied(&pull(&registry, &legacy).await, "GHSA-A");
}

#[tokio::test]
async fn a_blocked_image_pulled_through_its_sha512_alias_is_refused() {
    use sha2::{Digest as _, Sha512};
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(&marker_layer(b"layer")).await;
    let body = manifest(&config, (&layer, LAYER), None, None);
    assert_eq!(sha256(&serde_json::to_vec(&body).unwrap()), digest);
    let alias = format!(
        "sha512:{}",
        hex::encode(Sha512::digest(serde_json::to_vec(&body).unwrap()))
    );
    registry.push_manifest(&alias, &body).await;

    denied(&pull(&registry, &alias).await, "GHSA-A");
    denied(&pull(&registry, &digest).await, "GHSA-A");
}

#[tokio::test]
async fn the_preview_of_a_blocked_images_manifest_is_refused_and_a_clean_one_is_shown() {
    let registry = Registry::new().await;
    let blocked = blocked_image(&registry).await;
    let clean = registry.image(Some("clean"), b"other").await;
    let bom = registry
        .document(&clean, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    registry.settle().await;
    let preview = |path: String| {
        let registry = &registry;
        async move {
            let uri = format!("/service/rest/internal/ui/browse/hosted/content?path={path}");
            registry
                .fixture
                .call(Method::GET, &uri, Auth::Admin, None)
                .await
        }
    };

    for path in [
        "/v2/app/manifests/1.0".to_owned(),
        format!("/v2/app/manifests/{blocked}"),
    ] {
        let reply = preview(path.clone()).await;
        assert_eq!(
            reply.status,
            StatusCode::FORBIDDEN,
            "{path}: {}",
            reply.text()
        );
        assert!(
            reply
                .text()
                .contains("blocked by policy strict: 1 blocking advisory: GHSA-A"),
            "{path}: {}",
            reply.text()
        );
    }
    for path in [
        "/v2/app/manifests/clean".to_owned(),
        format!("/v2/app/manifests/{clean}"),
        format!("/v2/app/manifests/{bom}"),
    ] {
        let reply = preview(path.clone()).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
        assert!(!reply.body.is_empty());
    }
    let config = sha256(b"{\"os\":\"linux\"}");
    let blob = preview(format!("/v2/-/blobs/{config}")).await;
    assert_eq!(blob.status, StatusCode::OK, "{}", blob.text());
}

#[tokio::test]
async fn blobs_of_a_blocked_image_are_served_and_a_push_of_it_is_not_gated() {
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    let layer = sha256(&marker_layer(b"layer"));
    let blob = registry
        .send(
            Method::GET,
            &format!("/v2/hosted/app/blobs/{layer}"),
            Vec::new(),
            "application/json",
        )
        .await;
    assert_eq!(blob.status, StatusCode::OK);
    assert_eq!(registry.image(Some("again"), b"layer").await, digest);
    denied(&pull(&registry, "again").await, "GHSA-A");
}

#[tokio::test]
async fn a_group_is_gated_by_the_member_that_holds_the_image() {
    let registry = Registry::new().await;
    blocked_image(&registry).await;
    let free = registry.repository_without_policy("free").await;
    free.image(Some("1.0"), b"layer").await;
    for (name, members) in [
        ("strictfirst", ["hosted", "free"]),
        ("freefirst", ["free", "hosted"]),
    ] {
        let made = registry
            .fixture
            .call(
                Method::POST,
                "/service/rest/v1/repositories/docker/group",
                Auth::Admin,
                Some(json!({
                    "name": name, "online": true,
                    "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                    "group": {"memberNames": members},
                    "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
                })),
            )
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    }
    denied(
        &pull_from(&registry, "strictfirst", Method::GET, "1.0").await,
        "GHSA-A",
    );
    let through = pull_from(&registry, "freefirst", Method::GET, "1.0").await;
    assert_eq!(through.status, StatusCode::OK, "{}", through.text());
}

#[derive(Clone)]
struct Remote(Arc<HashMap<String, Vec<u8>>>);

async fn remote_registry(State(remote): State<Remote>, Path(path): Path<String>) -> Response {
    match remote.0.get(&path) {
        Some(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, OCI_MANIFEST.to_owned()),
                (header::CONTENT_LENGTH, bytes.len().to_string()),
                (
                    HeaderName::from_static("docker-content-digest"),
                    sha256(bytes),
                ),
            ],
            bytes.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start_remote() -> (SocketAddr, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut pages = HashMap::new();
    let mut first = String::new();
    for tag in ["1.0", "2.0"] {
        let body = manifest(
            &sha256(b"config"),
            (&sha256(tag.as_bytes()), LAYER),
            None,
            None,
        );
        let bytes = serde_json::to_vec(&body).unwrap();
        if first.is_empty() {
            first = sha256(&bytes);
        }
        pages.insert(format!("app/manifests/{}", sha256(&bytes)), bytes.clone());
        pages.insert(format!("app/manifests/{tag}"), bytes);
    }
    let app = Router::new()
        .route("/v2/{*path}", axum::routing::any(remote_registry))
        .with_state(Remote(Arc::new(pages)));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, first)
}

#[tokio::test]
async fn a_proxy_refuses_a_cached_blocked_manifest_and_its_first_pull_follows_the_unscanned_setting()
 {
    let registry = Registry::new().await;
    astrum_db::setting::put(
        registry.fixture.state.db().conn(),
        astrum_security::ssrf::SSRF_KEY,
        &astrum_security::ssrf::SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let (address, digest) = start_remote().await;
    let made = registry
        .fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/docker/proxy",
            Auth::Admin,
            Some(json!({
                "name": "mirror", "online": true, "policy": {"name": "strict"},
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "REGISTRY"}
            })),
        )
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let queued = "SELECT c.version AS n FROM scan_queue q JOIN component c ON c.id = q.component_id \
                  WHERE c.repository = 'mirror'";

    let first = pull_from(&registry, "mirror", Method::GET, "1.0").await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert_eq!(registry.rows(queued).await, vec![strings(&["1.0"])]);

    seed_blocked(&registry, "mirror", &digest).await;
    denied(
        &pull_from(&registry, "mirror", Method::GET, "1.0").await,
        "GHSA-A",
    );
    denied(
        &pull_from(&registry, "mirror", Method::GET, &digest).await,
        "GHSA-A",
    );

    unscanned_block(&registry).await;
    registry.execute("DELETE FROM scan_queue").await;
    denied(
        &pull_from(&registry, "mirror", Method::GET, "2.0").await,
        "the image is being scanned, retry",
    );
    assert_eq!(registry.rows(queued).await, vec![strings(&["2.0"])]);
}
