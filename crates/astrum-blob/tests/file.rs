mod support;

use std::os::unix::fs::MetadataExt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use astrum_blob::file::{CONTENT_DIR, TEMP_DIR};
use astrum_blob::{BlobId, BlobStore, Error, FileBlobStore, LinkFault, Totals};
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

const FAULTS: [Option<LinkFault>; 3] = [None, Some(LinkFault::Refused), Some(LinkFault::Unowned)];

async fn store_with(db: &Db, root: &std::path::Path, fault: Option<LinkFault>) -> FileBlobStore {
    let name = match fault {
        None => "linked",
        Some(LinkFault::Refused) => "refused",
        Some(LinkFault::Unowned) => "unowned",
    };
    db.register_file(name, &root.to_string_lossy()).await;
    let store = FileBlobStore::open(name, root, db.metrics(name))
        .await
        .unwrap();
    match fault {
        None => store,
        Some(fault) => store.with_link_fault(fault),
    }
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

#[tokio::test]
async fn listings_page_through_the_blobs_in_id_order() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let mut written = Vec::new();
    for size in 1..=5 {
        written.push(store.put(chunks(&pattern(size), 1)).await.unwrap());
    }
    written.sort_by_key(|blob| blob.id);
    let first = store.list(None, 3).await.unwrap();
    assert_eq!(
        first.iter().map(|blob| blob.id).collect::<Vec<_>>(),
        written[..3].iter().map(|blob| blob.id).collect::<Vec<_>>()
    );
    assert_eq!(first[0].size, written[0].size);
    let rest = store.list(Some(first[2].id), 3).await.unwrap();
    assert_eq!(
        rest.iter().map(|blob| blob.id).collect::<Vec<_>>(),
        written[3..].iter().map(|blob| blob.id).collect::<Vec<_>>()
    );
    assert!(store.list(Some(rest[1].id), 3).await.unwrap().is_empty());
    assert!(
        first[0].modified <= SystemTime::now()
            && first[0].modified > SystemTime::now() - Duration::from_secs(60)
    );
}

#[tokio::test]
async fn a_duplicate_outlives_the_original_whether_it_is_linked_or_copied() {
    let db = Db::new().await;
    for fault in FAULTS {
        let linked = fault.is_none();
        let dir = tempfile::tempdir().unwrap();
        let store = store_with(&db, dir.path(), fault).await;
        let bytes = pattern(300_000);
        let original = store.put(chunks(&bytes, 7_777)).await.unwrap();

        let copy = store
            .duplicate(original.id, &original.digests)
            .await
            .unwrap();
        assert_ne!(copy.id, original.id);
        assert_eq!((copy.store.as_str(), copy.size), (store.name(), 300_000));
        assert_eq!(copy.digests, original.digests);
        let first = std::fs::metadata(store.path_of(original.id)).unwrap();
        let second = std::fs::metadata(store.path_of(copy.id)).unwrap();
        assert_eq!(first.ino() == second.ino(), linked, "{fault:?}");
        assert_eq!(second.nlink(), if linked { 2 } else { 1 }, "{fault:?}");
        assert_eq!(read_all(&store, original.id).await, bytes);
        assert_eq!(store.totals().await.unwrap().blob_count, 2);

        assert!(store.delete(original.id).await.unwrap());
        assert_eq!(read_all(&store, copy.id).await, bytes);
        assert_eq!(
            store.totals().await.unwrap(),
            Totals {
                blob_count: 1,
                total_bytes: 300_000
            }
        );
        assert!(files_under(&dir.path().join(TEMP_DIR)).is_empty());
    }
}

#[tokio::test]
async fn a_duplicate_never_looks_old_to_compaction() {
    let db = Db::new().await;
    for fault in FAULTS {
        let linked = fault.is_none();
        let dir = tempfile::tempdir().unwrap();
        let store = store_with(&db, dir.path(), fault).await;
        let original = store.put(chunks(&pattern(5_000), 1_000)).await.unwrap();
        let long_ago = SystemTime::now() - Duration::from_secs(3 * 86_400);
        std::fs::File::options()
            .write(true)
            .open(store.path_of(original.id))
            .unwrap()
            .set_modified(long_ago)
            .unwrap();

        let copy = store
            .duplicate(original.id, &original.digests)
            .await
            .unwrap();
        let listed = store.list(None, 10).await.unwrap();
        let modified = |id| listed.iter().find(|blob| blob.id == id).unwrap().modified;
        let recent = SystemTime::now() - Duration::from_secs(60);
        assert!(modified(copy.id) > recent, "the copy is new: {fault:?}");
        assert_eq!(
            modified(original.id) > recent,
            linked,
            "a link shares the inode: {fault:?}"
        );
    }
}

