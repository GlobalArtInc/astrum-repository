use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, SystemTime};

use astrum_blob::{BlobStores, content_of};
use astrum_content::store::{self, Asset, ComponentKey, PutAsset};
use astrum_content::tiering::{Options, Outcome, Snapshot, move_asset};
use astrum_content::{Content, Ctx, holds};
use astrum_repo::config::{Attributes, Repository, Storage, WritePolicy};
use astrum_repo::{RepositoryType, format};
use astrum_security::{SecretBox, SecurityModel};
use astrum_tasks::kinds::compact::{self, Compact};
use astrum_tasks::kinds::expired::BlobHoldExpiry;
use astrum_tasks::{Expiry, Properties, Run, TaskKind};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use uuid::Uuid;

struct World {
    _test: TestDatabase,
    db: astrum_db::Database,
    dir: tempfile::TempDir,
    content: Content,
    dev: Ctx,
}

async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let dir = tempfile::tempdir().unwrap();
    astrum_blob::bootstrap(db.conn(), dir.path()).await.unwrap();
    std::fs::create_dir_all(dir.path().join("blobs").join("other")).unwrap();
    astrum_blob::config::insert(
        db.conn(),
        &astrum_blob::StoreConfig {
            name: "other".to_owned(),
            settings: astrum_blob::Settings::File(astrum_blob::FileSettings {
                path: "other".to_owned(),
            }),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    let secrets = SecretBox::from_key([7; 32]);
    let stores = Arc::new(BlobStores::new(
        db.conn().clone(),
        dir.path(),
        secrets.clone(),
    ));
    let content = Content::new(db.conn().clone(), stores, secrets);
    let created = astrum_repo::store::Writer::begin(db.conn())
        .await
        .unwrap()
        .create(Repository {
            name: "dev".to_owned(),
            format: format("raw").unwrap(),
            kind: RepositoryType::Hosted,
            online: true,
            attributes: Attributes {
                storage: Some(Storage {
                    blob_store_name: Some("default".to_owned()),
                    strict_content_type_validation: Some(false),
                    write_policy: Some(WritePolicy::Allow),
                    latest_policy: None,
                }),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    let dev = content.context(
        Arc::new(created),
        Arc::new(SecurityModel::default()),
        None,
        "",
        "",
    );
    World {
        _test: test,
        db,
        dir,
        content,
        dev,
    }
}

async fn seeded(world: &World) -> Asset {
    store::put(
        &world.dev,
        PutAsset::hosted(&world.dev, "/g/a.txt").component(ComponentKey {
            namespace: "g".to_owned(),
            name: "lib".to_owned(),
            version: "1".to_owned(),
            ..Default::default()
        }),
        content_of(&b"alpha"[..]),
    )
    .await
    .unwrap();
    store::find(world.db.conn(), "dev", "/g/a.txt")
        .await
        .unwrap()
        .unwrap()
}

async fn moved(world: &World, before: &Asset) -> Asset {
    let options = Options {
        drain: Duration::from_secs(3600),
        hold_ttl: Duration::from_secs(1800),
        limiter: None,
        duplicate_of: None,
    };
    let outcome = move_asset(&world.content, Snapshot::from(before), "other", &options).await;
    assert!(matches!(outcome, Outcome::Moved { .. }), "{outcome:?}");
    store::find(world.db.conn(), "dev", "/g/a.txt")
        .await
        .unwrap()
        .unwrap()
}

async fn moved_asset(world: &World) -> (Asset, Asset) {
    let before = seeded(world).await;
    let after = moved(world, &before).await;
    (before, after)
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

fn age(world: &World, store: &str, id: Uuid) {
    let path = world
        .dir
        .path()
        .join("blobs")
        .join(store)
        .join("content")
        .join(astrum_blob::BlobId::from(id).location());
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(7200))
        .unwrap();
}

async fn compact_default(world: &World) {
    let mut properties = Properties::new();
    properties.insert("blobstoreName".to_owned(), "default".to_owned());
    properties.insert("blobsOlderThan".to_owned(), "0".to_owned());
    let run = Run::new(
        1,
        Uuid::new_v4(),
        "compact",
        compact::ID,
        properties,
        world.content.clone(),
        Arc::new(AtomicBool::new(false)),
    );
    Compact.run(&run).await.unwrap();
}

#[tokio::test]
async fn compaction_spares_the_retired_blob_and_the_sweep_removes_it_after_the_drain() {
    let world = world().await;
    let (before, after) = moved_asset(&world).await;
    let orphan = world
        .content
        .blob_stores()
        .get("default")
        .await
        .unwrap()
        .put(content_of(&b"orphan"[..]))
        .await
        .unwrap();
    age(&world, "default", before.blob_id);
    age(&world, "default", orphan.id.uuid());

    compact_default(&world).await;

    assert!(
        exists(&world, "default", before.blob_id).await,
        "the retired hold protects the old blob from compaction"
    );
    assert!(
        !exists(&world, "default", orphan.id.uuid()).await,
        "compaction does delete an unprotected blob"
    );

    assert_eq!(BlobHoldExpiry.expire(&world.content).await.unwrap(), 0);
    assert!(exists(&world, "default", before.blob_id).await);

    lapse(&world, "default", before.blob_id).await;
    assert_eq!(BlobHoldExpiry.expire(&world.content).await.unwrap(), 1);
    assert!(!exists(&world, "default", before.blob_id).await);
    assert!(
        exists(&world, "other", after.blob_id).await,
        "the blob the asset names stays"
    );
}

#[tokio::test]
async fn the_sweep_removes_what_a_crashed_move_left_and_spares_what_an_asset_names() {
    let world = world().await;
    let (_, after) = moved_asset(&world).await;
    let other = world.content.blob_stores().get("other").await.unwrap();
    let leftover = other.put(content_of(&b"leftover"[..])).await.unwrap();
    for id in [leftover.id, after.blob_id.into()] {
        holds::hold_for(
            world.db.conn(),
            "other",
            id,
            Duration::from_secs(60),
            holds::TIERING,
        )
        .await
        .unwrap();
        lapse(&world, "other", id.uuid()).await;
    }

    assert_eq!(BlobHoldExpiry.expire(&world.content).await.unwrap(), 2);

    assert!(!exists(&world, "other", leftover.id.uuid()).await);
    assert!(exists(&world, "other", after.blob_id).await);
}

#[tokio::test]
async fn a_lapsed_hold_on_the_old_blob_does_not_let_the_sweep_remove_it_before_the_drain_ends() {
    let world = world().await;
    let before = seeded(&world).await;
    holds::hold(
        world.db.conn(),
        "default",
        before.blob_id.into(),
        Duration::from_secs(600),
    )
    .await
    .unwrap();
    lapse(&world, "default", before.blob_id).await;

    moved(&world, &before).await;

    assert_eq!(
        BlobHoldExpiry.expire(&world.content).await.unwrap(),
        0,
        "the sweep right after the commit finds nothing to take"
    );
    assert!(exists(&world, "default", before.blob_id).await);

    lapse(&world, "default", before.blob_id).await;
    assert_eq!(BlobHoldExpiry.expire(&world.content).await.unwrap(), 1);
    assert!(!exists(&world, "default", before.blob_id).await);
}
