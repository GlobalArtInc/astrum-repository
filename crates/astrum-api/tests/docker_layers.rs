mod inventory_common;
mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_content::inventory::Source;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use inventory_common::*;
use serde_json::{Value, json};

const ALPINE_OS: &[u8] = b"NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19.1\n";
const DEBIAN_OS: &[u8] = b"ID=debian\nVERSION_ID=\"12\"\n";
const APK_DB: &[u8] = b"P:musl-utils\nV:1.2.4-r2\nA:x86_64\no:musl\n\nP:busybox\nV:1.36.1-r15\n\n";
const DPKG_DB: &[u8] =
    b"Package: libc6\nStatus: install ok installed\nVersion: 2.36-9\nSource: glibc\n\n\
Package: libc-bin\nStatus: install ok installed\nVersion: 2.36-9\nSource: glibc\n\n";
const DB_CAP: usize = 64 * 1024 * 1024;

fn layer(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        builder.append_data(&mut header, path, *data).unwrap();
    }
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&builder.into_inner().unwrap()).unwrap();
    encoder.finish().unwrap()
}

fn alpine_base() -> Vec<u8> {
    layer(&[
        ("etc/os-release", ALPINE_OS),
        ("lib/apk/db/installed", APK_DB),
    ])
}

fn debian_base() -> Vec<u8> {
    layer(&[
        ("etc/os-release", DEBIAN_OS),
        ("var/lib/dpkg/status", DPKG_DB),
    ])
}

