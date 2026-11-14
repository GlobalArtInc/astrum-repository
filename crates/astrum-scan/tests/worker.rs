use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use astrum_blob::BlobStores;
use astrum_content::inventory::{Inventory, PackageRef};
use astrum_content::store::Component;
use astrum_content::{Content, ContentError, ContentRequest, Ctx, Format};
use astrum_scan::queue::{self, Selection};
use astrum_scan::worker::{self, Config, Metrics};
use astrum_security::{SecretBox, SecurityModel};
use astrum_testkit::TestDatabase;
use async_trait::async_trait;
use axum::response::Response;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

#[derive(Default)]
struct Fake {
    seen: Mutex<HashMap<String, usize>>,
    finished: Mutex<usize>,
    db: OnceLock<sea_orm::DatabaseConnection>,
}

fn package(name: &str) -> PackageRef {
    PackageRef {
        ecosystem: "Maven".to_owned(),
        name: name.to_owned(),
        version: "1".to_owned(),
        purl: format!("pkg:maven/{name}@1"),
    }
}

#[async_trait]
impl Format for Fake {
    fn id(&self) -> &'static str {
        "maven2"
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
        let first = {
            let mut seen = self.seen.lock().unwrap();
            let times = seen.entry(component.name.clone()).or_default();
            *times += 1;
            *times == 1
        };
        tokio::time::sleep(Duration::from_millis(3)).await;
        match component.name.as_str() {
            "again" if first => {
                let db = self.db.get().expect("the fake knows the database");
                queue::enqueue_component(db, &component.repository, component.id, "test")
                    .await
                    .map_err(|error| ContentError::BadRequest(error.to_string()))?;
                Ok(None)
            }
            "slow" => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                *self.finished.lock().unwrap() += 1;
                Ok(None)
            }
            "plain" => Ok(None),
            "boom" => Err(ContentError::BadRequest("cannot read it".to_owned())),
            "hostile" => Ok(Some(Inventory {
                key: "pkg:maven/hostile@1".to_owned(),
                packages: vec![package("hostile"), package("bad\u{0}name")],
                license: None,
                complete: true,
                ..Inventory::default()
            })),
            name => Ok(Some(Inventory {
                key: format!("pkg:maven/{name}@1"),
                packages: vec![package(name), package("shared"), package("shared")],
                license: None,
                complete: name != "partial",
                ..Inventory::default()
            })),
        }
    }
}

struct Setup {
    _test: TestDatabase,
    _dir: tempfile::TempDir,
    db: astrum_db::Database,
    content: Content,
    fake: Arc<Fake>,
}

async fn setup() -> Setup {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let dir = tempfile::tempdir().unwrap();
    let secrets = SecretBox::from_key([7; 32]);
    let blobs = Arc::new(BlobStores::new(
        db.conn().clone(),
        dir.path(),
        secrets.clone(),
    ));
    let content = Content::new(db.conn().clone(), blobs, secrets);
    let fake = Arc::new(Fake::default());
    content.register(fake.clone());
    fake.db.set(db.conn().clone()).unwrap();
    db.conn()
        .execute_unprepared(
            "INSERT INTO repository (name, format, type, attributes) \
             VALUES ('releases', 'maven2', 'hosted', '{}')",
        )
        .await
        .unwrap();
    Setup {
        _test: test,
        _dir: dir,
        db,
        content,
        fake,
    }
}

async fn add(s: &Setup, name: &str) -> i64 {
    let id =
        s.db.conn()
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                format!(
                    "INSERT INTO component (repository, format, name, version) \
                 VALUES ('releases', 'maven2', '{name}', '1') RETURNING id"
                ),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "id")
            .unwrap();
    queue::enqueue_component(s.db.conn(), "releases", id, "test")
        .await
        .unwrap();
    id
}

async fn scalar(s: &Setup, sql: &str) -> i64 {
    s.db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn text(s: &Setup, sql: &str) -> Option<String> {
    s.db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "t")
        .unwrap()
}

async fn drain_all(s: &Setup, config: &Config) -> usize {
    let security = Arc::new(SecurityModel::default());
    let metrics = Metrics::default();
    let mut total = 0;
    loop {
        let done = worker::drain(&s.content, config, &metrics, &security).await;
        if done == 0 {
            return total;
        }
        total += done;
    }
}

fn quick() -> Config {
    Config {
        batch: 4,
        ..Config::default()
    }
}

