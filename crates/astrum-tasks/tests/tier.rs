use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use astrum_blob::{BlobStores, content_of};
use astrum_content::store::{self, Asset, ComponentKey, PutAsset};
use astrum_content::{Content, Ctx, holds};
use astrum_repo::config::{Attributes, Repository, Storage, WritePolicy};
use astrum_repo::{RepositoryType, format};
use astrum_security::{SecretBox, SecurityModel};
use astrum_tasks::kinds::tier::{ID, Tier};
use astrum_tasks::{Properties, Run, TaskError, TaskKind};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use uuid::Uuid;

const MIB: usize = 1024 * 1024;

struct World {
    _test: TestDatabase,
    db: astrum_db::Database,
    _dir: tempfile::TempDir,
    content: Content,
    dev: Ctx,
    elsewhere: Ctx,
    cold: Ctx,
}

async fn repository(db: &astrum_db::Database, content: &Content, name: &str, store: &str) -> Ctx {
    let created = astrum_repo::store::Writer::begin(db.conn())
        .await
        .unwrap()
        .create(Repository {
            name: name.to_owned(),
            format: format("raw").unwrap(),
            kind: RepositoryType::Hosted,
            online: true,
            attributes: Attributes {
                storage: Some(Storage {
                    blob_store_name: Some(store.to_owned()),
                    strict_content_type_validation: Some(false),
                    write_policy: Some(WritePolicy::Allow),
                    latest_policy: None,
                    quota: None,
                }),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    content.context(
        Arc::new(created),
        Arc::new(SecurityModel::default()),
        None,
        "",
        "",
    )
}

async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let dir = tempfile::tempdir().unwrap();
    astrum_blob::bootstrap(db.conn(), dir.path()).await.unwrap();
    for name in ["other", "aside"] {
        std::fs::create_dir_all(dir.path().join("blobs").join(name)).unwrap();
        astrum_blob::config::insert(
            db.conn(),
            &astrum_blob::StoreConfig {
                name: name.to_owned(),
                settings: astrum_blob::Settings::File(astrum_blob::FileSettings {
                    path: name.to_owned(),
                }),
                soft_quota: None,
            },
        )
        .await
        .unwrap();
    }
    let secrets = SecretBox::from_key([7; 32]);
    let stores = Arc::new(BlobStores::new(
        db.conn().clone(),
        dir.path(),
        secrets.clone(),
    ));
    let content = Content::new(db.conn().clone(), stores, secrets);
    let dev = repository(&db, &content, "dev", "default").await;
    let elsewhere = repository(&db, &content, "elsewhere", "default").await;
    let cold = repository(&db, &content, "cold", "aside").await;
    World {
        _test: test,
        db,
        _dir: dir,
        content,
        dev,
        elsewhere,
        cold,
    }
}

async fn put(ctx: &Ctx, path: &str, bytes: Vec<u8>) {
    store::put(
        ctx,
        PutAsset::hosted(ctx, path).component(ComponentKey {
            namespace: "g".to_owned(),
            name: path.to_owned(),
            version: "1".to_owned(),
            ..Default::default()
        }),
        content_of(bytes),
    )
    .await
    .unwrap();
}

async fn execute(world: &World, sql: &str, path: &str) {
    world
        .db
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [path.into()],
        ))
        .await
        .unwrap();
}

async fn age(world: &World, path: &str) {
    execute(
        world,
        "UPDATE asset SET blob_updated_at = now() - interval '200 days' WHERE path = $1",
        path,
    )
    .await;
}

async fn download(world: &World, path: &str) {
    execute(
        world,
        "UPDATE asset SET last_downloaded_at = now() WHERE path = $1",
        path,
    )
    .await;
}

async fn lapse(world: &World, store: &str, id: Uuid) {
    world
        .db
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE blob_hold SET held_until = now() - interval '1 minute' \
             WHERE blob_store = $1 AND blob_id = $2",
            [store.into(), id.into()],
        ))
        .await
        .unwrap();
}

async fn exists(world: &World, store: &str, id: Uuid) -> bool {
    world
        .content
        .blob_stores()
        .get(store)
        .await
        .unwrap()
        .exists(id.into())
        .await
        .unwrap()
}

async fn asset(world: &World, repository: &str, path: &str) -> Asset {
    store::find(world.db.conn(), repository, path)
        .await
        .unwrap()
        .unwrap()
}

