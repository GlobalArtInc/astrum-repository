mod support;

use std::sync::Arc;

use astrum_format_docker::Connector;
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
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";

struct Registry {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
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
        connector: Option<u16>,
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
        let app = match connector {
            Some(port) => self.app.clone().layer(axum::Extension(Connector(port))),
            None => self.app.clone(),
        };
        let response = app
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

    async fn get(&self, path: &str, connector: Option<u16>, host: Option<&str>) -> Reply {
        let headers: Vec<(&str, &str)> = host.map(|host| ("host", host)).into_iter().collect();
        self.send(Method::GET, path, connector, &headers, Vec::new())
            .await
    }

    async fn create(&self, kind: &str, body: Value) -> Reply {
        self.fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/repositories/{kind}"),
                Auth::Admin,
                Some(body),
            )
            .await
    }

    async fn push(&self, connector: u16, image: &str, tag: &str) {
        for (blob, _) in [(b"{}".to_vec(), EMPTY), (b"layer bytes".to_vec(), LAYER)] {
            let digest = sha256(&blob);
            let reply = self
                .send(
                    Method::POST,
                    &format!("/v2/{image}/blobs/uploads/?digest={digest}"),
                    Some(connector),
                    &[("content-type", "application/octet-stream")],
                    blob,
                )
                .await;
            assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
            assert_eq!(
                reply.headers[header::LOCATION],
                format!("/v2/{image}/blobs/{digest}")
            );
        }
        let manifest = json!({
            "schemaVersion": 2,
            "mediaType": OCI_MANIFEST,
            "config": {"mediaType": EMPTY, "digest": sha256(b"{}"), "size": 2},
            "layers": [{"mediaType": LAYER, "digest": sha256(b"layer bytes"), "size": 11}],
        })
        .to_string();
        let reply = self
            .send(
                Method::PUT,
                &format!("/v2/{image}/manifests/{tag}"),
                Some(connector),
                &[("content-type", OCI_MANIFEST)],
                manifest.clone().into_bytes(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        assert_eq!(
            reply.headers[header::LOCATION],
            format!("/v2/{image}/manifests/{}", sha256(manifest.as_bytes()))
        );
    }
}

fn docker(name: &str, docker: Value) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "docker": docker,
    })
}

fn problems(reply: &Reply) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap_or_else(|| panic!("{}", reply.text()))
        .iter()
        .map(|error| {
            format!(
                "{}: {}",
                error["id"].as_str().unwrap(),
                error["message"].as_str().unwrap()
            )
        })
        .collect()
}

