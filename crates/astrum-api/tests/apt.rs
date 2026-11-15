mod support;

use std::io::{Read, Write};
use std::sync::Arc;

use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::transfer::{Bundle, Conflict, LocalSource, PROMOTED, Status, Terms};
use astrum_content::{ContentError, Format};
use astrum_format_apt::deb;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "aptboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_apt::Apt::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/apt/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str) -> Value {
    hosted_with(name, "ALLOW_ONCE")
}

fn hosted_with(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
        "apt": {"distribution": "bookworm"},
        "aptSigning": {"keypair": astrum_format_apt::testing::signing_key()},
    })
}

fn package(name: &str, version: &str, architecture: &str) -> Vec<u8> {
    deb::build(
        &format!(
            "Package: {name}\nVersion: {version}\nArchitecture: {architecture}\nMaintainer: Tests <t@example.org>\nDescription: {name} for the tests\n It has a long description.\n"
        ),
        &[(&format!("./usr/share/doc/{name}/README"), b"hello\n")],
    )
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn post(fixture: &Fixture, repository: &str, body: Vec<u8>, content_type: &str) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/"),
            Auth::Admin,
            Body::from(body),
            &[("content-type", content_type)],
        )
        .await
}

fn multipart(name: &str, file: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"package.deb\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[tokio::test]
async fn hosted_packages_get_signed_metadata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("debs")).await;
    let hello = package("hello-astrum", "1.0.0", "all");
    let reply = post(&fixture, "debs", hello.clone(), "multipart/form-data").await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let again = post(&fixture, "debs", hello.clone(), "application/octet-stream").await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    let lib = package("libastrum", "2.0.0~rc1", "amd64");
    let reply = post(
        &fixture,
        "debs",
        multipart("file", &lib),
        &format!("multipart/form-data; boundary={BOUNDARY}"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=debs",
            Auth::Admin,
            Body::from(multipart("apt.asset", &package("tool", "1:3.0-1", "amd64"))),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());
    let broken = post(
        &fixture,
        "debs",
        b"not a package".to_vec(),
        "application/octet-stream",
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);

    let release = get(&fixture, "/repository/debs/dists/bookworm/Release").await;
    assert_eq!(release.status, StatusCode::OK);
    let text = release.text();
    assert!(
        text.starts_with("Suite: bookworm\nCodename: bookworm\nComponents: main\nDate: "),
        "{text}"
    );
    assert!(text.contains("\nArchitectures: all amd64\n"), "{text}");
    let packages = get(
        &fixture,
        "/repository/debs/dists/bookworm/main/binary-amd64/Packages",
    )
    .await;
    let listing = packages.text();
    assert!(
        listing.contains("Package: libastrum\nVersion: 2.0.0~rc1\n"),
        "{listing}"
    );
    assert!(
        listing.contains("Filename: pool/t/tool/tool_3.0-1_amd64.deb\n"),
        "{listing}"
    );
    assert!(text.contains(&format!(
        " {} {} main/binary-amd64/Packages\n",
        sha256(&packages.body),
        packages.body.len()
    )));
    let compressed = get(
        &fixture,
        "/repository/debs/dists/bookworm/main/binary-amd64/Packages.gz",
    )
    .await;
    let mut unpacked = String::new();
    flate2::read::GzDecoder::new(compressed.body.as_slice())
        .read_to_string(&mut unpacked)
        .unwrap();
    assert_eq!(unpacked, listing);
    assert!(text.contains(&format!(
        " {} {} main/binary-amd64/Packages.gz\n",
        sha256(&compressed.body),
        compressed.body.len()
    )));
    let all = get(
        &fixture,
        "/repository/debs/dists/bookworm/main/binary-all/Packages",
    )
    .await
    .text();
    assert!(
        all.contains(&format!("SHA256: {}\n", sha256(&hello))),
        "{all}"
    );
    let inline = get(&fixture, "/repository/debs/dists/bookworm/InRelease")
        .await
        .text();
    assert!(
        inline.starts_with("-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\nSuite: bookworm\n")
    );
    let detached = get(&fixture, "/repository/debs/dists/bookworm/Release.gpg").await;
    assert!(detached.text().starts_with("-----BEGIN PGP SIGNATURE-----"));
    let file = get(
        &fixture,
        "/repository/debs/pool/h/hello-astrum/hello-astrum_1.0.0_all.deb",
    )
    .await;
    assert_eq!(file.body, hello);
    assert_eq!(file.headers["content-type"], "application/x-debian-package");
    assert_eq!(
        get(&fixture, "/repository/debs/dists/other/Release")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let components = get(&fixture, "/service/rest/v1/components?repository=debs").await;
    let mut names: Vec<(String, String, String)> = components.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["group"].as_str().unwrap().to_owned(),
                item["name"].as_str().unwrap().to_owned(),
                item["version"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            (
                "all".to_owned(),
                "hello-astrum".to_owned(),
                "1.0.0".to_owned()
            ),
            (
                "amd64".to_owned(),
                "libastrum".to_owned(),
                "2.0.0~rc1".to_owned()
            ),
            ("amd64".to_owned(), "tool".to_owned(), "1:3.0-1".to_owned()),
        ]
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/debs/pool/l/libastrum/libastrum_2.0.0~rc1_amd64.deb",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let listing = get(
        &fixture,
        "/repository/debs/dists/bookworm/main/binary-amd64/Packages",
    )
    .await
    .text();
    assert!(!listing.contains("libastrum"));
    let id = components.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "tool")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let release = get(&fixture, "/repository/debs/dists/bookworm/Release")
        .await
        .text();
    assert!(release.contains("\nArchitectures: all\n"), "{release}");
    assert_eq!(
        get(
            &fixture,
            "/repository/debs/dists/bookworm/main/binary-amd64/Packages"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn proxies_cache_metadata_and_packages_of_their_remote() {
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
    let remote = Fixture::new().await;
    astrum_blob::bootstrap(remote.state.db().conn(), remote.data.path())
        .await
        .unwrap();
    remote
        .state
        .content()
        .register(Arc::new(astrum_format_apt::Apt::default()));
    create(&remote, "hosted", hosted("origin")).await;
    let hello = package("hello-astrum", "1.0.0", "all");
    assert_eq!(
        post(&remote, "origin", hello.clone(), "application/octet-stream")
            .await
            .status,
        StatusCode::CREATED
    );
    let anonymous = remote
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::OK);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = astrum_api::router(remote.state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    create(
        &fixture,
        "proxy",
        json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/repository/origin/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
            "apt": {"distribution": "bookworm", "flat": false},
        }),
    )
    .await;
    let inline = get(&fixture, "/repository/mirror/dists/bookworm/InRelease").await;
    assert_eq!(inline.status, StatusCode::OK, "{}", inline.text());
    assert!(inline.text().contains("Suite: bookworm"));
    let file = get(
        &fixture,
        "/repository/mirror/pool/h/hello-astrum/hello-astrum_1.0.0_all.deb",
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, hello);
    let components = get(&fixture, "/service/rest/v1/components?repository=mirror").await;
    let item = &components.json()["items"][0];
    assert_eq!(item["name"], "hello-astrum");
    assert_eq!(item["group"], "all");
    let refused = post(&fixture, "mirror", hello, "application/octet-stream").await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

const HELLO: &str = "/pool/h/hello-astrum/hello-astrum_1.0.0_amd64.deb";

async fn promotion_pair(target_policy: &str) -> (Fixture, Bundle) {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("debs-source")).await;
    create(
        &fixture,
        "hosted",
        hosted_with("debs-target", target_policy),
    )
    .await;
    let reply = post(
        &fixture,
        "debs-source",
        package("hello-astrum", "1.0.0", "amd64"),
        "application/octet-stream",
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let bundle = support::bundle_of(
        &fixture,
        &astrum_format_apt::Apt::default(),
        "debs-source",
        HELLO,
    )
    .await;
    (fixture, bundle)
}

async fn promote(fixture: &Fixture, bundle: &Bundle) -> Result<(), ContentError> {
    let from = support::repository_ctx(fixture, "debs-source").await;
    let to = support::repository_ctx(fixture, "debs-target").await;
    astrum_format_apt::Apt::default()
        .transfer()
        .expect("apt takes part in promotion")
        .receive(
            &to,
            bundle,
            &LocalSource::new(from),
            Terms::new(Conflict::Fail, PROMOTED),
        )
        .await
        .map(|_| ())
}

async fn recorded(fixture: &Fixture, path: &str) -> bool {
    store::find(fixture.state.db().conn(), "debs-target", path)
        .await
        .unwrap()
        .is_some()
}

#[tokio::test]
async fn packages_are_promoted_and_the_target_lists_them() {
    let (fixture, bundle) = promotion_pair("ALLOW_ONCE").await;
    let format = astrum_format_apt::Apt::default();
    let outcome = support::receive(&fixture, &format, "debs-source", "debs-target", &bundle).await;
    assert_eq!(outcome.status, Status::Copied);
    assert!(outcome.warnings.is_empty(), "{:?}", outcome.warnings);
    assert!(recorded(&fixture, "/dists/bookworm/Release").await);
    let served = get(&fixture, &format!("/repository/debs-target{HELLO}")).await;
    assert_eq!(served.body, package("hello-astrum", "1.0.0", "amd64"));
    let listing = get(
        &fixture,
        "/repository/debs-target/dists/bookworm/main/binary-amd64/Packages",
    )
    .await
    .text();
    assert!(
        listing.contains("Package: hello-astrum\nVersion: 1.0.0\n"),
        "{listing}"
    );
    assert!(
        listing.contains("Filename: pool/h/hello-astrum/hello-astrum_1.0.0_amd64.deb\n"),
        "{listing}"
    );
    let signed = get(&fixture, "/repository/debs-target/dists/bookworm/InRelease").await;
    assert!(
        signed
            .text()
            .starts_with("-----BEGIN PGP SIGNED MESSAGE-----")
    );
}

#[tokio::test]
async fn promotion_into_a_read_only_repository_is_refused() {
    let (fixture, bundle) = promotion_pair("DENY").await;
    match promote(&fixture, &bundle).await {
        Err(ContentError::BadRequest(found)) => {
            assert_eq!(found, "the repository debs-target is read-only")
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert!(!recorded(&fixture, HELLO).await);
}

#[tokio::test]
async fn bundles_with_a_path_the_upload_would_not_use_are_refused() {
    let (fixture, bundle) = promotion_pair("ALLOW_ONCE").await;
    let mut elsewhere = bundle.clone();
    elsewhere.assets[0].path = "/pool/h/hello-astrum/hello-astrum_9.9.9_amd64.deb".to_owned();
    let mut renamed = bundle.clone();
    renamed.name = "other".to_owned();
    renamed.assets[0].path = "/pool/o/other/other_1.0.0_amd64.deb".to_owned();
    let mut udeb = bundle.clone();
    udeb.assets[0].path = HELLO.replace(".deb", ".udeb");
    let other = "/pool/o/other/other_1.0.0_amd64.deb";
    let from = support::repository_ctx(&fixture, "debs-source").await;
    let mut put = PutAsset::hosted(&from, other)
        .kind("deb")
        .content_type("application/x-debian-package")
        .component(ComponentKey {
            namespace: "amd64".to_owned(),
            name: "other".to_owned(),
            version: "1.0.0".to_owned(),
            kind: String::new(),
            attributes: json!({}),
        });
    put.check_content_type = false;
    store::put(
        &from,
        put,
        astrum_blob::content_of(package("hello-astrum", "1.0.0", "amd64")),
    )
    .await
    .unwrap();
    let swapped = support::bundle_of(
        &fixture,
        &astrum_format_apt::Apt::default(),
        "debs-source",
        other,
    )
    .await;
    for crafted in [elsewhere, renamed, udeb, swapped] {
        let path = crafted.assets[0].path.clone();
        match promote(&fixture, &crafted).await {
            Err(ContentError::BadRequest(found)) => {
                assert!(found.contains("where the upload would store"), "{found}")
            }
            other => panic!("expected a refusal for {path}, got {other:?}"),
        }
        assert!(!recorded(&fixture, &path).await);
    }
    assert!(!recorded(&fixture, HELLO).await);
}
