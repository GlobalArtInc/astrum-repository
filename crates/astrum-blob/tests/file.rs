mod support;

use std::time::{Duration, SystemTime};

use astrum_blob::file::{CONTENT_DIR, TEMP_DIR};
use astrum_blob::{BlobId, BlobStore, Error, FileBlobStore, Totals};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use support::{Db, chunks, collect, failing_after, files_under, pattern, read_all, stalled_after};

async fn store(db: &Db, root: &std::path::Path) -> FileBlobStore {
    db.register_file("files", &root.to_string_lossy()).await;
    FileBlobStore::open("files", root, db.metrics("files"))
        .await
        .unwrap()
}

#[tokio::test]
async fn a_written_blob_reads_back_with_its_size_and_checksums() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let bytes = pattern(300_000);

    let written = store.put(chunks(&bytes, 7_777)).await.unwrap();
    assert_eq!(written.store, "files");
    assert_eq!(written.size, 300_000);
    assert_eq!(written.digests.sha1, hex::encode(Sha1::digest(&bytes)));
    assert_eq!(written.digests.sha256, hex::encode(Sha256::digest(&bytes)));
    assert_eq!(written.digests.md5, hex::encode(Md5::digest(&bytes)));

    assert_eq!(read_all(&store, written.id).await, bytes);
    assert_eq!(store.size(written.id).await.unwrap(), Some(300_000));
    assert!(store.exists(written.id).await.unwrap());

    let location = store.path_of(written.id);
    let id = written.id.to_string();
    assert_eq!(
        location,
        dir.path()
            .join(CONTENT_DIR)
            .join(&id[0..2])
            .join(&id[2..4])
            .join(&id)
    );
    assert_eq!(files_under(dir.path()), vec![location]);
}

#[tokio::test]
async fn an_empty_blob_is_still_a_blob() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let written = store.put(chunks(&[], 1)).await.unwrap();
    assert_eq!(written.size, 0);
    assert_eq!(
        written.digests.sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert!(read_all(&store, written.id).await.is_empty());
    let range = store.read_range(written.id, 0, 10).await.unwrap().unwrap();
    assert_eq!(range.length, 0);
}

#[tokio::test]
async fn ranged_reads_return_the_asked_slice() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let bytes = pattern(200_000);
    let id = store.put(chunks(&bytes, 65_536)).await.unwrap().id;

    let middle = store
        .read_range(id, 70_000, 100_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (middle.size, middle.offset, middle.length),
        (200_000, 70_000, 100_000)
    );
    assert_eq!(collect(middle).await, &bytes[70_000..170_000]);

    let tail = store.read_range(id, 199_990, 1_000).await.unwrap().unwrap();
    assert_eq!(tail.length, 10, "a range past the end stops at the end");
    assert_eq!(collect(tail).await, &bytes[199_990..]);

    let at_end = store.read_range(id, 200_000, 5).await.unwrap().unwrap();
    assert_eq!(collect(at_end).await, Vec::<u8>::new());

    assert!(matches!(
        store.read_range(id, 200_001, 5).await,
        Err(Error::Range {
            offset: 200_001,
            size: 200_000
        })
    ));
    assert!(
        store
            .read_range(BlobId::new(), 0, 5)
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.read(BlobId::new()).await.unwrap().is_none());
}

#[tokio::test]
async fn deletes_remove_the_blob_once_and_keep_the_totals() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    assert_eq!(store.totals().await.unwrap(), Totals::default());

    let first = store.put(chunks(&pattern(1_000), 100)).await.unwrap();
    let second = store.put(chunks(&pattern(2_500), 100)).await.unwrap();
    assert_eq!(
        store.totals().await.unwrap(),
        Totals {
            blob_count: 2,
            total_bytes: 3_500
        }
    );

    assert!(store.delete(first.id).await.unwrap());
    assert!(!store.delete(first.id).await.unwrap(), "only once");
    assert!(!store.exists(first.id).await.unwrap());
    assert_eq!(store.size(first.id).await.unwrap(), None);
    assert!(store.exists(second.id).await.unwrap());
    assert_eq!(
        store.totals().await.unwrap(),
        Totals {
            blob_count: 1,
            total_bytes: 2_500
        }
    );

    let reopened = FileBlobStore::open("files", dir.path(), db.metrics("files"))
        .await
        .unwrap();
    assert_eq!(reopened.totals().await.unwrap().blob_count, 1);
}

#[tokio::test]
async fn a_failed_upload_leaves_nothing_behind() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;

    let error = store
        .put(failing_after(&pattern(10_000)))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Content(_)), "{error}");
    assert!(files_under(dir.path()).is_empty());
    assert_eq!(store.totals().await.unwrap(), Totals::default());
}

#[tokio::test]
async fn an_interrupted_upload_never_shows_under_a_final_name() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(store(&db, dir.path()).await);

    let writer = {
        let store = store.clone();
        tokio::spawn(async move { store.put(stalled_after(&pattern(50_000))).await })
    };
    let temp = dir.path().join(TEMP_DIR);
    for _ in 0..200 {
        if !files_under(&temp).is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(files_under(&temp).len(), 1, "the write is in progress");
    assert!(files_under(&dir.path().join(CONTENT_DIR)).is_empty());

    writer.abort();
    assert!(writer.await.unwrap_err().is_cancelled());
    assert!(files_under(dir.path()).is_empty());
    assert_eq!(store.totals().await.unwrap(), Totals::default());
}

#[tokio::test]
async fn leftovers_of_a_crash_are_swept_when_the_store_opens() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let kept = store.put(chunks(&pattern(10), 10)).await.unwrap();

    let temp = dir.path().join(TEMP_DIR);
    let stale = temp.join(format!("{}.part", BlobId::new()));
    let fresh = temp.join(format!("{}.part", BlobId::new()));
    let foreign = temp.join("notes.txt");
    for path in [&stale, &fresh, &foreign] {
        std::fs::write(path, b"half a blob").unwrap();
    }
    let long_ago = SystemTime::now() - Duration::from_secs(2 * 3600);
    for path in [&stale, &foreign] {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
    }

    let reopened = FileBlobStore::open("files", dir.path(), db.metrics("files"))
        .await
        .unwrap();
    assert!(!stale.exists(), "an old unfinished write is removed");
    assert!(fresh.exists(), "a write that may still run is kept");
    assert!(foreign.exists(), "files the store did not write are kept");
    assert!(reopened.exists(kept.id).await.unwrap());
}

#[tokio::test]
async fn the_store_reports_its_space_and_health() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = store(&db, &root).await;
    assert!(store.available_space().await.unwrap().unwrap() > 0);
    store.check().await.unwrap();
    assert!(
        files_under(&root).is_empty(),
        "the check cleans up after itself"
    );

    std::fs::remove_dir_all(&root).unwrap();
    assert!(store.check().await.is_err());
}

#[tokio::test]
async fn only_an_empty_store_removes_its_directory() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = store(&db, &root).await;
    let written = store.put(chunks(&pattern(10), 10)).await.unwrap();
    assert!(!store.remove_if_empty().await.unwrap());
    assert!(store.exists(written.id).await.unwrap());

    store.delete(written.id).await.unwrap();
    assert!(store.remove_if_empty().await.unwrap());
    assert!(!root.exists());
}