#[tokio::test]
async fn a_connector_port_and_a_subdomain_reach_one_repository() {
    let registry = Registry::new().await;
    let created = registry
        .create(
            "docker/hosted",
            docker("ported", json!({"forceBasicAuth": true, "v1Enabled": false, "httpPort": 18511, "subdomain": "Images"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created = registry
        .create(
            "docker/hosted",
            docker(
                "pathed",
                json!({"forceBasicAuth": true, "v1Enabled": false, "pathEnabled": true}),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    registry.push(18511, "team/app", "1.0").await;

    let tags = json!({"name": "team/app", "tags": ["1.0"]});
    for (connector, host) in [
        (Some(18511), None),
        (None, Some("images.example.org")),
        (None, Some("IMAGES.example.org:8443")),
    ] {
        let reply = registry
            .get("/v2/team/app/tags/list", connector, host)
            .await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{connector:?} {host:?}: {}",
            reply.text()
        );
        assert_eq!(reply.json(), tags);
        let catalog = registry.get("/v2/_catalog", connector, host).await;
        assert_eq!(catalog.json(), json!({"repositories": ["team/app"]}));
        let ping = registry.get("/v2/", connector, host).await;
        assert_eq!(ping.status, StatusCode::OK);
    }
    assert_eq!(
        registry
            .get("/repository/ported/v2/team/app/tags/list", None, None)
            .await
            .json(),
        tags
    );

    let by_path = registry
        .get("/v2/ported/team/app/tags/list", None, None)
        .await;
    assert_eq!(by_path.status, StatusCode::NOT_FOUND);
    assert_eq!(by_path.json()["errors"][0]["code"], "NAME_UNKNOWN");
    for host in ["127.0.0.1:8081", "other.example.org", "localhost"] {
        let reply = registry
            .get("/v2/team/app/tags/list", None, Some(host))
            .await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{host}");
    }
    let main = registry.get("/v2/_catalog", None, None).await;
    assert_eq!(main.json(), json!({"repositories": []}));

    let unused = registry
        .get("/v2/team/app/tags/list", Some(18599), None)
        .await;
    assert_eq!(unused.status, StatusCode::NOT_FOUND);
    assert_eq!(unused.json()["errors"][0]["code"], "NAME_UNKNOWN");

    let anonymous = registry
        .app
        .clone()
        .layer(axum::Extension(Connector(18511)))
        .oneshot(
            Request::get("/v2/team/app/tags/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers().contains_key(header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn the_port_suggestion_skips_ports_in_use() {
    let registry = Registry::new().await;
    let suggest = "/service/rest/internal/ui/docker/suggest-port";
    assert_eq!(registry.get(suggest, None, None).await.json(), json!(8082));
    for (name, ports) in [
        ("first", json!({"httpPort": 8082})),
        ("second", json!({"httpPort": 8084, "httpsPort": 8083})),
    ] {
        let mut settings = json!({"forceBasicAuth": true, "v1Enabled": false});
        settings
            .as_object_mut()
            .unwrap()
            .extend(ports.as_object().unwrap().clone());
        let created = registry
            .create("docker/hosted", docker(name, settings))
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    }
    assert_eq!(registry.get(suggest, None, None).await.json(), json!(8085));
    registry.fixture.state.set_application_port(8085);
    assert_eq!(registry.get(suggest, None, None).await.json(), json!(8086));
    assert_eq!(
        registry
            .get(&format!("{suggest}?exclude=8086&exclude=8087"), None, None)
            .await
            .json(),
        json!(8088)
    );
    let anonymous = registry
        .app
        .clone()
        .oneshot(Request::get(suggest).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ports_subdomains_and_registry_hosts_are_not_shared() {
    let registry = Registry::new().await;
    registry.fixture.state.set_application_port(8081);
    let refused = registry
        .create(
            "docker/hosted",
            docker("app-port", json!({"httpPort": 8081})),
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        problems(&refused),
        ["PARAMETER httpPort: Port 8081 is already used by the application"]
    );
    let created = registry
        .create(
            "docker/hosted",
            docker("first", json!({"subdomain": "Images"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let refused = registry
        .create("docker/proxy", {
            let mut body = docker("second", json!({"subdomain": "images"}));
            body["proxy"] = json!({"remoteUrl": "https://registry.example.org", "contentMaxAge": 1440, "metadataMaxAge": 1440});
            body["negativeCache"] = json!({"enabled": true, "timeToLive": 1440});
            body["httpClient"] = json!({"blocked": false, "autoBlock": true});
            body["dockerProxy"] = json!({"indexType": "REGISTRY"});
            body
        })
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        problems(&refused),
        ["PARAMETER subdomain: the subdomain images is used by the repository \"first\""]
    );
    let update = registry
        .fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/docker/hosted/first",
            Auth::Admin,
            Some(docker("first", json!({"subdomain": "images"}))),
        )
        .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT, "{}", update.text());

    let terraform = |name: &str, host: &str| {
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            "terraform": {"registryHost": host},
        })
    };
    let created = registry
        .create("terraform/hosted", terraform("tf-one", "tf.example.org"))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let refused = registry
        .create("terraform/hosted", terraform("tf-two", "TF.example.org"))
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        problems(&refused),
        [
            "PARAMETER registryHost: the registry host TF.example.org is used by the repository \"tf-one\""
        ]
    );
}
