mod support;

use std::time::Duration;

use astrum_blob::s3::MIN_PART_SIZE;
use astrum_blob::{BlobId, BlobStore, Error, S3BlobStore, S3EncryptionKind, S3Keys, Totals};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use support::{
    Db, S3Server, chunks, collect, failing_after, pattern, read_all, s3_keys, s3_settings, s3_store,
};

#[tokio::test]
async fn blobs_go_to_the_bucket_under_the_prefix_and_read_back() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let settings = s3_settings(&server, "astrum-blobs", "/team/blobs/");
    let store = s3_store(&db, "objects", &settings).await;
    store.check().await.unwrap();

    let bytes = pattern(100_000);
    let written = store.put(chunks(&bytes, 10_000)).await.unwrap();
    assert_eq!(written.size, 100_000);
    assert_eq!(written.store, "objects");
    assert_eq!(written.digests.sha1, hex::encode(Sha1::digest(&bytes)));
    assert_eq!(written.digests.sha256, hex::encode(Sha256::digest(&bytes)));
    assert_eq!(written.digests.md5, hex::encode(Md5::digest(&bytes)));

    let id = written.id.to_string();
    let key = format!("team/blobs/content/{}/{}/{id}", &id[0..2], &id[2..4]);
    assert_eq!(store.key_of(written.id), key);
    assert_eq!(server.objects("astrum-blobs"), vec![key]);

    assert_eq!(read_all(&store, written.id).await, bytes);
    assert_eq!(store.size(written.id).await.unwrap(), Some(100_000));
    assert!(store.exists(written.id).await.unwrap());
    assert!(!store.exists(BlobId::new()).await.unwrap());
    assert!(store.read(BlobId::new()).await.unwrap().is_none());
    assert_eq!(store.available_space().await.unwrap(), None);
}

#[tokio::test]
async fn ranged_reads_ask_s3_for_the_slice_only() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let store = s3_store(&db, "objects", &s3_settings(&server, "ranges", "")).await;
    let bytes = pattern(50_000);
    let id = store.put(chunks(&bytes, 50_000)).await.unwrap().id;

    let middle = store.read_range(id, 1_000, 2_000).await.unwrap().unwrap();
    assert_eq!(
        (middle.size, middle.offset, middle.length),
        (50_000, 1_000, 2_000)
    );
    assert_eq!(collect(middle).await, &bytes[1_000..3_000]);

    let tail = store.read_range(id, 49_000, 10_000).await.unwrap().unwrap();
    assert_eq!(collect(tail).await, &bytes[49_000..]);

    let empty = store.read_range(id, 50_000, 1).await.unwrap().unwrap();
    assert_eq!(collect(empty).await, Vec::<u8>::new());

    assert!(matches!(
        store.read_range(id, 50_001, 1).await,
        Err(Error::Range { .. })
    ));
    assert!(
        store
            .read_range(BlobId::new(), 0, 1)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn large_blobs_upload_in_parts() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let store = s3_store(&db, "objects", &s3_settings(&server, "parts", "p"))
        .await
        .with_part_size(MIN_PART_SIZE);
    let bytes = pattern(2 * MIN_PART_SIZE + 12_345);

    let written = store.put(chunks(&bytes, 1 << 20)).await.unwrap();
    assert_eq!(written.size, bytes.len() as u64);
    assert_eq!(written.digests.sha256, hex::encode(Sha256::digest(&bytes)));
    assert_eq!(read_all(&store, written.id).await, bytes);
    let range = store
        .read_range(written.id, MIN_PART_SIZE as u64 - 10, 20)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        collect(range).await,
        &bytes[MIN_PART_SIZE - 10..MIN_PART_SIZE + 10]
    );
}

#[tokio::test]
async fn a_failed_upload_leaves_no_object() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let store = s3_store(&db, "objects", &s3_settings(&server, "failures", ""))
        .await
        .with_part_size(MIN_PART_SIZE);

    let small = store.put(failing_after(&pattern(1_000))).await.unwrap_err();
    assert!(matches!(small, Error::Content(_)), "{small}");
    let large = store
        .put(failing_after(&pattern(MIN_PART_SIZE + 1)))
        .await
        .unwrap_err();
    assert!(matches!(large, Error::Content(_)), "{large}");

    assert!(server.objects("failures").is_empty());
    let leftovers: Vec<_> = support::files_under(server.root())
        .into_iter()
        .filter(|path| path.to_string_lossy().contains(".upload_id-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "the multipart upload was aborted: {leftovers:?}"
    );
    assert_eq!(store.totals().await.unwrap(), Totals::default());
}

