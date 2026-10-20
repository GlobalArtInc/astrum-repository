mod support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_nix::narinfo::NarInfo;
use astrum_format_nix::testing::{self, Node, StorePath};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const STORE: &str = "/nix/store";
const NARINFO_TYPE: &str = "text/x-nix-narinfo";
const NAR_TYPE: &str = "application/x-nix-nar";

#[derive(Clone, Copy)]
enum As<'a> {
    Admin,
    Nobody,
    User(&'a str, &'a str),
}

impl<'a> As<'a> {
    fn auth(self) -> Auth<'a> {
        match self {
            As::Admin => Auth::Admin,
            As::Nobody => Auth::None,
            As::User(user, password) => Auth::Basic(user, password),
        }
    }
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_nix::Nix));
    fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_nix::task::Gc));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/nix/{kind}"),
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

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
        "nix": {"priority": 30},
    })
}

fn program(name: &str, script: &'static [u8]) -> StorePath {
    StorePath::new(
        name,
        &Node::Directory(vec![(
            "bin",
            Node::Directory(vec![(
                "run",
                Node::File {
                    contents: script,
                    executable: true,
                },
            )]),
        )]),
    )
}

async fn send(
    fixture: &Fixture,
    method: Method,
    path: &str,
    who: As<'_>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    fixture
        .send_body(method, path, who.auth(), Body::from(body), headers)
        .await
}

async fn get(fixture: &Fixture, path: &str, who: As<'_>) -> Reply {
    send(fixture, Method::GET, path, who, Vec::new(), &[]).await
}

async fn head(fixture: &Fixture, path: &str, who: As<'_>) -> Reply {
    send(fixture, Method::HEAD, path, who, Vec::new(), &[]).await
}

async fn put(fixture: &Fixture, path: &str, who: As<'_>, body: &[u8], content_type: &str) -> Reply {
    send(
        fixture,
        Method::PUT,
        path,
        who,
        body.to_vec(),
        &[("content-type", content_type)],
    )
    .await
}

