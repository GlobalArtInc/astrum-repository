mod support;

use std::io::Write;
use std::sync::Arc;

use astrum_format_alpine::testing::{self, Spec};
use astrum_format_alpine::{index, package};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "alpineboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_alpine::Alpine::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/alpine/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, signing: Option<Value>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW_ONCE"},
    });
    if let Some(signing) = signing {
        body["alpineSigning"] = signing;
    }
    body
}

fn signing() -> Value {
    json!({"keypair": testing::signing_key()})
}

fn package(name: &str, version: &str, arch: &str) -> Vec<u8> {
    testing::build(&Spec::new(name, version, arch)).bytes
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn put(fixture: &Fixture, path: &str, body: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::Admin,
            Body::from(body),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn delete(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::DELETE, path, Auth::Admin, None).await
}

async fn upload(fixture: &Fixture, repository: &str, directory: &str, file: &[u8]) -> Reply {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"alpine.directory\"\r\n\r\n{directory}\r\n--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"alpine.asset\"; filename=\"package.apk\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await
}

fn names(file: &[u8]) -> Vec<(String, String, String)> {
    let read = index::read(file).unwrap();
    index::records(&read.index)
        .into_iter()
        .map(|record| (record.name, record.version, record.arch))
        .collect()
}

fn owned(items: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    items
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect()
}

#[tokio::test]
async fn hosted_packages_get_a_signed_index_per_directory() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("apks", Some(signing()))).await;
    let hello = package("hello", "1.0-r0", "x86_64");
    let target = "/repository/apks/v3.20/main/x86_64/hello-1.0-r0.apk";
    let reply = put(&fixture, target, hello.clone()).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(
        put(&fixture, target, hello.clone()).await.status,
        StatusCode::CONFLICT
    );
    for (path, body, wanted) in [
        (
            "/repository/apks/v3.20/main/x86_64/hello.apk",
            hello.clone(),
            "must be stored as hello-1.0-r0.apk",
        ),
        (
            "/repository/apks/v3.20/main/aarch64/hello-1.0-r0.apk",
            hello.clone(),
            "a package for x86_64",
        ),
        (
            "/repository/apks/hello-1.0-r0.apk",
            hello.clone(),
            "named after their architecture",
        ),
        (
            "/repository/apks/v3.20/main/x86_64/APKINDEX.tar.gz",
            hello.clone(),
            "are generated",
        ),
        (
            "/repository/apks/v3.20/main/x86_64/broken-1-r0.apk",
            b"not a package".to_vec(),
            "Not a valid Alpine package",
        ),
    ] {
        let reply = put(&fixture, path, body).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(reply.text().contains(wanted), "{path}: {}", reply.text());
    }
    let doc = package("hello-doc", "1.0_rc1-r0", "noarch");
    let reply = upload(&fixture, "apks", "v3.20/main/x86_64", &doc).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let tool = package("tool", "2.1-r3", "aarch64");
    let reply = upload(&fixture, "apks", "", &tool).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = upload(&fixture, "apks", "", &doc).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());

    let file = get(
        &fixture,
        "/repository/apks/v3.20/main/x86_64/APKINDEX.tar.gz",
    )
    .await;
    assert_eq!(file.status, StatusCode::OK, "{}", file.text());
    assert_eq!(file.headers["content-type"], "application/gzip");
    let read = index::read(&file.body).unwrap();
    assert_eq!(read.signatures, [".SIGN.RSA.apks.rsa.pub"]);
    assert_eq!(read.description, "apks v3.20/main");
    assert!(testing::verify_index(
        &file.body,
        "apks.rsa.pub",
        &testing::public_key()
    ));
    assert_eq!(
        names(&file.body),
        owned(&[
            ("hello", "1.0-r0", "x86_64"),
            ("hello-doc", "1.0_rc1-r0", "x86_64")
        ])
    );
    let checksum = package::read(&hello).unwrap().checksum;
    assert!(
        read.index.starts_with(&format!(
            "C:{checksum}\nP:hello\nV:1.0-r0\nA:x86_64\nS:{}\n",
            hello.len()
        )),
        "{}",
        read.index
    );
    let key = get(&fixture, "/repository/apks/apks.rsa.pub").await;
    assert_eq!(key.status, StatusCode::OK);
    assert_eq!(key.text(), testing::public_key());
    assert_eq!(
        get(&fixture, "/repository/apks/other.rsa.pub").await.status,
        StatusCode::NOT_FOUND
    );
    let arm = get(&fixture, "/repository/apks/aarch64/APKINDEX.tar.gz").await;
    assert_eq!(names(&arm.body), owned(&[("tool", "2.1-r3", "aarch64")]));
    let stored = get(&fixture, target).await;
    assert_eq!(stored.body, hello);
    let page = get(&fixture, "/repository/apks/").await;
    assert!(page.text().contains("/etc/apk/keys/apks.rsa.pub"));

    let components = get(&fixture, "/service/rest/v1/components?repository=apks").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    let mut found: Vec<(String, String, String)> = items
        .iter()
        .map(|item| {
            (
                item["group"].as_str().unwrap().to_owned(),
                item["name"].as_str().unwrap().to_owned(),
                item["version"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        owned(&[
            ("aarch64", "tool", "2.1-r3"),
            ("x86_64", "hello", "1.0-r0"),
            ("x86_64", "hello-doc", "1.0_rc1-r0")
        ])
    );
    let hello_item = items.iter().find(|item| item["name"] == "hello").unwrap();
    assert_eq!(hello_item["assets"][0]["alpine"]["checksum"], checksum);
    assert_eq!(hello_item["assets"][0]["alpine"]["license"], "MIT");

    let deleted = delete(
        &fixture,
        "/repository/apks/v3.20/main/x86_64/hello-doc-1.0_rc1-r0.apk",
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let file = get(
        &fixture,
        "/repository/apks/v3.20/main/x86_64/APKINDEX.tar.gz",
    )
    .await;
    assert_eq!(names(&file.body), owned(&[("hello", "1.0-r0", "x86_64")]));
    assert_eq!(
        delete(
            &fixture,
            "/repository/apks/v3.20/main/x86_64/APKINDEX.tar.gz"
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    let id = hello_item["id"].as_str().unwrap();
    let removed = delete(&fixture, &format!("/service/rest/v1/components/{id}")).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(
            &fixture,
            "/repository/apks/v3.20/main/x86_64/APKINDEX.tar.gz"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/apks/aarch64/APKINDEX.tar.gz")
            .await
            .status,
        StatusCode::OK
    );
    let spec = get(&fixture, "/service/rest/v1/formats/alpine/upload-specs").await;
    assert_eq!(spec.status, StatusCode::OK, "{}", spec.text());
    assert_eq!(spec.json()["componentFields"][0]["name"], "directory");
}

#[tokio::test]
async fn indexes_are_unsigned_without_a_key_and_keys_may_need_a_passphrase() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plain", None)).await;
    let hello = package("hello", "1.0-r0", "x86_64");
    let reply = put(
        &fixture,
        "/repository/plain/x86_64/hello-1.0-r0.apk",
        hello.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let file = get(&fixture, "/repository/plain/x86_64/APKINDEX.tar.gz").await;
    let read = index::read(&file.body).unwrap();
    assert!(read.signatures.is_empty());
    assert_eq!(read.description, "plain");
    assert_eq!(
        get(&fixture, "/repository/plain/plain.rsa.pub")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert!(
        !get(&fixture, "/repository/plain/")
            .await
            .text()
            .contains("/etc/apk/keys/")
    );

    create(
        &fixture,
        "hosted",
        hosted(
            "locked",
            Some(json!({
                "keypair": testing::encrypted_signing_key("open sesame"),
                "passphrase": "open sesame",
            })),
        ),
    )
    .await;
    let reply = put(
        &fixture,
        "/repository/locked/x86_64/hello-1.0-r0.apk",
        hello,
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let file = get(&fixture, "/repository/locked/x86_64/APKINDEX.tar.gz").await;
    assert!(testing::verify_index(
        &file.body,
        "locked.rsa.pub",
        &testing::public_key()
    ));
    let document = get(
        &fixture,
        "/service/rest/v1/repositories/alpine/hosted/locked",
    )
    .await;
    assert_eq!(document.json()["alpineSigning"], json!({}));

    let mut changed = hosted("locked", None);
    changed["alpineSigning"] = Value::Null;
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/alpine/hosted/locked",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let file = get(&fixture, "/repository/locked/x86_64/APKINDEX.tar.gz").await;
    assert!(index::read(&file.body).unwrap().signatures.is_empty());
}

#[tokio::test]
async fn proxies_and_groups_sign_what_they_serve() {
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
    let remote = ready().await;
    create(&remote, "hosted", hosted("origin", Some(signing()))).await;
    let hello = package("hello", "1.0-r0", "x86_64");
    let reply = put(
        &remote,
        "/repository/origin/v3.20/main/x86_64/hello-1.0-r0.apk",
        hello.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
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
            "alpineSigning": signing(),
        }),
    )
    .await;
    let path = "v3.20/main/x86_64/APKINDEX.tar.gz";
    let file = get(&fixture, &format!("/repository/mirror/{path}")).await;
    assert_eq!(file.status, StatusCode::OK, "{}", file.text());
    let read = index::read(&file.body).unwrap();
    assert_eq!(
        read.signatures,
        [".SIGN.RSA.mirror.rsa.pub", ".SIGN.RSA.origin.rsa.pub"]
    );
    for key in ["mirror.rsa.pub", "origin.rsa.pub"] {
        assert!(
            testing::verify_index(&file.body, key, &testing::public_key()),
            "{key}"
        );
    }
    let again = get(&fixture, &format!("/repository/mirror/{path}")).await;
    assert_eq!(again.body, file.body);
    let proxied = get(
        &fixture,
        "/repository/mirror/v3.20/main/x86_64/hello-1.0-r0.apk",
    )
    .await;
    assert_eq!(proxied.status, StatusCode::OK);
    assert_eq!(proxied.body, hello);
    let components = get(&fixture, "/service/rest/v1/components?repository=mirror").await;
    let item = &components.json()["items"][0];
    assert_eq!(
        (&item["group"], &item["name"], &item["version"]),
        (&json!("x86_64"), &json!("hello"), &json!("1.0-r0"))
    );
    assert_eq!(
        item["assets"][0]["alpine"]["checksum"],
        package::read(&hello).unwrap().checksum
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/mirror/v3.20/main/x86_64/hello-1.0-r0.apk",
            hello.clone()
        )
        .await
        .status,
        StatusCode::METHOD_NOT_ALLOWED
    );

    create(&fixture, "hosted", hosted("local", Some(signing()))).await;
    let tool = package("tool", "2.0-r1", "x86_64");
    let reply = put(
        &fixture,
        "/repository/local/v3.20/main/x86_64/tool-2.0-r1.apk",
        tool.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    let group = |name: &str, members: &[&str], signed: bool| {
        let mut body = json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": members},
        });
        if signed {
            body["alpineSigning"] = signing();
        }
        body
    };
    create(&fixture, "group", group("all", &["local", "mirror"], true)).await;
    create(
        &fixture,
        "group",
        group("plain", &["local", "mirror"], false),
    )
    .await;
    create(&fixture, "group", group("single", &["local"], false)).await;
    create(&fixture, "group", group("through", &["mirror"], false)).await;

    let merged = get(&fixture, &format!("/repository/all/{path}")).await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    assert_eq!(
        index::read(&merged.body).unwrap().signatures,
        [".SIGN.RSA.all.rsa.pub"]
    );
    assert!(testing::verify_index(
        &merged.body,
        "all.rsa.pub",
        &testing::public_key()
    ));
    assert_eq!(
        names(&merged.body),
        owned(&[("tool", "2.0-r1", "x86_64"), ("hello", "1.0-r0", "x86_64")])
    );
    let through = get(
        &fixture,
        "/repository/all/v3.20/main/x86_64/hello-1.0-r0.apk",
    )
    .await;
    assert_eq!(through.body, hello);
    assert_eq!(
        get(&fixture, "/repository/all/all.rsa.pub").await.text(),
        testing::public_key()
    );
    let plain = get(&fixture, &format!("/repository/plain/{path}")).await;
    let read = index::read(&plain.body).unwrap();
    assert!(read.signatures.is_empty());
    assert_eq!(index::records(&read.index).len(), 2);
    let single = get(&fixture, &format!("/repository/single/{path}")).await;
    assert_eq!(
        index::read(&single.body).unwrap().signatures,
        [".SIGN.RSA.local.rsa.pub"]
    );
    let through = get(&fixture, &format!("/repository/through/{path}")).await;
    assert_eq!(
        index::read(&through.body).unwrap().signatures,
        [".SIGN.RSA.mirror.rsa.pub", ".SIGN.RSA.origin.rsa.pub"]
    );
    assert_eq!(
        get(&fixture, "/repository/single/local.rsa.pub")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/all/v3.20/main/x86_64/tool-2.0-r1.apk",
            tool
        )
        .await
        .status,
        StatusCode::METHOD_NOT_ALLOWED
    );

    let removed = delete(
        &fixture,
        "/repository/local/v3.20/main/x86_64/tool-2.0-r1.apk",
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let merged = get(&fixture, &format!("/repository/all/{path}")).await;
    assert_eq!(names(&merged.body), owned(&[("hello", "1.0-r0", "x86_64")]));
}
