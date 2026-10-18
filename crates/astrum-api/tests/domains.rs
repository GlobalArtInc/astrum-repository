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
use support::{Fixture, Reply};
use tower::ServiceExt;

const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";

struct Server {
    fixture: Fixture,
    app: Router,
    admin: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn storage() -> Value {
    json!({"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"})
}

fn docker(name: &str, domain: Value) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": storage(),
        "docker": {"forceBasicAuth": true, "v1Enabled": false},
        "domain": domain,
    })
}

fn raw(name: &str, domain: Value) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": storage(),
        "raw": {"contentDisposition": "INLINE"},
        "domain": domain,
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

impl Server {
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
            .content()
            .register(Arc::new(astrum_format_raw::Raw));
        let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes());
        let admin = format!(
            "Basic {}",
            STANDARD.encode(format!("admin:{}", fixture.admin_password))
        );
        Server {
            fixture,
            app,
            admin,
        }
    }

    async fn send(
        &self,
        method: Method,
        host: &str,
        path: &str,
        signed: bool,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Reply {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, host);
        if signed {
            request = request.header(header::AUTHORIZATION, &self.admin);
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

    async fn get(&self, host: &str, path: &str) -> Reply {
        self.send(Method::GET, host, path, true, &[], Vec::new())
            .await
    }

    async fn create(&self, kind: &str, body: Value) -> Reply {
        self.call(
            Method::POST,
            &format!("/service/rest/v1/repositories/{kind}"),
            body,
        )
        .await
    }

    async fn call(&self, method: Method, path: &str, body: Value) -> Reply {
        self.send(
            method,
            "astrum.example.org",
            path,
            true,
            &[("content-type", "application/json")],
            body.to_string().into_bytes(),
        )
        .await
    }

    async fn push(&self, host: &str, image: &str, tag: &str) {
        for (blob, _) in [(b"{}".to_vec(), EMPTY), (b"layer bytes".to_vec(), LAYER)] {
            let digest = sha256(&blob);
            let reply = self
                .send(
                    Method::POST,
                    host,
                    &format!("/v2/{image}/blobs/uploads/?digest={digest}"),
                    true,
                    &[("content-type", "application/octet-stream")],
                    blob,
                )
                .await;
            assert_eq!(
                reply.status,
                StatusCode::CREATED,
                "{host}: {}",
                reply.text()
            );
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
                host,
                &format!("/v2/{image}/manifests/{tag}"),
                true,
                &[("content-type", OCI_MANIFEST)],
                manifest.into_bytes(),
            )
            .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{host}: {}",
            reply.text()
        );
    }
}