async fn store_of(world: &World, repository: &str, path: &str) -> String {
    asset(world, repository, path).await.blob_store
}

fn task_run(world: &World, pairs: &[(&str, &str)]) -> Run {
    let properties: Properties = pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    Run::new(
        1,
        Uuid::new_v4(),
        "tier",
        ID,
        properties,
        world.content.clone(),
        Arc::new(AtomicBool::new(false)),
    )
}

async fn run(
    world: &World,
    pairs: &[(&str, &str)],
) -> (Result<String, TaskError>, BTreeMap<String, i64>) {
    let run = task_run(world, pairs);
    let result = Tier.run(&run).await;
    (result, run.counts())
}

#[derive(Debug, PartialEq)]
struct State {
    assets: Vec<(i64, String, Uuid)>,
    holds: i64,
    notices: i64,
    blobs: Vec<usize>,
}

async fn count(world: &World, sql: &str) -> i64 {
    world
        .db
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

async fn state(world: &World) -> State {
    let rows = world
        .db
        .conn()
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT id, blob_store, blob_id FROM asset ORDER BY id",
        ))
        .await
        .unwrap();
    let mut assets: Vec<(i64, String, Uuid)> = Vec::new();
    for row in rows {
        assets.push((
            row.try_get("", "id").unwrap(),
            row.try_get("", "blob_store").unwrap(),
            row.try_get("", "blob_id").unwrap(),
        ));
    }
    let mut blobs = Vec::new();
    for name in ["default", "other"] {
        let store = world.content.blob_stores().get(name).await.unwrap();
        blobs.push(store.list(None, 1000).await.unwrap().len());
    }
    State {
        assets,
        holds: count(world, "SELECT count(*) AS n FROM blob_hold").await,
        notices: count(world, "SELECT count(*) AS n FROM notification_change").await,
        blobs,
    }
}

#[tokio::test]
async fn only_idle_assets_above_the_minimum_size_in_the_source_repository_move() {
    let world = world().await;
    put(&world.dev, "/dev/idle-a.bin", vec![1; MIB]).await;
    put(&world.dev, "/dev/idle-b.bin", vec![2; MIB]).await;
    put(&world.dev, "/dev/young.bin", vec![3; MIB]).await;
    put(&world.dev, "/dev/downloaded.bin", vec![4; MIB]).await;
    put(&world.dev, "/dev/small.bin", vec![5; 10]).await;
    put(&world.elsewhere, "/elsewhere/idle.bin", vec![6; MIB]).await;
    put(&world.cold, "/cold/idle.bin", vec![7; MIB]).await;
    for path in [
        "/dev/idle-a.bin",
        "/dev/idle-b.bin",
        "/dev/downloaded.bin",
        "/dev/small.bin",
        "/elsewhere/idle.bin",
        "/cold/idle.bin",
    ] {
        age(&world, path).await;
    }
    download(&world, "/dev/downloaded.bin").await;

    let (result, counts) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
        ],
    )
    .await;
    assert_eq!(
        result.unwrap(),
        "moved 2 assets (2.0 MB) from default to other; 0 changed meanwhile, 0 corrupt at source, 0 failed; freed 0 bytes from earlier runs"
    );
    assert_eq!(counts["candidates"], 2);
    for (repository, path, wanted) in [
        ("dev", "/dev/idle-a.bin", "other"),
        ("dev", "/dev/idle-b.bin", "other"),
        ("dev", "/dev/young.bin", "default"),
        ("dev", "/dev/downloaded.bin", "default"),
        ("dev", "/dev/small.bin", "default"),
        ("elsewhere", "/elsewhere/idle.bin", "default"),
        ("cold", "/cold/idle.bin", "aside"),
    ] {
        assert_eq!(store_of(&world, repository, path).await, wanted, "{path}");
    }
}