#[tokio::test]
async fn two_workers_on_one_queue_never_take_the_same_row() {
    let s = setup().await;
    for number in 0..30 {
        add(&s, &format!("c{number}")).await;
    }
    let config = quick();
    let (first, second) = tokio::join!(drain_all(&s, &config), drain_all(&s, &config));
    assert_eq!(first + second, 30);
    {
        let seen = s.fake.seen.lock().unwrap();
        assert_eq!(seen.len(), 30);
        assert!(seen.values().all(|times| *times == 1), "{seen:?}");
    }
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 0);
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM component_scan WHERE state = 'scanned'"
        )
        .await,
        30
    );
}

#[tokio::test]
async fn a_package_with_a_control_character_is_left_out_and_the_inventory_is_incomplete() {
    let s = setup().await;
    let hostile = add(&s, "hostile").await;
    drain_all(&s, &quick()).await;

    assert_eq!(
        text(
            &s,
            &format!("SELECT state AS t FROM component_scan WHERE component_id = {hostile}")
        )
        .await
        .as_deref(),
        Some("incomplete")
    );
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_package").await, 1);
}

#[tokio::test]
async fn a_scan_records_packages_links_and_the_state_of_each_component() {
    let s = setup().await;
    let good = add(&s, "good").await;
    let plain = add(&s, "plain").await;
    let partial = add(&s, "partial").await;
    drain_all(&s, &quick()).await;

    let state =
        |id: i64| format!("SELECT state AS t FROM component_scan WHERE component_id = {id}");
    assert_eq!(text(&s, &state(good)).await.as_deref(), Some("scanned"));
    assert_eq!(
        text(&s, &state(plain)).await.as_deref(),
        Some("unsupported")
    );
    assert_eq!(
        text(&s, &state(partial)).await.as_deref(),
        Some("incomplete")
    );
    assert_eq!(
        scalar(
            &s,
            &format!("SELECT count(*) AS n FROM component_scan WHERE component_id = {plain} AND inventory_id IS NULL")
        )
        .await,
        1
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_inventory").await,
        2
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        3
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_inventory_package").await,
        4
    );
}

#[tokio::test]
async fn a_failing_scan_is_retried_after_its_delay_and_then_given_up() {
    let s = setup().await;
    let boom = add(&s, "boom").await;
    let config = Config {
        backoff: vec![Duration::from_secs(3600)],
        ..quick()
    };
    assert_eq!(drain_all(&s, &config).await, 1);
    assert_eq!(drain_all(&s, &config).await, 0);
    let attempts =
        format!("SELECT attempts::bigint AS n FROM scan_queue WHERE component_id = {boom}");
    assert_eq!(scalar(&s, &attempts).await, 1);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM component_scan").await,
        0
    );

    s.db.conn()
        .execute_unprepared("UPDATE scan_queue SET next_attempt_at = now()")
        .await
        .unwrap();
    assert_eq!(drain_all(&s, &config).await, 1);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 0);
    assert_eq!(
        text(
            &s,
            &format!("SELECT state AS t FROM component_scan WHERE component_id = {boom}")
        )
        .await
        .as_deref(),
        Some("failed")
    );
    let error = text(&s, "SELECT error AS t FROM component_scan")
        .await
        .unwrap();
    assert!(error.contains("cannot read it"), "{error}");
    assert_eq!(s.fake.seen.lock().unwrap()["boom"], 2);
}

#[tokio::test]
async fn deleting_a_component_removes_its_scan_and_leaves_its_inventory_for_the_purge() {
    let s = setup().await;
    let good = add(&s, "good").await;
    drain_all(&s, &quick()).await;
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM component_scan").await,
        1
    );

    s.db.conn()
        .execute_unprepared(&format!("DELETE FROM component WHERE id = {good}"))
        .await
        .unwrap();
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM component_scan").await,
        0
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_inventory").await,
        1
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_inventory_package").await,
        2
    );
}