#[tokio::test]
async fn a_linked_duplicate_does_not_check_the_digests_it_is_given() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store_with(&db, dir.path(), None).await;
    let original = store.put(chunks(&pattern(5_000), 1_000)).await.unwrap();

    let mut other = original.digests.clone();
    other.sha256 = "0".repeat(64);
    let copy = store.duplicate(original.id, &other).await.unwrap();
    assert_eq!(copy.size, 5_000, "the size is the one on disk");
    assert_eq!(copy.digests, other, "the digests are the caller's own");

    let mut unknown = original.digests.clone();
    unknown.sha256 = String::new();
    let error = store.duplicate(original.id, &unknown).await.unwrap_err();
    assert!(matches!(error, Error::NoDigest { .. }), "{error}");
    assert_eq!(files_under(&dir.path().join(TEMP_DIR)).len(), 0);
}

#[tokio::test]
async fn a_copied_duplicate_with_other_digests_fails_and_leaves_nothing_behind() {
    let db = Db::new().await;
    for fault in [LinkFault::Refused, LinkFault::Unowned] {
        let dir = tempfile::tempdir().unwrap();
        let store = store_with(&db, dir.path(), Some(fault)).await;
        let original = store.put(chunks(&pattern(5_000), 1_000)).await.unwrap();

        let mut wrong = original.digests.clone();
        wrong.sha256 = "0".repeat(64);
        let error = store.duplicate(original.id, &wrong).await.unwrap_err();
        assert!(
            matches!(
                error,
                Error::Digest {
                    algorithm: "SHA-256",
                    ..
                }
            ),
            "{error}"
        );

        let mut other = original.digests.clone();
        other.md5 = "0".repeat(32);
        let error = store.duplicate(original.id, &other).await.unwrap_err();
        assert!(
            matches!(
                error,
                Error::Digest {
                    algorithm: "MD5",
                    ..
                }
            ),
            "{error}"
        );

        let mut unknown = original.digests.clone();
        unknown.sha256 = String::new();
        let error = store.duplicate(original.id, &unknown).await.unwrap_err();
        assert!(matches!(error, Error::NoDigest { .. }), "{error}");

        assert_eq!(
            files_under(dir.path()),
            vec![store.path_of(original.id)],
            "{fault:?}"
        );
        assert_eq!(store.totals().await.unwrap().blob_count, 1);
        assert_eq!(read_all(&store, original.id).await, pattern(5_000));
    }
}

#[tokio::test]
async fn a_duplicate_of_a_blob_that_is_not_there_is_reported() {
    let db = Db::new().await;
    for fault in FAULTS {
        let dir = tempfile::tempdir().unwrap();
        let store = store_with(&db, dir.path(), fault).await;
        let kept = store.put(chunks(&pattern(100), 100)).await.unwrap();
        let error = store
            .duplicate(BlobId::new(), &kept.digests)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Missing { .. }), "{fault:?}: {error}");
        assert_eq!(files_under(dir.path()), vec![store.path_of(kept.id)]);
    }
}

#[tokio::test]
async fn duplicating_twice_makes_two_independent_blobs() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = store(&db, dir.path()).await;
    let bytes = pattern(2_000);
    let original = store.put(chunks(&bytes, 500)).await.unwrap();
    let first = store
        .duplicate(original.id, &original.digests)
        .await
        .unwrap();
    let second = store
        .duplicate(original.id, &original.digests)
        .await
        .unwrap();
    assert_ne!(first.id, second.id);
    assert!(store.delete(first.id).await.unwrap());
    assert!(store.delete(original.id).await.unwrap());
    assert_eq!(read_all(&store, second.id).await, bytes);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_duplicate_racing_a_delete_is_whole_or_reported_missing() {
    let db = Db::new().await;
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(store(&db, dir.path()).await);
    let bytes = pattern(40_000);
    let mut copied = 0;
    for _ in 0..40 {
        let original = store.put(chunks(&bytes, 10_000)).await.unwrap();
        let duplicate = {
            let store = store.clone();
            let digests = original.digests.clone();
            tokio::spawn(async move { store.duplicate(original.id, &digests).await })
        };
        let delete = {
            let store = store.clone();
            tokio::spawn(async move { store.delete(original.id).await })
        };
        assert!(delete.await.unwrap().unwrap());
        match duplicate.await.unwrap() {
            Ok(copy) => {
                copied += 1;
                assert_eq!(read_all(store.as_ref(), copy.id).await, bytes);
            }
            Err(error) => assert!(matches!(error, Error::Missing { .. }), "{error}"),
        }
    }
    assert!(files_under(&dir.path().join(TEMP_DIR)).is_empty());
    assert_eq!(store.totals().await.unwrap().blob_count, copied);
}
