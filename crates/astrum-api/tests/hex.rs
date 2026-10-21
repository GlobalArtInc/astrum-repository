mod support;

use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_hex::keys::{self, RegistryKey};
use astrum_format_hex::proto::{Dependency, RetirementReason};
use astrum_format_hex::registry;
use astrum_format_hex::testing::{self, Spec};
use astrum_format_hex::{Hex, etf};
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const ERLANG: &str = "application/vnd.hex+erlang";
const BOUNDARY: &str = "hexboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture.state.content().register(Arc::new(Hex::default()));
    fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_hex::task::RebuildMetadata::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) -> Reply {
    fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/hex/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn created(fixture: &Fixture, kind: &str, body: Value) {
    let reply = create(fixture, kind, body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
    })
}

fn proxy(name: &str, remote: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": false},
    })
}

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
    })
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn token(fixture: &Fixture) -> String {
    fixture
        .state
        .security()
        .create_token("admin", "hex", Scope::Write, None)
        .await
        .unwrap()
        .1
}

async fn api(
    fixture: &Fixture,
    method: Method,
    path: &str,
    key: Option<&str>,
    body: Vec<u8>,
    content_type: &str,
) -> Reply {
    let mut headers = vec![
        ("accept", ERLANG),
        ("content-type", content_type),
        (
            "user-agent",
            "hex_core/0.10.3 (mix/2.1.1) (OTP/27) (erts/15.1)",
        ),
    ];
    if let Some(key) = key {
        headers.push(("authorization", key));
    }
    fixture
        .send_body(method, path, Auth::None, Body::from(body), &headers)
        .await
}

async fn publish(
    fixture: &Fixture,
    repository: &str,
    key: Option<&str>,
    tarball: &[u8],
    replace: bool,
) -> Reply {
    api(
        fixture,
        Method::POST,
        &format!("/repository/{repository}/api/publish?replace={replace}"),
        key,
        tarball.to_vec(),
        "application/octet-stream",
    )
    .await
}

fn term(reply: &Reply) -> Value {
    assert_eq!(reply.headers["content-type"], ERLANG, "{}", reply.text());
    etf::decode(&reply.body).unwrap()
}

async fn public_key(fixture: &Fixture, repository: &str) -> String {
    let reply = get(fixture, &format!("/repository/{repository}/public_key")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.text()
}

fn dependency<'a>(dependencies: &'a [Dependency], name: &str) -> &'a Dependency {
    dependencies
        .iter()
        .find(|dependency| dependency.package == name)
        .unwrap()
}

