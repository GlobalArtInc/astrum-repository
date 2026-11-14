mod support;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const HOSTED: &str = "jsr-hosted";
const MIB: usize = 1024 * 1024;

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_jsr::Jsr::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/jsr/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
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
        "proxy": {"remoteUrl": remote, "contentMaxAge": 0, "metadataMaxAge": 0},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": true},
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

async fn token(fixture: &Fixture, scope: Scope) -> String {
    fixture
        .state
        .security()
        .create_token("admin", "jsr", scope, None)
        .await
        .unwrap()
        .1
}

fn sha(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn gzip(raw: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

fn regular(path: &str, data: &[u8]) -> (tar::Header, Vec<u8>) {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    header.as_old_mut().name[..path.len()].copy_from_slice(path.as_bytes());
    header.set_cksum();
    (header, data.to_vec())
}

fn tar_gz(entries: Vec<(tar::Header, Vec<u8>)>) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (header, data) in entries {
        builder.append(&header, data.as_slice()).unwrap();
    }
    gzip(&builder.into_inner().unwrap())
}

fn deno_json(scope: &str, name: &str, version: &str) -> String {
    json!({"name": format!("@{scope}/{name}"), "version": version, "exports": {".": "./mod.ts", "./util": "./util.js"}})
        .to_string()
}

fn mod_ts(tag: &str) -> Vec<u8> {
    format!("export const tag = \"{tag}\";\n").into_bytes()
}

fn package(scope: &str, name: &str, version: &str, tag: &str) -> Vec<u8> {
    tar_gz(vec![
        regular("/deno.json", deno_json(scope, name, version).as_bytes()),
        regular("/mod.ts", &mod_ts(tag)),
        regular("/util.js", b"export default 1;\n"),
        regular("/view.tsx", b"export const v = <div/>;\n"),
        regular("/view.jsx", b"export const v = <div/>;\n"),
        regular("/README.md", b"# hello\n"),
    ])
}

async fn post(
    fixture: &Fixture,
    path: &str,
    credential: Option<&str>,
    body: Vec<u8>,
    encoding: bool,
    host: Option<&str>,
) -> Reply {
    let bearer = credential.map(|token| format!("Bearer {token}"));
    let mut headers: Vec<(&str, &str)> = Vec::new();
    if let Some(value) = bearer.as_deref() {
        headers.push(("authorization", value));
    }
    if encoding {
        headers.push(("content-encoding", "gzip"));
    }
    if let Some(host) = host {
        headers.push(("host", host));
    }
    fixture
        .send_body(Method::POST, path, Auth::None, Body::from(body), &headers)
        .await
}

fn publish_path(repository: &str, scope: &str, name: &str, version: &str) -> String {
    format!(
        "/repository/{repository}/api/scopes/{scope}/packages/{name}/versions/{version}?config=/deno.json"
    )
}

async fn publish(
    fixture: &Fixture,
    repository: &str,
    credential: &str,
    (scope, name, version): (&str, &str, &str),
    body: Vec<u8>,
) -> Reply {
    post(
        fixture,
        &publish_path(repository, scope, name, version),
        Some(credential),
        body,
        true,
        None,
    )
    .await
}

async fn get(fixture: &Fixture, path: &str, credential: Option<&str>) -> Reply {
    let bearer = credential.map(|token| format!("Bearer {token}"));
    let mut headers: Vec<(&str, &str)> = Vec::new();
    if let Some(value) = bearer.as_deref() {
        headers.push(("authorization", value));
    }
    fixture
        .send_body(Method::GET, path, Auth::None, Body::empty(), &headers)
        .await
}

fn versions(listing: &Value) -> Vec<String> {
    listing["versions"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

#[tokio::test]
async fn publishing_stores_the_files_and_the_client_paths_serve_them() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let target = ("acme", "hello", "0.1.0");

    let published = publish(
        &fixture,
        HOSTED,
        &writer,
        target,
        package("acme", "hello", "0.1.0", "a"),
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    let task = published.json();
    assert_eq!(task["status"], "success");
    assert_eq!(task["error"], Value::Null);
    assert_eq!(task["packageScope"], "acme");
    assert_eq!(task["packageName"], "hello");
    assert_eq!(task["packageVersion"], "0.1.0");
    assert!(task["id"].as_str().is_some() && task["createdAt"].as_str().is_some());

    let rc = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "0.2.0-rc.1"),
        package("acme", "hello", "0.2.0-rc.1", "rc"),
    )
    .await;
    assert_eq!(rc.status, StatusCode::OK, "{}", rc.text());

    let listing = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK);
    assert_eq!(listing.headers["content-type"], "application/json");
    let document = listing.json();
    assert_eq!(document["scope"], "acme");
    assert_eq!(document["name"], "hello");
    assert_eq!(
        document["latest"], "0.1.0",
        "a prerelease is never the latest"
    );
    assert_eq!(versions(&document), ["0.1.0", "0.2.0-rc.1"]);
    assert_eq!(document["versions"]["0.1.0"], json!({}));
    let etag = listing.headers["etag"].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send_body(
            Method::GET,
            &format!("/repository/{HOSTED}/@acme/hello/meta.json"),
            Auth::None,
            Body::empty(),
            &[
                ("authorization", &format!("Bearer {writer}")),
                ("if-none-match", &etag),
            ],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let meta = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/0.1.0_meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(meta.status, StatusCode::OK);
    assert_eq!(meta.headers["content-type"], "application/json");
    let meta = meta.json();
    assert_eq!(
        meta["exports"],
        json!({".": "./mod.ts", "./util": "./util.js"})
    );
    let manifest = meta["manifest"].as_object().unwrap();
    assert_eq!(manifest.len(), 6);
    let types = BTreeMap::from([
        ("/mod.ts", "application/typescript"),
        ("/util.js", "application/javascript"),
        ("/view.tsx", "text/tsx"),
        ("/view.jsx", "text/jsx"),
        ("/deno.json", "application/json"),
        ("/README.md", "application/octet-stream"),
    ]);
    for (file, entry) in manifest {
        let served = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hello/0.1.0{file}"),
            Some(&writer),
        )
        .await;
        assert_eq!(served.status, StatusCode::OK, "{file}");
        assert_eq!(
            served.headers["content-type"],
            types[file.as_str()],
            "{file}"
        );
        assert_eq!(
            entry["checksum"],
            format!("sha256-{}", sha(&served.body)),
            "{file}"
        );
        assert_eq!(entry["size"], served.body.len(), "{file}");
    }
    let module = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/0.1.0/mod.ts"),
        Some(&writer),
    )
    .await;
    assert_eq!(module.body, mod_ts("a"));

    let status = get(
        &fixture,
        &format!("/repository/{HOSTED}/api/publish_status/abc"),
        None,
    )
    .await;
    assert_eq!(status.status, StatusCode::NOT_FOUND);
    assert_eq!(status.json()["code"], "notFound");
}

#[tokio::test]
async fn a_gzip_content_encoding_is_not_decompressed_by_a_layer() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    for (version, encoding) in [("1.0.0", true), ("1.0.1", false)] {
        let reply = post(
            &fixture,
            &publish_path(HOSTED, "acme", "hello", version),
            Some(&writer),
            package("acme", "hello", version, "x"),
            encoding,
            None,
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{version}: {}", reply.text());
    }
}

