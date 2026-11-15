mod support;

use std::time::{Duration, Instant};

use astrum_blob::file::{CONTENT_DIR, TEMP_DIR};
use astrum_blob::{BlobId, BlobStore, Digests, Error, FileBlobStore, Limiter, copy};
use support::{Db, S3Server, chunks, files_under, pattern, read_all, s3_settings, s3_store};

async fn file_store(db: &Db, name: &str, root: &std::path::Path) -> FileBlobStore {
    db.register_file(name, &root.to_string_lossy()).await;
    FileBlobStore::open(name, root, db.metrics(name))
        .await
        .unwrap()
}

#[tokio::test]
async fn a_copy_between_file_stores_keeps_the_bytes_and_the_digests() {
    let db = Db::new().await;
    let (hot, cold) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let from = file_store(&db, "hot", hot.path()).await;
    let to = file_store(&db, "cold", cold.path()).await;
    let bytes = pattern(300_000);
    let original = from.put(chunks(&bytes, 7_777)).await.unwrap();

    let copied = copy(&from, &original.id, &original.digests, &to, None)
        .await
        .unwrap();
    assert_ne!(copied.id, original.id);
    assert_eq!((copied.store.as_str(), copied.size), ("cold", 300_000));
    assert_eq!(copied.digests, original.digests);
    assert_eq!(read_all(&to, copied.id).await, bytes);
    assert_eq!(read_all(&from, original.id).await, bytes);
    assert!(files_under(&cold.path().join(TEMP_DIR)).is_empty());
}

#[tokio::test]
async fn a_copy_to_an_s3_store_keeps_the_bytes_and_the_digests() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let hot = tempfile::tempdir().unwrap();
    let from = file_store(&db, "hot", hot.path()).await;
    let to = s3_store(&db, "objects", &s3_settings(&server, "cold-blobs", "")).await;
    let bytes = pattern(200_000);
    let original = from.put(chunks(&bytes, 10_000)).await.unwrap();

    let copied = copy(&from, &original.id, &original.digests, &to, None)
        .await
        .unwrap();
    assert_eq!((copied.store.as_str(), copied.size), ("objects", 200_000));
    assert_eq!(copied.digests, original.digests);
    assert_eq!(read_all(&to, copied.id).await, bytes);
}

#[tokio::test]
async fn a_source_that_does_not_match_its_digests_leaves_nothing_in_the_target() {
    let db = Db::new().await;
    let (hot, cold) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let from = file_store(&db, "hot", hot.path()).await;
    let to = file_store(&db, "cold", cold.path()).await;
    let original = from.put(chunks(&pattern(50_000), 5_000)).await.unwrap();
    let other = from.put(chunks(&pattern(49_999), 5_000)).await.unwrap();
    let wrong = Digests {
        sha256: other.digests.sha256.clone(),
        ..original.digests.clone()
    };

    let error = copy(&from, &original.id, &wrong, &to, None)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Digest { .. }), "{error}");
    assert!(to.list(None, 10).await.unwrap().is_empty());
    assert!(files_under(&cold.path().join(CONTENT_DIR)).is_empty());
    assert!(files_under(&cold.path().join(TEMP_DIR)).is_empty());

    let error = copy(&from, &BlobId::new(), &original.digests, &to, None)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Missing { .. }), "{error}");
}

#[tokio::test]
async fn a_limiter_paces_a_copy_to_its_rate() {
    let db = Db::new().await;
    let (hot, cold) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let from = file_store(&db, "hot", hot.path()).await;
    let to = file_store(&db, "cold", cold.path()).await;
    let original = from.put(chunks(&pattern(200_000), 10_000)).await.unwrap();
    let limiter = Limiter::new(200_000).unwrap();

    let started = Instant::now();
    copy(&from, &original.id, &original.digests, &to, Some(&limiter))
        .await
        .unwrap();
    let taken = started.elapsed();
    assert!(
        taken >= Duration::from_millis(900) && taken < Duration::from_millis(1_600),
        "{taken:?}"
    );
}
