use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use astrum_blob::BlobStores;
use astrum_content::{Content, zip};
use astrum_db::Database;
use astrum_scan::osv::{
    self, AdvisorySource, Fetched, HttpDump, Outcome, SourceError, SyncError, TempFile, Upload,
};
use astrum_security::SecretBox;
use astrum_testkit::TestDatabase;
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::json;
use tokio::sync::Notify;

fn never() -> bool {
    false
}

fn advisory(id: &str, modified: &str, package: &str) -> Vec<u8> {
    advisory_in(id, modified, "npm", package)
}

fn advisory_in(id: &str, modified: &str, ecosystem: &str, package: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": id,
        "modified": modified,
        "summary": "a flaw",
        "aliases": [format!("CVE-{id}")],
        "database_specific": {"severity": "HIGH"},
        "affected": [{
            "package": {"ecosystem": ecosystem, "name": package},
            "ranges": [{"type": "SEMVER", "events": [{"introduced": "0"}, {"fixed": "1.2.3"}]}],
            "versions": ["1.0.0"]
        }]
    }))
    .unwrap()
}

async fn stage(dir: &Path, archive: Vec<u8>) -> TempFile {
    TempFile::from_stream(
        dir,
        futures_util::stream::iter(vec![Ok::<_, Infallible>(archive)]),
        osv::MAX_ARCHIVE_BYTES,
    )
    .await
    .unwrap()
}

async fn import(db: &Database, dir: &Path, archive: Vec<u8>) -> Outcome {
    let source = Upload::new(stage(dir, archive).await);
    osv::sync(db.conn(), &source, "npm", &never).await.unwrap()
}

fn stats(outcome: Outcome) -> osv::Stats {
    match outcome {
        Outcome::Imported(stats) => stats,
        Outcome::NotModified => panic!("the archive was not imported"),
    }
}

async fn count(db: &Database, sql: &str) -> i64 {
    db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn execute(db: &Database, sql: &str) {
    db.conn()
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

struct Setup {
    _test: TestDatabase,
    db: Database,
    dir: tempfile::TempDir,
}

async fn setup() -> Setup {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    Setup {
        _test: test,
        db,
        dir: tempfile::tempdir().unwrap(),
    }
}

#[tokio::test]
async fn good_records_are_stored_and_the_feed_and_revision_move() {
    let s = setup().await;
    execute(
        &s.db,
        "INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) \
         VALUES ('npm', 'left-pad', '1.0.0', 'pkg:npm/left-pad@1.0.0', now()), \
                ('npm', 'other', '1.0.0', 'pkg:npm/other@1.0.0', now())",
    )
    .await;
    let archive = zip::write(&[
        (
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "left-pad"),
        ),
        (
            "GHSA-2.json",
            &advisory("GHSA-2", "2024-01-02T00:00:00Z", "right-pad"),
        ),
        ("notes/", b""),
    ]);
    let stats = stats(import(&s.db, s.dir.path(), archive).await);
    assert_eq!((stats.records, stats.changed, stats.skipped), (2, 2, 0));
    assert_eq!(stats.invalidated, 1);
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 2);
    assert_eq!(
        count(&s.db, "SELECT count(*) AS n FROM advisory_affected").await,
        2
    );
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE severity = 'high' AND aliases = '{CVE-GHSA-1}'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        1
    );
    let feeds = osv::feeds(s.db.conn()).await.unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!((feeds[0].records, feeds[0].last_error.clone()), (2, None));
    assert!(feeds[0].synced_at.is_some());
    assert_eq!(
        astrum_db::revision::current(s.db.conn(), astrum_db::revision::ADVISORIES)
            .await
            .unwrap(),
        1
    );
    assert_eq!(std::fs::read_dir(s.dir.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn malformed_oversized_and_unsafe_entries_are_skipped_and_counted() {
    let s = setup().await;
    let good = advisory("GHSA-1", "2024-01-01T00:00:00Z", "left-pad");
    let huge = vec![b'a'; 2 * 1024 * 1024];
    let archive = zip::write(&[
        ("GHSA-1.json", &good),
        ("GHSA-BAD.json", b"{not json"),
        ("GHSA-HUGE.json", &huge),
        (
            "../GHSA-3.json",
            &advisory("GHSA-3", "2024-01-01T00:00:00Z", "x"),
        ),
        (
            "/GHSA-4.json",
            &advisory("GHSA-4", "2024-01-01T00:00:00Z", "x"),
        ),
        (
            "a/GHSA-5.json",
            &advisory("GHSA-5", "2024-01-01T00:00:00Z", "x"),
        ),
        (
            "..\\GHSA-6.json",
            &advisory("GHSA-6", "2024-01-01T00:00:00Z", "x"),
        ),
        (
            "GHSA-7.json",
            &advisory("GHSA-OTHER", "2024-01-01T00:00:00Z", "x"),
        ),
        ("readme.txt", b"hello"),
    ]);
    let stats = stats(import(&s.db, s.dir.path(), archive).await);
    assert_eq!((stats.records, stats.changed), (1, 1));
    assert_eq!(stats.skipped, 8);
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 1);
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE id = 'GHSA-1'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn an_archive_without_a_readable_record_is_an_error_that_keeps_the_data() {
    let s = setup().await;
    let first = zip::write(&[(
        "GHSA-1.json",
        &advisory("GHSA-1", "2024-01-01T00:00:00Z", "a"),
    )]);
    import(&s.db, s.dir.path(), first).await;
    let source = Upload::new(stage(s.dir.path(), zip::write(&[("GHSA-1.json", b"{")])).await);
    let error = osv::sync(s.db.conn(), &source, "npm", &never)
        .await
        .unwrap_err();
    assert!(
        matches!(error, SyncError::Import(osv::ImportError::Empty)),
        "{error}"
    );
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 1);
    let source = Upload::new(stage(s.dir.path(), b"this is not a zip file at all".to_vec()).await);
    assert!(
        osv::sync(s.db.conn(), &source, "npm", &never)
            .await
            .is_err()
    );
    let feeds = osv::feeds(s.db.conn()).await.unwrap();
    assert!(feeds[0].last_error.is_some());
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 1);
}

