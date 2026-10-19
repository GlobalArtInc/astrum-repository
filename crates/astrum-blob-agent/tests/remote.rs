use std::io;
use std::path::Path;
use std::sync::Arc;

use astrum_blob::config::{self, Settings, StoreConfig};
use astrum_blob::{
    BlobId, BlobStore, ContentStream, Error, Metrics, RemoteBlobStore, RemoteSettings, StoreKind,
};
use astrum_blob_agent::{Agent, serve};
use astrum_db::Database;
use astrum_testkit::TestDatabase;
use astrum_testkit::tls::TestCertificates;
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use reqwest::StatusCode;
use tokio::net::TcpListener;

const TOKEN: &str = "a-test-token-that-is-long-enough-0123456789";

struct Db {
    _test: TestDatabase,
    db: Database,
}

impl Db {
    async fn new() -> Self {
        let test = TestDatabase::create().await;
        let db = test.migrated().await;
        Self { _test: test, db }
    }

    async fn store(&self, name: &str, settings: RemoteSettings) -> RemoteBlobStore {
        config::insert(
            self.db.conn(),
            &StoreConfig {
                name: name.to_owned(),
                settings: Settings::Remote(settings.clone()),
                soft_quota: None,
            },
        )
        .await
        .unwrap();
        RemoteBlobStore::open(
            name,
            &settings,
            TOKEN.to_owned(),
            Metrics::new(self.db.conn().clone(), name),
        )
        .unwrap()
    }
}

async fn start(dir: &Path, tls: Option<Arc<rustls::ServerConfig>>) -> std::net::SocketAddr {
    let agent = Agent::open(dir, TOKEN).await.unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(serve(listener, agent.router(), tls, std::future::pending()));
    address
}

fn settings(url: String) -> RemoteSettings {
    RemoteSettings {
        url,
        token: None,
        certificate: None,
    }
}

fn pattern(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index * 31 % 251) as u8).collect()
}

fn chunks(bytes: &[u8], size: usize) -> ContentStream {
    let parts: Vec<io::Result<Bytes>> = bytes
        .chunks(size)
        .map(|chunk| Ok(Bytes::copy_from_slice(chunk)))
        .collect();
    stream::iter(parts).boxed()
}

async fn collect(content: astrum_blob::Content) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut stream = content.stream;
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }
    bytes
}

