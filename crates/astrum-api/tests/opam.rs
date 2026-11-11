mod support;

use std::collections::BTreeMap;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_db::entity::asset;
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use sha2::{Digest, Sha256, Sha512};
use support::{Auth, Fixture, Reply};

const HOSTED: &str = "opam-hosted";
const ONCE: &str = "opam-once";
const PROXY: &str = "opam-proxy";
const REPO_TEXT: &str = "opam-version: \"2.0\"\narchive-mirrors: \"cache\"\n";

fn hosted_body(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
    })
}

fn proxy_body(name: &str, remote: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": true},
    })
}

async fn create(fixture: &Fixture, kind: &str, body: Value) -> Reply {
    fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/opam/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn ready_with(max_entries: usize) -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_opam::Opam::with_max_entries(
            max_entries,
        )));
    for (name, policy) in [(HOSTED, "ALLOW"), (ONCE, "ALLOW_ONCE")] {
        let reply = create(&fixture, "hosted", hosted_body(name, policy)).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    fixture
}

async fn ready() -> Fixture {
    ready_with(50_000).await
}

async fn send(
    fixture: &Fixture,
    method: Method,
    repository: &str,
    path: &str,
    body: Vec<u8>,
) -> Reply {
    fixture
        .send_body(
            method,
            &format!("/repository/{repository}/{path}"),
            Auth::Admin,
            Body::from(body),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn put(fixture: &Fixture, repository: &str, path: &str, body: impl Into<Vec<u8>>) -> Reply {
    send(fixture, Method::PUT, repository, path, body.into()).await
}

async fn get(fixture: &Fixture, repository: &str, path: &str) -> Reply {
    send(fixture, Method::GET, repository, path, vec![]).await
}

async fn delete(fixture: &Fixture, repository: &str, path: &str) -> Reply {
    send(fixture, Method::DELETE, repository, path, vec![]).await
}

struct Entry {
    path: String,
    mode: u32,
    uid: u64,
    gid: u64,
    mtime: u64,
    data: Vec<u8>,
}

fn entries(index: &[u8]) -> Vec<Entry> {
    let decoder = flate2::read::GzDecoder::new(index);
    let mut archive = tar::Archive::new(decoder);
    archive
        .entries()
        .unwrap()
        .map(|entry| {
            let mut entry = entry.unwrap();
            let header = entry.header().clone();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            Entry {
                path: entry.path().unwrap().to_string_lossy().into_owned(),
                mode: header.mode().unwrap(),
                uid: header.uid().unwrap(),
                gid: header.gid().unwrap(),
                mtime: header.mtime().unwrap(),
                data,
            }
        })
        .collect()
}

async fn index_paths(fixture: &Fixture, repository: &str) -> Vec<String> {
    let reply = get(fixture, repository, "index.tar.gz").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    entries(&reply.body)
        .into_iter()
        .map(|entry| entry.path)
        .collect()
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn sha512(data: &[u8]) -> String {
    hex::encode(Sha512::digest(data))
}

fn opam_path(name: &str, version: &str) -> String {
    format!("packages/{name}/{name}.{version}/opam")
}

async fn token(fixture: &Fixture, scope: Scope) -> String {
    fixture
        .state
        .security()
        .create_token("admin", "opam", scope, None)
        .await
        .unwrap()
        .1
}

async fn stored_asset(fixture: &Fixture, repository: &str, path: &str) -> asset::Model {
    asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.eq(format!("/{path}")))
        .one(fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap()
}

async fn set_write_policy(fixture: &Fixture, repository: &str, policy: &str) {
    let url = format!("/service/rest/v1/repositories/opam/hosted/{repository}");
    let mut document = fixture
        .call(Method::GET, &url, Auth::Admin, None)
        .await
        .json();
    document["storage"]["writePolicy"] = json!(policy);
    let reply = fixture
        .call(Method::PUT, &url, Auth::Admin, Some(document))
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

#[tokio::test]
async fn the_index_is_rebuilt_on_every_write_and_delete() {
    let fixture = ready().await;
    let first = put(
        &fixture,
        HOSTED,
        &opam_path("zeta", "1.0"),
        "opam-version: \"2.0\"\n",
    )
    .await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.text());
    assert_eq!(
        index_paths(&fixture, HOSTED).await,
        ["repo", "packages/zeta/zeta.1.0/opam"]
    );
    let second = put(&fixture, HOSTED, &opam_path("alpha", "2.0"), "alpha").await;
    assert_eq!(second.status, StatusCode::CREATED);
    let patch = put(
        &fixture,
        HOSTED,
        "packages/alpha/alpha.2.0/files/fix.patch",
        "patch",
    )
    .await;
    assert_eq!(patch.status, StatusCode::CREATED);
    let reply = get(&fixture, HOSTED, "index.tar.gz").await;
    let listed = entries(&reply.body);
    let paths: Vec<&str> = listed.iter().map(|entry| entry.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "repo",
            "packages/alpha/alpha.2.0/files/fix.patch",
            "packages/alpha/alpha.2.0/opam",
            "packages/zeta/zeta.1.0/opam",
        ]
    );
    assert_eq!(listed[0].data, REPO_TEXT.as_bytes());
    assert_eq!(listed[2].data, b"alpha");
    for entry in &listed {
        assert_eq!(
            (entry.mode, entry.uid, entry.gid),
            (0o644, 0, 0),
            "{}",
            entry.path
        );
        assert!(entry.mtime > 0, "{}", entry.path);
    }
    let removed = delete(&fixture, HOSTED, &opam_path("zeta", "1.0")).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 3);
    let missing = delete(&fixture, HOSTED, &opam_path("zeta", "1.0")).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    delete(&fixture, HOSTED, "packages/alpha/alpha.2.0/files/fix.patch").await;
    delete(&fixture, HOSTED, &opam_path("alpha", "2.0")).await;
    assert_eq!(index_paths(&fixture, HOSTED).await, ["repo"]);
    let repo = get(&fixture, HOSTED, "repo").await;
    assert_eq!(repo.text(), REPO_TEXT);
}

#[tokio::test]
async fn a_missing_index_is_built_when_it_is_read() {
    let fixture = ready().await;
    assert_eq!(index_paths(&fixture, HOSTED).await, ["repo"]);
    put(&fixture, HOSTED, &opam_path("a", "1"), "a").await;
    asset::Entity::delete_many()
        .filter(asset::Column::Repository.eq(HOSTED))
        .filter(asset::Column::Path.eq("/index.tar.gz"))
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(
        index_paths(&fixture, HOSTED).await,
        ["repo", "packages/a/a.1/opam"]
    );
}

#[tokio::test]
async fn a_replaced_opam_file_answers_204_or_409_by_the_write_policy() {
    let fixture = ready().await;
    let path = opam_path("hello", "1.0");
    assert_eq!(
        put(&fixture, HOSTED, &path, "one").await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, HOSTED, &path, "two").await.status,
        StatusCode::NO_CONTENT
    );
    let reply = get(&fixture, HOSTED, "index.tar.gz").await;
    assert_eq!(entries(&reply.body)[1].data, b"two");
    assert_eq!(
        put(&fixture, ONCE, &path, "one").await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, ONCE, &path, "two").await.status,
        StatusCode::CONFLICT
    );
    assert_eq!(get(&fixture, ONCE, &path).await.body, b"one");
}

