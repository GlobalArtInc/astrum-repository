use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use astrum_blob::BlobStores;
use astrum_content::inventory::{Inventory, PackageRef, Source};
use astrum_content::store::Component;
use astrum_content::{Content, ContentError, ContentRequest, Ctx, Format};
use astrum_policy::cache::Cache;
use astrum_scan::matcher;
use astrum_scan::queue;
use astrum_scan::verdict::{self, SweepState};
use astrum_scan::worker::{self, Config, Metrics};
use astrum_security::{SecretBox, SecurityModel};
use astrum_testkit::TestDatabase;
use async_trait::async_trait;
use axum::response::Response;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

const COMPONENTS: i64 = 6;
const ROUNDS: usize = 100;

#[derive(Default)]
struct Images {
    calls: AtomicUsize,
}

fn package(name: &str) -> PackageRef {
    PackageRef {
        ecosystem: "npm".to_owned(),
        name: name.to_owned(),
        version: "1".to_owned(),
        purl: format!("pkg:npm/{name}@1"),
    }
}

#[async_trait]
impl Format for Images {
    fn id(&self) -> &'static str {
        "docker"
    }

    async fn handle(&self, _ctx: Ctx, _request: ContentRequest) -> Result<Response, ContentError> {
        Err(ContentError::not_found("nothing here"))
    }

    async fn get(
        &self,
        _ctx: &Ctx,
        _path: &str,
    ) -> Result<Option<astrum_content::Asset>, ContentError> {
        Ok(None)
    }

    async fn inventory(
        &self,
        _ctx: &Ctx,
        _component: &Component,
    ) -> Result<Option<Inventory>, ContentError> {
        let turn = self.calls.fetch_add(1, Ordering::SeqCst);
        let second = if turn.is_multiple_of(2) {
            "dock-b"
        } else {
            "dock-c"
        };
        Ok(Some(Inventory {
            key: "oci:img:sha256:aa".to_owned(),
            packages: vec![package("dock-a"), package(second)],
            license: None,
            complete: true,
            source: Source::Sbom,
            digest: Some("sha256:aa".to_owned()),
        }))
    }
}

async fn number(db: &sea_orm::DatabaseConnection, sql: &str) -> i64 {
    db.query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
async fn sharing_scans_rejudging_and_sweeping_for_many_rounds_never_deadlock() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn().clone();
    let dir = tempfile::tempdir().unwrap();
    let secrets = SecretBox::from_key([7; 32]);
    let blobs = Arc::new(BlobStores::new(conn.clone(), dir.path(), secrets.clone()));
    let content = Content::new(conn.clone(), blobs, secrets);
    content.register(Arc::new(Images::default()));
    conn.execute_unprepared(
        "INSERT INTO content_policy (name, vulnerabilities) \
           VALUES ('base', '{\"mode\":\"block\",\"severity\":\"high\"}'); \
         INSERT INTO repository (name, format, type, attributes) \
           VALUES ('img', 'docker', 'hosted', '{\"policy\":{\"name\":\"base\"}}'); \
         INSERT INTO advisory (id, aliases, severity, modified_at, document) \
           VALUES ('GHSA-1', '{CVE-1}', 'high', now(), '{}'); \
         INSERT INTO advisory_affected (advisory_id, ecosystem, name, versions) \
           VALUES ('GHSA-1', 'npm', 'dock-a', '{1}'), ('GHSA-1', 'npm', 'dock-c', '{1}')",
    )
    .await
    .unwrap();
    for id in 1..=COMPONENTS {
        conn.execute_unprepared(&format!(
            "INSERT INTO component (id, repository, format, name, version) \
             VALUES ({id}, 'img', 'docker', 'app', 'v{id}')"
        ))
        .await
        .unwrap();
    }
    let before = number(
        &conn,
        "SELECT deadlocks::bigint AS n FROM pg_stat_database WHERE datname = current_database()",
    )
    .await;
    let security = Arc::new(SecurityModel::default());
    let scanning = Config {
        batch: 2,
        ..Config::default()
    };
    let scans = Arc::new(Metrics::default());
    let matching = matcher::Config::default();
    let matches = matcher::Metrics::default();
    let judging = verdict::Config::default();
    let cache = Cache::default();
    for _ in 0..ROUNDS {
        for id in 1..=COMPONENTS {
            queue::enqueue_component(&conn, "img", id, "round")
                .await
                .unwrap();
        }
        let mut state = SweepState::default();
        let (_, _, _, _, swept, judged, again) = tokio::join!(
            worker::drain(&content, &scanning, &scans, &security),
            worker::drain(&content, &scanning, &scans, &security),
            worker::drain(&content, &scanning, &scans, &security),
            matcher::step(&conn, &matching, &matches),
            verdict::sweep(&conn, &judging, &mut state),
            verdict::judge_batch(&conn, &cache, &judging),
            verdict::judge_batch(&conn, &cache, &judging),
        );
        swept.unwrap();
        judged.unwrap();
        again.unwrap();
    }
    while worker::drain(&content, &scanning, &scans, &security).await > 0 {}
    tokio::time::sleep(Duration::from_millis(1800)).await;
    let after = number(
        &conn,
        "SELECT deadlocks::bigint AS n FROM pg_stat_database WHERE datname = current_database()",
    )
    .await;
    assert_eq!(after, before, "a deadlock was detected");
    assert_eq!(
        number(
            &conn,
            "SELECT count(*) AS n FROM component_scan WHERE state = 'scanned' AND digest = 'sha256:aa'"
        )
        .await,
        COMPONENTS
    );
    assert_eq!(
        number(&conn, "SELECT count(*) AS n FROM scan_queue").await,
        0
    );
}