#[tokio::test]
async fn deletes_and_totals_follow_the_writes() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let store = s3_store(&db, "objects", &s3_settings(&server, "totals", "")).await;
    let first = store.put(chunks(&pattern(300), 100)).await.unwrap();
    store.put(chunks(&pattern(700), 100)).await.unwrap();
    assert_eq!(
        store.totals().await.unwrap(),
        Totals {
            blob_count: 2,
            total_bytes: 1_000
        }
    );
    assert!(store.delete(first.id).await.unwrap());
    assert!(!store.delete(first.id).await.unwrap());
    assert!(!store.exists(first.id).await.unwrap());
    assert_eq!(
        store.totals().await.unwrap(),
        Totals {
            blob_count: 1,
            total_bytes: 700
        }
    );
    assert_eq!(server.objects("totals").len(), 1);
}

#[tokio::test]
async fn server_side_encryption_settings_are_sent() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let mut settings = s3_settings(&server, "encrypted", "");
    settings.encryption.kind = S3EncryptionKind::S3Managed;
    let store = s3_store(&db, "objects", &settings).await;
    let written = store.put(chunks(&pattern(64), 64)).await.unwrap();
    assert_eq!(read_all(&store, written.id).await, pattern(64));
}

#[tokio::test]
async fn wrong_keys_and_missing_buckets_make_the_store_unavailable() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let settings = s3_settings(&server, "guarded", "");
    s3_store(&db, "objects", &settings).await;

    let wrong = S3BlobStore::open(
        "wrong",
        &settings,
        Some(S3Keys {
            secret_access_key: "not-the-secret".to_owned(),
            ..s3_keys()
        }),
        db.metrics("wrong"),
    )
    .await
    .unwrap();
    assert!(matches!(wrong.check().await, Err(Error::S3 { .. })));
    assert!(wrong.put(chunks(b"x", 1)).await.is_err());

    let missing = S3BlobStore::open(
        "missing",
        &s3_settings(&server, "no-such-bucket", ""),
        Some(s3_keys()),
        db.metrics("missing"),
    )
    .await
    .unwrap();
    assert!(missing.check().await.is_err());
    missing.ensure_bucket().await.unwrap();
    missing.check().await.unwrap();
}

#[tokio::test]
async fn the_pool_size_limits_requests_in_flight() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let mut settings = s3_settings(&server, "pooled", "");
    settings.connection.max_connection_pool_size = Some(1);
    let store = std::sync::Arc::new(s3_store(&db, "objects", &settings).await);
    let id = store.put(chunks(&pattern(1_000), 1_000)).await.unwrap().id;

    let open = store.read(id).await.unwrap().unwrap();
    let second = {
        let store = store.clone();
        tokio::spawn(async move { store.size(id).await })
    };
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        !second.is_finished(),
        "the only connection is held by the open read"
    );
    assert_eq!(collect(open).await, pattern(1_000));
    let size = tokio::time::timeout(Duration::from_secs(10), second)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(size, Some(1_000));
}

#[tokio::test]
async fn listings_page_through_the_bucket_under_the_prefix() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let store = s3_store(&db, "objects", &s3_settings(&server, "listed", "/team/")).await;
    let mut ids = Vec::new();
    for size in [3, 5, 7] {
        ids.push(store.put(chunks(&pattern(size), size)).await.unwrap().id);
    }
    ids.sort();
    let first = store.list(None, 2).await.unwrap();
    assert_eq!(
        first.iter().map(|blob| blob.id).collect::<Vec<_>>(),
        ids[..2]
    );
    let rest = store.list(Some(first[1].id), 2).await.unwrap();
    assert_eq!(
        rest.iter().map(|blob| blob.id).collect::<Vec<_>>(),
        ids[2..]
    );
    assert_eq!(
        first.iter().chain(&rest).map(|blob| blob.size).sum::<u64>(),
        15
    );
}