#[tokio::test]
async fn a_repository_is_queued_in_chunks_and_unscanned_components_can_be_picked() {
    let s = setup().await;
    for number in 0..7 {
        let id = add(&s, &format!("c{number}")).await;
        if number < 3 {
            s.db.conn()
                .execute_unprepared(&format!(
                    "INSERT INTO component_scan (component_id, repository, state) \
                     VALUES ({id}, 'releases', 'unsupported')"
                ))
                .await
                .unwrap();
        }
    }
    s.db.conn()
        .execute_unprepared("DELETE FROM scan_queue")
        .await
        .unwrap();
    let never = || false;
    let unscanned = queue::enqueue_repository(
        s.db.conn(),
        "releases",
        "test",
        Selection::Unscanned,
        2,
        &never,
    )
    .await
    .unwrap();
    assert_eq!((unscanned.components, unscanned.queued), (4, 4));

    let every =
        queue::enqueue_repository(s.db.conn(), "releases", "test", Selection::Every, 3, &never)
            .await
            .unwrap();
    assert_eq!((every.components, every.queued), (7, 7));
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 7);

    let stopped = queue::enqueue_repository(
        s.db.conn(),
        "releases",
        "test",
        Selection::Every,
        3,
        &|| true,
    )
    .await;
    assert!(matches!(stopped, Err(queue::QueueError::Cancelled)));
    let depth = queue::depth(s.db.conn()).await.unwrap();
    assert_eq!(depth.queued, 7);
}

async fn drain_once(s: &Setup, config: &Config) -> usize {
    let security = Arc::new(SecurityModel::default());
    worker::drain(&s.content, config, &Metrics::default(), &security).await
}

fn due(id: i64) -> String {
    format!(
        "SELECT count(*) AS n FROM scan_queue WHERE component_id = {id} AND next_attempt_at <= now()"
    )
}

#[tokio::test]
async fn a_component_queued_again_while_it_is_scanned_keeps_its_row_and_is_due_at_once() {
    let s = setup().await;
    let again = add(&s, "again").await;
    s.db.conn()
        .execute_unprepared("UPDATE scan_queue SET queued_at = now() - interval '1 minute'")
        .await
        .unwrap();
    let config = quick();
    assert_eq!(drain_once(&s, &config).await, 1);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 1);
    assert_eq!(
        scalar(
            &s,
            &format!("SELECT attempts::bigint AS n FROM scan_queue WHERE component_id = {again}")
        )
        .await,
        0
    );
    assert_eq!(scalar(&s, &due(again)).await, 1);

    assert_eq!(drain_once(&s, &config).await, 1);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 0);
    assert_eq!(s.fake.seen.lock().unwrap()["again"], 2);
}

#[tokio::test]
async fn a_rescan_makes_a_row_in_backoff_due_now() {
    let s = setup().await;
    let boom = add(&s, "boom").await;
    let config = Config {
        backoff: vec![Duration::from_secs(1800)],
        ..quick()
    };
    assert_eq!(drain_once(&s, &config).await, 1);
    assert_eq!(scalar(&s, &due(boom)).await, 0);

    let again = queue::enqueue_repository(
        s.db.conn(),
        "releases",
        "rescan",
        Selection::Every,
        10,
        &|| false,
    )
    .await
    .unwrap();
    assert_eq!(again.queued, 1);
    assert_eq!(scalar(&s, &due(boom)).await, 1);
    assert_eq!(drain_once(&s, &config).await, 1);
    assert_eq!(s.fake.seen.lock().unwrap()["boom"], 2);
}

#[tokio::test]
async fn a_row_whose_lease_ran_out_comes_back_and_is_processed_once() {
    let s = setup().await;
    let good = add(&s, "good").await;
    s.db.conn()
        .execute_unprepared(
            "UPDATE scan_queue SET attempts = 1, next_attempt_at = now() + interval '1 hour'",
        )
        .await
        .unwrap();
    let config = quick();
    assert_eq!(drain_once(&s, &config).await, 0);

    s.db.conn()
        .execute_unprepared("UPDATE scan_queue SET next_attempt_at = now() - interval '1 minute'")
        .await
        .unwrap();
    assert_eq!(drain_all(&s, &config).await, 1);
    assert_eq!(s.fake.seen.lock().unwrap()["good"], 1);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 0);
    assert_eq!(
        scalar(
            &s,
            &format!("SELECT count(*) AS n FROM component_scan WHERE component_id = {good}")
        )
        .await,
        1
    );
}

#[tokio::test]
async fn a_scan_that_times_out_is_stopped_before_its_outcome_is_written() {
    let s = setup().await;
    let slow = add(&s, "slow").await;
    let config = Config {
        timeout: Duration::from_millis(40),
        backoff: vec![Duration::from_secs(3600)],
        ..quick()
    };
    assert_eq!(drain_once(&s, &config).await, 1);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(*s.fake.finished.lock().unwrap(), 0);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM component_scan").await,
        0
    );
    assert_eq!(scalar(&s, &due(slow)).await, 0);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM scan_queue").await, 1);
}
