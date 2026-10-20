mod support;

use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use astrum_format_pacman::archive::Compression;
use astrum_format_pacman::db::{self, Entry, Record};
use astrum_format_pacman::paths::Variant;
use astrum_format_pacman::testing::{self, Spec};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use base64::Engine;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "pacmanboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pacman::Pacman::default()));
    fixture.state.tasks().register(Arc::new(
        astrum_format_pacman::task::RebuildMetadata::default(),
    ));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/pacman/{kind}"),
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
        body["pacmanSigning"] = signing;
    }
    body
}

fn proxy(name: &str, remote: &str, signing: Option<Value>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": true},
    });
    if let Some(signing) = signing {
        body["pacmanSigning"] = signing;
    }
    body
}

fn group(name: &str, members: &[&str], signing: Option<Value>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
    });
    if let Some(signing) = signing {
        body["pacmanSigning"] = signing;
    }
    body
}

fn package(name: &str, version: &str, arch: &str) -> Vec<u8> {
    testing::build(&Spec::new(name, version, arch))
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn put_as(fixture: &Fixture, path: &str, body: Vec<u8>, auth: Auth<'_>) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            auth,
            Body::from(body),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn put(fixture: &Fixture, path: &str, body: Vec<u8>) -> Reply {
    put_as(fixture, path, body, Auth::Admin).await
}

async fn delete(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::DELETE, path, Auth::Admin, None).await
}

async fn upload(
    fixture: &Fixture,
    repository: &str,
    directory: &str,
    files: &[(&str, &[u8])],
) -> Reply {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"pacman.directory\"\r\n\r\n{directory}\r\n"
    )
    .unwrap();
    for (index, (file_name, data)) in files.iter().enumerate() {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"pacman.asset{}\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
            index + 1
        )
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
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

fn untar(file: &[u8]) -> BTreeMap<String, Vec<u8>> {
    assert_eq!(&file[..2], &[0x1f, 0x8b], "databases are gzip files");
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let mut entries = BTreeMap::new();
    for entry in tar.entries().unwrap() {
        let mut entry = entry.unwrap();
        let name = String::from_utf8(entry.path_bytes().to_vec()).unwrap();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        entries.insert(name, data);
    }
    entries
}

fn sections(data: &[u8]) -> BTreeMap<String, Vec<String>> {
    let text = String::from_utf8(data.to_vec()).unwrap();
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.starts_with('%') && line.ends_with('%') && line.len() > 2 {
            current = line.trim_matches('%').to_owned();
            found.entry(current.clone()).or_default();
        } else if !line.is_empty() {
            found
                .entry(current.clone())
                .or_default()
                .push(line.to_owned());
        }
    }
    found
}

fn packages_of(file: &[u8]) -> Vec<String> {
    untar(file)
        .keys()
        .filter_map(|name| name.strip_suffix("/desc").map(str::to_owned))
        .collect()
}

fn desc(file: &[u8], directory: &str) -> BTreeMap<String, Vec<String>> {
    sections(&untar(file)[&format!("{directory}/desc")])
}

fn one(fields: &BTreeMap<String, Vec<String>>, key: &str) -> String {
    fields
        .get(key)
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or_else(|| panic!("{key} is missing from {fields:?}"))
}