#[tokio::test]
async fn names_versions_directories_and_paths_are_checked() {
    let fixture = ready().await;
    let long = "n".repeat(65);
    let refused = [
        "packages/hello/other.1.0/opam".to_owned(),
        "packages/hello/hello/opam".to_owned(),
        "packages/-hello/-hello.1/opam".to_owned(),
        "packages/he%20llo/he%20llo.1/opam".to_owned(),
        "packages/a.b/a.b.1/opam".to_owned(),
        format!("packages/{long}/{long}.1/opam"),
        "packages/hello/hello.1$/opam".to_owned(),
        format!("packages/hello/hello.{}/opam", "1".repeat(65)),
        "packages/hello/hello.1/opam2".to_owned(),
        "packages/hello/hello.1/files".to_owned(),
        "packages/hello/hello.1/files/a%20b".to_owned(),
        "packages/hello/hello.1/files/a/b/c/d/e/f/g/h/i".to_owned(),
        "packages/hello/hello.1/files/a@b".to_owned(),
        "notes.txt".to_owned(),
    ];
    for path in &refused {
        let reply = put(&fixture, HOSTED, path, "x").await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{path}: {}",
            reply.text()
        );
    }
    assert_eq!(index_paths(&fixture, HOSTED).await, ["repo"]);
    let accepted = [
        "packages/a_b+c-d/a_b+c-d.1.0~rc+git-2/opam",
        "packages/hello/hello.1.0/files/a/b/c/d/e/f/g/h",
    ];
    for path in accepted {
        let reply = put(&fixture, HOSTED, path, "x").await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{path}: {}",
            reply.text()
        );
    }
    let reply = put(
        &fixture,
        HOSTED,
        "packages/hello/hello.1.0/opam",
        vec![0xff, 0xfe],
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_sizes_of_stored_files_are_capped() {
    let fixture = ready().await;
    let path = opam_path("big", "1");
    let limit = 256 * 1024;
    assert_eq!(
        put(&fixture, HOSTED, &path, vec![b'a'; limit]).await.status,
        StatusCode::CREATED
    );
    let over = put(
        &fixture,
        HOSTED,
        &opam_path("big", "2"),
        vec![b'a'; limit + 1],
    )
    .await;
    assert_eq!(over.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        get(&fixture, HOSTED, &opam_path("big", "2")).await.status,
        StatusCode::NOT_FOUND
    );
    let file = "packages/big/big.1/files/blob";
    let mebibyte = 1024 * 1024;
    assert_eq!(
        put(&fixture, HOSTED, file, vec![1u8; mebibyte])
            .await
            .status,
        StatusCode::CREATED
    );
    let over = put(
        &fixture,
        HOSTED,
        "packages/big/big.1/files/more",
        vec![1u8; mebibyte + 1],
    )
    .await;
    assert_eq!(over.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_cache_file_must_match_its_digest_and_directory() {
    let fixture = ready().await;
    let data = b"source archive".to_vec();
    let hex = sha256(&data);
    let path = format!("cache/sha256/{}/{hex}", &hex[..2]);
    let other = put(&fixture, HOSTED, &path, "different bytes").await;
    assert_eq!(other.status, StatusCode::BAD_REQUEST, "{}", other.text());
    assert_eq!(
        get(&fixture, HOSTED, &path).await.status,
        StatusCode::NOT_FOUND
    );
    let wrong_directory = format!(
        "cache/sha256/{}/{hex}",
        if &hex[..2] == "00" { "01" } else { "00" }
    );
    assert_eq!(
        put(&fixture, HOSTED, &wrong_directory, data.clone())
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    let upper = format!(
        "cache/sha256/{}/{}",
        hex[..2].to_uppercase(),
        hex.to_uppercase()
    );
    assert_eq!(
        put(&fixture, HOSTED, &upper, data.clone()).await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        put(&fixture, HOSTED, &path, data.clone()).await.status,
        StatusCode::CREATED
    );
    let served = get(&fixture, HOSTED, &path).await;
    assert_eq!(served.status, StatusCode::OK);
    assert_eq!(served.body, data);
    assert_eq!(
        put(&fixture, ONCE, &path, data.clone()).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, ONCE, &path, data.clone()).await.status,
        StatusCode::NO_CONTENT
    );
    let strong = sha512(&data);
    let strong_path = format!("cache/sha512/{}/{strong}", &strong[..2]);
    assert_eq!(
        put(&fixture, HOSTED, &strong_path, data.clone())
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(get(&fixture, HOSTED, &strong_path).await.body, data);
    assert_eq!(
        delete(&fixture, HOSTED, &path).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&fixture, HOSTED, &path).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(index_paths(&fixture, HOSTED).await, ["repo"]);
}

#[tokio::test]
async fn the_generated_files_cannot_be_written_or_deleted() {
    let fixture = ready().await;
    for path in ["repo", "index.tar.gz"] {
        for method in [Method::PUT, Method::DELETE] {
            let reply = send(&fixture, method.clone(), HOSTED, path, b"x".to_vec()).await;
            assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{method} {path}");
        }
    }
    let reply = send(&fixture, Method::POST, HOSTED, "repo", vec![]).await;
    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(index_paths(&fixture, HOSTED).await, ["repo"]);
}

#[tokio::test]
async fn a_read_token_reads_but_cannot_write_or_delete() {
    let fixture = ready().await;
    let path = opam_path("hello", "1.0");
    put(&fixture, HOSTED, &path, "x").await;
    let reader = token(&fixture, Scope::Read).await;
    let authorization = format!("Bearer {reader}");
    let headers = [("authorization", authorization.as_str())];
    let attempt = |method: Method, path: String| {
        let fixture = &fixture;
        let headers = &headers;
        async move {
            fixture
                .send_body(
                    method,
                    &format!("/repository/{HOSTED}/{path}"),
                    Auth::None,
                    Body::from("y"),
                    headers,
                )
                .await
        }
    };
    assert_eq!(
        attempt(Method::GET, path.clone()).await.status,
        StatusCode::OK
    );
    assert_eq!(
        attempt(Method::GET, "index.tar.gz".to_owned()).await.status,
        StatusCode::OK
    );
    assert_eq!(
        attempt(Method::PUT, opam_path("other", "1")).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        attempt(Method::PUT, path.clone()).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        attempt(Method::DELETE, path.clone()).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(get(&fixture, HOSTED, &path).await.body, b"x");
}

#[tokio::test]
async fn concurrent_writers_leave_an_index_with_every_package() {
    let fixture = Arc::new(ready().await);
    let rounds = 6;
    for round in 0..rounds {
        let mut tasks = Vec::new();
        for number in 0..8 {
            let fixture = fixture.clone();
            tasks.push(tokio::spawn(async move {
                let name = format!("pkg{number}");
                let version = format!("1.{round}");
                put(
                    &fixture,
                    HOSTED,
                    &opam_path(&name, &version),
                    format!("opam {number}"),
                )
                .await
            }));
        }
        for task in tasks {
            let reply = task.await.unwrap();
            assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        }
        let written =
            astrum_content::store::find(fixture.state.db().conn(), HOSTED, "/index.tar.gz")
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            written.attributes["opam"]["entries"],
            1 + 8 * (round + 1),
            "round {round}"
        );
    }
    let mut expected = vec!["repo".to_owned()];
    for number in 0..8 {
        for round in 0..rounds {
            expected.push(format!("packages/pkg{number}/pkg{number}.1.{round}/opam"));
        }
    }
    assert_eq!(index_paths(&fixture, HOSTED).await, expected);
}

#[tokio::test]
async fn the_number_of_index_entries_is_capped() {
    let fixture = ready_with(3).await;
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("a", "1"), "a")
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("b", "1"), "b")
            .await
            .status,
        StatusCode::CREATED
    );
    let over = put(&fixture, HOSTED, &opam_path("c", "1"), "c").await;
    assert_eq!(over.status, StatusCode::CONFLICT);
    assert!(
        over.text().contains("too many package files"),
        "{}",
        over.text()
    );
    assert_eq!(
        get(&fixture, HOSTED, &opam_path("c", "1")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("a", "1"), "again")
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete(&fixture, HOSTED, &opam_path("a", "1")).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("c", "1"), "c")
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        index_paths(&fixture, HOSTED).await,
        ["repo", "packages/b/b.1/opam", "packages/c/c.1/opam"]
    );
}