async fn copy(fixture: &Fixture, repository: &str, path: &StorePath, who: As<'_>) {
    let base = format!("/repository/{repository}");
    let narinfo = format!("{base}/{}.narinfo", path.hash);
    assert_eq!(
        head(fixture, &narinfo, who).await.status,
        StatusCode::NOT_FOUND
    );
    let nar = format!("{base}/{}", path.url);
    if head(fixture, &nar, who).await.status == StatusCode::NOT_FOUND {
        let stored = put(fixture, &nar, who, &path.file, NAR_TYPE).await;
        assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    }
    let listing = put(
        fixture,
        &format!("{base}/{}.ls", path.hash),
        who,
        path.listing().as_bytes(),
        "application/json",
    )
    .await;
    assert!(
        matches!(listing.status, StatusCode::CREATED | StatusCode::NO_CONTENT),
        "{}",
        listing.text()
    );
    let stored = put(
        fixture,
        &narinfo,
        who,
        path.narinfo(STORE).as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
}

async fn substitute(fixture: &Fixture, repository: &str, path: &StorePath, who: As<'_>) {
    let base = format!("/repository/{repository}");
    let info = get(fixture, &format!("{base}/{}.narinfo", path.hash), who).await;
    assert_eq!(info.status, StatusCode::OK, "{}", info.text());
    assert_eq!(info.headers[header::CONTENT_TYPE], NARINFO_TYPE);
    let parsed = NarInfo::parse(&info.body, STORE).unwrap();
    assert_eq!(parsed.store_path, path.store_path(STORE));
    let url = parsed
        .relative_url()
        .expect("the narinfo names a relative URL");
    let nar = get(fixture, &format!("{base}/{url}"), who).await;
    assert_eq!(nar.status, StatusCode::OK, "{}", nar.text());
    assert_eq!(
        parsed.file_hash.unwrap().nix32(),
        format!("sha256:{}", testing::sha256_nix32(&nar.body))
    );
    let unpacked = match parsed.compression.as_str() {
        "xz" => testing::unxz(&nar.body),
        _ => nar.body.clone(),
    };
    assert_eq!(
        parsed.nar_hash.nix32(),
        format!("sha256:{}", testing::sha256_nix32(&unpacked))
    );
    assert_eq!(unpacked, path.nar);
}

async fn search(fixture: &Fixture, query: &str) -> Vec<Value> {
    let reply = get(
        fixture,
        &format!("/service/rest/v1/search?{query}"),
        As::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["items"].as_array().unwrap().clone()
}

fn asset_paths(item: &Value) -> Vec<String> {
    sorted(
        item["assets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|asset| {
                asset["path"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches('/')
                    .to_owned()
            })
            .collect(),
    )
}

fn sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths
}

#[tokio::test]
async fn hosted_repositories_take_nix_copy_and_serve_substituters() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("cache", "ALLOW_ONCE")).await;
    let info = get(&fixture, "/repository/cache/nix-cache-info", As::Admin).await;
    assert_eq!(info.status, StatusCode::OK);
    assert_eq!(info.headers[header::CONTENT_TYPE], "text/x-nix-cache-info");
    assert_eq!(
        info.text(),
        "StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 40\n"
    );
    let info_head = head(&fixture, "/repository/cache/nix-cache-info", As::Admin).await;
    assert_eq!(info_head.status, StatusCode::OK);
    assert!(info_head.body.is_empty());

    let library = program("libgreeting-1.0", b"#!/bin/sh\necho greeting\n");
    let mut hello = program("hello-2.12.1", b"#!/bin/sh\necho hello\n");
    hello.references = vec![library.base(), hello.base()];
    hello.deriver = Some(format!(
        "{}-hello-2.12.1.drv",
        testing::store_hash("hello.drv")
    ));
    copy(&fixture, "cache", &library, As::Admin).await;
    copy(&fixture, "cache", &hello, As::Admin).await;

    let found = head(
        &fixture,
        &format!("/repository/cache/{}.narinfo", hello.hash),
        As::Admin,
    )
    .await;
    assert_eq!(found.status, StatusCode::OK);
    assert!(found.body.is_empty());
    let served = get(
        &fixture,
        &format!("/repository/cache/{}.narinfo", hello.hash),
        As::Admin,
    )
    .await;
    assert_eq!(served.text(), hello.narinfo(STORE));
    substitute(&fixture, "cache", &hello, As::Admin).await;
    substitute(&fixture, "cache", &library, As::Admin).await;
    let parsed = NarInfo::parse(&served.body, STORE).unwrap();
    assert_eq!(parsed.references, [library.base(), hello.base()]);
    let listing = get(
        &fixture,
        &format!("/repository/cache/{}.ls", hello.hash),
        As::Admin,
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK);
    assert_eq!(listing.json()["version"], 1);
    let ranged = send(
        &fixture,
        Method::GET,
        &format!("/repository/cache/{}", hello.url),
        As::Admin,
        Vec::new(),
        &[("range", "bytes=0-5")],
    )
    .await;
    assert_eq!(ranged.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(ranged.body, hello.file[..6]);
    let queried = get(
        &fixture,
        &format!("/repository/cache/{}?sha256=whatever", hello.url),
        As::Admin,
    )
    .await;
    assert_eq!(queried.body, hello.file);

    let items = search(&fixture, "repository=cache&name=hello").await;
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["group"], hello.hash);
    assert_eq!(items[0]["version"], "2.12.1");
    assert_eq!(items[0]["format"], "nix");
    assert_eq!(
        asset_paths(&items[0]),
        sorted(vec![
            format!("{}.ls", hello.hash),
            format!("{}.narinfo", hello.hash),
            hello.url.clone()
        ])
    );
    let narinfo_asset = items[0]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"].as_str().unwrap().ends_with(".narinfo"))
        .unwrap();
    assert_eq!(narinfo_asset["nix"]["storePath"], hello.store_path(STORE));
    assert_eq!(narinfo_asset["nix"]["compression"], "xz");
    let by_path = search(
        &fixture,
        &format!(
            "repository=cache&nix.storePath={}",
            library.store_path(STORE)
        ),
    )
    .await;
    assert_eq!(by_path.len(), 1);
    assert_eq!(by_path[0]["name"], "libgreeting");
    let by_keyword = search(&fixture, "format=nix&q=libgreeting").await;
    assert_eq!(by_keyword.len(), 1);

    let again = put(
        &fixture,
        &format!("/repository/cache/{}", hello.url),
        As::Admin,
        &hello.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(again.status, StatusCode::NO_CONTENT, "{}", again.text());
    let changed = put(
        &fixture,
        &format!("/repository/cache/{}", hello.url),
        As::Admin,
        b"other bytes",
        NAR_TYPE,
    )
    .await;
    assert_eq!(changed.status, StatusCode::CONFLICT, "{}", changed.text());
    let redeploy = put(
        &fixture,
        &format!("/repository/cache/{}.narinfo", hello.hash),
        As::Admin,
        hello.narinfo(STORE).as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(
        redeploy.status,
        StatusCode::NO_CONTENT,
        "{}",
        redeploy.text()
    );
    let resigned = format!(
        "{}Sig: {}\n",
        hello.narinfo(STORE),
        testing::signature("other-1", &hello.hash)
    );
    let resigned = put(
        &fixture,
        &format!("/repository/cache/{}.narinfo", hello.hash),
        As::Admin,
        resigned.as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(resigned.status, StatusCode::CONFLICT);
    let interrupted = program("interrupted-1.0", b"#!/bin/sh\necho again\n");
    put(
        &fixture,
        &format!("/repository/cache/{}", interrupted.url),
        As::Admin,
        &interrupted.file,
        NAR_TYPE,
    )
    .await;
    put(
        &fixture,
        &format!("/repository/cache/{}.ls", interrupted.hash),
        As::Admin,
        interrupted.listing().as_bytes(),
        "application/json",
    )
    .await;
    copy(&fixture, "cache", &interrupted, As::Admin).await;
    substitute(&fixture, "cache", &interrupted, As::Admin).await;

    let drv = hello.deriver.clone().unwrap();
    let log = put(
        &fixture,
        &format!("/repository/cache/log/{drv}"),
        As::Admin,
        b"building hello\n",
        "text/plain; charset=utf-8",
    )
    .await;
    assert_eq!(log.status, StatusCode::CREATED, "{}", log.text());
    let log = get(&fixture, &format!("/repository/cache/log/{drv}"), As::Admin).await;
    assert_eq!(log.text(), "building hello\n");
    let squeezed_path = format!("/repository/cache/{}.ls", testing::store_hash("squeezed"));
    let squeezed = send(
        &fixture,
        Method::PUT,
        &squeezed_path,
        As::Admin,
        b"\x8b\x02\x80{}\x03".to_vec(),
        &[
            ("content-type", "application/json"),
            ("content-encoding", "br"),
        ],
    )
    .await;
    assert_eq!(squeezed.status, StatusCode::CREATED, "{}", squeezed.text());
    let squeezed = get(&fixture, &squeezed_path, As::Admin).await;
    assert_eq!(squeezed.headers[header::CONTENT_ENCODING], "br");
    assert_eq!(squeezed.body, b"\x8b\x02\x80{}\x03");
    let realisation = format!(
        "/repository/cache/realisations/sha256:{}!out.doi",
        testing::sha256_nix32(b"drv")
    );
    let document = json!({"id": "x!out", "outPath": hello.base(), "signatures": [], "dependentRealisations": {}});
    let stored = put(
        &fixture,
        &realisation,
        As::Admin,
        document.to_string().as_bytes(),
        "application/json",
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(
        get(&fixture, &realisation, As::Admin).await.json(),
        document
    );
    let broken = put(
        &fixture,
        "/repository/cache/realisations/broken!out.doi",
        As::Admin,
        b"not json",
        "application/json",
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    let trace = format!("/repository/cache/build-trace-v2/{drv}/out.doi");
    assert_eq!(
        put(&fixture, &trace, As::Admin, b"{}", "application/json")
            .await
            .status,
        StatusCode::CREATED
    );

    let same = put(
        &fixture,
        "/repository/cache/nix-cache-info",
        As::Admin,
        b"StoreDir: /nix/store\n",
        "text/x-nix-cache-info",
    )
    .await;
    assert_eq!(same.status, StatusCode::NO_CONTENT);
    let other = put(
        &fixture,
        "/repository/cache/nix-cache-info",
        As::Admin,
        b"StoreDir: /gnu/store\n",
        "text/x-nix-cache-info",
    )
    .await;
    assert_eq!(other.status, StatusCode::BAD_REQUEST);
    assert!(other.text().contains("/nix/store"), "{}", other.text());
    let missing = get(
        &fixture,
        &format!(
            "/repository/cache/{}.narinfo",
            testing::store_hash("nothing")
        ),
        As::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let stray = put(
        &fixture,
        "/repository/cache/elsewhere/file.txt",
        As::Admin,
        b"x",
        "text/plain",
    )
    .await;
    assert_eq!(stray.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn narinfo_uploads_are_checked_against_the_store_and_the_nar() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("strict", "ALLOW")).await;
    let path = program("tool-1.0", b"#!/bin/sh\necho tool\n");
    let narinfo_url = format!("/repository/strict/{}.narinfo", path.hash);
    let early = put(
        &fixture,
        &narinfo_url,
        As::Admin,
        path.narinfo(STORE).as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(early.status, StatusCode::BAD_REQUEST);
    assert!(
        early.text().contains("upload the NAR file"),
        "{}",
        early.text()
    );
    let nar = put(
        &fixture,
        &format!("/repository/strict/{}", path.url),
        As::Admin,
        &path.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(nar.status, StatusCode::CREATED, "{}", nar.text());
    let good = path.narinfo(STORE);
    let file_size = format!("FileSize: {}", path.file.len());
    let file_hash = format!("FileHash: sha256:{}", testing::sha256_nix32(&path.file));
    let cases = [
        good.replace(&file_size, "FileSize: 1"),
        good.replace(
            &file_hash,
            &format!("FileHash: sha256:{}", testing::sha256_nix32(b"x")),
        ),
        good.replace("/nix/store/", "/gnu/store/"),
        good.replace(&path.hash, &testing::store_hash("other")),
        good.replace(
            &format!("URL: {}", path.url),
            &format!("URL: https://elsewhere.example/{}", path.url),
        ),
        good.replace("NarSize:", "NarSize: many\nOld:"),
        "garbage".to_owned(),
    ];
    for text in cases {
        let reply = put(
            &fixture,
            &narinfo_url,
            As::Admin,
            text.as_bytes(),
            NARINFO_TYPE,
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{text}: {}",
            reply.text()
        );
    }
    let compressed = send(
        &fixture,
        Method::PUT,
        &narinfo_url,
        As::Admin,
        testing::xz(good.as_bytes()),
        &[("content-type", NARINFO_TYPE), ("content-encoding", "xz")],
    )
    .await;
    assert_eq!(compressed.status, StatusCode::BAD_REQUEST);
    assert!(compressed.text().contains("narinfo-compression"));
    let accepted = put(
        &fixture,
        &narinfo_url,
        As::Admin,
        good.as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(accepted.status, StatusCode::CREATED, "{}", accepted.text());

    let replacement = program("tool-1.0", b"#!/bin/sh\necho tool, rebuilt\n").uncompressed();
    let mut moved = StorePath {
        hash: path.hash.clone(),
        ..replacement
    };
    moved.signatures = path.signatures.clone();
    let stored = put(
        &fixture,
        &format!("/repository/strict/{}", moved.url),
        As::Admin,
        &moved.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED);
    let replaced = put(
        &fixture,
        &narinfo_url,
        As::Admin,
        moved.narinfo(STORE).as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(replaced.status, StatusCode::CREATED, "{}", replaced.text());
    let old = get(
        &fixture,
        &format!("/repository/strict/{}", path.url),
        As::Admin,
    )
    .await;
    assert_eq!(old.status, StatusCode::NOT_FOUND);
    substitute(&fixture, "strict", &moved, As::Admin).await;

    create(
        &fixture,
        "hosted",
        json!({
            "name": "elsewhere",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            "nix": {"storeDir": "/opt/nix/store", "priority": 10, "wantMassQuery": false},
        }),
    )
    .await;
    let info = get(&fixture, "/repository/elsewhere/nix-cache-info", As::Admin).await;
    assert_eq!(
        info.text(),
        "StoreDir: /opt/nix/store\nWantMassQuery: 0\nPriority: 10\n"
    );
    put(
        &fixture,
        &format!("/repository/elsewhere/{}", path.url),
        As::Admin,
        &path.file,
        NAR_TYPE,
    )
    .await;
    let wrong_store = put(
        &fixture,
        &format!("/repository/elsewhere/{}.narinfo", path.hash),
        As::Admin,
        path.narinfo(STORE).as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(wrong_store.status, StatusCode::BAD_REQUEST);
    let right_store = put(
        &fixture,
        &format!("/repository/elsewhere/{}.narinfo", path.hash),
        As::Admin,
        path.narinfo("/opt/nix/store").as_bytes(),
        NARINFO_TYPE,
    )
    .await;
    assert_eq!(
        right_store.status,
        StatusCode::CREATED,
        "{}",
        right_store.text()
    );

    create(&fixture, "hosted", hosted("frozen", "DENY")).await;
    let denied = put(
        &fixture,
        &format!("/repository/frozen/{}", path.url),
        As::Admin,
        &path.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(denied.status, StatusCode::BAD_REQUEST);
    assert!(denied.text().contains("read-only"));
}

#[tokio::test]
async fn deleting_a_store_path_keeps_a_nar_another_narinfo_needs() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("shared", "ALLOW_ONCE")).await;
    let root = Node::File {
        contents: b"same bytes\n",
        executable: false,
    };
    let first = StorePath::new("first-1.0", &root);
    let second = StorePath::new("second-1.0", &root);
    assert_eq!(first.url, second.url);
    copy(&fixture, "shared", &first, As::Admin).await;
    copy(&fixture, "shared", &second, As::Admin).await;
    let first_item = search(&fixture, "repository=shared&name=first").await;
    assert_eq!(
        asset_paths(&first_item[0]),
        sorted(vec![
            format!("{}.ls", first.hash),
            format!("{}.narinfo", first.hash)
        ])
    );

    let deleted = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/shared/{}.narinfo", first.hash),
        As::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/shared/{}.narinfo", first.hash),
            As::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/shared/{}.ls", first.hash),
            As::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert!(
        search(&fixture, "repository=shared&name=first")
            .await
            .is_empty()
    );
    substitute(&fixture, "shared", &second, As::Admin).await;
    let second_item = search(&fixture, "repository=shared&name=second").await;
    assert_eq!(
        asset_paths(&second_item[0]),
        sorted(vec![
            format!("{}.ls", second.hash),
            format!("{}.narinfo", second.hash),
            second.url.clone()
        ])
    );

    let component = second_item[0]["id"].as_str().unwrap();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{component}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    for gone in [
        format!("{}.narinfo", second.hash),
        format!("{}.ls", second.hash),
        second.url.clone(),
    ] {
        assert_eq!(
            get(&fixture, &format!("/repository/shared/{gone}"), As::Admin)
                .await
                .status,
            StatusCode::NOT_FOUND,
            "{gone}"
        );
    }

    let alone = program("alone-2.0", b"#!/bin/sh\necho alone\n");
    copy(&fixture, "shared", &alone, As::Admin).await;
    let nar = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/shared/{}", alone.url),
        As::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(nar.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/shared/{}.narinfo", alone.hash),
            As::Admin
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let narinfo = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/shared/{}.narinfo", alone.hash),
        As::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(narinfo.status, StatusCode::NO_CONTENT);
    assert!(search(&fixture, "repository=shared").await.is_empty());
    let again = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/shared/{}.narinfo", alone.hash),
        As::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reading_and_writing_need_their_privileges() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("private", "ALLOW_ONCE")).await;
    let path = program("secret-1.0", b"#!/bin/sh\necho secret\n");
    copy(&fixture, "private", &path, As::Admin).await;

    let anonymous = get(&fixture, "/repository/private/nix-cache-info", As::Nobody).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        anonymous.headers[header::WWW_AUTHENTICATE],
        "Basic realm=\"Astrum Repository\""
    );
    let narinfo = format!("/repository/private/{}.narinfo", path.hash);
    assert_eq!(
        get(&fixture, &narinfo, As::Nobody).await.status,
        StatusCode::UNAUTHORIZED
    );
    let other = program("other-1.0", b"#!/bin/sh\necho other\n");
    let anonymous_put = put(
        &fixture,
        &format!("/repository/private/{}", other.url),
        As::Nobody,
        &other.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(anonymous_put.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous_put.headers.contains_key(header::WWW_AUTHENTICATE));

    fixture.add_user("stranger", "stranger-password", &[]).await;
    let stranger = As::User("stranger", "stranger-password");
    assert_eq!(
        get(&fixture, &narinfo, stranger).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&fixture, "/repository/private/nix-cache-info", stranger)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(
            &fixture,
            &format!("/repository/private/{}", other.url),
            stranger,
            &other.file,
            NAR_TYPE
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );

    for (role, privileges) in [
        (
            "nix-readers",
            json!([
                "as-repository-view-nix-private-read",
                "as-repository-view-nix-private-browse"
            ]),
        ),
        (
            "nix-writers",
            json!([
                "as-repository-view-nix-private-read",
                "as-repository-view-nix-private-add"
            ]),
        ),
    ] {
        let reply = fixture
            .call(
                Method::POST,
                "/service/rest/v1/security/roles",
                Auth::Admin,
                Some(json!({"id": role, "name": role, "description": "", "privileges": privileges, "roles": []})),
            )
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-password", &["nix-readers"])
        .await;
    fixture
        .add_user("ci", "ci-password", &["nix-writers"])
        .await;
    let reader = As::User("reader", "reader-password");
    substitute(&fixture, "private", &path, reader).await;
    assert_eq!(
        put(
            &fixture,
            &format!("/repository/private/{}", other.url),
            reader,
            &other.file,
            NAR_TYPE
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    let ci = As::User("ci", "ci-password");
    copy(&fixture, "private", &other, ci).await;
    substitute(&fixture, "private", &other, ci).await;
    let not_allowed = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/private/{}.narinfo", other.hash),
        ci,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(not_allowed.status, StatusCode::FORBIDDEN);
}

type Served = (String, Vec<u8>, Option<&'static str>);

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<String>>>,
    files: Arc<Mutex<Vec<Served>>>,
}

impl Upstream {
    fn serve(&self, path: &str, body: Vec<u8>, encoding: Option<&'static str>) {
        self.files
            .lock()
            .unwrap()
            .push((path.to_owned(), body, encoding));
    }

    fn count(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|hit| *hit == path)
            .count()
    }
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    upstream.hits.lock().unwrap().push(path.clone());
    let found = upstream
        .files
        .lock()
        .unwrap()
        .iter()
        .find(|(known, _, _)| *known == path)
        .cloned();
    match found {
        Some((_, body, encoding)) => {
            let mut response = body.into_response();
            if let Some(encoding) = encoding {
                response
                    .headers_mut()
                    .insert(header::CONTENT_ENCODING, encoding.parse().unwrap());
            }
            response
        }
        None => (StatusCode::NOT_FOUND, "404").into_response(),
    }
}

#[tokio::test]
async fn proxies_cache_the_upstream_and_groups_take_the_first_member() {
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
    let upstream = Upstream::default();
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let remote = format!("http://{address}/cache");

    let near = program("near-1.0", b"#!/bin/sh\necho near\n");
    let far = program("far-3.1", b"#!/bin/sh\necho far\n");
    let bad = program("bad-0.1", b"#!/bin/sh\necho bad\n");
    upstream.serve(
        "cache/nix-cache-info",
        b"StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 10\n".to_vec(),
        None,
    );
    upstream.serve(
        &format!("cache/{}.narinfo", near.hash),
        near.narinfo(STORE).into_bytes(),
        None,
    );
    upstream.serve(&format!("cache/{}", near.url), near.file.clone(), None);
    upstream.serve(
        &format!("cache/{}.ls", near.hash),
        b"(zstd bytes)".to_vec(),
        Some("zstd"),
    );
    upstream.serve(
        &format!("cache/{}.narinfo", far.hash),
        far.narinfo(STORE)
            .replace(
                &format!("URL: {}", far.url),
                &format!("URL: {remote}/{}", far.url),
            )
            .into_bytes(),
        None,
    );
    upstream.serve(&format!("cache/{}", far.url), far.file.clone(), None);
    upstream.serve(
        &format!("cache/{}.narinfo", bad.hash),
        far.narinfo(STORE).into_bytes(),
        None,
    );
    let drv = format!("{}-near-1.0.drv", testing::store_hash("near.drv"));
    upstream.serve(&format!("cache/log/{drv}"), b"built near\n".to_vec(), None);

    create(
        &fixture,
        "proxy",
        json!({
            "name": "nixos",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": remote, "contentMaxAge": -1, "metadataMaxAge": 60},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    let info = get(&fixture, "/repository/nixos/nix-cache-info", As::Admin).await;
    assert_eq!(
        info.text(),
        "StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 40\n"
    );
    assert_eq!(upstream.count("cache/nix-cache-info"), 0);

    let served = get(
        &fixture,
        &format!("/repository/nixos/{}.narinfo", near.hash),
        As::Admin,
    )
    .await;
    assert_eq!(served.text(), near.narinfo(STORE));
    for _ in 0..2 {
        substitute(&fixture, "nixos", &near, As::Admin).await;
    }
    assert_eq!(upstream.count(&format!("cache/{}", near.url)), 1);
    assert_eq!(upstream.count(&format!("cache/{}.narinfo", near.hash)), 1);

    let rewritten = get(
        &fixture,
        &format!("/repository/nixos/{}.narinfo", far.hash),
        As::Admin,
    )
    .await;
    assert_eq!(rewritten.status, StatusCode::OK, "{}", rewritten.text());
    assert!(
        rewritten.text().contains(&format!("\nURL: {}\n", far.url)),
        "{}",
        rewritten.text()
    );
    assert_eq!(rewritten.text(), far.narinfo(STORE));
    substitute(&fixture, "nixos", &far, As::Admin).await;

    let absent = format!(
        "/repository/nixos/{}.narinfo",
        testing::store_hash("absent")
    );
    for _ in 0..3 {
        assert_eq!(
            get(&fixture, &absent, As::Admin).await.status,
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        upstream.count(&format!("cache/{}.narinfo", testing::store_hash("absent"))),
        1
    );
    let invalid = get(
        &fixture,
        &format!("/repository/nixos/{}.narinfo", bad.hash),
        As::Admin,
    )
    .await;
    assert_eq!(
        invalid.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        invalid.text()
    );

    for _ in 0..2 {
        let listing = get(
            &fixture,
            &format!("/repository/nixos/{}.ls", near.hash),
            As::Admin,
        )
        .await;
        assert_eq!(listing.status, StatusCode::OK);
        assert_eq!(listing.headers[header::CONTENT_ENCODING], "zstd");
        assert_eq!(listing.body, b"(zstd bytes)");
    }
    assert_eq!(upstream.count(&format!("cache/{}.ls", near.hash)), 1);
    let log = get(&fixture, &format!("/repository/nixos/log/{drv}"), As::Admin).await;
    assert_eq!(log.text(), "built near\n");
    let no_log = get(
        &fixture,
        &format!("/repository/nixos/log/{}-x.drv", testing::store_hash("x")),
        As::Admin,
    )
    .await;
    assert_eq!(no_log.status, StatusCode::NOT_FOUND);

    let cached = search(&fixture, "repository=nixos&name=near").await;
    assert_eq!(cached.len(), 1, "{cached:?}");
    assert_eq!(cached[0]["version"], "1.0");
    assert_eq!(
        asset_paths(&cached[0]),
        sorted(vec![
            format!("{}.ls", near.hash),
            format!("{}.narinfo", near.hash),
            near.url.clone()
        ])
    );
    let refused = put(
        &fixture,
        &format!("/repository/nixos/{}", near.url),
        As::Admin,
        &near.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);

    create(&fixture, "hosted", hosted("own", "ALLOW_ONCE")).await;
    let own = program("own-0.1", b"#!/bin/sh\necho own\n");
    copy(&fixture, "own", &own, As::Admin).await;
    create(&fixture, "group", group("everything", &["own", "nixos"])).await;
    let info = get(&fixture, "/repository/everything/nix-cache-info", As::Admin).await;
    assert_eq!(
        info.text(),
        "StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 30\n"
    );
    substitute(&fixture, "everything", &own, As::Admin).await;
    substitute(&fixture, "everything", &near, As::Admin).await;
    substitute(&fixture, "everything", &far, As::Admin).await;
    let through = get(
        &fixture,
        &format!("/repository/everything/{}.narinfo", far.hash),
        As::Admin,
    )
    .await;
    assert_eq!(through.text(), far.narinfo(STORE));
    let listing = get(
        &fixture,
        &format!("/repository/everything/{}.ls", near.hash),
        As::Admin,
    )
    .await;
    assert_eq!(listing.headers[header::CONTENT_ENCODING], "zstd");
    assert_eq!(
        get(&fixture, &absent.replace("nixos", "everything"), As::Admin)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let to_group = put(
        &fixture,
        &format!("/repository/everything/{}", own.url),
        As::Admin,
        &own.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(to_group.status, StatusCode::METHOD_NOT_ALLOWED);

    let forget = send(
        &fixture,
        Method::DELETE,
        &format!("/repository/nixos/{}.narinfo", near.hash),
        As::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(forget.status, StatusCode::NO_CONTENT);
    substitute(&fixture, "nixos", &near, As::Admin).await;
    assert_eq!(upstream.count(&format!("cache/{}.narinfo", near.hash)), 2);
    assert_eq!(upstream.count(&format!("cache/{}", near.url)), 1);
}

#[tokio::test]
async fn the_rest_api_creates_nix_repositories_with_their_settings() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plain", "ALLOW_ONCE")).await;
    let stored = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/nix/hosted/plain",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.status, StatusCode::OK);
    let document = stored.json();
    assert_eq!(document["format"], "nix");
    assert_eq!(document["type"], "hosted");
    assert_eq!(
        document["nix"],
        json!({"storeDir": "/nix/store", "priority": 40, "wantMassQuery": true})
    );
    let mut changed = document.clone();
    changed["nix"]["priority"] = json!(20);
    let update = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/nix/hosted/plain",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT, "{}", update.text());
    assert_eq!(
        get(&fixture, "/repository/plain/nix-cache-info", As::Admin)
            .await
            .text(),
        "StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 20\n"
    );
    let mut bad = hosted("bad", "ALLOW");
    bad["nix"] = json!({"storeDir": "relative/store"});
    let refused = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/nix/hosted",
            Auth::Admin,
            Some(bad),
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(refused.text().contains("storeDir"), "{}", refused.text());
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(hosted("files", "ALLOW")),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let mixed = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/nix/group",
            Auth::Admin,
            Some(group("mixed", &["plain", "files"])),
        )
        .await;
    assert_eq!(mixed.status, StatusCode::BAD_REQUEST, "{}", mixed.text());
}

async fn run_task(fixture: &Fixture, name: &str, properties: Value) -> Value {
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "repository.nix.gc", "name": name, "properties": properties})),
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
    get(fixture, &format!("/service/rest/v1/tasks/{id}"), As::Admin)
        .await
        .json()
}

#[tokio::test]
async fn a_task_deletes_the_nar_files_of_interrupted_uploads() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("tidy", "ALLOW_ONCE")).await;
    let kept = program("kept-1.0", b"#!/bin/sh\necho kept\n");
    copy(&fixture, "tidy", &kept, As::Admin).await;
    let orphan = program("orphan-1.0", b"#!/bin/sh\necho orphan\n");
    let nar = format!("/repository/tidy/{}", orphan.url);
    assert_eq!(
        put(&fixture, &nar, As::Admin, &orphan.file, NAR_TYPE)
            .await
            .status,
        StatusCode::CREATED
    );
    let recent = run_task(&fixture, "recent", json!({"repositoryName": "tidy"})).await;
    assert_eq!(recent["lastRunResult"], "OK", "{recent}");
    assert_eq!(get(&fixture, &nar, As::Admin).await.status, StatusCode::OK);
    let all = run_task(
        &fixture,
        "all",
        json!({"repositoryName": "tidy", "deployOffset": "0"}),
    )
    .await;
    assert_eq!(all["lastRunResult"], "OK", "{all}");
    assert_eq!(
        get(&fixture, &nar, As::Admin).await.status,
        StatusCode::NOT_FOUND
    );
    substitute(&fixture, "tidy", &kept, As::Admin).await;
    let templates = get(&fixture, "/service/rest/v1/tasks/templates", As::Admin)
        .await
        .json();
    assert!(
        templates
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["type"] == "repository.nix.gc"),
        "{templates}"
    );
}

async fn tree(fixture: &Fixture, repository: &str, path: &str) -> Vec<(String, String, String)> {
    let reply = get(
        fixture,
        &format!("/service/rest/internal/ui/browse/{repository}?path={path}"),
        As::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["name"].as_str().unwrap().to_owned(),
                node["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn nodes(expected: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    expected
        .iter()
        .map(|(kind, name, path)| ((*kind).to_owned(), (*name).to_owned(), (*path).to_owned()))
        .collect()
}

#[tokio::test]
async fn the_browse_tree_shows_store_paths_by_name_and_version() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("tree", "ALLOW_ONCE")).await;
    let hello = program("hello-2.12.1", b"#!/bin/sh\necho hello\n");
    let source = StorePath::new(
        "source",
        &Node::File {
            contents: b"source\n",
            executable: false,
        },
    );
    copy(&fixture, "tree", &hello, As::Admin).await;
    copy(&fixture, "tree", &source, As::Admin).await;
    let orphan = program("orphan-1.0", b"#!/bin/sh\necho orphan\n");
    put(
        &fixture,
        &format!("/repository/tree/{}", orphan.url),
        As::Admin,
        &orphan.file,
        NAR_TYPE,
    )
    .await;
    assert_eq!(
        tree(&fixture, "tree", "/").await,
        nodes(&[
            ("folder", "hello", "/hello/"),
            ("folder", "nar", "/nar/"),
            ("folder", "source", "/source/"),
            ("asset", "nix-cache-info", "/nix-cache-info"),
        ])
    );
    let files: Vec<String> = tree(&fixture, "tree", "/source/")
        .await
        .into_iter()
        .map(|(kind, name, _)| format!("{kind} {name}"))
        .collect();
    assert_eq!(files.len(), 3, "{files:?}");
    assert!(files.contains(&format!("asset {}.narinfo", source.hash)));
    assert_eq!(
        tree(&fixture, "tree", "/hello/").await,
        nodes(&[("component", "2.12.1", "/hello/2.12.1/")])
    );
    let file = hello.url.trim_start_matches("nar/");
    let mut expected = vec![
        (
            "asset".to_owned(),
            format!("{}.ls", hello.hash),
            format!("/hello/2.12.1/{}.ls", hello.hash),
        ),
        (
            "asset".to_owned(),
            format!("{}.narinfo", hello.hash),
            format!("/hello/2.12.1/{}.narinfo", hello.hash),
        ),
        (
            "asset".to_owned(),
            file.to_owned(),
            format!("/hello/2.12.1/{file}"),
        ),
    ];
    expected.sort_by(|a, b| a.1.cmp(&b.1));
    assert_eq!(tree(&fixture, "tree", "/hello/2.12.1/").await, expected);
}