#[tokio::test]
async fn importing_twice_changes_nothing_and_updates_only_newer_records() {
    let s = setup().await;
    let archive = |modified: &str| {
        zip::write(&[
            ("GHSA-1.json", &advisory("GHSA-1", modified, "a")),
            (
                "GHSA-2.json",
                &advisory("GHSA-2", "2024-01-01T00:00:00Z", "b"),
            ),
        ])
    };
    stats(import(&s.db, s.dir.path(), archive("2024-01-01T00:00:00Z")).await);
    execute(&s.db, "INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) VALUES ('npm', 'a', '1.0.0', 'p', now())").await;
    let again = stats(import(&s.db, s.dir.path(), archive("2024-01-01T00:00:00Z")).await);
    assert_eq!(
        (again.changed, again.unchanged, again.invalidated),
        (0, 2, 0)
    );
    assert_eq!(
        count(&s.db, "SELECT count(*) AS n FROM advisory_affected").await,
        2
    );
    let newer = stats(import(&s.db, s.dir.path(), archive("2024-02-01T00:00:00Z")).await);
    assert_eq!(
        (newer.changed, newer.unchanged, newer.invalidated),
        (1, 1, 1)
    );
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 2);
    assert_eq!(
        count(&s.db, "SELECT count(*) AS n FROM advisory_affected").await,
        2
    );
}

#[tokio::test]
async fn advisories_missing_from_the_dump_are_deleted_but_unreadable_ones_stay() {
    let s = setup().await;
    let both = zip::write(&[
        (
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "a"),
        ),
        (
            "GHSA-2.json",
            &advisory("GHSA-2", "2024-01-01T00:00:00Z", "b"),
        ),
        (
            "GHSA-3.json",
            &advisory("GHSA-3", "2024-01-01T00:00:00Z", "c"),
        ),
    ]);
    import(&s.db, s.dir.path(), both).await;
    execute(&s.db, "INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) VALUES ('npm', 'b', '1.0.0', 'p', now())").await;
    let fewer = zip::write(&[
        (
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "a"),
        ),
        ("GHSA-3.json", b"{broken"),
    ]);
    let stats = stats(import(&s.db, s.dir.path(), fewer).await);
    assert_eq!(stats.removed, 1);
    assert_eq!(stats.invalidated, 1);
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE id IN ('GHSA-1', 'GHSA-3')"
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE id = 'GHSA-2'"
        )
        .await,
        0
    );
}