#[tokio::test]
async fn a_published_version_is_never_replaced() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let target = ("acme", "hello", "0.1.0");
    let first = publish(
        &fixture,
        HOSTED,
        &writer,
        target,
        package("acme", "hello", "0.1.0", "first"),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    let again = publish(
        &fixture,
        HOSTED,
        &writer,
        target,
        package("acme", "hello", "0.1.0", "second"),
    )
    .await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    let body = again.json();
    assert_eq!(body["code"], "duplicateVersionPublish");
    assert_eq!(body["task"]["status"], "success");
    let admin = fixture
        .send_body(
            Method::POST,
            &publish_path(HOSTED, "acme", "hello", "0.1.0"),
            Auth::Admin,
            Body::from(package("acme", "hello", "0.1.0", "third")),
            &[("content-encoding", "gzip")],
        )
        .await;
    assert_eq!(admin.json()["code"], "duplicateVersionPublish");
    let stored = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/0.1.0/mod.ts"),
        Some(&writer),
    )
    .await;
    assert_eq!(stored.body, mod_ts("first"));
}

#[tokio::test]
async fn racing_publishes_of_one_version_have_one_winner() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let target = ("acme", "race", "1.0.0");
    let (one, two) = tokio::join!(
        publish(
            &fixture,
            HOSTED,
            &writer,
            target,
            package("acme", "race", "1.0.0", "one")
        ),
        publish(
            &fixture,
            HOSTED,
            &writer,
            target,
            package("acme", "race", "1.0.0", "two")
        ),
    );
    let mut statuses = [one.status, two.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::BAD_REQUEST]);
    let loser = if one.status == StatusCode::OK {
        &two
    } else {
        &one
    };
    assert_eq!(loser.json()["code"], "duplicateVersionPublish");
    let winner_tag = if one.status == StatusCode::OK {
        "one"
    } else {
        "two"
    };
    let stored = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/race/1.0.0/mod.ts"),
        Some(&writer),
    )
    .await;
    assert_eq!(stored.body, mod_ts(winner_tag));
    let meta = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/race/1.0.0_meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(
        meta.json()["manifest"]["/mod.ts"]["checksum"],
        format!("sha256-{}", sha(&mod_ts(winner_tag)))
    );
}

fn symlink_archive() -> Vec<u8> {
    let mut link = tar::Header::new_gnu();
    link.set_entry_type(tar::EntryType::Symlink);
    link.set_size(0);
    link.as_old_mut().name[..6].copy_from_slice(b"mod.ts");
    link.set_link_name("/etc/passwd").unwrap();
    link.set_cksum();
    tar_gz(vec![
        regular("deno.json", deno_json("acme", "bad", "1.0.0").as_bytes()),
        (link, Vec::new()),
    ])
}

fn bomb_archive() -> Vec<u8> {
    let size = 100 * MIB as u64 + 1;
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_size(size);
    header.as_old_mut().name[..9].copy_from_slice(b"zeros.bin");
    header.set_cksum();
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
    builder
        .append(&header, std::io::repeat(0).take(size))
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

#[tokio::test]
async fn an_untrusted_archive_is_refused_before_anything_is_stored() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let config = || regular("deno.json", deno_json("acme", "bad", "1.0.0").as_bytes());
    let module = || regular("mod.ts", b"export {};\n");
    let many: Vec<(tar::Header, Vec<u8>)> = std::iter::once(config())
        .chain((0..2000).map(|number| regular(&format!("f{number}.ts"), b"")))
        .collect();
    let cases: Vec<(&str, Vec<u8>, StatusCode, &str)> = vec![
        (
            "symlink",
            symlink_archive(),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
        (
            "traversal",
            tar_gz(vec![config(), module(), regular("../evil.ts", b"x")]),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
        (
            "absolute",
            tar_gz(vec![config(), module(), regular("//etc/passwd", b"x")]),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
        (
            "duplicate",
            tar_gz(vec![config(), module(), module()]),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
        (
            "2001 entries",
            tar_gz(many),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
        (
            "bomb",
            bomb_archive(),
            StatusCode::PAYLOAD_TOO_LARGE,
            "tarballTooLarge",
        ),
        (
            "over 20 MiB",
            vec![0u8; 20 * MIB + 1],
            StatusCode::PAYLOAD_TOO_LARGE,
            "tarballTooLarge",
        ),
        (
            "not gzip",
            b"plain text".to_vec(),
            StatusCode::BAD_REQUEST,
            "invalidTarball",
        ),
    ];
    for (label, body, status, code) in cases {
        let reply = publish(&fixture, HOSTED, &writer, ("acme", "bad", "1.0.0"), body).await;
        assert_eq!(reply.status, status, "{label}: {}", reply.text());
        assert_eq!(reply.json()["code"], code, "{label}");
    }
    let nothing = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/bad/meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
    let components = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/components?repository={HOSTED}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(components.json()["items"], json!([]));
}

#[tokio::test]
async fn names_versions_and_the_config_must_agree() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let archive = |config: Value, extra: &[(&str, &[u8])]| {
        let mut entries = vec![
            regular("deno.json", config.to_string().as_bytes()),
            regular("mod.ts", b"x"),
        ];
        entries.extend(extra.iter().map(|(path, data)| regular(path, data)));
        tar_gz(entries)
    };
    let named = |name: &str, version: &str| json!({"name": name, "version": version, "exports": "./mod.ts"});
    let wrong_name = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        archive(named("@acme/other", "1.0.0"), &[]),
    )
    .await;
    let wrong_version = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        archive(named("@acme/hello", "1.0.1"), &[]),
    )
    .await;
    let missing_target = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        archive(
            json!({"name": "@acme/hello", "version": "1.0.0", "exports": "./gone.ts"}),
            &[],
        ),
    )
    .await;
    let no_config = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        tar_gz(vec![regular("mod.ts", b"x")]),
    )
    .await;
    for (label, reply, code) in [
        ("name", &wrong_name, "invalidConfig"),
        ("version", &wrong_version, "invalidConfig"),
        ("target", &missing_target, "invalidConfig"),
        ("config", &no_config, "invalidConfig"),
    ] {
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{label}: {}",
            reply.text()
        );
        assert_eq!(reply.json()["code"], code, "{label}");
    }
    let bad_version = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0"),
        archive(named("@acme/hello", "1.0"), &[]),
    )
    .await;
    assert_eq!(bad_version.json()["code"], "invalidVersion");
    for (scope, name) in [
        ("a", "hello"),
        ("-acme", "hello"),
        ("Acme", "hello"),
        ("acme", "-x"),
        ("acme", "A"),
    ] {
        let reply = publish(
            &fixture,
            HOSTED,
            &writer,
            (scope, name, "1.0.0"),
            archive(named("@acme/hello", "1.0.0"), &[]),
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{scope}/{name}");
        assert_eq!(reply.json()["code"], "invalidPackageName", "{scope}/{name}");
    }
    let fine = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        archive(named("@acme/hello", "1.0.0"), &[]),
    )
    .await;
    assert_eq!(fine.status, StatusCode::OK, "{}", fine.text());
}