#[tokio::test]
async fn the_asset_limit_stops_the_run_and_the_next_run_continues() {
    let world = world().await;
    for (index, path) in ["/dev/one.bin", "/dev/two.bin", "/dev/three.bin"]
        .into_iter()
        .enumerate()
    {
        put(&world.dev, path, vec![index as u8 + 1; MIB]).await;
        age(&world, path).await;
    }
    let pairs = [
        ("source", "default"),
        ("target", "other"),
        ("repository", "dev"),
        ("maxAssetsPerRun", "2"),
    ];

    let (first, _) = run(&world, &pairs).await;
    assert!(
        first
            .unwrap()
            .starts_with("moved 2 assets (2.0 MB) from default to other;")
    );
    let (second, _) = run(&world, &pairs).await;
    assert!(
        second
            .unwrap()
            .starts_with("moved 1 asset (1.0 MB) from default to other;")
    );
    for path in ["/dev/one.bin", "/dev/two.bin", "/dev/three.bin"] {
        assert_eq!(store_of(&world, "dev", path).await, "other", "{path}");
    }
}

#[tokio::test]
async fn a_dry_run_writes_nothing_and_says_what_it_would_move() {
    let world = world().await;
    put(&world.dev, "/dev/idle-a.bin", vec![1; MIB]).await;
    put(&world.dev, "/dev/idle-b.bin", vec![2; MIB]).await;
    age(&world, "/dev/idle-a.bin").await;
    age(&world, "/dev/idle-b.bin").await;
    let orphan = world
        .content
        .blob_stores()
        .get("other")
        .await
        .unwrap()
        .put(content_of(&b"orphan"[..]))
        .await
        .unwrap();
    holds::hold_for(
        world.db.conn(),
        "other",
        orphan.id,
        Duration::from_secs(60),
        holds::TIERING,
    )
    .await
    .unwrap();
    lapse(&world, "other", orphan.id.uuid()).await;
    let before = state(&world).await;

    let (result, _) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
            ("dryRun", "true"),
        ],
    )
    .await;

    assert!(
        result
            .unwrap()
            .starts_with("would move 2 assets (2.0 MB) from default to other")
    );
    assert_eq!(state(&world).await, before);
    assert!(
        exists(&world, "other", orphan.id.uuid()).await,
        "a dry run leaves the lapsed hold and its blob for the next real run"
    );
}

#[tokio::test]
async fn a_reverse_run_restores_every_asset_to_the_source() {
    let world = world().await;
    put(&world.dev, "/dev/idle-a.bin", vec![1; MIB]).await;
    put(&world.dev, "/dev/idle-b.bin", vec![2; MIB]).await;
    age(&world, "/dev/idle-a.bin").await;
    age(&world, "/dev/idle-b.bin").await;
    run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
        ],
    )
    .await
    .0
    .unwrap();

    let (result, _) = run(
        &world,
        &[
            ("source", "other"),
            ("target", "default"),
            ("repository", "dev"),
            ("idleDays", "0"),
        ],
    )
    .await;

    assert!(
        result
            .unwrap()
            .starts_with("moved 2 assets (2.0 MB) from other to default;")
    );
    for path in ["/dev/idle-a.bin", "/dev/idle-b.bin"] {
        assert_eq!(store_of(&world, "dev", path).await, "default", "{path}");
    }
}

#[tokio::test]
async fn equal_content_moves_as_a_duplicate_without_reading_the_source_again() {
    let world = world().await;
    put(&world.dev, "/dev/twin-a.bin", vec![3; MIB]).await;
    put(&world.dev, "/dev/twin-b.bin", vec![3; MIB]).await;
    age(&world, "/dev/twin-a.bin").await;
    age(&world, "/dev/twin-b.bin").await;
    let second = asset(&world, "dev", "/dev/twin-b.bin").await;
    world
        .content
        .blob_stores()
        .get("default")
        .await
        .unwrap()
        .delete(second.blob_id.into())
        .await
        .unwrap();

    let (result, counts) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
        ],
    )
    .await;

    assert!(
        result
            .unwrap()
            .starts_with("moved 2 assets (2.0 MB) from default to other;")
    );
    assert_eq!(counts["failed"], 0);
    for path in ["/dev/twin-a.bin", "/dev/twin-b.bin"] {
        let moved = asset(&world, "dev", path).await;
        assert_eq!(moved.blob_store, "other");
        let size = world
            .content
            .blob_stores()
            .get("other")
            .await
            .unwrap()
            .size(moved.blob_id.into())
            .await
            .unwrap();
        assert_eq!(size, Some(MIB as u64), "{path}");
    }
}

