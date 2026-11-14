mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_content::inventory::Source;
use astrum_scan::worker;
use astrum_security::SecurityModel;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
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
const SPDX: &str = "application/spdx+json";
const PREFIX: &str = "/v2/hosted/app";
const ALPINE_OS: &[u8] = b"NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19.1\n";
const DEBIAN_OS: &[u8] = b"ID=debian\nVERSION_ID=\"12\"\n";
const APK_DB: &[u8] = b"P:musl-utils\nV:1.2.4-r2\nA:x86_64\no:musl\n\nP:busybox\nV:1.36.1-r15\n\n";
const DPKG_DB: &[u8] =
    b"Package: libc6\nStatus: install ok installed\nVersion: 2.36-9\nSource: glibc\n\n\
Package: libc-bin\nStatus: install ok installed\nVersion: 2.36-9\nSource: glibc\n\n";
const MUSL: &str = "pkg:apk/alpine/musl@1.2.4-r2?distro=alpine-3.19.1";
const DB_CAP: usize = 64 * 1024 * 1024;

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

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

fn manifest(config: &str, layers: &[String]) -> Value {
    json!({
        "schemaVersion": 2, "mediaType": OCI_MANIFEST,
        "config": {"mediaType": OCI_CONFIG, "digest": config, "size": 2},
        "layers": layers.iter().map(|digest| json!({"mediaType": LAYER, "digest": digest, "size": 1})).collect::<Vec<_>>(),
    })
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
        registry.create("hosted", repository).await;
        registry
    }

    async fn create(&self, kind: &str, body: Value) {
        let made = self
            .fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/repositories/docker/{kind}"),
                Auth::Admin,
                Some(body),
            )
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
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
        let path = format!("{PREFIX}/manifests/{reference}");
        let reply = self
            .send(Method::PUT, &path, bytes.clone(), OCI_MANIFEST)
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        sha256(&bytes)
    }

    async fn image(&self, tag: &str, layers: &[Vec<u8>]) -> String {
        let config = self.push_blob(b"{\"os\":\"linux\"}").await;
        let mut digests = Vec::new();
        for bytes in layers {
            digests.push(self.push_blob(bytes).await);
        }
        self.push_manifest(tag, &manifest(&config, &digests)).await
    }

    async fn sbom_of(&self, subject: &str, document: &str) {
        let config = self.push_blob(b"{}").await;
        let payload = self.push_blob(document.as_bytes()).await;
        let mut body = manifest(&config, &[payload]);
        body["layers"][0]["mediaType"] = SPDX.into();
        body["artifactType"] = SPDX.into();
        body["config"]["mediaType"] = "application/vnd.oci.empty.v1+json".into();
        body["subject"] = json!({"mediaType": OCI_MANIFEST, "digest": subject, "size": 2});
        let reference = sha256(&serde_json::to_vec(&body).unwrap());
        self.push_manifest(&reference, &body).await;
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
                row.column_names()
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
        let security = Arc::new(SecurityModel::default());
        let config = worker::Config::default();
        let metrics = worker::Metrics::default();
        while worker::drain(self.fixture.state.content(), &config, &metrics, &security).await > 0 {}
    }

    async fn scan_of(&self, tag: &str) -> Vec<String> {
        let sql = format!(
            "SELECT cs.state, COALESCE(i.source, '-') AS source, \
                    (SELECT count(*) FROM sca_inventory_package ip WHERE ip.inventory_id = i.id)::text AS packages \
             FROM component c JOIN component_scan cs ON cs.component_id = c.id \
             LEFT JOIN sca_inventory i ON i.id = cs.inventory_id \
             WHERE c.name = 'app' AND c.version = '{tag}'"
        );
        self.rows(&sql).await.remove(0)
    }

    async fn packages_of(&self, tag: &str) -> Vec<String> {
        let sql = format!(
            "SELECT p.ecosystem || '|' || p.name || '|' || p.version AS item \
             FROM component c JOIN component_scan cs ON cs.component_id = c.id \
             JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
             JOIN sca_package p ON p.id = ip.package_id \
             WHERE c.name = 'app' AND c.version = '{tag}' ORDER BY 1"
        );
        self.rows(&sql)
            .await
            .into_iter()
            .map(|mut row| row.remove(0))
            .collect()
    }

    async fn scanned(&self, tag: &str, source: Source, packages: usize) {
        assert_eq!(
            self.scan_of(tag).await,
            vec![
                "scanned".to_owned(),
                source.as_str().to_owned(),
                packages.to_string()
            ],
            "{tag}"
        );
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[tokio::test]
async fn an_alpine_image_gets_its_packages_from_the_apk_database_under_the_origin_name() {
    let registry = Registry::new().await;
    registry
        .image("3.19", &[alpine_base(), layer(&[("app/run", &b"x"[..])])])
        .await;
    registry.settle().await;
    registry.scanned("3.19", Source::Layers, 2).await;
    assert_eq!(
        registry.packages_of("3.19").await,
        strings(&[
            "Alpine:v3.19|busybox|1.36.1-r15",
            "Alpine:v3.19|musl|1.2.4-r2"
        ])
    );
}

#[tokio::test]
async fn a_debian_image_gets_its_packages_from_dpkg_under_the_source_package() {
    let registry = Registry::new().await;
    registry.image("12", &[debian_base()]).await;
    registry.settle().await;
    registry.scanned("12", Source::Layers, 1).await;
    assert_eq!(
        registry.packages_of("12").await,
        strings(&["Debian:12|glibc|2.36-9"])
    );
}

#[tokio::test]
async fn an_image_without_an_os_release_and_one_whose_upper_layer_deletes_the_dpkg_status_have_no_packages()
 {
    let registry = Registry::new().await;
    registry
        .image("static", &[layer(&[("app/server", &b"binary"[..])])])
        .await;
    registry
        .image(
            "wiped",
            &[
                debian_base(),
                layer(&[("var/lib/dpkg/.wh.status", &b""[..])]),
            ],
        )
        .await;
    registry.image("kept", &[debian_base()]).await;
    registry.settle().await;
    for tag in ["static", "wiped"] {
        assert_eq!(
            registry.scan_of(tag).await,
            strings(&["unsupported", "-", "0"]),
            "{tag}"
        );
    }
    registry.scanned("kept", Source::Layers, 1).await;
}

#[tokio::test]
async fn sbom_and_layer_packages_are_merged_without_duplicates_and_the_source_says_both() {
    let registry = Registry::new().await;
    let digest = registry.image("1.0", &[alpine_base()]).await;
    registry
        .sbom_of(&digest, &spdx(&[MUSL, "pkg:npm/lodash@4.17.20"]))
        .await;
    registry.settle().await;
    registry.scanned("1.0", Source::SbomAndLayers, 3).await;
    assert_eq!(
        registry.packages_of("1.0").await,
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
    registry
        .image(
            "huge",
            &[layer(&[
                ("etc/os-release", DEBIAN_OS),
                ("var/lib/dpkg/status", huge.as_slice()),
            ])],
        )
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("huge").await,
        strings(&["incomplete", Source::Layers.as_str(), "0"])
    );
}

async fn cache_rows(registry: &Registry, digest: &str) -> Vec<Vec<String>> {
    registry
        .rows(&format!(
            "SELECT digest, extracted_at::text AS at FROM oci_layer_facts WHERE digest = '{digest}'"
        ))
        .await
}

#[tokio::test]
async fn a_shared_base_layer_is_extracted_once_and_read_from_the_cache_afterwards() {
    let registry = Registry::new().await;
    let base = sha256(&debian_base());
    registry
        .image("first", &[debian_base(), layer(&[("a", &b"1"[..])])])
        .await;
    registry.settle().await;
    let before = cache_rows(&registry, &base).await;
    assert_eq!(before.len(), 1);
    registry
        .image("second", &[debian_base(), layer(&[("b", &b"2"[..])])])
        .await;
    registry.settle().await;
    assert_eq!(cache_rows(&registry, &base).await, before);
    let total = registry
        .rows("SELECT count(*)::text AS n FROM oci_layer_facts")
        .await;
    assert_eq!(total, vec![strings(&["3"])]);
    registry.scanned("second", Source::Layers, 1).await;
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
    let body = serde_json::to_vec(&manifest(&sha256(config), &[sha256(&layer)])).unwrap();
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
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let (address, remote) = start_remote(alpine_base()).await;
    registry
        .create(
            "proxy",
            json!({
                "name": "mirror", "online": true, "policy": {"name": "strict"},
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false},
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "REGISTRY"}
            }),
        )
        .await;
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
    registry.scanned("1.0", Source::Layers, 2).await;
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