async fn selector_user(
    fixture: &Fixture,
    user: &str,
    repository: &str,
    actions: Value,
    expression: &str,
) {
    let steps = [
        (
            "/service/rest/v1/security/content-selectors".to_owned(),
            json!({"name": format!("{user}-selector"), "description": "", "expression": expression}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector".to_owned(),
            json!({"name": format!("{user}-privilege"), "description": "", "actions": actions, "format": "jsr", "repository": repository, "contentSelector": format!("{user}-selector")}),
        ),
        (
            "/service/rest/v1/security/roles".to_owned(),
            json!({"id": format!("{user}-role"), "name": format!("{user}-role"), "description": "", "privileges": [format!("{user}-privilege")], "roles": []}),
        ),
    ];
    for (path, body) in steps {
        let reply = fixture
            .call(Method::POST, &path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.text());
    }
    fixture
        .add_user(user, "selected-pass", &[&format!("{user}-role")])
        .await;
}

#[tokio::test]
async fn reads_and_publishes_follow_the_rights_of_the_caller() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let reader = token(&fixture, Scope::Read).await;
    let target = ("acme", "hello", "0.1.0");
    assert_eq!(
        publish(
            &fixture,
            HOSTED,
            &writer,
            target,
            package("acme", "hello", "0.1.0", "a")
        )
        .await
        .status,
        StatusCode::OK
    );

    let denied = publish(
        &fixture,
        HOSTED,
        &reader,
        ("acme", "hello", "0.2.0"),
        package("acme", "hello", "0.2.0", "b"),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let anonymous = post(
        &fixture,
        &publish_path(HOSTED, "acme", "hello", "0.2.0"),
        None,
        package("acme", "hello", "0.2.0", "b"),
        true,
        None,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["code"], "unauthorized");

    let file = format!("/repository/{HOSTED}/@acme/hello/0.1.0/mod.ts");
    assert_eq!(
        get(&fixture, &file, None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert!(
        get(&fixture, &file, None)
            .await
            .headers
            .contains_key(header::WWW_AUTHENTICATE)
    );
    assert_eq!(
        get(&fixture, &file, Some(&reader)).await.status,
        StatusCode::OK
    );

    let probe =
        |name: &'static str| format!("/repository/{HOSTED}/api/scopes/acme/packages/{name}");
    let known_to_reader = get(&fixture, &probe("hello"), Some(&reader)).await;
    assert_eq!(known_to_reader.status, StatusCode::OK);
    assert_eq!(
        known_to_reader.json(),
        json!({"scope": "acme", "name": "hello"})
    );
    assert_eq!(
        get(&fixture, &probe("fresh"), Some(&writer)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &probe("fresh"), Some(&reader)).await.status,
        StatusCode::NOT_FOUND
    );
    let probe_anonymous = get(&fixture, &probe("hello"), None).await;
    assert_eq!(probe_anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(
        probe_anonymous
            .headers
            .contains_key(header::WWW_AUTHENTICATE)
    );

    selector_user(
        &fixture,
        "metaonly",
        HOSTED,
        json!(["ADD", "READ", "BROWSE"]),
        "format == \"jsr\" && path == \"/@acme/narrow/1.0.0_meta.json\"",
    )
    .await;
    let narrow = fixture
        .send_body(
            Method::POST,
            &publish_path(HOSTED, "acme", "narrow", "1.0.0"),
            Auth::Basic("metaonly", "selected-pass"),
            Body::from(package("acme", "narrow", "1.0.0", "n")),
            &[("content-encoding", "gzip")],
        )
        .await;
    assert_eq!(narrow.status, StatusCode::FORBIDDEN, "{}", narrow.text());
    let nothing = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/narrow/1.0.0_meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn selectors_hide_the_versions_a_caller_may_not_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    for version in ["0.1.0", "0.2.0"] {
        let reply = publish(
            &fixture,
            HOSTED,
            &writer,
            ("acme", "hello", version),
            package("acme", "hello", version, version),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK);
    }
    selector_user(
        &fixture,
        "selected",
        HOSTED,
        json!(["READ", "BROWSE"]),
        "format == \"jsr\" && (path == \"/@acme/hello/meta.json\" || path =^ \"/@acme/hello/0.1.0\")",
    )
    .await;
    let selected = || Auth::Basic("selected", "selected-pass");
    let listing = fixture
        .send(
            Method::GET,
            &format!("/repository/{HOSTED}/@acme/hello/meta.json"),
            selected(),
            None,
            &[],
        )
        .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(versions(&listing.json()), ["0.1.0"]);
    assert_eq!(listing.json()["latest"], "0.1.0");
    let visible = fixture
        .send(
            Method::GET,
            &format!("/repository/{HOSTED}/@acme/hello/0.1.0/mod.ts"),
            selected(),
            None,
            &[],
        )
        .await;
    assert_eq!(visible.status, StatusCode::OK);
    let hidden = fixture
        .send(
            Method::GET,
            &format!("/repository/{HOSTED}/@acme/hello/0.2.0/mod.ts"),
            selected(),
            None,
            &[],
        )
        .await;
    assert_eq!(hidden.status, StatusCode::FORBIDDEN);
}

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    seen: Arc<Mutex<Vec<(String, String)>>>,
}

impl Upstream {
    fn publish(&self, path: &str, body: impl Into<Vec<u8>>) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_owned(), body.into());
    }

    fn accepts(&self, path: &str) -> Vec<String> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(seen, _)| seen == path)
            .map(|(_, accept)| accept.clone())
            .collect()
    }

    fn asked(&self, prefix: &str) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .count()
    }

    fn package(&self, scope: &str, name: &str, version: &str, tag: &str) {
        self.publish(
            &format!("@{scope}/{name}/meta.json"),
            json!({"scope": scope, "name": name, "latest": version, "versions": {version: {}}})
                .to_string(),
        );
        self.publish(
            &format!("@{scope}/{name}/{version}_meta.json"),
            json!({"manifest": {}, "exports": {".": "./mod.ts"}}).to_string(),
        );
        self.publish(&format!("@{scope}/{name}/{version}/mod.ts"), mod_ts(tag));
    }
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    upstream
        .seen
        .lock()
        .unwrap()
        .push((path.clone(), accept.clone()));
    if accept.contains("text/html") {
        return ([(header::CONTENT_TYPE, "text/html")], "<html>a page</html>").into_response();
    }
    let found = upstream.files.lock().unwrap().get(&path).cloned();
    match found {
        Some(body) => ([(header::CONTENT_TYPE, "application/octet-stream")], body).into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn serve_upstream(fixture: &Fixture) -> (Upstream, SocketAddr) {
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
    let upstream = Upstream::default();
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (upstream, address)
}

#[tokio::test]
async fn proxies_ask_the_remote_for_json_and_never_for_html() {
    let fixture = ready().await;
    let (upstream, address) = serve_upstream(&fixture).await;
    upstream.package("std", "path", "1.0.0", "std");
    create(
        &fixture,
        "proxy",
        proxy("jsr-proxy", &format!("http://{address}")),
    )
    .await;
    let admin = |path: &str| {
        let path = path.to_owned();
        let fixture = &fixture;
        async move {
            fixture
                .send(Method::GET, &path, Auth::Admin, None, &[])
                .await
        }
    };

    let listing = admin("/repository/jsr-proxy/@std/path/meta.json").await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(listing.headers["content-type"], "application/json");
    assert_eq!(versions(&listing.json()), ["1.0.0"]);
    let meta = admin("/repository/jsr-proxy/@std/path/1.0.0_meta.json").await;
    assert_eq!(meta.status, StatusCode::OK);
    let file = admin("/repository/jsr-proxy/@std/path/1.0.0/mod.ts").await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, mod_ts("std"));
    assert_eq!(file.headers["content-type"], "application/typescript");
    assert_eq!(
        admin("/repository/jsr-proxy/@std/path/9.9.9/mod.ts")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    assert_eq!(
        upstream.accepts("@std/path/meta.json"),
        ["application/json"]
    );
    assert_eq!(
        upstream.accepts("@std/path/1.0.0_meta.json"),
        ["application/json"]
    );
    assert_eq!(upstream.accepts("@std/path/1.0.0/mod.ts"), ["*/*"]);
    assert!(
        upstream
            .seen
            .lock()
            .unwrap()
            .iter()
            .all(|(_, accept)| !accept.contains("text/html"))
    );

    let refused = post(
        &fixture,
        &publish_path("jsr-proxy", "acme", "hello", "0.1.0"),
        Some(&token(&fixture, Scope::Write).await),
        package("acme", "hello", "0.1.0", "a"),
        true,
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn groups_merge_listings_and_hosted_names_shadow_the_proxies() {
    let fixture = ready().await;
    let (upstream, address) = serve_upstream(&fixture).await;
    upstream.package("std", "path", "1.0.0", "std");
    upstream.package("acme", "lib", "0.5.0", "remote");
    create(&fixture, "hosted", hosted("jsr-a", "ALLOW")).await;
    create(&fixture, "hosted", hosted("jsr-b", "ALLOW")).await;
    create(
        &fixture,
        "proxy",
        proxy("jsr-proxy", &format!("http://{address}")),
    )
    .await;
    create(
        &fixture,
        "group",
        group("jsr-group", &["jsr-a", "jsr-b", "jsr-proxy"]),
    )
    .await;
    let writer = token(&fixture, Scope::Write).await;
    for (repository, version, tag) in [
        ("jsr-a", "1.0.0", "a1"),
        ("jsr-a", "2.0.0", "a2"),
        ("jsr-b", "1.0.0", "b1"),
        ("jsr-b", "3.0.0", "b3"),
    ] {
        let reply = publish(
            &fixture,
            repository,
            &writer,
            ("acme", "lib", version),
            package("acme", "lib", version, tag),
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "{repository} {version}: {}",
            reply.text()
        );
    }

    let listing = get(
        &fixture,
        "/repository/jsr-group/@acme/lib/meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(versions(&listing.json()), ["1.0.0", "2.0.0", "3.0.0"]);
    assert_eq!(listing.json()["latest"], "3.0.0");
    assert_eq!(
        get(
            &fixture,
            "/repository/jsr-group/@acme/lib/1.0.0/mod.ts",
            Some(&writer)
        )
        .await
        .body,
        mod_ts("a1")
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/jsr-group/@acme/lib/3.0.0/mod.ts",
            Some(&writer)
        )
        .await
        .body,
        mod_ts("b3")
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/jsr-group/@acme/lib/0.5.0/mod.ts",
            Some(&writer)
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        upstream.asked("@acme"),
        0,
        "a name a hosted member knows is never asked of the proxy"
    );

    let remote = get(
        &fixture,
        "/repository/jsr-group/@std/path/meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(remote.status, StatusCode::OK, "{}", remote.text());
    assert_eq!(versions(&remote.json()), ["1.0.0"]);
    assert_eq!(
        get(
            &fixture,
            "/repository/jsr-group/@std/path/1.0.0/mod.ts",
            Some(&writer)
        )
        .await
        .body,
        mod_ts("std")
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/jsr-group/@nobody/none/meta.json",
            Some(&writer)
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let without_writable = publish(
        &fixture,
        "jsr-group",
        &writer,
        ("acme", "lib", "4.0.0"),
        package("acme", "lib", "4.0.0", "g"),
    )
    .await;
    assert_eq!(without_writable.status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(without_writable.headers.contains_key(header::ALLOW));
    assert!(
        without_writable.json()["message"]
            .as_str()
            .unwrap()
            .contains("no writable member")
    );
    let known = get(
        &fixture,
        "/repository/jsr-group/api/scopes/acme/packages/lib",
        Some(&writer),
    )
    .await;
    assert_eq!(known.status, StatusCode::OK);
    let unknown = get(
        &fixture,
        "/repository/jsr-group/api/scopes/acme/packages/none",
        Some(&writer),
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

fn writable_group(name: &str, members: &[&str], writable: &str) -> Value {
    let mut body = group(name, members);
    body["group"]["writableMember"] = json!(writable);
    body
}

async fn group_with_writable_member() -> (Fixture, String) {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("jsr-a", "ALLOW")).await;
    create(&fixture, "hosted", hosted("jsr-b", "ALLOW")).await;
    create(
        &fixture,
        "group",
        writable_group("jsr-wgroup", &["jsr-a", "jsr-b"], "jsr-b"),
    )
    .await;
    let writer = token(&fixture, Scope::Write).await;
    (fixture, writer)
}

#[tokio::test]
async fn a_group_forwards_publishing_to_its_writable_member() {
    let (fixture, writer) = group_with_writable_member().await;
    let target = ("acme", "hello", "0.1.0");
    let published = publish(
        &fixture,
        "jsr-wgroup",
        &writer,
        target,
        package("acme", "hello", "0.1.0", "via-group"),
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    assert_eq!(published.json()["status"], "success");

    let from_group = get(
        &fixture,
        "/repository/jsr-wgroup/@acme/hello/0.1.0/mod.ts",
        Some(&writer),
    )
    .await;
    assert_eq!(from_group.body, mod_ts("via-group"));
    let listing = get(
        &fixture,
        "/repository/jsr-wgroup/@acme/hello/meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(versions(&listing.json()), ["0.1.0"]);
    let member = get(
        &fixture,
        "/repository/jsr-b/@acme/hello/0.1.0/mod.ts",
        Some(&writer),
    )
    .await;
    assert_eq!(member.body, mod_ts("via-group"));
    let other = get(
        &fixture,
        "/repository/jsr-a/@acme/hello/meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(other.status, StatusCode::NOT_FOUND);

    let again = publish(
        &fixture,
        "jsr-wgroup",
        &writer,
        target,
        package("acme", "hello", "0.1.0", "second"),
    )
    .await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert_eq!(again.json()["code"], "duplicateVersionPublish");
    let kept = get(
        &fixture,
        "/repository/jsr-wgroup/@acme/hello/0.1.0/mod.ts",
        Some(&writer),
    )
    .await;
    assert_eq!(kept.body, mod_ts("via-group"));
}

#[tokio::test]
async fn publishing_through_a_group_needs_the_add_right_on_the_group() {
    let (fixture, writer) = group_with_writable_member().await;
    let reader = token(&fixture, Scope::Read).await;
    let denied = publish(
        &fixture,
        "jsr-wgroup",
        &reader,
        ("acme", "hello", "0.1.0"),
        package("acme", "hello", "0.1.0", "a"),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN, "{}", denied.text());
    let anonymous = post(
        &fixture,
        &publish_path("jsr-wgroup", "acme", "hello", "0.1.0"),
        None,
        package("acme", "hello", "0.1.0", "a"),
        true,
        None,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));

    selector_user(
        &fixture,
        "onmember",
        "jsr-b",
        json!(["ADD", "READ", "BROWSE"]),
        "format == \"jsr\" && path =^ \"/@acme/\"",
    )
    .await;
    selector_user(
        &fixture,
        "ongroup",
        "jsr-wgroup",
        json!(["ADD"]),
        "format == \"jsr\" && path =^ \"/@acme/\"",
    )
    .await;
    let as_user = |user: &'static str, version: &'static str| {
        let fixture = &fixture;
        async move {
            fixture
                .send_body(
                    Method::POST,
                    &publish_path("jsr-wgroup", "acme", "hello", version),
                    Auth::Basic(user, "selected-pass"),
                    Body::from(package("acme", "hello", version, user)),
                    &[("content-encoding", "gzip")],
                )
                .await
        }
    };
    let wrong_member = as_user("onmember", "0.2.0").await;
    assert_eq!(wrong_member.status, StatusCode::FORBIDDEN);
    let nothing = get(
        &fixture,
        "/repository/jsr-b/@acme/hello/0.2.0_meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
    let on_group = as_user("ongroup", "0.3.0").await;
    assert_eq!(on_group.status, StatusCode::OK, "{}", on_group.text());
    let stored = get(
        &fixture,
        "/repository/jsr-b/@acme/hello/0.3.0/mod.ts",
        Some(&writer),
    )
    .await;
    assert_eq!(stored.body, mod_ts("ongroup"));
}

#[tokio::test]
async fn a_group_without_a_writable_member_takes_no_publishes() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("jsr-a", "ALLOW")).await;
    create(&fixture, "group", group("jsr-plain", &["jsr-a"])).await;
    let writer = token(&fixture, Scope::Write).await;
    let refused = publish(
        &fixture,
        "jsr-plain",
        &writer,
        ("acme", "hello", "0.1.0"),
        package("acme", "hello", "0.1.0", "a"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        refused.json()["message"]
            .as_str()
            .unwrap()
            .contains("no writable member")
    );
    let nothing = get(
        &fixture,
        "/repository/jsr-a/@acme/hello/meta.json",
        Some(&writer),
    )
    .await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_package_probe_of_a_group_follows_its_members_and_the_writable_member() {
    let (fixture, writer) = group_with_writable_member().await;
    let reader = token(&fixture, Scope::Read).await;
    let probe = |name: &str| format!("/repository/jsr-wgroup/api/scopes/acme/packages/{name}");
    let fresh_for_writer = get(&fixture, &probe("fresh"), Some(&writer)).await;
    assert_eq!(fresh_for_writer.status, StatusCode::OK);
    assert_eq!(
        fresh_for_writer.json(),
        json!({"scope": "acme", "name": "fresh"})
    );
    assert_eq!(
        get(&fixture, &probe("fresh"), Some(&reader)).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        publish(
            &fixture,
            "jsr-wgroup",
            &writer,
            ("acme", "fresh", "1.0.0"),
            package("acme", "fresh", "1.0.0", "f")
        )
        .await
        .status,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &probe("fresh"), Some(&reader)).await.status,
        StatusCode::OK
    );
    let anonymous = get(&fixture, &probe("fresh"), None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn a_root_domain_takes_the_publish_api_calls_without_a_path_prefix() {
    let fixture = ready().await;
    let mut body = hosted(HOSTED, "ALLOW");
    body["domain"] = json!({"host": "jsr.example.test"});
    create(&fixture, "hosted", body).await;
    let writer = token(&fixture, Scope::Write).await;
    let host = Some("jsr.example.test:8081");
    let probe = fixture
        .send_body(
            Method::GET,
            "/api/scopes/acme/packages/hello",
            Auth::None,
            Body::empty(),
            &[
                ("host", "jsr.example.test"),
                ("authorization", &format!("Bearer {writer}")),
            ],
        )
        .await;
    assert_eq!(probe.status, StatusCode::OK, "{}", probe.text());
    let published = post(
        &fixture,
        "/api/scopes/acme/packages/hello/versions/0.1.0?config=/deno.json",
        Some(&writer),
        package("acme", "hello", "0.1.0", "root"),
        true,
        host,
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    let read = fixture
        .send_body(
            Method::GET,
            "/@acme/hello/0.1.0/mod.ts",
            Auth::None,
            Body::empty(),
            &[
                ("host", "jsr.example.test"),
                ("authorization", &format!("Bearer {writer}")),
            ],
        )
        .await;
    assert_eq!(read.body, mod_ts("root"));
}

#[tokio::test]
async fn read_only_repositories_and_other_methods_refuse_writes() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("jsr-frozen", "DENY")).await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let frozen = publish(
        &fixture,
        "jsr-frozen",
        &writer,
        ("acme", "hello", "0.1.0"),
        package("acme", "hello", "0.1.0", "a"),
    )
    .await;
    assert_eq!(frozen.status, StatusCode::BAD_REQUEST);
    assert!(
        frozen.json()["message"]
            .as_str()
            .unwrap()
            .contains("read-only")
    );
    for path in ["@acme/hello/meta.json", "@acme/hello/0.1.0/mod.ts"] {
        let put = fixture
            .send_body(
                Method::PUT,
                &format!("/repository/{HOSTED}/{path}"),
                Auth::None,
                Body::from("x"),
                &[("authorization", &format!("Bearer {writer}"))],
            )
            .await;
        assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED, "{path}");
    }
    let authorization = fixture
        .call(
            Method::POST,
            &format!("/repository/{HOSTED}/api/authorizations"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(authorization.status, StatusCode::NOT_FOUND);
}

async fn store_ctx(fixture: &Fixture, repository: &str) -> astrum_content::Ctx {
    let repository = fixture
        .state
        .content()
        .repository(repository)
        .await
        .unwrap()
        .unwrap();
    fixture.state.content().context(
        repository,
        fixture.state.security().model(),
        None,
        "http://localhost",
        "127.0.0.1",
    )
}

fn files_package(version: &str, order: &[usize], tag: &str) -> Vec<u8> {
    let config = deno_json("acme", "race", version);
    let files: Vec<(String, Vec<u8>)> =
        std::iter::once(("deno.json".to_owned(), config.into_bytes()))
            .chain(std::iter::once(("mod.ts".to_owned(), mod_ts(tag))))
            .chain((0..6).map(|n| (format!("f{n}.ts"), format!("// {tag} {n}\n").into_bytes())))
            .chain(std::iter::once((
                "util.js".to_owned(),
                b"export default 1;\n".to_vec(),
            )))
            .collect();
    tar_gz(
        order
            .iter()
            .map(|index| regular(&files[*index].0, &files[*index].1))
            .collect(),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn racing_archives_of_one_version_with_different_file_orders_leave_a_whole_version() {
    let fixture = Arc::new(ready().await);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let forward: Vec<usize> = (0..9).collect();
    let reverse: Vec<usize> = (0..9).rev().collect();
    let rotated: Vec<usize> = (0..9).map(|n| (n + 4) % 9).collect();
    for round in 0..25 {
        let version = format!("1.0.{round}");
        let mut tasks = Vec::new();
        for (index, (tag, order)) in [("one", &forward), ("two", &reverse), ("three", &rotated)]
            .into_iter()
            .enumerate()
        {
            let delay = std::time::Duration::from_millis((index * (round % 6 + 1) * 3) as u64);
            let fixture = fixture.clone();
            let writer = writer.clone();
            let version = version.clone();
            let body = files_package(&version, order, tag);
            tasks.push(tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                publish(&fixture, HOSTED, &writer, ("acme", "race", &version), body).await
            }));
        }
        let mut statuses = Vec::new();
        for task in tasks {
            statuses.push(task.await.unwrap().status);
        }
        statuses.sort();
        assert_eq!(
            statuses,
            [
                StatusCode::OK,
                StatusCode::BAD_REQUEST,
                StatusCode::BAD_REQUEST
            ],
            "round {round}"
        );
        let meta = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/race/{version}_meta.json"),
            Some(&writer),
        )
        .await;
        assert_eq!(meta.status, StatusCode::OK, "round {round}");
        let manifest = meta.json()["manifest"].as_object().unwrap().clone();
        assert_eq!(manifest.len(), 9, "round {round}");
        for (file, entry) in manifest {
            let served = get(
                &fixture,
                &format!("/repository/{HOSTED}/@acme/race/{version}{file}"),
                Some(&writer),
            )
            .await;
            assert_eq!(served.status, StatusCode::OK, "round {round} {file}");
            assert_eq!(
                entry["checksum"],
                format!("sha256-{}", sha(&served.body)),
                "round {round} {file}"
            );
        }
    }
}

async fn ready_with(max_connections: u32, limits: astrum_format_jsr::Limits) -> Fixture {
    let test = astrum_testkit::TestDatabase::create().await;
    let data = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(data.path().join("tmp")).unwrap();
    let mut config = test.config();
    config.max_connections = max_connections;
    let db = astrum_db::Database::connect(&config).await.unwrap();
    db.migrate().await.unwrap();
    let created = astrum_security::bootstrap(db.conn(), data.path())
        .await
        .unwrap();
    let admin_password = std::fs::read_to_string(created.admin_password_file.unwrap()).unwrap();
    let state = astrum_api::AppState::load(
        db,
        data.path().to_owned(),
        astrum_security::SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap();
    astrum_blob::bootstrap(state.db().conn(), data.path())
        .await
        .unwrap();
    state
        .content()
        .register(Arc::new(astrum_format_jsr::Jsr::with_limits(limits)));
    Fixture {
        test,
        data,
        state,
        admin_password,
        peer: None,
    }
}

async fn hammer(pool: u32, publishes: usize) {
    let fixture = Arc::new(ready_with(pool, astrum_format_jsr::Limits::default()).await);
    let size = fixture
        .state
        .db()
        .conn()
        .get_postgres_connection_pool()
        .options()
        .get_max_connections();
    assert_eq!(size, pool);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let mut tasks = Vec::new();
    for number in 0..publishes {
        let fixture = fixture.clone();
        let writer = writer.clone();
        tasks.push(tokio::spawn(async move {
            let version = format!("1.0.{number}");
            let reply = publish(
                &fixture,
                HOSTED,
                &writer,
                ("acme", "many", &version),
                package("acme", "many", &version, "m"),
            )
            .await;
            (
                reply.status,
                reply.headers.contains_key(header::RETRY_AFTER),
            )
        }));
    }
    let mut readers = Vec::new();
    for _ in 0..10 {
        let fixture = fixture.clone();
        let writer = writer.clone();
        readers.push(tokio::spawn(async move {
            let mut statuses = Vec::new();
            for _ in 0..5 {
                statuses.push(
                    get(
                        &fixture,
                        &format!("/repository/{HOSTED}/@acme/many/meta.json"),
                        Some(&writer),
                    )
                    .await
                    .status,
                );
            }
            statuses
        }));
    }
    let mut counts: BTreeMap<u16, usize> = BTreeMap::new();
    for task in tasks {
        let (status, retry) = task.await.unwrap();
        if status == StatusCode::SERVICE_UNAVAILABLE {
            assert!(retry, "a 503 carries Retry-After");
        }
        *counts.entry(status.as_u16()).or_default() += 1;
    }
    assert!(
        counts.keys().all(|status| *status == 200 || *status == 503),
        "pool {pool}, {publishes} publishes: {counts:?}"
    );
    for reader in readers {
        for status in reader.await.unwrap() {
            assert!(
                status == StatusCode::OK || status == StatusCode::NOT_FOUND,
                "a read answered {status}; pool {pool}, {publishes} publishes: {counts:?}"
            );
        }
    }
    assert_eq!(
        counts.get(&200).copied().unwrap_or_default(),
        publishes,
        "pool {pool}: {counts:?}"
    );
    let listing = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/many/meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(versions(&listing.json()).len(), publishes);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn forty_parallel_publishes_at_the_default_pool_answer_without_a_server_error() {
    hammer(20, 40).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_publishes_beyond_a_small_pool_answer_without_a_server_error() {
    hammer(6, 24).await;
}

#[tokio::test]
async fn files_named_like_version_metadata_are_files() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let config = deno_json("acme", "hello", "1.0.0");
    let archive = tar_gz(vec![
        regular("deno.json", config.as_bytes()),
        regular("mod.ts", b"export {};\n"),
        regular("util.js", b"export default 1;\n"),
        regular("data_meta.json", b"{\"a\":1}"),
        regular("sub/schema_meta.json", b"{\"b\":2}"),
    ]);
    let published = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        archive,
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    for (file, body) in [
        ("data_meta.json", "{\"a\":1}"),
        ("sub/schema_meta.json", "{\"b\":2}"),
    ] {
        let served = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hello/1.0.0/{file}"),
            Some(&writer),
        )
        .await;
        assert_eq!(served.status, StatusCode::OK, "{file}");
        assert_eq!(served.body, body.as_bytes(), "{file}");
        assert_eq!(served.headers["content-type"], "application/json");
    }

    let (upstream, address) = serve_upstream(&fixture).await;
    upstream.package("acme", "lib", "1.0.0", "p");
    upstream.publish("@acme/lib/1.0.0/data_meta.json", "{\"c\":3}");
    upstream.publish("@acme/lib/1.0.0/sub/schema_meta.json", "{\"d\":4}");
    create(
        &fixture,
        "proxy",
        proxy("jsr-proxy", &format!("http://{address}")),
    )
    .await;
    for (file, body) in [
        ("data_meta.json", "{\"c\":3}"),
        ("sub/schema_meta.json", "{\"d\":4}"),
    ] {
        let served = get(
            &fixture,
            &format!("/repository/jsr-proxy/@acme/lib/1.0.0/{file}"),
            Some(&writer),
        )
        .await;
        assert_eq!(served.status, StatusCode::OK, "{file}");
        assert_eq!(served.body, body.as_bytes(), "{file}");
    }
    assert_eq!(upstream.accepts("@acme/lib/1.0.0/data_meta.json"), ["*/*"]);
}

#[tokio::test]
async fn add_on_the_version_metadata_is_needed_apart_from_add_on_the_files() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    selector_user(
        &fixture,
        "filesonly",
        HOSTED,
        json!(["ADD", "READ", "BROWSE"]),
        "format == \"jsr\" && path =^ \"/@acme/hello/0.1.0/\"",
    )
    .await;
    let refused = fixture
        .send_body(
            Method::POST,
            &publish_path(HOSTED, "acme", "hello", "0.1.0"),
            Auth::Basic("filesonly", "selected-pass"),
            Body::from(package("acme", "hello", "0.1.0", "f")),
            &[("content-encoding", "gzip")],
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    for path in ["0.1.0/mod.ts", "0.1.0_meta.json", "meta.json"] {
        let nothing = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hello/{path}"),
            Some(&writer),
        )
        .await;
        assert_eq!(nothing.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn a_duplicate_version_is_answered_before_the_body_is_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let target = ("acme", "hello", "0.1.0");
    assert_eq!(
        publish(
            &fixture,
            HOSTED,
            &writer,
            target,
            package("acme", "hello", "0.1.0", "a")
        )
        .await
        .status,
        StatusCode::OK
    );
    let junk = publish(
        &fixture,
        HOSTED,
        &writer,
        target,
        b"not an archive".to_vec(),
    )
    .await;
    assert_eq!(junk.status, StatusCode::BAD_REQUEST);
    assert_eq!(junk.json()["code"], "duplicateVersionPublish");
    let new_version = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "0.1.1"),
        b"not an archive".to_vec(),
    )
    .await;
    assert_eq!(new_version.json()["code"], "invalidTarball");
}

#[tokio::test]
async fn a_failed_write_leaves_no_version_and_the_metadata_is_written_last() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let ctx = store_ctx(&fixture, HOSTED).await;
    let mut blocker =
        astrum_content::PutAsset::hosted(&ctx, "/@acme/hole/1.0.0/b.ts").kind("other");
    blocker.check_content_type = false;
    astrum_content::store::put(&ctx, blocker, astrum_blob::content_of(b"x".to_vec()))
        .await
        .unwrap();
    let config = deno_json("acme", "hole", "1.0.0");
    let archive = tar_gz(vec![
        regular("deno.json", config.as_bytes()),
        regular("mod.ts", b"export {};\n"),
        regular("util.js", b"export default 1;\n"),
        regular("a.ts", b"export {};\n"),
        regular("b.ts", b"export {};\n"),
        regular("c.ts", b"export {};\n"),
    ]);
    let failed = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hole", "1.0.0"),
        archive,
    )
    .await;
    assert_eq!(failed.status, StatusCode::BAD_REQUEST, "{}", failed.text());
    for path in [
        "meta.json",
        "1.0.0_meta.json",
        "1.0.0/mod.ts",
        "1.0.0/a.ts",
        "1.0.0/c.ts",
    ] {
        let missing = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hole/{path}"),
            Some(&writer),
        )
        .await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }
    let kept = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hole/1.0.0/b.ts"),
        Some(&writer),
    )
    .await;
    assert_eq!(kept.body, b"x");

    let good = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "fine", "1.0.0"),
        package("acme", "fine", "1.0.0", "g"),
    )
    .await;
    assert_eq!(good.status, StatusCode::OK);
    let meta = astrum_content::store::find(ctx.db(), HOSTED, "/@acme/fine/1.0.0_meta.json")
        .await
        .unwrap()
        .unwrap();
    for file in [
        "deno.json",
        "mod.ts",
        "util.js",
        "view.tsx",
        "view.jsx",
        "README.md",
    ] {
        let stored =
            astrum_content::store::find(ctx.db(), HOSTED, &format!("/@acme/fine/1.0.0/{file}"))
                .await
                .unwrap()
                .unwrap();
        assert!(meta.created_at > stored.created_at, "{file}");
    }
}

#[tokio::test]
async fn a_version_left_without_its_metadata_is_cleared_and_published_again() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let target = ("acme", "hello", "0.1.0");
    assert_eq!(
        publish(
            &fixture,
            HOSTED,
            &writer,
            target,
            package("acme", "hello", "0.1.0", "old")
        )
        .await
        .status,
        StatusCode::OK
    );
    let ctx = store_ctx(&fixture, HOSTED).await;
    assert!(
        astrum_content::store::delete(&ctx, "/@acme/hello/0.1.0_meta.json")
            .await
            .unwrap()
    );
    let listing = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(listing.status, StatusCode::NOT_FOUND);
    let again = publish(
        &fixture,
        HOSTED,
        &writer,
        target,
        package("acme", "hello", "0.1.0", "new"),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    let served = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/0.1.0/mod.ts"),
        Some(&writer),
    )
    .await;
    assert_eq!(served.body, mod_ts("new"));
}

#[tokio::test]
async fn the_package_probe_does_not_tell_what_the_caller_may_not_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    for name in ["hello", "secret"] {
        let reply = publish(
            &fixture,
            HOSTED,
            &writer,
            ("acme", name, "1.0.0"),
            package("acme", name, "1.0.0", name),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK);
    }
    selector_user(
        &fixture,
        "narrow",
        HOSTED,
        json!(["READ", "BROWSE"]),
        "format == \"jsr\" && path =^ \"/@acme/hello/\"",
    )
    .await;
    let as_narrow = |path: String| {
        let fixture = &fixture;
        async move {
            fixture
                .send(
                    Method::GET,
                    &path,
                    Auth::Basic("narrow", "selected-pass"),
                    None,
                    &[],
                )
                .await
        }
    };
    let own = as_narrow(format!(
        "/repository/{HOSTED}/api/scopes/acme/packages/hello"
    ))
    .await;
    assert_eq!(own.status, StatusCode::OK);
    let hidden = as_narrow(format!(
        "/repository/{HOSTED}/api/scopes/acme/packages/secret"
    ))
    .await;
    assert_eq!(hidden.status, StatusCode::FORBIDDEN);
    let absent = as_narrow(format!(
        "/repository/{HOSTED}/api/scopes/acme/packages/nothing"
    ))
    .await;
    assert_eq!(absent.status, StatusCode::FORBIDDEN);
    let listing = as_narrow(format!("/repository/{HOSTED}/@acme/secret/meta.json")).await;
    assert_eq!(listing.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_config_with_comments_and_trailing_commas_is_read() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let mut config = vec![0xEF, 0xBB, 0xBF];
    config.extend_from_slice(
        b"{\n  // the package\n  \"name\": \"@acme/hello\", /* scope and name */\n  \"version\": \"0.1.0\",\n  \"exports\": {\n    \".\": \"./mod.ts\", // entry\n  },\n}\n",
    );
    let archive = tar_gz(vec![
        regular("deno.jsonc", &config),
        regular("mod.ts", b"export {};\n"),
    ]);
    let published = post(
        &fixture,
        &format!(
            "/repository/{HOSTED}/api/scopes/acme/packages/hello/versions/0.1.0?config=/deno.jsonc"
        ),
        Some(&writer),
        archive,
        true,
        None,
    )
    .await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    let meta = get(
        &fixture,
        &format!("/repository/{HOSTED}/@acme/hello/0.1.0_meta.json"),
        Some(&writer),
    )
    .await;
    assert_eq!(meta.json()["exports"], json!({".": "./mod.ts"}));
    let unclosed = tar_gz(vec![
        regular("deno.jsonc", b"{ /* never closed"),
        regular("mod.ts", b"export {};\n"),
    ]);
    let refused = post(
        &fixture,
        &format!(
            "/repository/{HOSTED}/api/scopes/acme/packages/hello/versions/0.2.0?config=/deno.jsonc"
        ),
        Some(&writer),
        unclosed,
        true,
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.json()["code"], "invalidConfig");
}

fn blocking_hook(
    on: &'static str,
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Semaphore>,
) -> astrum_format_jsr::Hook {
    Arc::new(move |path: String| {
        let started = started.clone();
        let release = release.clone();
        Box::pin(async move {
            if path.ends_with(on) {
                started.notify_one();
                release.acquire().await.unwrap().forget();
            }
        })
    })
}

fn limits(
    intake: usize,
    writes: usize,
    wait_ms: u64,
    deadline_ms: u64,
) -> astrum_format_jsr::Limits {
    astrum_format_jsr::Limits {
        intake,
        writes: Some(writes),
        permit_wait: std::time::Duration::from_millis(wait_ms),
        body_deadline: std::time::Duration::from_millis(deadline_ms),
        ..Default::default()
    }
}

fn slow_body() -> Body {
    use futures_util::StreamExt;
    Body::from_stream(
        futures_util::stream::iter([Ok::<_, std::io::Error>(axum::body::Bytes::from_static(
            b"\x1f\x8b",
        ))])
        .chain(futures_util::stream::pending()),
    )
}

async fn slow_publish(
    fixture: &Fixture,
    repository: &str,
    credential: &str,
    version: &str,
) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &publish_path(repository, "acme", "slow", version),
            Auth::None,
            slow_body(),
            &[
                ("authorization", &format!("Bearer {credential}")),
                ("content-encoding", "gzip"),
            ],
        )
        .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_used_up_write_gate_answers_503_with_retry_after() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let mut gate = limits(4, 1, 300, 60_000);
    gate.after_file = Some(blocking_hook("/mod.ts", started.clone(), release.clone()));
    let fixture = Arc::new(ready_with(20, gate).await);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let first = {
        let fixture = fixture.clone();
        let writer = writer.clone();
        tokio::spawn(async move {
            publish(
                &fixture,
                HOSTED,
                &writer,
                ("acme", "gate", "1.0.0"),
                package("acme", "gate", "1.0.0", "a"),
            )
            .await
        })
    };
    started.notified().await;
    let refused = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        publish(
            &fixture,
            HOSTED,
            &writer,
            ("acme", "gate", "1.0.1"),
            package("acme", "gate", "1.0.1", "b"),
        ),
    )
    .await
    .expect("the second publish is answered while the first is stuck");
    assert_eq!(
        refused.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "{}",
        refused.text()
    );
    assert!(refused.headers.contains_key(header::RETRY_AFTER));
    assert_eq!(refused.json()["code"], "busy");
    release.add_permits(100);
    assert_eq!(first.await.unwrap().status, StatusCode::OK);
    let again = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "gate", "1.0.1"),
        package("acme", "gate", "1.0.1", "b"),
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
}