#[tokio::test]
async fn an_asset_replaced_during_the_run_is_counted_as_changed_and_the_run_goes_on() {
    let world = world().await;
    for path in ["/dev/busy-a.bin", "/dev/busy-b.bin", "/dev/busy-c.bin"] {
        put(&world.dev, path, vec![8; MIB]).await;
        age(&world, path).await;
    }
    world
        .db
        .conn()
        .execute_unprepared(
            "CREATE FUNCTION replace_busy_b() RETURNS trigger LANGUAGE plpgsql AS $$ \
             BEGIN UPDATE asset SET blob_id = gen_random_uuid() WHERE path = '/dev/busy-b.bin'; RETURN NEW; END $$; \
             CREATE TRIGGER replace_busy_b AFTER INSERT ON blob_hold FOR EACH ROW \
             WHEN (NEW.reason = 'tiering') EXECUTE FUNCTION replace_busy_b()",
        )
        .await
        .unwrap();

    let (result, counts) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
        ],
    )
    .await;

    assert_eq!(
        result.unwrap(),
        "moved 2 assets (2.0 MB) from default to other; 1 changed meanwhile, 0 corrupt at source, 0 failed; freed 0 bytes from earlier runs"
    );
    assert_eq!(counts["changed"], 1);
    assert_eq!(store_of(&world, "dev", "/dev/busy-b.bin").await, "default");
    assert_eq!(store_of(&world, "dev", "/dev/busy-c.bin").await, "other");
}

#[tokio::test]
async fn an_asset_whose_source_blob_is_missing_fails_and_the_others_move() {
    let world = world().await;
    put(&world.dev, "/dev/bad-a.bin", vec![1; MIB]).await;
    put(&world.dev, "/dev/bad-b.bin", vec![2; MIB]).await;
    put(&world.dev, "/dev/bad-c.bin", vec![3; MIB]).await;
    for path in ["/dev/bad-a.bin", "/dev/bad-b.bin", "/dev/bad-c.bin"] {
        age(&world, path).await;
    }
    let broken = asset(&world, "dev", "/dev/bad-b.bin").await;
    world
        .content
        .blob_stores()
        .get("default")
        .await
        .unwrap()
        .delete(broken.blob_id.into())
        .await
        .unwrap();

    let (result, counts) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
        ],
    )
    .await;

    assert_eq!(
        result.unwrap(),
        "moved 2 assets (2.0 MB) from default to other; 0 changed meanwhile, 0 corrupt at source, 1 failed; freed 0 bytes from earlier runs"
    );
    assert_eq!(counts["failed"], 1);
    assert_eq!(store_of(&world, "dev", "/dev/bad-a.bin").await, "other");
    assert_eq!(store_of(&world, "dev", "/dev/bad-b.bin").await, "default");
    assert_eq!(store_of(&world, "dev", "/dev/bad-c.bin").await, "other");
}

async fn three_distinct_blobs(world: &World) {
    for (index, path) in ["/dev/paced-a.bin", "/dev/paced-b.bin", "/dev/paced-c.bin"]
        .into_iter()
        .enumerate()
    {
        put(&world.dev, path, vec![index as u8 + 1; 2 * MIB]).await;
        age(world, path).await;
    }
}

#[tokio::test]
async fn the_speed_limit_paces_a_run_of_distinct_blobs() {
    let world = world().await;
    three_distinct_blobs(&world).await;

    let started = Instant::now();
    let (result, counts) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
            ("maxMiBPerSecond", "2"),
        ],
    )
    .await;
    let elapsed = started.elapsed();

    assert!(
        result
            .unwrap()
            .starts_with("moved 3 assets (6.0 MB) from default to other;")
    );
    assert_eq!(counts["failed"], 0);
    assert!(elapsed >= Duration::from_millis(2200), "{elapsed:?}");
    assert!(elapsed <= Duration::from_secs(8), "{elapsed:?}");
}

#[tokio::test]
async fn a_zero_speed_limit_means_no_limit() {
    let world = world().await;
    three_distinct_blobs(&world).await;

    let started = Instant::now();
    let (result, _) = run(
        &world,
        &[
            ("source", "default"),
            ("target", "other"),
            ("repository", "dev"),
            ("maxMiBPerSecond", "0"),
        ],
    )
    .await;

    assert!(
        result
            .unwrap()
            .starts_with("moved 3 assets (6.0 MB) from default to other;")
    );
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{:?}",
        started.elapsed()
    );
}
