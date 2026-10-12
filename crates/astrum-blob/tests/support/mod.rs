#![allow(dead_code, unused_imports)]

use std::io;

use astrum_blob::config::{self, FileSettings, Settings, StoreConfig};
use astrum_blob::{
    BlobStore, Content, ContentStream, Metrics, S3BlobStore, S3Bucket, S3Connection, S3Keys,
    S3Settings,
};
use astrum_db::Database;
use astrum_testkit::TestDatabase;
use bytes::Bytes;
use futures_util::{StreamExt, stream};

pub use astrum_testkit::s3::{ACCESS_KEY, S3Server, SECRET_KEY, files_under};

pub struct Db {
    pub test: TestDatabase,
    pub db: Database,
}

impl Db {
    pub async fn new() -> Self {
        let test = TestDatabase::create().await;
        let db = test.migrated().await;
        Self { test, db }
    }

    pub fn metrics(&self, store: &str) -> Metrics {
        Metrics::new(self.db.conn().clone(), store)
    }

    pub async fn register(&self, config: &StoreConfig) {
        config::insert(self.db.conn(), config).await.unwrap();
    }

    pub async fn register_file(&self, name: &str, path: &str) {
        self.register(&file_config(name, path)).await;
    }
}

pub fn file_config(name: &str, path: &str) -> StoreConfig {
    StoreConfig {
        name: name.to_owned(),
        settings: Settings::File(FileSettings {
            path: path.to_owned(),
        }),
        soft_quota: None,
    }
}

pub fn pattern(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index * 31 % 251) as u8).collect()
}

pub fn chunks(bytes: &[u8], size: usize) -> ContentStream {
    let parts: Vec<io::Result<Bytes>> = bytes
        .chunks(size.max(1))
        .map(|chunk| Ok(Bytes::copy_from_slice(chunk)))
        .collect();
    stream::iter(parts).boxed()
}

pub fn failing_after(bytes: &[u8]) -> ContentStream {
    let parts = vec![
        Ok(Bytes::copy_from_slice(bytes)),
        Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "the client went away",
        )),
    ];
    stream::iter(parts).boxed()
}

pub fn stalled_after(bytes: &[u8]) -> ContentStream {
    stream::once(std::future::ready(Ok(Bytes::copy_from_slice(bytes))))
        .chain(stream::pending())
        .boxed()
}

pub async fn collect(content: Content) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut stream = content.stream;
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }
    assert_eq!(bytes.len() as u64, content.length);
    bytes
}

pub async fn read_all(store: &dyn BlobStore, id: astrum_blob::BlobId) -> Vec<u8> {
    collect(store.read(id).await.unwrap().expect("the blob exists")).await
}

pub fn s3_settings(server: &S3Server, bucket: &str, prefix: &str) -> S3Settings {
    S3Settings {
        bucket: S3Bucket {
            region: "DEFAULT".to_owned(),
            name: bucket.to_owned(),
            prefix: prefix.to_owned(),
            expiration: None,
        },
        connection: S3Connection {
            endpoint: Some(server.endpoint()),
            force_path_style: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn s3_keys() -> S3Keys {
    S3Keys {
        access_key_id: ACCESS_KEY.to_owned(),
        secret_access_key: SECRET_KEY.to_owned(),
        session_token: None,
    }
}

pub async fn s3_store(db: &Db, name: &str, settings: &S3Settings) -> S3BlobStore {
    let config = StoreConfig {
        name: name.to_owned(),
        settings: Settings::S3(Box::new(settings.clone())),
        soft_quota: None,
    };
    db.register(&config).await;
    let store = S3BlobStore::open(name, settings, Some(s3_keys()), db.metrics(name))
        .await
        .unwrap();
    store.ensure_bucket().await.unwrap();
    store
}