#[tokio::test]
async fn failed_publishes_give_their_slots_back() {
    let fixture = ready_with(20, limits(2, 2, 300, 60_000)).await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let ctx = store_ctx(&fixture, HOSTED).await;
    let mut blocker =
        astrum_content::PutAsset::hosted(&ctx, "/@acme/leak/2.0.0/mod.ts").kind("other");
    blocker.check_content_type = false;
    astrum_content::store::put(&ctx, blocker, astrum_blob::content_of(b"x".to_vec()))
        .await
        .unwrap();
    let mismatch = || {
        tar_gz(vec![
            regular("deno.json", deno_json("acme", "other", "1.0.0").as_bytes()),
            regular("mod.ts", b"export {};\n"),
        ])
    };
    let mut codes = Vec::new();
    for round in 0..9 {
        let reply = match round % 4 {
            0 => {
                publish(
                    &fixture,
                    HOSTED,
                    &writer,
                    ("acme", "leak", "1.0.0"),
                    b"plain".to_vec(),
                )
                .await
            }
            1 => {
                publish(
                    &fixture,
                    HOSTED,
                    &writer,
                    ("acme", "leak", "1.0.0"),
                    mismatch(),
                )
                .await
            }
            2 => {
                publish(
                    &fixture,
                    HOSTED,
                    &writer,
                    ("acme", "leak", "2.0.0"),
                    package("acme", "leak", "2.0.0", "c"),
                )
                .await
            }
            _ => {
                publish(
                    &fixture,
                    HOSTED,
                    &writer,
                    ("acme", "leak", "1.0.0"),
                    vec![0u8; 20 * MIB + 1],
                )
                .await
            }
        };
        codes.push(reply.status.as_u16());
    }
    assert_eq!(codes, [400, 400, 400, 413, 400, 400, 400, 413, 400]);
    let fine = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "leak", "3.0.0"),
        package("acme", "leak", "3.0.0", "ok"),
    )
    .await;
    assert_eq!(fine.status, StatusCode::OK, "{}", fine.text());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_slow_upload_times_out_and_gives_its_intake_slot_back() {
    let fixture = Arc::new(ready_with(20, limits(1, 1, 5_000, 1_000)).await);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let started = std::time::Instant::now();
    let slow = {
        let fixture = fixture.clone();
        let writer = writer.clone();
        tokio::spawn(async move { slow_publish(&fixture, HOSTED, &writer, "1.0.0").await })
    };
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let normal = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "fast", "1.0.0"),
        package("acme", "fast", "1.0.0", "f"),
    )
    .await;
    assert_eq!(normal.status, StatusCode::OK, "{}", normal.text());
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(900),
        "the normal publish waited for the intake slot of the slow one"
    );
    let timed_out = tokio::time::timeout(std::time::Duration::from_secs(10), slow)
        .await
        .expect("the slow upload is answered")
        .unwrap();
    assert_eq!(
        timed_out.status,
        StatusCode::REQUEST_TIMEOUT,
        "{}",
        timed_out.text()
    );
    assert_eq!(timed_out.json()["code"], "requestTimeout");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_slow_upload_does_not_hold_the_write_slot() {
    let fixture = Arc::new(ready_with(20, limits(2, 1, 300, 2_000)).await);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    create(&fixture, "hosted", hosted("jsr-other", "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let slow = {
        let fixture = fixture.clone();
        let writer = writer.clone();
        tokio::spawn(async move { slow_publish(&fixture, HOSTED, &writer, "1.0.0").await })
    };
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let normal = publish(
        &fixture,
        "jsr-other",
        &writer,
        ("acme", "fast", "1.0.0"),
        package("acme", "fast", "1.0.0", "f"),
    )
    .await;
    assert_eq!(normal.status, StatusCode::OK, "{}", normal.text());
    assert!(!slow.is_finished(), "the slow upload is still going");
    let timed_out = tokio::time::timeout(std::time::Duration::from_secs(10), slow)
        .await
        .expect("the slow upload is answered")
        .unwrap();
    assert_eq!(timed_out.status, StatusCode::REQUEST_TIMEOUT);
}

#[tokio::test]
async fn a_spool_that_cannot_be_written_is_a_server_error() {
    let mut broken = limits(2, 2, 300, 60_000);
    broken.spool_dir = Some(std::path::PathBuf::from("/nonexistent/jsr-spool"));
    let fixture = ready_with(20, broken).await;
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let reply = publish(
        &fixture,
        HOSTED,
        &writer,
        ("acme", "hello", "1.0.0"),
        package("acme", "hello", "1.0.0", "a"),
    )
    .await;
    assert_eq!(
        reply.status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "{}",
        reply.text()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelling_the_request_in_the_middle_of_a_write_leaves_a_whole_version() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let mut gate = limits(4, 2, 5_000, 60_000);
    gate.after_file = Some(blocking_hook("/mod.ts", started.clone(), release.clone()));
    let fixture = Arc::new(ready_with(20, gate).await);
    create(&fixture, "hosted", hosted(HOSTED, "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let call = {
        let fixture = fixture.clone();
        let writer = writer.clone();
        tokio::spawn(async move {
            publish(
                &fixture,
                HOSTED,
                &writer,
                ("acme", "hello", "0.1.0"),
                package("acme", "hello", "0.1.0", "kept"),
            )
            .await
        })
    };
    started.notified().await;
    call.abort();
    assert!(call.await.is_err());
    release.add_permits(100);
    let mut meta = None;
    for _ in 0..50 {
        let reply = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hello/0.1.0_meta.json"),
            Some(&writer),
        )
        .await;
        if reply.status == StatusCode::OK {
            meta = Some(reply.json());
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let meta = meta.expect("the write went on after the request was dropped");
    let manifest = meta["manifest"].as_object().unwrap();
    assert_eq!(manifest.len(), 6);
    for (file, entry) in manifest {
        let served = get(
            &fixture,
            &format!("/repository/{HOSTED}/@acme/hello/0.1.0{file}"),
            Some(&writer),
        )
        .await;
        assert_eq!(served.status, StatusCode::OK, "{file}");
        assert_eq!(
            entry["checksum"],
            format!("sha256-{}", sha(&served.body)),
            "{file}"
        );
    }
}