#[tokio::test]
async fn an_index_that_no_longer_matches_the_packages_is_rebuilt_without_the_lost_asset() {
    let fixture = ready().await;
    put(&fixture, HOSTED, &opam_path("a", "1"), "a").await;
    put(&fixture, HOSTED, &opam_path("b", "1"), "b").await;
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 3);
    asset::Entity::delete_many()
        .filter(asset::Column::Repository.eq(HOSTED))
        .filter(asset::Column::Path.eq("/packages/a/a.1/opam"))
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(
        index_paths(&fixture, HOSTED).await,
        ["repo", "packages/b/b.1/opam"]
    );
}

#[tokio::test]
async fn a_failing_rebuild_serves_the_previous_index_or_answers_503() {
    let fixture = ready().await;
    for name in [HOSTED, "opam-fresh"] {
        if name != HOSTED {
            let reply = create(&fixture, "hosted", hosted_body(name, "ALLOW")).await;
            assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
        }
        put(&fixture, name, &opam_path("a", "1"), "a").await;
        put(&fixture, name, &opam_path("b", "1"), "b").await;
    }
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 3);
    let broken = stored_asset(&fixture, HOSTED, "packages/a/a.1/opam").await;
    let lost = broken.blob_id;
    let mut changed: asset::ActiveModel = broken.into();
    changed.blob_id = Set(uuid::Uuid::new_v4());
    changed.size = Set(2);
    changed.update(fixture.state.db().conn()).await.unwrap();
    asset::Entity::delete_many()
        .filter(asset::Column::Repository.eq(HOSTED))
        .filter(asset::Column::Path.eq("/packages/b/b.1/opam"))
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 3);
    let mut healed: asset::ActiveModel = stored_asset(&fixture, HOSTED, "packages/a/a.1/opam")
        .await
        .into();
    healed.blob_id = Set(lost);
    healed.update(fixture.state.db().conn()).await.unwrap();
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 3);

    asset::Entity::delete_many()
        .filter(asset::Column::Repository.eq("opam-fresh"))
        .filter(asset::Column::Path.eq("/index.tar.gz"))
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    let broken = stored_asset(&fixture, "opam-fresh", "packages/a/a.1/opam").await;
    let mut changed: asset::ActiveModel = broken.into();
    changed.blob_id = Set(uuid::Uuid::new_v4());
    changed.update(fixture.state.db().conn()).await.unwrap();
    let reply = get(&fixture, "opam-fresh", "index.tar.gz").await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(reply.text().contains("cannot be built"), "{}", reply.text());
    assert_eq!(get(&fixture, HOSTED, "repo").await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_package_files_of_a_repository_are_capped_at_32_mebibytes() {
    let fixture = ready().await;
    let mebibyte = 1024 * 1024;
    for number in 0..32 {
        let reply = put(
            &fixture,
            HOSTED,
            &format!("packages/big/big.1/files/f{number}"),
            vec![1u8; mebibyte],
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    let over = put(&fixture, HOSTED, &opam_path("big", "1"), "x").await;
    assert_eq!(over.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(
        over.text().contains("at most 33554432 bytes"),
        "{}",
        over.text()
    );
    assert_eq!(
        get(&fixture, HOSTED, &opam_path("big", "1")).await.status,
        StatusCode::NOT_FOUND
    );
    let same = put(
        &fixture,
        HOSTED,
        "packages/big/big.1/files/f0",
        vec![2u8; mebibyte],
    )
    .await;
    assert_eq!(same.status, StatusCode::NO_CONTENT, "{}", same.text());
    let smaller = put(
        &fixture,
        HOSTED,
        "packages/big/big.1/files/f0",
        vec![2u8; mebibyte - 10],
    )
    .await;
    assert_eq!(smaller.status, StatusCode::NO_CONTENT);
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("big", "2"), vec![b'a'; 11])
            .await
            .status,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        put(&fixture, HOSTED, &opam_path("big", "3"), vec![b'a'; 10])
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(index_paths(&fixture, HOSTED).await.len(), 34);
}

#[tokio::test]
async fn a_cache_file_that_exists_is_refused_like_any_write_when_the_repository_denies_writes() {
    let fixture = ready().await;
    let data = b"source archive".to_vec();
    let hex = sha256(&data);
    let path = format!("cache/sha256/{}/{hex}", &hex[..2]);
    assert_eq!(
        put(&fixture, HOSTED, &path, data.clone()).await.status,
        StatusCode::CREATED
    );
    set_write_policy(&fixture, HOSTED, "DENY").await;
    let again = put(&fixture, HOSTED, &path, data.clone()).await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert!(again.text().contains("read-only"), "{}", again.text());
    let package = put(&fixture, HOSTED, &opam_path("a", "1"), "a").await;
    assert_eq!(package.status, StatusCode::BAD_REQUEST);
    assert!(package.text().contains("read-only"), "{}", package.text());
}

type Files = BTreeMap<String, (String, Vec<u8>)>;

#[derive(Clone, Default)]
struct Stub {
    files: Arc<Mutex<Files>>,
    seen: Arc<Mutex<Vec<String>>>,
}

impl Stub {
    fn publish(&self, path: &str, body: impl Into<Vec<u8>>) {
        self.files.lock().unwrap().insert(
            path.to_owned(),
            ("application/octet-stream".to_owned(), body.into()),
        );
    }

    fn calls(&self, path: &str) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|seen| *seen == path)
            .count()
    }

    fn total(&self) -> usize {
        self.seen.lock().unwrap().len()
    }
}

async fn answer(State(stub): State<Stub>, uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/').to_owned();
    stub.seen.lock().unwrap().push(path.clone());
    let found = stub.files.lock().unwrap().get(&path).cloned();
    match found {
        Some((content_type, body)) => {
            ([(header::CONTENT_TYPE, content_type)], body).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn serve_stub() -> (Stub, SocketAddr) {
    let stub = Stub::default();
    let app = Router::new()
        .route("/{*path}", routing::get(answer))
        .with_state(stub.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (stub, address)
}

async fn ready_proxy() -> (Fixture, Stub) {
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
    let (stub, address) = serve_stub().await;
    let reply = create(
        &fixture,
        "proxy",
        proxy_body(PROXY, &format!("http://{address}")),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    (fixture, stub)
}

async fn stored(fixture: &Fixture, repository: &str, path: &str) -> bool {
    astrum_content::store::find(fixture.state.db().conn(), repository, path)
        .await
        .unwrap()
        .is_some()
}

#[tokio::test]
async fn the_index_and_repo_pass_through_unchanged_and_are_cached() {
    let (fixture, stub) = ready_proxy().await;
    let index = vec![0x1f, 0x8b, 8, 0, 1, 2, 3];
    stub.publish("index.tar.gz", index.clone());
    stub.publish("repo", REPO_TEXT);
    for _ in 0..2 {
        let reply = get(&fixture, PROXY, "index.tar.gz").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        assert_eq!(reply.body, index);
        assert_eq!(get(&fixture, PROXY, "repo").await.text(), REPO_TEXT);
    }
    assert_eq!(stub.calls("index.tar.gz"), 1);
    assert_eq!(stub.calls("repo"), 1);
}

#[tokio::test]
async fn a_cache_file_is_checked_against_its_digest_before_it_is_stored() {
    let (fixture, stub) = ready_proxy().await;
    let data = b"upstream archive".to_vec();
    let hex = sha256(&data);
    let path = format!("cache/sha256/{}/{hex}", &hex[..2]);
    stub.publish(&path, data.clone());
    for _ in 0..2 {
        let reply = get(&fixture, PROXY, &path).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        assert_eq!(reply.body, data);
    }
    assert_eq!(stub.calls(&path), 1);
    let strong = sha512(&data);
    let strong_path = format!("cache/sha512/{}/{strong}", &strong[..2]);
    stub.publish(&strong_path, data.clone());
    assert_eq!(get(&fixture, PROXY, &strong_path).await.body, data);
    let bad_hex = sha256(b"expected");
    let bad_path = format!("cache/sha256/{}/{bad_hex}", &bad_hex[..2]);
    stub.publish(&bad_path, "tampered");
    let reply = get(&fixture, PROXY, &bad_path).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY, "{}", reply.text());
    assert!(!stored(&fixture, PROXY, &format!("/{bad_path}")).await);
    let bad_strong = sha512(b"expected");
    let bad_strong_path = format!("cache/sha512/{}/{bad_strong}", &bad_strong[..2]);
    stub.publish(&bad_strong_path, "tampered");
    assert_eq!(
        get(&fixture, PROXY, &bad_strong_path).await.status,
        StatusCode::BAD_GATEWAY
    );
    assert!(!stored(&fixture, PROXY, &format!("/{bad_strong_path}")).await);
}

#[tokio::test]
async fn only_the_fixed_paths_reach_the_upstream() {
    let (fixture, stub) = ready_proxy().await;
    let hex = sha256(b"x");
    let wrong_directory = format!(
        "cache/sha256/{}/{hex}",
        if &hex[..2] == "00" { "01" } else { "00" }
    );
    let refused = [
        "packages/x/x.1/opam".to_owned(),
        "packages/x/x.1/files/a.patch".to_owned(),
        "packages".to_owned(),
        wrong_directory,
        format!(
            "cache/sha256/{}/{}",
            hex[..2].to_uppercase(),
            hex.to_uppercase()
        ),
        format!("cache/md5/{}/{}", &hex[..2], &hex[..32]),
        format!("cache/sha256/{}/{}/x", &hex[..2], hex),
        "cache".to_owned(),
        "version".to_owned(),
        "urls.txt".to_owned(),
        "https://example.org/x".to_owned(),
        "index.tar.gz/x".to_owned(),
        String::new(),
    ];
    for path in &refused {
        let reply = get(&fixture, PROXY, path).await;
        assert_eq!(
            reply.status,
            StatusCode::NOT_FOUND,
            "{path}: {}",
            reply.text()
        );
    }
    assert_eq!(stub.total(), 0, "{:?}", stub.seen.lock().unwrap());
    for method in [Method::PUT, Method::DELETE, Method::POST] {
        let reply = send(
            &fixture,
            method.clone(),
            PROXY,
            "index.tar.gz",
            b"x".to_vec(),
        )
        .await;
        assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
    }
    let hosted = send(&fixture, Method::POST, HOSTED, "index.tar.gz", vec![]).await;
    assert_eq!(hosted.status, StatusCode::METHOD_NOT_ALLOWED);
    let anonymous = fixture
        .send(
            Method::GET,
            &format!("/repository/{PROXY}/index.tar.gz"),
            Auth::None,
            None,
            &[],
        )
        .await;
    assert!(
        matches!(
            anonymous.status,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ),
        "{}",
        anonymous.status
    );
    assert_eq!(stub.total(), 0);
}

#[tokio::test]
async fn a_group_is_not_offered() {
    let fixture = ready().await;
    let reply = create(
        &fixture,
        "group",
        json!({"name": "opam-group", "online": true, "group": {"memberNames": [HOSTED]}}),
    )
    .await;
    assert!(reply.status.is_client_error(), "{}", reply.status);
}
