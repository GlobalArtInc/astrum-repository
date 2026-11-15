use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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

const COMPONENTS: i64 = 24;
const DIGESTS: i64 = 4;
const PER_REPOSITORY: i64 = 8;
const ROUNDS: usize = 100;
const EXPIRED: &str = "8d3f6c1e-0000-4000-8000-000000000001";

fn repository_of(id: i64) -> &'static str {
    match (id - 1) / PER_REPOSITORY {
        0 => "img",
        1 => "img2",
        _ => "img3",
    }
}

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
        component: &Component,
    ) -> Result<Option<Inventory>, ContentError> {
        let turn = self.calls.fetch_add(1, Ordering::SeqCst);
        let second = if turn.is_multiple_of(2) {
            "dock-b"
        } else {
            "dock-c"
        };
        let digest = format!("sha256:{:02}", component.id % DIGESTS);
        Ok(Some(Inventory {
            key: format!("oci:{}:{digest}", component.repository),
            packages: vec![package("dock-a"), package(second)],
            license: None,
            complete: true,
            source: Source::Sbom,
            digest: Some(digest),
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

const DEADLOCKS: &str =
    "SELECT deadlocks::bigint AS n FROM pg_stat_database WHERE datname = current_database()";

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
    conn.execute_unprepared(&format!(
        "INSERT INTO content_policy (name, vulnerabilities) \
           VALUES ('base', '{{\"mode\":\"block\",\"severity\":\"high\"}}'); \
         INSERT INTO repository (name, format, type, attributes) VALUES \
           ('img', 'docker', 'hosted', '{{\"policy\":{{\"name\":\"base\"}}}}'), \
           ('img2', 'docker', 'hosted', '{{\"policy\":{{\"name\":\"base\"}}}}'), \
           ('img3', 'docker', 'hosted', '{{\"policy\":{{\"name\":\"base\"}}}}'); \
         INSERT INTO advisory (id, aliases, severity, modified_at, document) \
           VALUES ('GHSA-1', '{{CVE-1}}', 'high', now(), '{{}}'); \
         INSERT INTO advisory_affected (advisory_id, ecosystem, name, versions) \
           VALUES ('GHSA-1', 'npm', 'dock-a', '{{1}}'), ('GHSA-1', 'npm', 'dock-c', '{{1}}'); \
         INSERT INTO policy_exception (id, rule, advisory, reason, created_by, expires_at) \
           VALUES ('{EXPIRED}', 'vulnerability', 'CVE-1', 'old', 'tester', now() - interval '10 minutes')"
    ))
    .await
    .unwrap();
    for id in 1..=COMPONENTS {
        let repository = repository_of(id);
        conn.execute_unprepared(&format!(
            "INSERT INTO component (id, repository, format, name, version) \
             VALUES ({id}, '{repository}', 'docker', 'app', 'v{id}')"
        ))
        .await
        .unwrap();
    }
    let before = number(&conn, DEADLOCKS).await;
    let security = Arc::new(SecurityModel::default());
    let scanning = Config {
        batch: 2,
        ..Config::default()
    };
    let scans = Arc::new(Metrics::default());
    let matching = matcher::Config::default();
    let judging = verdict::Config::default();
    let cache = Cache::default();
    let aged = format!(
        "UPDATE policy_verdict SET computed_at = now() - interval '1 hour', max_severity = 'high', \
           reasons = jsonb_build_array(jsonb_build_object('kind', 'exception', 'id', '{EXPIRED}'))"
    );
    for _ in 0..ROUNDS {
        for id in 1..=COMPONENTS {
            let repository = repository_of(id);
            queue::enqueue_component(&conn, repository, id, "round")
                .await
                .unwrap();
        }
        conn.execute_unprepared(&aged).await.unwrap();
        conn.execute_unprepared("DELETE FROM policy_dirty")
            .await
            .unwrap();
        let mut state = SweepState::default();
        let bump = conn.clone();
        let (_, _, _, matched, swept, judged, again) = tokio::join!(
            worker::drain(&content, &scanning, &scans, &security),
            worker::drain(&content, &scanning, &scans, &security),
            worker::drain(&content, &scanning, &scans, &security),
            async {
                let batch = matcher::match_batch(&conn, &matching).await.unwrap();
                let fanned = matcher::fan_out(&conn, &matching, &batch.marked).await;
                let swept = matcher::sweep(&conn, &matching).await.unwrap();
                fanned.failed + swept.failed
            },
            verdict::sweep(&conn, &judging, &mut state),
            verdict::judge_batch(&conn, &cache, &judging),
            verdict::judge_batch_with(&conn, &cache, &judging, || async move {
                astrum_db::revision::bump(&bump, astrum_db::revision::POLICIES)
                    .await
                    .unwrap();
            }),
        );
        assert_eq!(matched, 0, "the matcher failed to queue a subject");
        swept.unwrap();
        judged.unwrap();
        again.unwrap();
    }
    while worker::drain(&content, &scanning, &scans, &security).await > 0 {}
    assert_eq!(
        number(&conn, "SELECT count(*) AS n FROM scan_queue").await,
        0
    );
    assert_eq!(
        number(
            &conn,
            "SELECT count(*) AS n FROM component_scan WHERE state = 'scanned' AND digest IS NOT NULL"
        )
        .await,
        COMPONENTS
    );
    assert_eq!(
        number(&conn, "SELECT count(*) AS n FROM sca_inventory").await,
        3 * DIGESTS
    );
    assert_eq!(
        number(&conn, DEADLOCKS).await,
        before,
        "a deadlock was detected"
    );
}
