use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use astrum_content::zip;
use astrum_scan::matcher::{self, Config};
use astrum_scan::osv::{self, TempFile, Upload};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::json;

const RUN: Duration = Duration::from_secs(12);

#[derive(Default)]
struct Tally {
    deadlocks: AtomicU64,
    failures: AtomicU64,
    matched: AtomicU64,
    imports: AtomicU64,
    purges: AtomicU64,
}

impl Tally {
    fn judge<T, E: std::fmt::Display>(&self, outcome: Result<T, E>) -> Option<T> {
        match outcome {
            Ok(value) => Some(value),
            Err(error) => {
                let text = error.to_string();
                let counter = if text.contains("deadlock detected") {
                    &self.deadlocks
                } else {
                    eprintln!("{text}");
                    &self.failures
                };
                counter.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
}

fn advisory_archive(sequence: u64) -> Vec<u8> {
    let modified = format!(
        "2027-01-01T{:02}:{:02}:{:02}Z",
        sequence / 3600,
        sequence / 60 % 60,
        sequence % 60
    );
    let file = serde_json::to_vec(&json!({
        "id": "A",
        "modified": modified,
        "summary": "a flaw",
        "database_specific": {"severity": "HIGH"},
        "affected": [{
            "package": {"ecosystem": "npm", "name": format!("lib{}", sequence % 5)},
            "ranges": [{"type": "SEMVER", "events": [{"introduced": "0"}]}]
        }]
    }))
    .unwrap();
    zip::write(&[("A.json", file.as_slice())])
}

async fn import(
    db: &DatabaseConnection,
    dir: &std::path::Path,
    sequence: u64,
) -> Result<(), String> {
    let staged = TempFile::from_stream(
        dir,
        futures_util::stream::iter(vec![Ok::<_, Infallible>(advisory_archive(sequence))]),
        osv::MAX_ARCHIVE_BYTES,
    )
    .await
    .map_err(|error| error.to_string())?;
    osv::sync_with(db, &Upload::new(staged), "npm", true, &|| false)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn matchers_import_and_purge_running_together_never_deadlock() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await.conn().clone();
    let dir = tempfile::tempdir().unwrap();
    let tally = Arc::new(Tally::default());
    let stop = Arc::new(AtomicBool::new(false));
    let config = Config {
        batch: 50,
        ..Config::default()
    };
    let mut tasks = Vec::new();
    for _ in 0..3 {
        let (db, tally, stop, config) = (db.clone(), tally.clone(), stop.clone(), config.clone());
        tasks.push(tokio::spawn(async move {
            while !stop.load(Ordering::Relaxed) {
                match tally.judge(matcher::match_batch(&db, &config).await) {
                    Some(batch) if batch.matched > 0 => {
                        tally.matched.fetch_add(batch.matched, Ordering::Relaxed);
                        matcher::fan_out(&db, &config, &batch.marked).await;
                    }
                    _ => tokio::task::yield_now().await,
                }
            }
        }));
    }
    {
        let (db, tally, stop, path) = (
            db.clone(),
            tally.clone(),
            stop.clone(),
            dir.path().to_owned(),
        );
        tasks.push(tokio::spawn(async move {
            let mut sequence = 0;
            while !stop.load(Ordering::Relaxed) {
                sequence += 1;
                if tally.judge(import(&db, &path, sequence).await).is_some() {
                    tally.imports.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    {
        let (db, tally, stop) = (db.clone(), tally.clone(), stop.clone());
        tasks.push(tokio::spawn(async move {
            while !stop.load(Ordering::Relaxed) {
                if tally.judge(osv::purge::run(&db).await).is_some() {
                    tally.purges.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    {
        let (db, stop) = (db.clone(), stop.clone());
        tasks.push(tokio::spawn(async move {
            let mut round = 0u64;
            while !stop.load(Ordering::Relaxed) {
                round += 1;
                db.execute_unprepared(&format!(
                    "INSERT INTO sca_package (ecosystem, name, version, purl) \
                     SELECT 'npm', 'lib' || (n % 5), '{round}.' || n || '.0', 'p{round}-' || n \
                     FROM generate_series(1, 200) n \
                     WHERE (SELECT count(*) FROM sca_package) < 3000 ON CONFLICT DO NOTHING"
                ))
                .await
                .unwrap();
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }));
    }
    tokio::time::sleep(RUN).await;
    stop.store(true, Ordering::Relaxed);
    for task in tasks {
        tokio::time::timeout(Duration::from_secs(60), task)
            .await
            .expect("a task did not stop")
            .unwrap();
    }
    assert_eq!(tally.deadlocks.load(Ordering::Relaxed), 0);
    assert_eq!(tally.failures.load(Ordering::Relaxed), 0);
    assert!(tally.matched.load(Ordering::Relaxed) > 0);
    assert!(tally.imports.load(Ordering::Relaxed) > 0);
    assert!(tally.purges.load(Ordering::Relaxed) > 0);
}