fn files_under(dir: &Path) -> usize {
    let mut count = 0;
    let mut pending = vec![dir.to_owned()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

#[tokio::test]
async fn the_agent_answers_only_with_the_token() {
    let dir = tempfile::tempdir().unwrap();
    let address = start(dir.path(), None).await;
    let client = reqwest::Client::new();
    let url = format!("http://{address}/v1/status");
    assert_eq!(
        client.get(&url).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .get(&url)
            .bearer_auth("a-test-token-that-is-long-enough-012345678x")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let reply = client.get(&url).bearer_auth(TOKEN).send().await.unwrap();
    assert_eq!(reply.status(), StatusCode::OK);
    let status: serde_json::Value = serde_json::from_slice(&reply.bytes().await.unwrap()).unwrap();
    assert!(status["availableSpace"].as_u64().unwrap() > 0, "{status}");
    assert_eq!(
        client
            .get(format!("http://{address}/v1/blobs/not-an-id"))
            .bearer_auth(TOKEN)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn a_short_token_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let error = Agent::open(dir.path(), "short").await.err().unwrap();
    assert!(error.to_string().contains("at least 32"), "{error}");
}

#[tokio::test]
async fn a_remote_store_keeps_blobs_on_the_agent() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let address = start(dir.path(), None).await;
    let store = db
        .store("remote", settings(format!("http://{address}/")))
        .await;
    assert_eq!(store.kind(), StoreKind::Remote);
    store.check().await.unwrap();
    assert!(store.available_space().await.unwrap().unwrap() > 0);

    let bytes = pattern(300_000);
    let written = store.put(chunks(&bytes, 7_000)).await.unwrap();
    assert_eq!(written.store, "remote");
    assert_eq!(written.size, bytes.len() as u64);
    assert_eq!(written.digests.sha1.len(), 40);
    assert!(
        dir.path()
            .join("content")
            .join(written.id.location())
            .is_file()
    );

    let content = store.read(written.id).await.unwrap().unwrap();
    assert_eq!(
        (content.size, content.offset, content.length),
        (300_000, 0, 300_000)
    );
    assert_eq!(collect(content).await, bytes);

    let range = store
        .read_range(written.id, 1_000, 2_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (range.size, range.offset, range.length),
        (300_000, 1_000, 2_000)
    );
    assert_eq!(collect(range).await, bytes[1_000..3_000]);
    let tail = store
        .read_range(written.id, 299_990, 100)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tail.length, 10);
    assert_eq!(collect(tail).await, bytes[299_990..]);
    assert!(matches!(
        store.read_range(written.id, 300_001, 1).await,
        Err(Error::Range {
            offset: 300_001,
            size: 300_000
        })
    ));

    assert_eq!(store.size(written.id).await.unwrap(), Some(300_000));
    assert!(store.exists(written.id).await.unwrap());
    let unknown = BlobId::new();
    assert_eq!(store.size(unknown).await.unwrap(), None);
    assert!(store.read(unknown).await.unwrap().is_none());

    let second = store.put(astrum_blob::content_of("x")).await.unwrap();
    let totals = store.totals().await.unwrap();
    assert_eq!((totals.blob_count, totals.total_bytes), (2, 300_001));

    let mut listed: Vec<BlobId> = store
        .list(None, 10)
        .await
        .unwrap()
        .into_iter()
        .map(|blob| blob.id)
        .collect();
    let mut expected = vec![written.id, second.id];
    expected.sort();
    listed.sort();
    assert_eq!(listed, expected);
    let after = store.list(Some(expected[0]), 10).await.unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, expected[1]);
    assert_eq!(store.list(None, 1).await.unwrap().len(), 1);

    assert!(store.delete(written.id).await.unwrap());
    assert!(!store.delete(written.id).await.unwrap());
    assert_eq!(store.size(written.id).await.unwrap(), None);
    let totals = store.totals().await.unwrap();
    assert_eq!((totals.blob_count, totals.total_bytes), (1, 1));
}

#[tokio::test]
async fn an_upload_that_breaks_leaves_nothing_behind() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let address = start(dir.path(), None).await;
    let store = db
        .store("remote", settings(format!("http://{address}")))
        .await;
    let broken: ContentStream = stream::iter(vec![
        Ok(Bytes::from(pattern(100_000))),
        Err(io::Error::other("the client went away")),
    ])
    .boxed();
    let error = store.put(broken).await.unwrap_err();
    assert!(matches!(error, Error::Content(_)), "{error}");
    let mut left = 0;
    for _ in 0..50 {
        left = files_under(dir.path());
        if left == 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(left, 0);
    assert_eq!(store.totals().await.unwrap().blob_count, 0);
}

#[tokio::test]
async fn an_agent_that_is_gone_is_reported() {
    let db = Db::new().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let store = db
        .store("remote", settings(format!("http://{address}")))
        .await;
    let error = store.check().await.unwrap_err();
    assert!(matches!(error, Error::Remote { .. }), "{error}");
    assert!(error.to_string().contains("\"remote\""), "{error}");
    assert!(store.read(BlobId::new()).await.is_err());
    assert!(store.put(astrum_blob::content_of("x")).await.is_err());
}

#[tokio::test]
async fn a_wrong_token_is_reported() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let address = start(dir.path(), None).await;
    let settings = settings(format!("http://{address}"));
    config::insert(
        db.db.conn(),
        &StoreConfig {
            name: "remote".to_owned(),
            settings: Settings::Remote(settings.clone()),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    let store = RemoteBlobStore::open(
        "remote",
        &settings,
        "not-the-token-of-this-agent-at-all-000000".to_owned(),
        Metrics::new(db.db.conn().clone(), "remote"),
    )
    .unwrap();
    let error = store.check().await.unwrap_err();
    assert!(error.to_string().contains("401"), "{error}");
    assert!(
        error
            .to_string()
            .contains("a valid bearer token is required"),
        "{error}"
    );
}

#[tokio::test]
async fn an_agent_with_its_own_certificate_is_trusted_when_told() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let certificates = TestCertificates::generate(&["localhost"]);
    let address = start(dir.path(), Some(certificates.server_config())).await;
    let url = format!("https://localhost:{}", address.port());

    let untrusted = db.store("untrusted", settings(url.clone())).await;
    assert!(untrusted.check().await.is_err());

    let trusted = db
        .store(
            "trusted",
            RemoteSettings {
                certificate: Some(certificates.ca_pem.clone()),
                ..settings(url)
            },
        )
        .await;
    trusted.check().await.unwrap();
    let bytes = pattern(70_000);
    let written = trusted.put(chunks(&bytes, 9_000)).await.unwrap();
    let content = trusted.read(written.id).await.unwrap().unwrap();
    assert_eq!(collect(content).await, bytes);
}

#[test]
fn settings_are_checked_before_use() {
    let metrics = || Metrics::detached("remote");
    let error = RemoteBlobStore::open(
        "remote",
        &settings("ftp://x".to_owned()),
        TOKEN.to_owned(),
        metrics(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("http://"), "{error}");
    let error = RemoteBlobStore::open(
        "remote",
        &settings("http://x".to_owned()),
        String::new(),
        metrics(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("token"), "{error}");
    let error = RemoteBlobStore::open(
        "remote",
        &RemoteSettings {
            certificate: Some("not a certificate".to_owned()),
            ..settings("https://x".to_owned())
        },
        TOKEN.to_owned(),
        metrics(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("certificate"), "{error}");
}