fn layered(config: &str, layers: &[String]) -> Value {
    json!({
        "schemaVersion": 2, "mediaType": OCI_MANIFEST,
        "config": {"mediaType": OCI_CONFIG, "digest": config, "size": 2},
        "layers": layers.iter().map(|digest| json!({"mediaType": LAYER, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    })
}

async fn image(registry: &Registry, tag: &str, layers: &[Vec<u8>]) -> String {
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let mut digests = Vec::new();
    for bytes in layers {
        digests.push(registry.push_blob(bytes).await);
    }
    registry
        .push_manifest(tag, &layered(&config, &digests))
        .await
}

async fn packages_of(registry: &Registry, tag: &str) -> Vec<String> {
    let sql = format!(
        "SELECT p.ecosystem || '|' || p.name || '|' || p.version AS item \
         FROM component c JOIN component_scan cs ON cs.component_id = c.id \
         JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
         JOIN sca_package p ON p.id = ip.package_id \
         WHERE c.name = 'app' AND c.version = '{tag}' AND c.repository = 'hosted' ORDER BY 1"
    );
    registry
        .rows(&sql)
        .await
        .into_iter()
        .map(|mut row| row.remove(0))
        .collect()
}

async fn expect(registry: &Registry, tag: &str, state: &str, source: Option<Source>, count: usize) {
    let scan = registry.scan_of(tag).await;
    let source = source.map_or("-", Source::as_str);
    assert_eq!(
        (scan[0].as_str(), scan[3].as_str(), scan[4].as_str()),
        (state, source, count.to_string().as_str()),
        "{tag}: {scan:?}"
    );
}

#[tokio::test]
async fn an_alpine_image_gets_its_packages_from_the_apk_database_under_the_origin_name() {
    let registry = Registry::new().await;
    image(
        &registry,
        "3.19",
        &[alpine_base(), layer(&[("app/run", &b"x"[..])])],
    )
    .await;
    registry.settle().await;
    expect(&registry, "3.19", "scanned", Some(Source::Layers), 2).await;
    assert_eq!(
        packages_of(&registry, "3.19").await,
        strings(&[
            "Alpine:v3.19|busybox|1.36.1-r15",
            "Alpine:v3.19|musl|1.2.4-r2"
        ])
    );
}

#[tokio::test]
async fn a_debian_image_gets_its_packages_from_dpkg_under_the_source_package() {
    let registry = Registry::new().await;
    image(&registry, "12", &[debian_base()]).await;
    registry.settle().await;
    expect(&registry, "12", "scanned", Some(Source::Layers), 1).await;
    assert_eq!(
        packages_of(&registry, "12").await,
        strings(&["Debian:12|glibc|2.36-9"])
    );
}

async fn child(registry: &Registry, layers: &[Vec<u8>]) -> String {
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let mut digests = Vec::new();
    for bytes in layers {
        digests.push(registry.push_blob(bytes).await);
    }
    let body = layered(&config, &digests);
    let reference = sha256(&serde_json::to_vec(&body).unwrap());
    registry.push_manifest(&reference, &body).await
}

#[tokio::test]
async fn a_multi_arch_image_without_sboms_is_judged_on_the_layers_of_its_children() {
    let registry = Registry::new().await;
    let alpine = child(&registry, &[alpine_base()]).await;
    let debian = child(&registry, &[debian_base()]).await;
    let top = registry
        .push_manifest("multi", &index(&[(&alpine, None), (&debian, None)]))
        .await;
    registry.settle().await;

    let scan = registry.scan_of("multi").await;
    assert_eq!(
        (
            scan[0].as_str(),
            scan[1].as_str(),
            scan[3].as_str(),
            scan[4].as_str()
        ),
        ("scanned", top.as_str(), Source::Index.as_str(), "3")
    );
    assert_eq!(
        registry.verdicts().await.last().unwrap(),
        &strings(&["oci", "block", "true", "2"])
    );
    let refused = registry
        .send(
            Method::GET,
            "/v2/hosted/app/manifests/multi",
            Vec::new(),
            "application/json",
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    assert!(
        refused
            .text()
            .contains("2 blocking advisories: GHSA-A, GHSA-D"),
        "{}",
        refused.text()
    );
}

#[tokio::test]
async fn an_image_without_an_os_release_and_one_whose_upper_layer_deletes_the_dpkg_status_have_no_packages()
 {
    let registry = Registry::new().await;
    image(
        &registry,
        "static",
        &[layer(&[("app/server", &b"binary"[..])])],
    )
    .await;
    let wipe = layer(&[("var/lib/dpkg/.wh.status", &b""[..])]);
    image(&registry, "wiped", &[debian_base(), wipe]).await;
    image(&registry, "kept", &[debian_base()]).await;
    registry.settle().await;
    expect(&registry, "static", "unsupported", None, 0).await;
    expect(&registry, "wiped", "unsupported", None, 0).await;
    expect(&registry, "kept", "scanned", Some(Source::Layers), 1).await;
}

#[tokio::test]
async fn sbom_and_layer_packages_are_merged_without_duplicates_and_the_source_says_both() {
    let registry = Registry::new().await;
    let digest = image(&registry, "1.0", &[alpine_base()]).await;
    registry
        .document(&digest, None, SPDX, &spdx(&[MUSL, LODASH]))
        .await;
    registry.settle().await;
    expect(&registry, "1.0", "scanned", Some(Source::SbomAndLayers), 3).await;
    assert_eq!(
        packages_of(&registry, "1.0").await,
        strings(&[
            "Alpine:v3.19|busybox|1.36.1-r15",
            "Alpine:v3.19|musl|1.2.4-r2",
            "npm|lodash|4.17.20"
        ])
    );
    let purls = registry
        .rows("SELECT purl FROM sca_package WHERE name = 'musl'")
        .await;
    assert_eq!(purls, vec![strings(&[MUSL])]);
}

#[tokio::test]
async fn a_package_database_over_the_size_cap_makes_the_inventory_incomplete() {
    let registry = Registry::new().await;
    let huge = vec![b'a'; DB_CAP + 1];
    let files = [
        ("etc/os-release", DEBIAN_OS),
        ("var/lib/dpkg/status", huge.as_slice()),
    ];
    image(&registry, "huge", &[layer(&files)]).await;
    registry.settle().await;
    expect(&registry, "huge", "incomplete", Some(Source::Layers), 0).await;
}

async fn cache_rows(registry: &Registry, digest: &str) -> Vec<Vec<String>> {
    registry
        .rows(&format!(
            "SELECT extractor::text AS extractor, extracted_at::text AS at \
             FROM oci_layer_facts WHERE digest = '{digest}'"
        ))
        .await
}

#[tokio::test]
async fn a_shared_base_layer_is_extracted_once_and_read_from_the_cache_afterwards() {
    let registry = Registry::new().await;
    let base = sha256(&debian_base());
    image(
        &registry,
        "first",
        &[debian_base(), layer(&[("a", &b"1"[..])])],
    )
    .await;
    registry.settle().await;
    let before = cache_rows(&registry, &base).await;
    assert_eq!(before.len(), 1);
    image(
        &registry,
        "second",
        &[debian_base(), layer(&[("b", &b"2"[..])])],
    )
    .await;
    registry.settle().await;
    assert_eq!(cache_rows(&registry, &base).await, before);
    let total = registry
        .rows("SELECT count(*)::text AS n FROM oci_layer_facts")
        .await;
    assert_eq!(total, vec![strings(&["3"])]);
    expect(&registry, "second", "scanned", Some(Source::Layers), 1).await;
}

#[tokio::test]
async fn facts_cached_by_an_older_extractor_are_extracted_again() {
    let registry = Registry::new().await;
    let base = sha256(&debian_base());
    image(&registry, "first", &[debian_base()]).await;
    registry.settle().await;
    let current = cache_rows(&registry, &base).await[0][0].clone();
    registry
        .execute(&format!(
            "UPDATE oci_layer_facts SET extractor = 0, facts = '{{}}'::jsonb, \
             extracted_at = now() - interval '1 day' WHERE digest = '{base}'"
        ))
        .await;
    image(
        &registry,
        "second",
        &[debian_base(), layer(&[("b", &b"2"[..])])],
    )
    .await;
    registry.settle().await;
    expect(&registry, "second", "scanned", Some(Source::Layers), 1).await;
    let after = cache_rows(&registry, &base).await;
    assert_eq!(after[0][0], current);
}

#[derive(Clone)]
struct Remote {
    manifest: Vec<u8>,
    layer: Vec<u8>,
    layer_hits: Arc<AtomicUsize>,
}

async fn remote_registry(State(remote): State<Remote>, Path(path): Path<String>) -> Response {
    let manifest_digest = sha256(&remote.manifest);
    let layer_digest = sha256(&remote.layer);
    let found = if path == "app/manifests/1.0" || path == format!("app/manifests/{manifest_digest}")
    {
        Some((OCI_MANIFEST, remote.manifest.clone(), manifest_digest))
    } else if path == format!("app/blobs/{layer_digest}") {
        remote.layer_hits.fetch_add(1, Ordering::SeqCst);
        Some((
            "application/octet-stream",
            remote.layer.clone(),
            layer_digest,
        ))
    } else {
        None
    };
    match found {
        Some((content_type, bytes, digest)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, content_type.to_owned()),
                (header::CONTENT_LENGTH, bytes.len().to_string()),
                (
                    header::HeaderName::from_static("docker-content-digest"),
                    digest,
                ),
            ],
            bytes,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start_remote(layer: Vec<u8>) -> (SocketAddr, Remote) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = b"{\"os\":\"linux\"}";
    let body = serde_json::to_vec(&layered(&sha256(config), &[sha256(&layer)])).unwrap();
    let remote = Remote {
        manifest: body,
        layer,
        layer_hits: Arc::default(),
    };
    let app = Router::new()
        .route("/v2/{*path}", axum::routing::any(remote_registry))
        .with_state(remote.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, remote)
}

#[tokio::test]
async fn a_proxy_fetches_an_uncached_layer_through_the_proxy_path_and_keeps_it_for_pulls() {
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
    let (address, remote) = start_remote(alpine_base()).await;
    let body = json!({
        "name": "mirror", "online": true, "policy": {"name": "strict"},
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": false, "timeToLive": 1},
        "httpClient": {"blocked": false, "autoBlock": false},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
        "dockerProxy": {"indexType": "REGISTRY"}
    });
    let made = registry
        .fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/docker/proxy",
            support::Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let pulled = registry
        .send(
            Method::GET,
            "/v2/mirror/app/manifests/1.0",
            Vec::new(),
            "application/json",
        )
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    assert_eq!(remote.layer_hits.load(Ordering::SeqCst), 0);
    registry.settle().await;
    let scan = registry
        .rows(
            "SELECT cs.state, i.source, \
                    (SELECT count(*) FROM sca_inventory_package ip WHERE ip.inventory_id = i.id)::text AS packages \
             FROM component c JOIN component_scan cs ON cs.component_id = c.id \
             JOIN sca_inventory i ON i.id = cs.inventory_id \
             WHERE c.repository = 'mirror' AND c.name = 'app' AND c.version = '1.0'",
        )
        .await;
    assert_eq!(
        scan,
        vec![strings(&["scanned", Source::Layers.as_str(), "2"])]
    );
    assert_eq!(remote.layer_hits.load(Ordering::SeqCst), 1);
    let blob = registry
        .send(
            Method::GET,
            &format!("/v2/mirror/app/blobs/{}", sha256(&remote.layer)),
            Vec::new(),
            "application/json",
        )
        .await;
    assert_eq!(blob.status, StatusCode::OK);
    assert_eq!(blob.body, remote.layer);
    assert_eq!(remote.layer_hits.load(Ordering::SeqCst), 1);
}