#[tokio::test]
async fn hosted_packages_build_signed_databases_per_architecture() {
    let fixture = ready().await;
    let key = testing::signing_key(None);
    let public = testing::public_key(&key);
    let packager = testing::signing_key(None);
    create(
        &fixture,
        "hosted",
        hosted("pkgs", Some(json!({"keypair": key}))),
    )
    .await;
    let hello = package("hello", "1.0-1", "x86_64");
    let target = "/repository/pkgs/x86_64/hello-1.0-1-x86_64.pkg.tar.zst";
    let reply = put(&fixture, target, hello.clone()).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(
        put(&fixture, target, hello.clone()).await.status,
        StatusCode::CONFLICT
    );
    for (path, body, wanted) in [
        (
            "/repository/pkgs/x86_64/hello-1.0-2-x86_64.pkg.tar.zst",
            hello.clone(),
            "must be stored as hello-1.0-1-x86_64.pkg.tar.zst",
        ),
        (
            "/repository/pkgs/aarch64/hello-1.0-1-x86_64.pkg.tar.zst",
            hello.clone(),
            "a package for x86_64",
        ),
        (
            "/repository/pkgs/hello-1.0-1-x86_64.pkg.tar.zst",
            hello.clone(),
            "named after their architecture",
        ),
        (
            "/repository/pkgs/x86_64/pkgs.db",
            hello.clone(),
            "are generated",
        ),
        (
            "/repository/pkgs/x86_64/README",
            hello.clone(),
            "packages (.pkg.tar.*) and their signatures (.sig) only",
        ),
        (
            "/repository/pkgs/x86_64/broken-1-1-x86_64.pkg.tar.zst",
            b"not a package".to_vec(),
            "Not a valid pacman package",
        ),
        (
            "/repository/pkgs/x86_64/hello-1.0-1-x86_64.pkg.tar.zst.sig",
            b"not a signature".to_vec(),
            "Not a valid OpenPGP signature",
        ),
    ] {
        let reply = put(&fixture, path, body).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(reply.text().contains(wanted), "{path}: {}", reply.text());
    }

    let newer = testing::build(&Spec {
        compression: Compression::Xz,
        depends: vec!["glibc".to_owned(), "bash>=5".to_owned()],
        ..Spec::new("hello", "1:0.9-1", "x86_64")
    });
    let newer_path = "/repository/pkgs/x86_64/hello-1:0.9-1-x86_64.pkg.tar.xz";
    let reply = put(&fixture, newer_path, newer.clone()).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let newer_signature = testing::sign(&packager, &newer);
    let reply = put(
        &fixture,
        &format!("{newer_path}.sig"),
        newer_signature.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    let doc = package("hello-doc", "1.0-1", "any");
    let doc_signature = testing::sign(&packager, &doc);
    let reply = upload(
        &fixture,
        "pkgs",
        "",
        &[
            ("whatever.pkg.tar.zst", &doc),
            ("hello-doc-1.0-1-any.pkg.tar.zst.sig", &doc_signature),
        ],
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let tool = package("tool", "2.0-1", "aarch64");
    let reply = upload(&fixture, "pkgs", "", &[("tool.pkg.tar.zst", &tool)]).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());

    let file = get(&fixture, "/repository/pkgs/x86_64/custom.db").await;
    assert_eq!(file.status, StatusCode::OK, "{}", file.text());
    assert_eq!(file.headers["content-type"], "application/gzip");
    assert_eq!(
        packages_of(&file.body),
        ["hello-1:0.9-1", "hello-doc-1.0-1"]
    );
    let entries = untar(&file.body);
    assert!(entries.contains_key("hello-1:0.9-1/"));
    assert!(!entries.keys().any(|name| name.ends_with("/files")));
    let fields = desc(&file.body, "hello-1:0.9-1");
    assert_eq!(one(&fields, "FILENAME"), "hello-1:0.9-1-x86_64.pkg.tar.xz");
    assert_eq!(one(&fields, "NAME"), "hello");
    assert_eq!(one(&fields, "BASE"), "hello");
    assert_eq!(one(&fields, "VERSION"), "1:0.9-1");
    assert_eq!(one(&fields, "DESC"), "hello for the tests");
    assert_eq!(one(&fields, "CSIZE"), newer.len().to_string());
    assert_eq!(one(&fields, "SHA256SUM"), sha256(&newer));
    assert_eq!(
        one(&fields, "PGPSIG"),
        base64::engine::general_purpose::STANDARD.encode(&newer_signature)
    );
    assert_eq!(one(&fields, "URL"), "https://example.org/hello");
    assert_eq!(fields["LICENSE"], ["MIT"]);
    assert_eq!(one(&fields, "ARCH"), "x86_64");
    assert_eq!(one(&fields, "BUILDDATE"), "1700000000");
    assert_eq!(one(&fields, "PACKAGER"), "Astrum Tests <tests@example.org>");
    assert_eq!(fields["DEPENDS"], ["glibc", "bash>=5"]);
    assert!(fields.contains_key("ISIZE"));
    let doc_fields = desc(&file.body, "hello-doc-1.0-1");
    assert_eq!(one(&doc_fields, "ARCH"), "any");
    assert_eq!(
        one(&doc_fields, "FILENAME"),
        "hello-doc-1.0-1-any.pkg.tar.zst"
    );
    assert_eq!(
        one(&doc_fields, "PGPSIG"),
        base64::engine::general_purpose::STANDARD.encode(&doc_signature)
    );
    for alias in [
        "/repository/pkgs/x86_64/custom.db.tar.gz",
        "/repository/pkgs/x86_64/pkgs.db",
    ] {
        assert_eq!(get(&fixture, alias).await.body, file.body, "{alias}");
    }
    let signature = get(&fixture, "/repository/pkgs/x86_64/custom.db.sig").await;
    assert_eq!(signature.status, StatusCode::OK);
    assert!(testing::verify(&signature.body, &file.body, &public));
    let files = get(&fixture, "/repository/pkgs/x86_64/custom.files").await;
    assert_eq!(files.status, StatusCode::OK);
    let listed = untar(&files.body);
    assert_eq!(
        listed["hello-doc-1.0-1/files"],
        b"%FILES%\nusr/\nusr/bin/\nusr/bin/hello-doc\nusr/share/\nusr/share/doc/\nusr/share/doc/hello-doc/\nusr/share/doc/hello-doc/README\n"
    );
    assert!(listed.contains_key("hello-1:0.9-1/desc"));
    let files_signature = get(&fixture, "/repository/pkgs/x86_64/custom.files.sig").await;
    assert!(testing::verify(&files_signature.body, &files.body, &public));
    assert_eq!(
        get(&fixture, "/repository/pkgs/x86_64/custom.db.tar.zst")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let key_file = get(&fixture, "/repository/pkgs/pkgs.key").await;
    assert_eq!(key_file.status, StatusCode::OK);
    assert_eq!(key_file.text(), public);

    let shared = get(
        &fixture,
        "/repository/pkgs/x86_64/hello-doc-1.0-1-any.pkg.tar.zst",
    )
    .await;
    assert_eq!(shared.status, StatusCode::OK);
    assert_eq!(shared.body, doc);
    assert_eq!(
        get(
            &fixture,
            "/repository/pkgs/aarch64/hello-doc-1.0-1-any.pkg.tar.zst.sig"
        )
        .await
        .body,
        doc_signature
    );
    assert_eq!(get(&fixture, target).await.body, hello);
    assert_eq!(get(&fixture, newer_path).await.body, newer);
    let arm = get(&fixture, "/repository/pkgs/aarch64/pkgs.db").await;
    assert_eq!(packages_of(&arm.body), ["hello-doc-1.0-1", "tool-2.0-1"]);
    let other = get(&fixture, "/repository/pkgs/armv7h/pkgs.db").await;
    assert_eq!(other.status, StatusCode::OK);
    assert_eq!(packages_of(&other.body), ["hello-doc-1.0-1"]);
    assert_eq!(
        get(&fixture, "/repository/pkgs/any/pkgs.db").await.body,
        other.body
    );
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/pkgs/x86_64/custom.db",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
    let modified = file.headers[header::LAST_MODIFIED].to_str().unwrap();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/pkgs/x86_64/custom.db",
            Auth::Admin,
            None,
            &[("if-modified-since", modified)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    let page = get(&fixture, "/repository/pkgs/").await.text();
    assert!(
        page.contains("Server = http://localhost/repository/pkgs/$arch"),
        "{page}"
    );
    assert!(page.contains("pacman-key --lsign-key "), "{page}");

    let components = get(&fixture, "/service/rest/v1/components?repository=pkgs").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    let mut found: Vec<(String, String, String, usize)> = items
        .iter()
        .map(|item| {
            (
                item["group"].as_str().unwrap().to_owned(),
                item["name"].as_str().unwrap().to_owned(),
                item["version"].as_str().unwrap().to_owned(),
                item["assets"].as_array().unwrap().len(),
            )
        })
        .collect();
    found.sort();
    let owned = |group: &str, name: &str, version: &str, assets| {
        (
            group.to_owned(),
            name.to_owned(),
            version.to_owned(),
            assets,
        )
    };
    assert_eq!(
        found,
        [
            owned("aarch64", "tool", "2.0-1", 1),
            owned("any", "hello-doc", "1.0-1", 2),
            owned("x86_64", "hello", "1.0-1", 1),
            owned("x86_64", "hello", "1:0.9-1", 2),
        ]
    );
    let newest = items
        .iter()
        .find(|item| item["version"] == "1:0.9-1")
        .unwrap();
    let package_asset = newest["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"].as_str().unwrap().ends_with(".xz"))
        .unwrap();
    assert_eq!(
        package_asset["pacman"]["depends"],
        json!(["glibc", "bash>=5"])
    );
    assert_eq!(package_asset["contentType"], "application/x-xz");

    let deleted = delete(&fixture, newer_path).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    assert_eq!(
        get(&fixture, &format!("{newer_path}.sig")).await.status,
        StatusCode::NOT_FOUND
    );
    let file = get(&fixture, "/repository/pkgs/x86_64/custom.db").await;
    assert_eq!(packages_of(&file.body), ["hello-1.0-1", "hello-doc-1.0-1"]);
    assert!(!desc(&file.body, "hello-1.0-1").contains_key("PGPSIG"));
    let signature = get(&fixture, "/repository/pkgs/x86_64/custom.db.sig").await;
    assert!(testing::verify(&signature.body, &file.body, &public));
    assert_eq!(
        delete(&fixture, "/repository/pkgs/x86_64/pkgs.db.tar.gz")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    let doc_item = items
        .iter()
        .find(|item| item["name"] == "hello-doc")
        .unwrap();
    let id = doc_item["id"].as_str().unwrap();
    let removed = delete(&fixture, &format!("/service/rest/v1/components/{id}")).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&fixture, "/repository/pkgs/armv7h/pkgs.db")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let file = get(&fixture, "/repository/pkgs/x86_64/custom.db").await;
    assert_eq!(packages_of(&file.body), ["hello-1.0-1"]);
    let files = get(&fixture, "/repository/pkgs/x86_64/custom.files").await;
    assert_eq!(
        untar(&files.body)["hello-1.0-1/files"],
        b"%FILES%\nusr/\nusr/bin/\nusr/bin/hello\nusr/share/\nusr/share/doc/\nusr/share/doc/hello/\nusr/share/doc/hello/README\n"
    );
    let spec = get(&fixture, "/service/rest/v1/formats/pacman/upload-specs").await;
    assert_eq!(spec.status, StatusCode::OK, "{}", spec.text());
    assert_eq!(spec.json()["componentFields"][0]["name"], "directory");
    assert_eq!(spec.json()["multipleUpload"], true);
}

#[tokio::test]
async fn mirror_layouts_unsigned_databases_and_permissions() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plain", None)).await;
    let hello = package("hello", "1.0-1", "x86_64");
    let reply = upload(
        &fixture,
        "plain",
        "custom/os",
        &[("hello.pkg.tar.zst", &hello)],
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let db_path = "/repository/plain/custom/os/x86_64/custom.db";
    let file = get(&fixture, db_path).await;
    assert_eq!(file.status, StatusCode::OK, "{}", file.text());
    assert_eq!(packages_of(&file.body), ["hello-1.0-1"]);
    assert_eq!(
        get(
            &fixture,
            "/repository/plain/custom/os/x86_64/hello-1.0-1-x86_64.pkg.tar.zst"
        )
        .await
        .body,
        hello
    );
    assert_eq!(
        get(&fixture, &format!("{db_path}.sig")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/plain/plain.key").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/plain/x86_64/custom.db")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert!(
        !get(&fixture, "/repository/plain/")
            .await
            .text()
            .contains("pacman-key")
    );

    let anonymous = fixture.call(Method::GET, db_path, Auth::None, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    fixture.add_user("nobody", "nobody-password-1", &[]).await;
    assert_eq!(
        fixture
            .call(
                Method::GET,
                db_path,
                Auth::Basic("nobody", "nobody-password-1"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let reader = fixture
        .call(
            Method::GET,
            db_path,
            Auth::Basic("reader", "reader-password-1"),
            None,
        )
        .await;
    assert_eq!(reader.status, StatusCode::OK);
    assert_eq!(reader.body, file.body);
    let tool = package("tool", "1.0-1", "x86_64");
    let tool_path = "/repository/plain/x86_64/tool-1.0-1-x86_64.pkg.tar.zst";
    assert_eq!(
        put_as(
            &fixture,
            tool_path,
            tool.clone(),
            Auth::Basic("reader", "reader-password-1")
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    let enabled = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(enabled.status, StatusCode::OK);
    let open = fixture.call(Method::GET, db_path, Auth::None, None).await;
    assert_eq!(open.status, StatusCode::OK);
    assert_eq!(open.body, file.body);
    assert_eq!(
        put_as(&fixture, tool_path, tool.clone(), Auth::None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "deployer-role", "name": "deployer-role", "description": "", "privileges": ["as-repository-view-pacman-plain-add"], "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user("deployer", "deployer-password-1", &["deployer-role"])
        .await;
    let deployed = put_as(
        &fixture,
        tool_path,
        tool.clone(),
        Auth::Basic("deployer", "deployer-password-1"),
    )
    .await;
    assert_eq!(deployed.status, StatusCode::CREATED, "{}", deployed.text());
    assert_eq!(
        packages_of(
            &get(&fixture, "/repository/plain/x86_64/plain.db")
                .await
                .body
        ),
        ["tool-1.0-1"]
    );

    let key = testing::signing_key(Some("open sesame"));
    let mut changed = hosted("plain", None);
    changed["pacmanSigning"] = json!({"keypair": key, "passphrase": "open sesame"});
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/pacman/hosted/plain",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let signature = get(&fixture, &format!("{db_path}.sig")).await;
    assert_eq!(signature.status, StatusCode::OK);
    assert!(testing::verify(
        &signature.body,
        &get(&fixture, db_path).await.body,
        &testing::public_key(&key)
    ));
    let document = get(
        &fixture,
        "/service/rest/v1/repositories/pacman/hosted/plain",
    )
    .await;
    assert_eq!(document.json()["pacmanSigning"], json!({}));
    let mut wrong = hosted("plain", None);
    wrong["pacmanSigning"] =
        json!({"keypair": "-----BEGIN PRIVATE KEY-----\nMII\n-----END PRIVATE KEY-----\n"});
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/pacman/hosted/plain",
            Auth::Admin,
            Some(wrong),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
}

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Arc<Mutex<HashMap<String, usize>>>,
}

impl Upstream {
    fn hits(&self, path: &str) -> usize {
        self.hits.lock().unwrap().get(path).copied().unwrap_or(0)
    }
}

async fn upstream(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    *upstream
        .hits
        .lock()
        .unwrap()
        .entry(path.clone())
        .or_default() += 1;
    match upstream.files.lock().unwrap().get(&path) {
        Some(body) => body.clone().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn remote_database(packages: &[(&str, &str, &[u8])], variant: Variant) -> Vec<u8> {
    let records: Vec<Record> = packages
        .iter()
        .map(|(name, version, data)| {
            let entry = Entry {
                file_name: format!("{name}-{version}-x86_64.pkg.tar.zst"),
                name: name.to_string(),
                version: version.to_string(),
                description: format!("{name} from the mirror"),
                compressed_size: data.len() as u64,
                sha256: sha256(data),
                arch: "x86_64".to_owned(),
                build_date: 1_700_000_000,
                ..Entry::default()
            };
            Record {
                directory: entry.directory(),
                name: name.to_string(),
                members: vec![
                    (db::DESC.to_owned(), entry.desc().into_bytes()),
                    (
                        db::FILES.to_owned(),
                        format!("%FILES%\nusr/\nusr/bin/{name}\n").into_bytes(),
                    ),
                ],
            }
        })
        .collect();
    db::archive(&records, variant, 1_700_000_000).unwrap()
}

#[tokio::test]
async fn proxies_cache_mirrors_and_groups_merge_their_databases() {
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
    let mirror_key = testing::signing_key(None);
    let bash = package("bash", "5.2.037-1", "x86_64");
    let glibc = package("glibc", "2.40-1", "x86_64");
    let vim = package("vim", "9.1-1", "x86_64");
    let core_db = remote_database(
        &[("bash", "5.2.037-1", &bash), ("glibc", "2.40-1", &glibc)],
        Variant::Db,
    );
    let core_files = remote_database(
        &[("bash", "5.2.037-1", &bash), ("glibc", "2.40-1", &glibc)],
        Variant::Files,
    );
    let extra_db = remote_database(&[("vim", "9.1-1", &vim)], Variant::Db);
    let extra_signature = testing::sign(&mirror_key, &extra_db);
    let bash_signature = testing::sign(&mirror_key, &bash);
    let state = Upstream::default();
    {
        let mut files = state.files.lock().unwrap();
        files.insert("core/os/x86_64/core.db".to_owned(), core_db.clone());
        files.insert("core/os/x86_64/core.files".to_owned(), core_files.clone());
        files.insert(
            "core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst".to_owned(),
            bash.clone(),
        );
        files.insert(
            "core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst.sig".to_owned(),
            bash_signature.clone(),
        );
        files.insert(
            "core/os/x86_64/glibc-2.40-1-x86_64.pkg.tar.zst".to_owned(),
            glibc.clone(),
        );
        files.insert("extra/os/x86_64/extra.db".to_owned(), extra_db.clone());
        files.insert(
            "extra/os/x86_64/extra.db.sig".to_owned(),
            extra_signature.clone(),
        );
        files.insert("lastsync".to_owned(), b"1700000000\n".to_vec());
    }
    let app = Router::new()
        .route("/{*path}", routing::get(upstream))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let remote = format!("http://{address}/");
    create(&fixture, "proxy", proxy("mirror", &remote, None)).await;

    let core = "/repository/mirror/core/os/x86_64/core.db";
    let file = get(&fixture, core).await;
    assert_eq!(file.status, StatusCode::OK, "{}", file.text());
    assert_eq!(file.body, core_db);
    assert_eq!(get(&fixture, core).await.body, core_db);
    assert_eq!(state.hits("core/os/x86_64/core.db"), 1);
    assert_eq!(
        get(&fixture, &format!("{core}.sig")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/mirror/extra/os/x86_64/extra.db.sig")
            .await
            .body,
        extra_signature
    );
    let fetched = get(
        &fixture,
        "/repository/mirror/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst",
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, bash);
    assert_eq!(
        get(
            &fixture,
            "/repository/mirror/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst.sig"
        )
        .await
        .body,
        bash_signature
    );
    assert_eq!(
        get(&fixture, "/repository/mirror/lastsync").await.text(),
        "1700000000\n"
    );
    let components = get(&fixture, "/service/rest/v1/components?repository=mirror").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(
        (&items[0]["group"], &items[0]["name"], &items[0]["version"]),
        (&json!("x86_64"), &json!("bash"), &json!("5.2.037-1"))
    );
    let bash_asset = items[0]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"].as_str().unwrap().ends_with(".zst"))
        .unwrap();
    assert_eq!(bash_asset["pacman"]["name"], "bash");
    assert_eq!(
        put(
            &fixture,
            "/repository/mirror/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst",
            bash.clone()
        )
        .await
        .status,
        StatusCode::METHOD_NOT_ALLOWED
    );

    let proxy_key = testing::signing_key(None);
    create(
        &fixture,
        "proxy",
        proxy("signer", &remote, Some(json!({"keypair": proxy_key}))),
    )
    .await;
    let signed = get(&fixture, "/repository/signer/core/os/x86_64/core.db").await;
    let signature = get(&fixture, "/repository/signer/core/os/x86_64/core.db.sig").await;
    assert_eq!(signature.status, StatusCode::OK);
    assert!(testing::verify(
        &signature.body,
        &signed.body,
        &testing::public_key(&proxy_key)
    ));

    create(&fixture, "hosted", hosted("local", None)).await;
    let own_bash = package("bash", "5.3-1", "x86_64");
    let own_path = "/repository/local/core/os/x86_64/bash-5.3-1-x86_64.pkg.tar.zst";
    assert_eq!(
        put(&fixture, own_path, own_bash.clone()).await.status,
        StatusCode::CREATED
    );
    let mine = package("mine", "1.0-1", "x86_64");
    assert_eq!(
        put(
            &fixture,
            "/repository/local/x86_64/mine-1.0-1-x86_64.pkg.tar.zst",
            mine.clone()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let group_key = testing::signing_key(None);
    create(&fixture, "group", group("all", &["local", "mirror"], None)).await;
    create(
        &fixture,
        "group",
        group(
            "signed",
            &["local", "mirror"],
            Some(json!({"keypair": group_key})),
        ),
    )
    .await;

    let merged = get(&fixture, "/repository/all/core/os/x86_64/core.db").await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    assert_eq!(packages_of(&merged.body), ["bash-5.3-1", "glibc-2.40-1"]);
    assert_eq!(
        one(&desc(&merged.body, "bash-5.3-1"), "SHA256SUM"),
        sha256(&own_bash)
    );
    assert_eq!(
        one(&desc(&merged.body, "glibc-2.40-1"), "DESC"),
        "glibc from the mirror"
    );
    assert_eq!(
        get(&fixture, "/repository/all/core/os/x86_64/core.db")
            .await
            .body,
        merged.body
    );
    let merged_files = get(&fixture, "/repository/all/core/os/x86_64/core.files").await;
    let listed = untar(&merged_files.body);
    assert_eq!(
        listed["glibc-2.40-1/files"],
        b"%FILES%\nusr/\nusr/bin/glibc\n"
    );
    assert!(listed.contains_key("bash-5.3-1/files"));
    assert_eq!(
        get(&fixture, "/repository/all/core/os/x86_64/core.db.sig")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/all/core/os/x86_64/bash-5.3-1-x86_64.pkg.tar.zst"
        )
        .await
        .body,
        own_bash
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/all/core/os/x86_64/glibc-2.40-1-x86_64.pkg.tar.zst"
        )
        .await
        .body,
        glibc
    );
    let own = get(&fixture, "/repository/local/x86_64/mine.db").await;
    assert_eq!(
        get(&fixture, "/repository/all/x86_64/mine.db").await.body,
        own.body
    );
    assert_eq!(
        get(&fixture, "/repository/all/extra/os/x86_64/extra.db.sig")
            .await
            .body,
        extra_signature
    );
    let group_public = testing::public_key(&group_key);
    let signed_core = get(&fixture, "/repository/signed/core/os/x86_64/core.db").await;
    assert_eq!(untar(&signed_core.body), untar(&merged.body));
    assert!(testing::verify(
        &get(&fixture, "/repository/signed/core/os/x86_64/core.db.sig")
            .await
            .body,
        &signed_core.body,
        &group_public
    ));
    assert!(testing::verify(
        &get(&fixture, "/repository/signed/extra/os/x86_64/extra.db.sig")
            .await
            .body,
        &extra_db,
        &group_public
    ));
    assert_eq!(
        get(&fixture, "/repository/signed/signed.key").await.text(),
        group_public
    );
    assert_eq!(
        put(
            &fixture,
            "/repository/all/x86_64/mine-1.0-1-x86_64.pkg.tar.zst",
            mine
        )
        .await
        .status,
        StatusCode::METHOD_NOT_ALLOWED
    );

    let removed = delete(&fixture, own_path).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let merged = get(&fixture, "/repository/all/core/os/x86_64/core.db").await;
    assert_eq!(merged.body, core_db);
    assert_eq!(
        get(
            &fixture,
            "/repository/all/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst"
        )
        .await
        .body,
        bash
    );
}

async fn stored_in(fixture: &Fixture, repository: &str) -> i64 {
    use sea_orm::ConnectionTrait;
    fixture
        .state
        .db()
        .conn()
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            "SELECT count(*) AS n FROM asset WHERE repository = $1",
            [repository.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

async fn run_task(fixture: &Fixture, properties: Value) -> Value {
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({
                "type": "repository.pacman.rebuild.metadata",
                "name": format!("Rebuild pacman {}", properties["repositoryName"]),
                "properties": properties,
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
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
    get(fixture, &format!("/service/rest/v1/tasks/{id}"))
        .await
        .json()
}

#[tokio::test]
async fn the_rebuild_task_rewrites_hosted_databases_and_resets_merged_ones() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("one", None)).await;
    create(&fixture, "hosted", hosted("two", None)).await;
    create(&fixture, "group", group("both", &["one", "two"], None)).await;
    for (repository, name) in [("one", "alpha"), ("two", "beta")] {
        let reply = put(
            &fixture,
            &format!("/repository/{repository}/x86_64/{name}-1.0-1-x86_64.pkg.tar.zst"),
            package(name, "1.0-1", "x86_64"),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    let before = get(&fixture, "/repository/one/x86_64/one.files").await;
    let merged = get(&fixture, "/repository/both/x86_64/both.db").await;
    assert_eq!(packages_of(&merged.body), ["alpha-1.0-1", "beta-1.0-1"]);
    assert_eq!(stored_in(&fixture, "both").await, 1);
    let task = run_task(&fixture, json!({"repositoryName": "one"})).await;
    assert_eq!(task["lastRunResult"], "OK", "{task}");
    let after = get(&fixture, "/repository/one/x86_64/one.files").await;
    assert_eq!(untar(&after.body), untar(&before.body));
    let task = run_task(
        &fixture,
        json!({"repositoryName": "*", "resetProxyMetadata": "true"}),
    )
    .await;
    assert_eq!(task["lastRunResult"], "OK", "{task}");
    assert_eq!(stored_in(&fixture, "both").await, 0);
    let again = get(&fixture, "/repository/both/x86_64/both.db").await;
    assert_eq!(packages_of(&again.body), ["alpha-1.0-1", "beta-1.0-1"]);
    let templates = get(&fixture, "/service/rest/v1/tasks/templates")
        .await
        .json();
    assert!(
        templates
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["type"] == "repository.pacman.rebuild.metadata"),
        "{templates}"
    );
}