fn upload_form(file: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"hex.asset\"; filename=\"package.tar\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

#[tokio::test]
async fn hosted_packages_publish_through_the_hex_api_and_get_a_signed_registry() {
    let fixture = ready().await;
    created(&fixture, "hosted", hosted("hex-local", "ALLOW")).await;
    let key = token(&fixture).await;
    let first = Spec::new("hello", "1.0.0")
        .requirement("jason", "~> 1.4", false)
        .local("tools", "~> 0.1", "hex-local")
        .build();
    let reply = publish(&fixture, "hex-local", Some(&key), &first.bytes, false).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let body = term(&reply);
    assert_eq!(body["version"], "1.0.0");
    assert_eq!(body["checksum"], hex::encode(first.outer_checksum));
    assert!(
        body["url"]
            .as_str()
            .unwrap()
            .ends_with("/repository/hex-local/api/packages/hello/releases/1.0.0"),
        "{body}"
    );
    assert!(body["html_url"].as_str().unwrap().contains("hex-local"));

    let again = publish(&fixture, "hex-local", Some(&key), &first.bytes, false).await;
    assert_eq!(again.status, StatusCode::UNPROCESSABLE_ENTITY);
    let refused = term(&again);
    assert_eq!(refused["status"], 422);
    assert!(
        refused["message"].as_str().unwrap().contains("--replace"),
        "{refused}"
    );
    let replaced = publish(&fixture, "hex-local", Some(&key), &first.bytes, true).await;
    assert_eq!(replaced.status, StatusCode::OK, "{}", replaced.text());

    let second = Spec::new("hello", "1.1.0")
        .requirement("jason", "~> 1.4", false)
        .optional("plug", "~> 1.15")
        .build();
    let reply = publish(&fixture, "hex-local", Some(&key), &second.bytes, false).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let tools = Spec::new("tools", "0.1.0").build();
    let reply = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/packages/tools/releases",
        Some(&key),
        tools.bytes.clone(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let organisation = Spec::new("extra", "2.0.0").build();
    let reply = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/repos/acme/publish?replace=false",
        Some(&key),
        organisation.bytes.clone(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let mismatch = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/packages/other/releases",
        Some(&key),
        tools.bytes.clone(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(mismatch.status, StatusCode::UNPROCESSABLE_ENTITY);

    let broken = publish(&fixture, "hex-local", Some(&key), b"not a tarball", false).await;
    assert_eq!(broken.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        term(&broken)["message"]
            .as_str()
            .unwrap()
            .contains("refused")
    );
    let mut wrong = Spec::new("hello", "1.2.0");
    wrong.checksum = Some("AB".repeat(32));
    let wrong = publish(
        &fixture,
        "hex-local",
        Some(&key),
        &wrong.build().bytes,
        false,
    )
    .await;
    assert_eq!(wrong.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        term(&wrong)["message"]
            .as_str()
            .unwrap()
            .contains("CHECKSUM")
    );

    let anonymous = publish(&fixture, "hex-local", None, &first.bytes, true).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    let message = term(&anonymous)["message"].as_str().unwrap().to_owned();
    assert!(message.contains("HEX_API_KEY"), "{message}");
    let unknown = publish(
        &fixture,
        "hex-local",
        Some("astrum_0000000000000000000000000000000000000000"),
        &first.bytes,
        true,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    fixture.add_user("visitor", "visitor-password", &[]).await;
    let visitor = fixture
        .send_body(
            Method::POST,
            "/repository/hex-local/api/publish",
            Auth::Basic("visitor", "visitor-password"),
            Body::from(first.bytes.clone()),
            &[("accept", "application/json")],
        )
        .await;
    assert_eq!(visitor.status, StatusCode::FORBIDDEN);
    assert_eq!(visitor.headers["content-type"], "application/json");
    assert_eq!(visitor.json()["status"], 403);

    let public = public_key(&fixture, "hex-local").await;
    assert!(public.starts_with("-----BEGIN PUBLIC KEY-----"), "{public}");
    let names_file = get(&fixture, "/repository/hex-local/names").await;
    assert_eq!(names_file.status, StatusCode::OK);
    assert_eq!(&names_file.body[..2], &[0x1f, 0x8b]);
    let names = testing::names(&names_file.body, &public).unwrap();
    assert_eq!(names.repository, "hex-local");
    let listed: Vec<&str> = names
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(listed, ["extra", "hello", "tools"]);
    assert!(names.packages[1].updated_at.is_some());
    let versions = testing::versions(
        &get(&fixture, "/repository/hex-local/versions").await.body,
        &public,
    )
    .unwrap();
    assert_eq!(versions.repository, "hex-local");
    assert_eq!(versions.packages[1].versions, ["1.0.0", "1.1.0"]);
    assert!(versions.packages[1].retired.is_empty());
    let package_file = get(&fixture, "/repository/hex-local/packages/hello").await;
    assert_eq!(package_file.status, StatusCode::OK);
    let package = testing::package(&package_file.body, &public).unwrap();
    assert_eq!(
        (package.name.as_str(), package.repository.as_str()),
        ("hello", "hex-local")
    );
    assert_eq!(package.releases.len(), 2);
    let release = &package.releases[0];
    assert_eq!(release.version, "1.0.0");
    assert_eq!(release.inner_checksum, first.inner_checksum);
    assert_eq!(
        release.outer_checksum.as_deref(),
        Some(&first.outer_checksum[..])
    );
    let jason = dependency(&release.dependencies, "jason");
    assert_eq!(jason.requirement, "~> 1.4");
    assert_eq!(jason.repository.as_deref(), Some("hexpm"));
    assert_eq!(jason.optional, None);
    assert_eq!(dependency(&release.dependencies, "tools").repository, None);
    let plug = dependency(&package.releases[1].dependencies, "plug");
    assert_eq!(plug.optional, Some(true));
    assert!(testing::package(&package_file.body, &testing_key_public()).is_none());

    let etag = package_file.headers["etag"].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/hex-local/packages/hello",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/hex-local/names",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
    let tarball = get(&fixture, "/repository/hex-local/tarballs/hello-1.0.0.tar").await;
    assert_eq!(tarball.status, StatusCode::OK);
    assert_eq!(tarball.body, first.bytes);
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&tarball.body)),
        first.outer_checksum
    );
    assert_eq!(
        get(&fixture, "/repository/hex-local/packages/missing")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/hex-local/tarballs/hello-9.9.9.tar")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let page = get(&fixture, "/repository/hex-local/").await;
    let fingerprint = keys::fingerprint(&keys::public_key_of(&public).unwrap());
    assert!(
        page.text().contains("mix hex.repo add hex-local "),
        "{}",
        page.text()
    );
    assert!(page.text().contains(&fingerprint), "{}", page.text());
    assert!(page.text().contains("HEX_API_URL="));

    for (path, auth, wanted) in [
        (
            "/repository/hex-local/names",
            Auth::None,
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/repository/hex-local/packages/hello",
            Auth::Basic("visitor", "visitor-password"),
            StatusCode::FORBIDDEN,
        ),
        (
            "/repository/hex-local/tarballs/hello-1.0.0.tar",
            Auth::None,
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/repository/hex-local/public_key",
            Auth::Basic("visitor", "visitor-password"),
            StatusCode::FORBIDDEN,
        ),
    ] {
        assert_eq!(
            fixture.call(Method::GET, path, auth, None).await.status,
            wanted,
            "{path}"
        );
    }
    let hidden = fixture
        .call(Method::GET, "/repository/hex-local/", Auth::None, None)
        .await;
    assert_eq!(hidden.status, StatusCode::OK);
    assert!(!hidden.text().contains(&fingerprint));

    let retirement = etf::encode(&json!({"reason": "security", "message": "use 1.1.0"}));
    let retired = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/packages/hello/releases/1.0.0/retire",
        Some(&key),
        retirement,
        ERLANG,
    )
    .await;
    assert_eq!(retired.status, StatusCode::NO_CONTENT, "{}", retired.text());
    let versions = testing::versions(
        &get(&fixture, "/repository/hex-local/versions").await.body,
        &public,
    )
    .unwrap();
    assert_eq!(versions.packages[1].retired, [0]);
    let package = testing::package(
        &get(&fixture, "/repository/hex-local/packages/hello")
            .await
            .body,
        &public,
    )
    .unwrap();
    let status = package.releases[0].retired.clone().unwrap();
    assert_eq!(status.reason, RetirementReason::RetiredSecurity as i32);
    assert_eq!(status.message.as_deref(), Some("use 1.1.0"));
    let bad_reason = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/packages/hello/releases/1.0.0/retire",
        Some(&key),
        etf::encode(&json!({"reason": "boring"})),
        ERLANG,
    )
    .await;
    assert_eq!(bad_reason.status, StatusCode::UNPROCESSABLE_ENTITY);
    let information = fixture
        .send(
            Method::GET,
            "/repository/hex-local/api/packages/hello",
            Auth::Admin,
            None,
            &[("accept", "application/json")],
        )
        .await;
    assert_eq!(information.status, StatusCode::OK, "{}", information.text());
    let information = information.json();
    assert_eq!(information["name"], "hello");
    assert_eq!(information["releases"][0]["version"], "1.1.0");
    assert_eq!(information["meta"]["licenses"], json!(["Apache-2.0"]));
    assert_eq!(information["retirements"]["1.0.0"]["reason"], "security");
    assert_eq!(information["latest_stable_version"], "1.1.0");
    let unretired = api(
        &fixture,
        Method::DELETE,
        "/repository/hex-local/api/packages/hello/releases/1.0.0/retire",
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(unretired.status, StatusCode::NO_CONTENT);
    let package = testing::package(
        &get(&fixture, "/repository/hex-local/packages/hello")
            .await
            .body,
        &public,
    )
    .unwrap();
    assert!(package.releases[0].retired.is_none());
    let release = api(
        &fixture,
        Method::GET,
        "/repository/hex-local/api/packages/hello/releases/1.1.0",
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    let release = term(&release);
    assert_eq!(release["requirements"]["plug"]["optional"], true);
    assert_eq!(release["checksum"], hex::encode(second.outer_checksum));
    assert_eq!(release["meta"]["build_tools"], json!(["mix"]));
    let me = term(
        &api(
            &fixture,
            Method::GET,
            "/repository/hex-local/api/users/me",
            Some(&key),
            Vec::new(),
            ERLANG,
        )
        .await,
    );
    assert_eq!(me["username"], "admin");
    let docs_path = "/repository/hex-local/api/packages/hello/releases/1.1.0/docs";
    let plain = api(
        &fixture,
        Method::POST,
        docs_path,
        Some(&key),
        b"docs".to_vec(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(plain.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(term(&plain)["message"].as_str().unwrap().contains("gzip"));
    let documentation = astrum_format_hex::proto::gzip(b"index.html");
    let docs = api(
        &fixture,
        Method::POST,
        docs_path,
        Some(&key),
        documentation.clone(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(docs.status, StatusCode::CREATED, "{}", docs.text());
    let orphan = api(
        &fixture,
        Method::POST,
        "/repository/hex-local/api/packages/hello/releases/7.0.0/docs",
        Some(&key),
        documentation.clone(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(orphan.status, StatusCode::NOT_FOUND);
    let served = get(&fixture, "/repository/hex-local/docs/hello-1.1.0.tar.gz").await;
    assert_eq!(served.status, StatusCode::OK);
    assert_eq!(served.body, documentation);
    let redirect = api(
        &fixture,
        Method::GET,
        docs_path,
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(redirect.status, StatusCode::FOUND);
    assert!(
        redirect.headers["location"]
            .to_str()
            .unwrap()
            .ends_with("/repository/hex-local/docs/hello-1.1.0.tar.gz")
    );
    let release = term(
        &api(
            &fixture,
            Method::GET,
            "/repository/hex-local/api/packages/hello/releases/1.1.0",
            Some(&key),
            Vec::new(),
            ERLANG,
        )
        .await,
    );
    assert_eq!(release["has_docs"], true);
    assert_eq!(release["configs"]["erlang.mk"], "dep_hello = hex 1.1.0");
    assert_eq!(
        release["configs"]["mix.exs"],
        "{:hello, \"~> 1.1\", repo: \"hex-local\"}"
    );
    let owners = term(
        &api(
            &fixture,
            Method::GET,
            "/repository/hex-local/api/packages/hello/owners",
            Some(&key),
            Vec::new(),
            ERLANG,
        )
        .await,
    );
    assert_eq!(owners, json!([]));
    for (method, wanted) in [
        (Method::PUT, StatusCode::NO_CONTENT),
        (Method::DELETE, StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let reply = api(
            &fixture,
            method,
            "/repository/hex-local/api/packages/hello/owners/jane",
            Some(&key),
            Vec::new(),
            ERLANG,
        )
        .await;
        assert_eq!(reply.status, wanted);
    }
    let auth = api(
        &fixture,
        Method::GET,
        "/repository/hex-local/api/auth",
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(auth.status, StatusCode::NO_CONTENT);
    let auth = api(
        &fixture,
        Method::GET,
        "/repository/hex-local/api/auth",
        None,
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(auth.status, StatusCode::UNAUTHORIZED);
    let index = fixture
        .send(
            Method::GET,
            "/repository/hex-local/api/",
            Auth::Admin,
            None,
            &[("accept", "application/json")],
        )
        .await;
    assert!(
        index.json()["package_url"]
            .as_str()
            .unwrap()
            .ends_with("/repository/hex-local/api/packages/{name}")
    );

    let reverted = api(
        &fixture,
        Method::DELETE,
        "/repository/hex-local/api/packages/tools/releases/0.1.0",
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(
        reverted.status,
        StatusCode::NO_CONTENT,
        "{}",
        reverted.text()
    );
    assert_eq!(
        get(&fixture, "/repository/hex-local/packages/tools")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let names = testing::names(
        &get(&fixture, "/repository/hex-local/names").await.body,
        &public,
    )
    .unwrap();
    assert!(!names.packages.iter().any(|package| package.name == "tools"));
    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/hex-local/tarballs/extra-2.0.0.tar",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(
        fixture
            .call(
                Method::DELETE,
                "/repository/hex-local/names",
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::BAD_REQUEST
    );

    let components = get(&fixture, "/service/rest/v1/components?repository=hex-local").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    let hello = items
        .iter()
        .find(|item| item["name"] == "hello" && item["version"] == "1.0.0")
        .unwrap();
    assert_eq!(hello["format"], "hex");
    let tarball_asset = hello["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"] == "/tarballs/hello-1.0.0.tar")
        .unwrap();
    assert_eq!(tarball_asset["hex"]["licenses"], json!(["Apache-2.0"]));
    let id = hello["id"].as_str().unwrap();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let package = testing::package(
        &get(&fixture, "/repository/hex-local/packages/hello")
            .await
            .body,
        &public,
    )
    .unwrap();
    let left: Vec<&str> = package
        .releases
        .iter()
        .map(|release| release.version.as_str())
        .collect();
    assert_eq!(left, ["1.1.0"]);
    let search = get(
        &fixture,
        "/service/rest/v1/search?repository=hex-local&q=hello%20package",
    )
    .await;
    assert_eq!(
        search.json()["items"][0]["name"],
        "hello",
        "{}",
        search.text()
    );
    let licensed = get(
        &fixture,
        "/service/rest/v1/search?format=hex&hex.license=Apache-2.0",
    )
    .await;
    assert!(!licensed.json()["items"].as_array().unwrap().is_empty());

    let uploaded = Spec::new("widget", "0.3.0").build();
    let reply = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=hex-local",
            Auth::Admin,
            Body::from(upload_form(&uploaded.bytes)),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/hex-local/tarballs/gadget-1.0.0.tar",
            Auth::Admin,
            Body::from(Spec::new("gadget", "1.0.0").build().bytes),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.text());
    let misnamed = fixture
        .send_body(
            Method::PUT,
            "/repository/hex-local/tarballs/gadget-2.0.0.tar",
            Auth::Admin,
            Body::from(Spec::new("gadget", "1.0.0").build().bytes),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(misnamed.status, StatusCode::BAD_REQUEST);
    let names = testing::names(
        &get(&fixture, "/repository/hex-local/names").await.body,
        &public,
    )
    .unwrap();
    let listed: Vec<&str> = names
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(listed, ["gadget", "hello", "widget"]);
    let spec = get(&fixture, "/service/rest/v1/formats/hex/upload-specs").await;
    assert_eq!(spec.status, StatusCode::OK, "{}", spec.text());

    let task = fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "repository.hex.rebuild.metadata", "name": "Rebuild Hex", "properties": {"repositoryName": "hex-local"}})),
        )
        .await;
    assert_eq!(task.status, StatusCode::CREATED, "{}", task.text());
    let id = task.json()["id"].as_str().unwrap().to_owned();
    let run = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/tasks/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(run.status, StatusCode::NO_CONTENT, "{}", run.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let task = get(&fixture, &format!("/service/rest/v1/tasks/{id}"))
        .await
        .json();
    assert_eq!(task["lastRunResult"], "OK", "{task}");
    assert!(
        testing::names(
            &get(&fixture, "/repository/hex-local/names").await.body,
            &public
        )
        .is_some()
    );
}

fn testing_key_public() -> String {
    RegistryKey::read(&testing::private_key_pem())
        .unwrap()
        .public_key
}

#[tokio::test]
async fn signing_keys_are_kept_sealed_in_the_repository_configuration() {
    let fixture = ready().await;
    created(&fixture, "hosted", hosted("hex-keys", "ALLOW_ONCE")).await;
    let document = get(
        &fixture,
        "/service/rest/v1/repositories/hex/hosted/hex-keys",
    )
    .await;
    assert_eq!(document.status, StatusCode::OK, "{}", document.text());
    assert_eq!(document.json()["hexSigning"], Value::Null);
    let first = public_key(&fixture, "hex-keys").await;
    assert_eq!(public_key(&fixture, "hex-keys").await, first);
    let document = get(
        &fixture,
        "/service/rest/v1/repositories/hex/hosted/hex-keys",
    )
    .await;
    assert_eq!(document.json()["hexSigning"], json!({}));
    let stored = fixture
        .state
        .db()
        .conn()
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT attributes::text AS a FROM repository WHERE name = 'hex-keys'",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "a")
        .unwrap();
    assert!(stored.contains("hexSigning"), "{stored}");
    assert!(!stored.contains("PRIVATE KEY"), "{stored}");

    let mut changed = hosted("hex-keys", "ALLOW");
    changed["online"] = json!(true);
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/hex/hosted/hex-keys",
            Auth::Admin,
            Some(changed.clone()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(public_key(&fixture, "hex-keys").await, first);
    changed["hexSigning"] = Value::Null;
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/hex/hosted/hex-keys",
            Auth::Admin,
            Some(changed.clone()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(public_key(&fixture, "hex-keys").await, first);

    let own = testing::private_key_pem();
    changed["hexSigning"] = json!({"keypair": own});
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/hex/hosted/hex-keys",
            Auth::Admin,
            Some(changed.clone()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(public_key(&fixture, "hex-keys").await, testing_key_public());

    let mut supplied = hosted("hex-supplied", "ALLOW_ONCE");
    supplied["hexSigning"] = json!({"keypair": testing::private_key_pem()});
    created(&fixture, "hosted", supplied).await;
    assert_eq!(
        public_key(&fixture, "hex-supplied").await,
        testing_key_public()
    );
    let mut encrypted = hosted("hex-encrypted", "ALLOW_ONCE");
    encrypted["hexSigning"] = json!({
        "keypair": "-----BEGIN ENCRYPTED PRIVATE KEY-----\nMII\n-----END ENCRYPTED PRIVATE KEY-----\n",
    });
    let refused = create(&fixture, "hosted", encrypted).await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    let mut with_proxy = proxy("hex-signed-proxy", "https://repo.hex.pm");
    with_proxy["hexSigning"] = json!({"keypair": testing::private_key_pem()});
    created(&fixture, "proxy", with_proxy).await;
    let document = get(
        &fixture,
        "/service/rest/v1/repositories/hex/proxy/hex-signed-proxy",
    )
    .await
    .json();
    assert!(document.get("hexSigning").is_none(), "{document}");

    let key = token(&fixture).await;
    let built = Spec::new("once", "1.0.0").build();
    let reply = publish(&fixture, "hex-supplied", Some(&key), &built.bytes, false).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let replaced = publish(&fixture, "hex-supplied", Some(&key), &built.bytes, true).await;
    assert_eq!(replaced.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        term(&replaced)["message"]
            .as_str()
            .unwrap()
            .contains("redeploying")
    );
}

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Arc<Mutex<Vec<String>>>,
}

impl Upstream {
    fn put(&self, path: &str, body: Vec<u8>) {
        self.files.lock().unwrap().insert(path.to_owned(), body);
    }

    fn hits(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|hit| *hit == path)
            .count()
    }
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    let path = format!("/{path}");
    upstream.hits.lock().unwrap().push(path.clone());
    match upstream.files.lock().unwrap().get(&path) {
        Some(body) => (
            [(header::CONTENT_TYPE, "application/octet-stream")],
            body.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start(upstream: &Upstream) -> SocketAddr {
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

fn serve_registry(upstream: &Upstream, prefix: &str, key: &RegistryKey, signer: &RegistryKey) {
    let jason = Spec::new("jason", "1.4.4").build();
    let decimal = Spec::new("decimal", "2.1.1").build();
    let records = vec![jason.record(1_700_000_000), decimal.record(1_700_000_100)];
    upstream.put(
        &format!("{prefix}/public_key"),
        key.public_key.clone().into_bytes(),
    );
    upstream.put(
        &format!("{prefix}/names"),
        testing::signed(signer, &registry::names("hexpm", &records)),
    );
    upstream.put(
        &format!("{prefix}/versions"),
        testing::signed(signer, &registry::versions("hexpm", &records)),
    );
    for name in ["jason", "decimal"] {
        upstream.put(
            &format!("{prefix}/packages/{name}"),
            testing::signed(signer, &registry::package("hexpm", name, &records)),
        );
    }
    upstream.put(&format!("{prefix}/tarballs/jason-1.4.4.tar"), jason.bytes);
    upstream.put(
        &format!("{prefix}/tarballs/decimal-2.1.1.tar"),
        decimal.bytes,
    );
}

#[tokio::test]
async fn proxies_pass_the_signed_registry_through_and_groups_sign_their_own() {
    let fixture = ready().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let hexpm = RegistryKey::read(&testing::private_key_pem()).unwrap();
    let stranger = RegistryKey::read(&RegistryKey::generate().unwrap()).unwrap();
    let upstream = Upstream::default();
    serve_registry(&upstream, "", &hexpm, &hexpm);
    serve_registry(&upstream, "/tampered", &hexpm, &stranger);
    upstream.put("/installs/hex-1.x.csv", b"2.1.1,abc,1.17.0\n".to_vec());
    let address = start(&upstream).await;
    created(
        &fixture,
        "proxy",
        proxy("hexpm-proxy", &format!("http://{address}")),
    )
    .await;
    created(
        &fixture,
        "proxy",
        proxy("tampered-proxy", &format!("http://{address}/tampered/")),
    )
    .await;

    for path in [
        "/names",
        "/versions",
        "/packages/jason",
        "/tarballs/jason-1.4.4.tar",
        "/public_key",
        "/installs/hex-1.x.csv",
    ] {
        let reply = get(&fixture, &format!("/repository/hexpm-proxy{path}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
        assert_eq!(
            reply.body,
            upstream.files.lock().unwrap()[path],
            "{path} must pass through unchanged"
        );
    }
    let names = testing::names(
        &get(&fixture, "/repository/hexpm-proxy/names").await.body,
        &hexpm.public_key,
    )
    .unwrap();
    assert_eq!(names.repository, "hexpm");
    let package = testing::package(
        &get(&fixture, "/repository/hexpm-proxy/packages/jason")
            .await
            .body,
        &hexpm.public_key,
    )
    .unwrap();
    assert_eq!(package.repository, "hexpm");
    assert_eq!(upstream.hits("/packages/jason"), 1);
    assert_eq!(upstream.hits("/tarballs/jason-1.4.4.tar"), 1);
    let again = get(&fixture, "/repository/hexpm-proxy/tarballs/jason-1.4.4.tar").await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(upstream.hits("/tarballs/jason-1.4.4.tar"), 1);
    assert_eq!(
        get(&fixture, "/repository/hexpm-proxy/packages/nothing")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=hexpm-proxy",
    )
    .await;
    let item = &components.json()["items"][0];
    assert_eq!(
        (&item["name"], &item["version"]),
        (&json!("jason"), &json!("1.4.4"))
    );
    assert_eq!(item["assets"][0]["hex"]["licenses"], json!(["Apache-2.0"]));
    let page = get(&fixture, "/repository/hexpm-proxy/").await;
    assert!(page.text().contains("HEX_MIRROR="), "{}", page.text());
    let refused = publish(
        &fixture,
        "hexpm-proxy",
        Some(&token(&fixture).await),
        &Spec::new("mine", "1.0.0").build().bytes,
        false,
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        term(&refused)["message"]
            .as_str()
            .unwrap()
            .contains("hosted")
    );
    let put = fixture
        .send_body(
            Method::PUT,
            "/repository/hexpm-proxy/tarballs/mine-1.0.0.tar",
            Auth::Admin,
            Body::from(Vec::new()),
            &[],
        )
        .await;
    assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED);
    let forgotten = fixture
        .call(
            Method::DELETE,
            "/repository/hexpm-proxy/packages/jason",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(forgotten.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&fixture, "/repository/hexpm-proxy/packages/jason")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(upstream.hits("/packages/jason"), 2);

    created(&fixture, "hosted", hosted("hex-local", "ALLOW")).await;
    let key = token(&fixture).await;
    let hello = Spec::new("hello", "1.0.0")
        .requirement("jason", "~> 1.4", false)
        .build();
    let fork = Spec::new("jason", "9.0.0").build();
    for built in [&hello, &fork] {
        let reply = publish(&fixture, "hex-local", Some(&key), &built.bytes, false).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    created(
        &fixture,
        "group",
        group("hex-all", &["hex-local", "hexpm-proxy"]),
    )
    .await;
    created(
        &fixture,
        "group",
        group("hex-tampered", &["tampered-proxy", "hex-local"]),
    )
    .await;

    let group_key = public_key(&fixture, "hex-all").await;
    assert_ne!(group_key, hexpm.public_key);
    assert_ne!(group_key, public_key(&fixture, "hex-local").await);
    let names_file = get(&fixture, "/repository/hex-all/names").await;
    assert_eq!(names_file.status, StatusCode::OK, "{}", names_file.text());
    assert!(testing::names(&names_file.body, &hexpm.public_key).is_none());
    let names = testing::names(&names_file.body, &group_key).unwrap();
    assert_eq!(names.repository, "hex-all");
    let listed: Vec<&str> = names
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(listed, ["decimal", "hello", "jason"]);
    let versions = testing::versions(
        &get(&fixture, "/repository/hex-all/versions").await.body,
        &group_key,
    )
    .unwrap();
    let jason = versions
        .packages
        .iter()
        .find(|package| package.name == "jason")
        .unwrap();
    assert_eq!(jason.versions, ["1.4.4", "9.0.0"]);
    let package = testing::package(
        &get(&fixture, "/repository/hex-all/packages/jason")
            .await
            .body,
        &group_key,
    )
    .unwrap();
    assert_eq!(
        (package.name.as_str(), package.repository.as_str()),
        ("jason", "hex-all")
    );
    let merged: Vec<&str> = package
        .releases
        .iter()
        .map(|release| release.version.as_str())
        .collect();
    assert_eq!(merged, ["1.4.4", "9.0.0"]);
    assert_eq!(package.releases[1].inner_checksum, fork.inner_checksum);
    let cached = get(&fixture, "/repository/hex-all/packages/jason").await;
    assert_eq!(testing::package(&cached.body, &group_key).unwrap(), package);
    let from_proxy = get(&fixture, "/repository/hex-all/tarballs/jason-1.4.4.tar").await;
    assert_eq!(
        from_proxy.body,
        upstream.files.lock().unwrap()["/tarballs/jason-1.4.4.tar"]
    );
    let from_hosted = get(&fixture, "/repository/hex-all/tarballs/hello-1.0.0.tar").await;
    assert_eq!(from_hosted.body, hello.bytes);
    let page = get(&fixture, "/repository/hex-all/").await;
    assert!(
        page.text().contains("mix hex.repo add hex-all"),
        "{}",
        page.text()
    );

    let tampered = testing::names(
        &get(&fixture, "/repository/hex-tampered/names").await.body,
        &public_key(&fixture, "hex-tampered").await,
    )
    .unwrap();
    let listed: Vec<&str> = tampered
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(listed, ["hello", "jason"]);
    assert_eq!(
        get(&fixture, "/repository/hex-tampered/packages/decimal")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let removed = api(
        &fixture,
        Method::DELETE,
        "/repository/hex-local/api/packages/jason/releases/9.0.0",
        Some(&key),
        Vec::new(),
        ERLANG,
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let package = testing::package(
        &get(&fixture, "/repository/hex-all/packages/jason")
            .await
            .body,
        &group_key,
    )
    .unwrap();
    assert_eq!(package.releases.len(), 1);
    for (method, path) in [
        (Method::PUT, "/repository/hex-all/tarballs/hello-1.0.0.tar"),
        (Method::DELETE, "/repository/hex-all/packages/jason"),
    ] {
        let reply = fixture.call(method, path, Auth::Admin, None).await;
        assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED, "{path}");
    }
    fixture.add_user("visitor", "visitor-password", &[]).await;
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/hex-all/names", Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                "/repository/hex-all/packages/jason",
                Auth::Basic("visitor", "visitor-password"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}