struct Scripted {
    archive: Vec<u8>,
    dir: PathBuf,
}

#[async_trait]
impl AdvisorySource for Scripted {
    async fn fetch(&self, _ecosystem: &str, etag: Option<&str>) -> Result<Fetched, SourceError> {
        if etag == Some("\"v1\"") {
            return Ok(Fetched::NotModified);
        }
        Ok(Fetched::Data {
            file: stage(&self.dir, self.archive.clone()).await,
            etag: Some("\"v1\"".to_owned()),
        })
    }
}

#[tokio::test]
async fn an_unchanged_etag_ends_in_not_modified_and_keeps_the_data() {
    let s = setup().await;
    let source = Scripted {
        archive: zip::write(&[(
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "a"),
        )]),
        dir: s.dir.path().to_owned(),
    };
    let first = osv::sync(s.db.conn(), &source, "npm", &never)
        .await
        .unwrap();
    assert!(matches!(first, Outcome::Imported(_)));
    let before = osv::feeds(s.db.conn()).await.unwrap().remove(0);
    assert_eq!(before.etag.as_deref(), Some("\"v1\""));
    let second = osv::sync(s.db.conn(), &source, "npm", &never)
        .await
        .unwrap();
    assert_eq!(second, Outcome::NotModified);
    let after = osv::feeds(s.db.conn()).await.unwrap().remove(0);
    assert_eq!(after.etag, before.etag);
    assert!(after.synced_at >= before.synced_at);
    assert_eq!(
        astrum_db::revision::current(s.db.conn(), astrum_db::revision::ADVISORIES)
            .await
            .unwrap(),
        1
    );
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 1);
}

#[tokio::test]
async fn a_private_address_is_refused_before_any_connection() {
    let s = setup().await;
    let secrets = SecretBox::from_key([7; 32]);
    let blobs = Arc::new(BlobStores::new(
        s.db.conn().clone(),
        s.dir.path(),
        secrets.clone(),
    ));
    let content = Content::new(s.db.conn().clone(), blobs, secrets);
    for url in [
        "http://127.0.0.1:9/",
        "http://10.1.2.3/osv",
        "http://169.254.169.254/",
    ] {
        let source = HttpDump::new(content.clone(), url, s.dir.path().join("tmp")).unwrap();
        let error = osv::sync(s.db.conn(), &source, "npm", &never)
            .await
            .unwrap_err();
        assert!(
            matches!(error, SyncError::Source(SourceError::Refused(_))),
            "{url}: {error}"
        );
    }
    let localhost = HttpDump::new(
        content.clone(),
        "http://localhost:9/",
        s.dir.path().join("tmp"),
    )
    .unwrap();
    assert!(
        osv::sync(s.db.conn(), &localhost, "npm", &never)
            .await
            .is_err()
    );
    assert!(
        HttpDump::new(
            content.clone(),
            "http://user:pw@example.org/",
            s.dir.path().join("tmp")
        )
        .is_err()
    );
    assert!(HttpDump::new(content, "ftp://example.org/", s.dir.path().join("tmp")).is_err());
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 0);
    let feed = osv::feeds(s.db.conn()).await.unwrap().remove(0);
    assert!(feed.last_error.is_some() && feed.synced_at.is_none());
}

#[tokio::test]
async fn the_purge_drops_packages_no_inventory_holds() {
    let s = setup().await;
    for sql in [
        "INSERT INTO sca_package (id, ecosystem, name, version, purl) VALUES \
            (1, 'npm', 'held', '1.0.0', 'p1'), (2, 'npm', 'orphan', '1.0.0', 'p2')",
        "INSERT INTO sca_inventory (id, key, source, extractor) VALUES (1, 'pkg:p1', 'metadata', 1)",
        "INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES (1, 1)",
        "INSERT INTO oci_layer_facts (digest, extractor, facts, extracted_at) \
            VALUES ('sha256:aa', 1, '{}', now() - interval '2 hours')",
    ] {
        execute(&s.db, sql).await;
    }
    let purged = osv::purge::run(s.db.conn()).await.unwrap();
    assert_eq!(
        (purged.inventories, purged.packages, purged.layer_facts),
        (0, 1, 1)
    );
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM sca_package WHERE name = 'held'"
        )
        .await,
        1
    );
}