#[tokio::test]
async fn a_root_domain_serves_one_repository_and_the_old_path_still_works() {
    let server = Server::new().await;
    let created = server
        .create(
            "docker/hosted",
            docker("docker-onprem", json!({"host": "Docker-Prem.Example.org"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created = server
        .create(
            "raw/hosted",
            raw("files", json!({"host": "files.example.org"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let stored = server
        .call(
            Method::GET,
            "/service/rest/v1/repositories/docker/hosted/docker-onprem",
            Value::Null,
        )
        .await;
    assert_eq!(
        stored.json()["domain"],
        json!({"host": "docker-prem.example.org", "routing": "ROOT"})
    );

    server
        .push("docker-prem.example.org", "team/app", "1.0")
        .await;
    let tags = json!({"name": "team/app", "tags": ["1.0"]});
    for (host, path) in [
        ("docker-prem.example.org", "/v2/team/app/tags/list"),
        ("DOCKER-PREM.example.org:443", "/v2/team/app/tags/list"),
        (
            "astrum.example.org",
            "/repository/docker-onprem/v2/team/app/tags/list",
        ),
    ] {
        let reply = server.get(host, path).await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{host}{path}: {}",
            reply.text()
        );
        assert_eq!(reply.json(), tags);
    }
    let catalog = server.get("docker-prem.example.org", "/v2/_catalog").await;
    assert_eq!(catalog.json(), json!({"repositories": ["team/app"]}));
    let main = server.get("astrum.example.org", "/v2/_catalog").await;
    assert_eq!(main.json(), json!({"repositories": []}));

    astrum_db::setting::put(
        server.fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "DockerToken"],
    )
    .await
    .unwrap();
    server.fixture.state.security().reload().await.unwrap();
    let anonymous = server
        .send(
            Method::GET,
            "docker-prem.example.org",
            "/v2/",
            false,
            &[("x-forwarded-proto", "https")],
            Vec::new(),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous.headers[header::WWW_AUTHENTICATE],
        "Bearer realm=\"https://docker-prem.example.org/v2/token\",service=\"docker-prem.example.org\""
    );
    let issued = server
        .get(
            "docker-prem.example.org",
            "/v2/token?service=docker-prem.example.org&scope=repository:team/app:pull",
        )
        .await;
    assert_eq!(issued.status, StatusCode::OK, "{}", issued.text());
    let bearer = format!("Bearer {}", issued.json()["token"].as_str().unwrap());
    let pulled = server
        .send(
            Method::GET,
            "docker-prem.example.org",
            "/v2/team/app/tags/list",
            false,
            &[("authorization", bearer.as_str())],
            Vec::new(),
        )
        .await;
    assert_eq!(pulled.status, StatusCode::OK, "{}", pulled.text());
    assert_eq!(pulled.json(), tags);

    let put = server
        .send(
            Method::PUT,
            "files.example.org",
            "/tools/1.0/install.sh",
            true,
            &[],
            b"echo hello\n".to_vec(),
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.text());
    for (host, path) in [
        ("files.example.org", "/tools/1.0/install.sh?download=1"),
        (
            "files.example.org",
            "/repository/files/tools/1.0/install.sh",
        ),
        (
            "astrum.example.org",
            "/repository/files/tools/1.0/install.sh",
        ),
    ] {
        let reply = server.get(host, path).await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{host}{path}: {}",
            reply.text()
        );
        assert_eq!(reply.body, b"echo hello\n");
    }
    let other = server
        .get(
            "files.example.org",
            "/repository/docker-onprem/v2/team/app/tags/list",
        )
        .await;
    assert_eq!(other.status, StatusCode::NOT_FOUND);
    let status = server
        .get("files.example.org", "/service/rest/v1/status")
        .await;
    assert_eq!(status.status, StatusCode::OK);
}

#[tokio::test]
async fn a_shared_domain_leads_to_each_repository_by_its_segment() {
    let server = Server::new().await;
    for (name, domain) in [
        (
            "docker-onprem",
            json!({"host": "images.example.org", "routing": "path", "path": "prod"}),
        ),
        (
            "docker-dev",
            json!({"host": "images.example.org", "routing": "PATH"}),
        ),
        ("docker-hidden", json!(null)),
    ] {
        let created = server.create("docker/hosted", docker(name, domain)).await;
        assert_eq!(
            created.status,
            StatusCode::CREATED,
            "{name}: {}",
            created.text()
        );
    }
    let created = server
        .create(
            "raw/hosted",
            raw(
                "files",
                json!({"host": "images.example.org", "routing": "PATH", "path": "files"}),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let stored = server
        .call(
            Method::GET,
            "/service/rest/v1/repositories/docker/hosted/docker-onprem",
            Value::Null,
        )
        .await;
    assert_eq!(
        stored.json()["domain"],
        json!({"host": "images.example.org", "routing": "PATH", "path": "prod"})
    );
    let hidden = server
        .call(
            Method::GET,
            "/service/rest/v1/repositories/docker/hosted/docker-hidden",
            Value::Null,
        )
        .await;
    assert!(hidden.json().get("domain").is_none(), "{}", hidden.text());

    server
        .push("images.example.org", "prod/team/app", "1.0")
        .await;
    server
        .push("images.example.org", "docker-dev/team/tool", "2.0")
        .await;
    for (host, path, name) in [
        (
            "images.example.org",
            "/v2/prod/team/app/tags/list",
            "prod/team/app",
        ),
        (
            "astrum.example.org",
            "/repository/docker-onprem/v2/team/app/tags/list",
            "team/app",
        ),
    ] {
        let reply = server.get(host, path).await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{host}{path}: {}",
            reply.text()
        );
        assert_eq!(reply.json(), json!({"name": name, "tags": ["1.0"]}));
    }
    for path in [
        "/v2/docker-onprem/team/app/tags/list",
        "/v2/docker-hidden/team/app/tags/list",
        "/v2/files/team/app/tags/list",
    ] {
        let reply = server.get("images.example.org", path).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(reply.json()["errors"][0]["code"], "NAME_UNKNOWN", "{path}");
    }
    let catalog = server.get("images.example.org", "/v2/_catalog").await;
    assert_eq!(
        catalog.json(),
        json!({"repositories": ["docker-dev/team/tool", "prod/team/app"]})
    );

    let put = server
        .send(
            Method::PUT,
            "images.example.org",
            "/files/a.txt",
            true,
            &[],
            b"shared\n".to_vec(),
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.text());
    let reply = server
        .get("astrum.example.org", "/repository/files/a.txt")
        .await;
    assert_eq!(reply.body, b"shared\n");
    for path in ["/", "/docker-hidden/a.txt", "/repository/docker-hidden/v2/"] {
        let reply = server.get("images.example.org", path).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn domains_and_segments_are_not_shared() {
    let server = Server::new().await;
    let created = server
        .create(
            "docker/hosted",
            docker("docker-onprem", json!({"host": "docker-prem.example.org"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created = server
        .create(
            "docker/hosted",
            docker(
                "docker-prod",
                json!({"host": "images.example.org", "routing": "PATH", "path": "prod"}),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    for (name, domain, expected) in [
        (
            "second",
            json!({"host": "DOCKER-PREM.example.org"}),
            "PARAMETER domainHost: the domain docker-prem.example.org is used by the repository \"docker-onprem\"; a domain is shared only when every repository on it uses path routing",
        ),
        (
            "second",
            json!({"host": "docker-prem.example.org", "routing": "PATH"}),
            "PARAMETER domainHost: the domain docker-prem.example.org is used by the repository \"docker-onprem\"; a domain is shared only when every repository on it uses path routing",
        ),
        (
            "second",
            json!({"host": "images.example.org"}),
            "PARAMETER domainHost: the domain images.example.org is used by the repository \"docker-prod\"; a domain is shared only when every repository on it uses path routing",
        ),
        (
            "second",
            json!({"host": "images.example.org", "routing": "PATH", "path": "PROD"}),
            "PARAMETER domainPath: images.example.org/PROD already leads to the repository \"docker-prod\"",
        ),
        (
            "prod",
            json!({"host": "images.example.org", "routing": "PATH"}),
            "PARAMETER domainPath: images.example.org/prod already leads to the repository \"docker-prod\"",
        ),
        (
            "second",
            json!({"host": "astrum.example.org"}),
            "PARAMETER domainHost: astrum.example.org is the address of this server, not a domain of its own",
        ),
        (
            "second",
            json!({"host": "https://images.example.org"}),
            "PARAMETER domainHost: https://images.example.org is not a domain name such as docker.example.com",
        ),
        (
            "second",
            json!({"host": "images.example.org", "routing": "PATH", "path": "v2"}),
            "PARAMETER domainPath: v2 is not a path segment: use letters, digits, '-', '_' and '.', and not v2, repository or service",
        ),
    ] {
        let refused = server
            .create("docker/hosted", docker(name, domain.clone()))
            .await;
        assert_eq!(
            refused.status,
            StatusCode::BAD_REQUEST,
            "{domain}: {}",
            refused.text()
        );
        assert_eq!(problems(&refused), [expected], "{domain}");
    }

    let shared = server
        .create(
            "docker/hosted",
            docker(
                "second",
                json!({"host": "images.example.org", "routing": "PATH"}),
            ),
        )
        .await;
    assert_eq!(shared.status, StatusCode::CREATED, "{}", shared.text());

    let kept = server
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/docker/hosted/docker-onprem",
            docker("docker-onprem", json!({"host": "docker-prem.example.org"})),
        )
        .await;
    assert_eq!(kept.status, StatusCode::NO_CONTENT, "{}", kept.text());
    let moved = server
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/docker/hosted/docker-onprem",
            docker("docker-onprem", json!(null)),
        )
        .await;
    assert_eq!(moved.status, StatusCode::NO_CONTENT, "{}", moved.text());
    let gone = server.get("docker-prem.example.org", "/v2/_catalog").await;
    assert_eq!(gone.json(), json!({"repositories": []}));
    let freed = server
        .create(
            "docker/hosted",
            docker("third", json!({"host": "docker-prem.example.org"})),
        )
        .await;
    assert_eq!(freed.status, StatusCode::CREATED, "{}", freed.text());
}

#[tokio::test]
async fn the_dns_check_reports_what_a_domain_points_to() {
    let server = Server::new().await;
    let check = |host: &str| format!("/service/rest/internal/ui/domains/check?host={host}");
    let invalid = server
        .get("astrum.example.org", &check("not%20a%20host"))
        .await;
    assert_eq!(invalid.status, StatusCode::OK, "{}", invalid.text());
    assert_eq!(invalid.json()["status"], "invalid");
    let missing = server
        .get("127.0.0.1:8081", &check("Nothing.Invalid"))
        .await;
    assert_eq!(
        missing.json(),
        json!({"host": "nothing.invalid", "status": "missing", "addresses": [], "serverAddresses": ["127.0.0.1"]})
    );
    let anonymous = server
        .send(
            Method::GET,
            "astrum.example.org",
            &check("images.example.org"),
            false,
            &[],
            Vec::new(),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}