fn numbered(count: usize) -> Vec<(String, Vec<u8>)> {
    (0..count)
        .map(|n| {
            (
                format!("R-{n}.json"),
                advisory(
                    &format!("R-{n}"),
                    "2024-01-01T00:00:00Z",
                    &format!("pkg-{n}"),
                ),
            )
        })
        .collect()
}

fn archive_of(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let listed: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    zip::write(&listed)
}

async fn sync_forced(
    db: &Database,
    dir: &Path,
    archive: Vec<u8>,
    force: bool,
) -> Result<Outcome, SyncError> {
    let source = Upload::new(stage(dir, archive).await);
    osv::sync_with(db.conn(), &source, "npm", force, &never).await
}

#[tokio::test]
async fn records_the_database_cannot_store_are_skipped_and_the_rest_is_imported() {
    let s = setup().await;
    let with_nul = serde_json::to_vec(&json!({
        "id": "GHSA-NUL", "modified": "2024-01-01T00:00:00Z", "details": "a\u{0}b",
        "affected": [{"package": {"ecosystem": "npm", "name": "x"}}]
    }))
    .unwrap();
    let year_zero = serde_json::to_vec(&json!({
        "id": "GHSA-YEAR", "modified": "2024-01-01T00:00:00Z", "withdrawn": "0000-01-01T00:00:00Z",
        "affected": [{"package": {"ecosystem": "npm", "name": "y"}}]
    }))
    .unwrap();
    let archive = zip::write(&[
        ("GHSA-NUL.json", &with_nul),
        ("GHSA-YEAR.json", &year_zero),
        (
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "left-pad"),
        ),
    ]);
    let stats = stats(import(&s.db, s.dir.path(), archive).await);
    assert_eq!((stats.records, stats.changed, stats.skipped), (1, 1, 2));
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE id = 'GHSA-1'"
        )
        .await,
        1
    );
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 1);
}

#[tokio::test]
async fn a_batch_the_database_rejects_is_retried_record_by_record() {
    let s = setup().await;
    execute(
        &s.db,
        "ALTER TABLE advisory ADD CONSTRAINT refuse_one CHECK (id <> 'GHSA-REFUSED')",
    )
    .await;
    let archive = zip::write(&[
        (
            "GHSA-1.json",
            &advisory("GHSA-1", "2024-01-01T00:00:00Z", "a"),
        ),
        (
            "GHSA-REFUSED.json",
            &advisory("GHSA-REFUSED", "2024-01-01T00:00:00Z", "b"),
        ),
        (
            "GHSA-2.json",
            &advisory("GHSA-2", "2024-01-01T00:00:00Z", "c"),
        ),
    ]);
    let stats = stats(import(&s.db, s.dir.path(), archive).await);
    assert_eq!((stats.records, stats.changed, stats.skipped), (2, 2, 1));
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 2);
}

#[tokio::test]
async fn a_dump_of_another_ecosystem_is_refused_and_keeps_the_data() {
    let s = setup().await;
    import(&s.db, s.dir.path(), archive_of(&numbered(3))).await;
    let before = osv::feeds(s.db.conn()).await.unwrap().remove(0);
    let wrong = zip::write(&[(
        "PYSEC-1.json",
        &advisory_in("PYSEC-1", "2024-01-01T00:00:00Z", "PyPI", "django"),
    )]);
    let error = sync_forced(&s.db, s.dir.path(), wrong, true)
        .await
        .unwrap_err();
    assert!(
        matches!(error, SyncError::Import(osv::ImportError::OutOfScope(_))),
        "{error}"
    );
    assert_eq!(
        count(
            &s.db,
            "SELECT count(*) AS n FROM advisory WHERE id LIKE 'R-%'"
        )
        .await,
        3
    );
    let after = osv::feeds(s.db.conn()).await.unwrap().remove(0);
    assert_eq!(
        (after.etag, after.synced_at),
        (before.etag, before.synced_at)
    );
}

#[tokio::test]
async fn a_large_shrink_needs_force_and_a_small_one_does_not() {
    let s = setup().await;
    let all = numbered(150);
    import(&s.db, s.dir.path(), archive_of(&all)).await;
    let error = sync_forced(&s.db, s.dir.path(), archive_of(&all[..10]), false)
        .await
        .unwrap_err();
    assert!(
        matches!(
            error,
            SyncError::Import(osv::ImportError::Shrinks {
                removing: 140,
                held: 150
            })
        ),
        "{error}"
    );
    assert!(error.to_string().contains("force=true"));
    assert_eq!(
        count(&s.db, "SELECT count(*) AS n FROM advisory").await,
        150
    );
    let small = stats(
        sync_forced(&s.db, s.dir.path(), archive_of(&all[..140]), false)
            .await
            .unwrap(),
    );
    assert_eq!(small.removed, 10);
    let forced = stats(
        sync_forced(&s.db, s.dir.path(), archive_of(&all[..10]), true)
            .await
            .unwrap(),
    );
    assert_eq!(forced.removed, 130);
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 10);
}

#[tokio::test]
async fn entries_sharing_one_local_header_make_the_archive_malformed() {
    let s = setup().await;
    import(&s.db, s.dir.path(), archive_of(&numbered(3))).await;
    let mut archive = archive_of(&numbered(2));
    let end = archive.len() - 22;
    let directory = u32::from_le_bytes(archive[end + 16..end + 20].try_into().unwrap()) as usize;
    let first = archive[directory + 42..directory + 46].to_vec();
    let name = u16::from_le_bytes(archive[directory + 28..directory + 30].try_into().unwrap());
    let second = directory + 46 + name as usize;
    archive[second + 42..second + 46].copy_from_slice(&first);
    let error = sync_forced(&s.db, s.dir.path(), archive, true)
        .await
        .unwrap_err();
    assert!(
        matches!(error, SyncError::Import(osv::ImportError::Archive(_))),
        "{error}"
    );
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 3);
}

struct Gate {
    started: Notify,
    proceed: Notify,
    archive: Vec<u8>,
    dir: PathBuf,
}

#[async_trait]
impl AdvisorySource for Gate {
    async fn fetch(&self, _ecosystem: &str, _etag: Option<&str>) -> Result<Fetched, SourceError> {
        self.started.notify_one();
        self.proceed.notified().await;
        Ok(Fetched::Data {
            file: stage(&self.dir, self.archive.clone()).await,
            etag: None,
        })
    }
}

#[tokio::test]
async fn a_second_import_of_a_running_ecosystem_is_turned_away() {
    let s = setup().await;
    let gate = Arc::new(Gate {
        started: Notify::new(),
        proceed: Notify::new(),
        archive: archive_of(&numbered(2)),
        dir: s.dir.path().to_owned(),
    });
    let running = {
        let (db, gate) = (s.db.conn().clone(), gate.clone());
        tokio::spawn(async move { osv::sync(&db, gate.as_ref(), "npm", &never).await })
    };
    gate.started.notified().await;
    let error = import_error(&s.db, s.dir.path()).await;
    assert!(matches!(error, SyncError::Busy(_)), "{error}");
    let other = Upload::new(stage(s.dir.path(), archive_of(&numbered(1))).await);
    assert!(
        osv::sync(s.db.conn(), &other, "PyPI", &never)
            .await
            .is_err_and(|error| !matches!(error, SyncError::Busy(_)))
    );
    gate.proceed.notify_one();
    assert!(matches!(
        running.await.unwrap().unwrap(),
        Outcome::Imported(_)
    ));
    assert_eq!(count(&s.db, "SELECT count(*) AS n FROM advisory").await, 2);
    assert!(
        osv::EcosystemLock::try_take(s.db.conn(), "npm")
            .await
            .unwrap()
            .is_some()
    );
}

async fn import_error(db: &Database, dir: &Path) -> SyncError {
    sync_forced(db, dir, archive_of(&numbered(1)), false)
        .await
        .unwrap_err()
}
