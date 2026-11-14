use std::time::Duration;

use astrum_blob::{BlobId, S3Bucket, S3Connection, S3Security, S3Settings, Settings, StoreConfig};
use astrum_content::holds;
use astrum_content::store::{self, Asset};
use astrum_content::tiering::{Options, Outcome, Snapshot, move_asset};
use astrum_content::{Ctx, Serve, serve};
use astrum_repo::config::WritePolicy;
use astrum_security::SecretBox;
use astrum_testkit::s3::{ACCESS_KEY, S3Server, SECRET_KEY};
use axum::http::{HeaderMap, Method};
use sea_orm::{ConnectionTrait, DatabaseTransaction, TransactionTrait};
use tokio::task::JoinHandle;

mod support;
use support::*;

fn options() -> Options {
    Options {
        drain: Duration::from_secs(3600),
        hold_ttl: Duration::from_secs(1800),
        limiter: None,
        duplicate_of: None,
    }
}

async fn move_to(world: &World, ctx: &Ctx, path: &str, target: &str) -> Outcome {
    let snapshot = Snapshot::from(&row(ctx, path).await);
    move_asset(&world.content, snapshot, target, &options()).await
}

fn new_blob(outcome: Outcome) -> BlobId {
    match outcome {
        Outcome::Moved { new_id, .. } => new_id,
        other => panic!("the asset was not moved: {other:?}"),
    }
}

async fn served(ctx: &Ctx, asset: &Asset) -> Vec<u8> {
    let response = serve(
        ctx,
        asset,
        &Method::GET,
        &HeaderMap::new(),
        Serve::default(),
    )
    .await
    .unwrap();
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

async fn blocked_move(
    world: &World,
    ctx: &Ctx,
    path: &str,
) -> (DatabaseTransaction, JoinHandle<Outcome>) {
    let holder = world.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT 1 FROM blob_store WHERE name = 'other' FOR NO KEY UPDATE")
        .await
        .unwrap();
    let content = world.content.clone();
    let snapshot = Snapshot::from(&row(ctx, path).await);
    let task =
        tokio::spawn(async move { move_asset(&content, snapshot, "other", &options()).await });
    wait_for_lock(world).await;
    (holder, task)
}

async fn finish(holder: DatabaseTransaction, task: JoinHandle<Outcome>) -> Outcome {
    holder.commit().await.unwrap();
    task.await.unwrap()
}

async fn usage(world: &World) -> (i64, i64, String) {
    let found = world
        .db
        .conn()
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT assets::bigint AS assets, bytes::bigint AS bytes, updated_at::text AS updated \
             FROM repository_usage WHERE repository = 'dev'",
        ))
        .await
        .unwrap()
        .unwrap();
    (
        found.try_get("", "assets").unwrap(),
        found.try_get("", "bytes").unwrap(),
        found.try_get("", "updated").unwrap(),
    )
}

#[tokio::test]
async fn a_moved_asset_is_served_from_the_target_and_an_old_reader_still_gets_the_old_blob() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let before = row(&dev, "/g/a.txt").await;

    let new_id = new_blob(move_to(&world, &dev, "/g/a.txt", "other").await);
    let after = row(&dev, "/g/a.txt").await;
    assert_eq!(
        (after.blob_store.as_str(), after.blob_id),
        ("other", new_id.uuid())
    );
    assert_eq!(served(&dev, &after).await, b"alpha");
    assert!(blob_file(&world, "other", new_id.uuid()).is_file());

    assert_eq!(
        served(&dev, &before).await,
        b"alpha",
        "a reader that loaded the old row keeps a valid blob"
    );
    let retired = number(
        &world,
        &format!(
            "SELECT count(*) AS n FROM blob_hold WHERE blob_store = 'default' \
             AND blob_id = '{}' AND reason = 'retired' AND held_until > now() + interval '55 minutes'",
            before.blob_id
        ),
    )
    .await;
    assert_eq!(retired, 1, "the old blob is held for the drain");
    assert_eq!(
        hold_rows(&world).await,
        1,
        "the hold of the new blob is consumed"
    );
}

#[tokio::test]
async fn a_move_changes_only_where_the_blob_lives() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    world
        .db
        .conn()
        .execute_unprepared(
            "INSERT INTO notification_channel (id, name, type) \
             VALUES (gen_random_uuid(), 'watch', 'webhook')",
        )
        .await
        .unwrap();
    seed(&dev, "/g/a.txt", b"alpha").await;
    world
        .db
        .conn()
        .execute_unprepared("UPDATE asset SET last_downloaded_at = now() - interval '3 days'")
        .await
        .unwrap();
    let before = row(&dev, "/g/a.txt").await;
    let notices = number(&world, "SELECT count(*) AS n FROM notification_change").await;
    assert!(notices > 0, "an ordinary write is noticed");
    let counter = usage(&world).await;

    let new_id = new_blob(move_to(&world, &dev, "/g/a.txt", "other").await);

    let mut after = row(&dev, "/g/a.txt").await;
    assert!(before.last_downloaded_at.is_some());
    assert_eq!(after.blob_id, new_id.uuid());
    after.blob_store = before.blob_store.clone();
    after.blob_id = before.blob_id;
    assert_eq!(after, before, "every other column is untouched");
    assert_eq!(
        number(&world, "SELECT count(*) AS n FROM notification_change").await,
        notices,
        "a move is not a change of the component"
    );
    assert_eq!(usage(&world).await, counter);
}

#[tokio::test]
async fn a_reupload_between_the_copy_and_the_commit_wins() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let (holder, task) = blocked_move(&world, &dev, "/g/a.txt").await;

    seed(&dev, "/g/a.txt", b"alpha, replaced").await;
    let outcome = finish(holder, task).await;

    assert_eq!(outcome, Outcome::Changed);
    let after = row(&dev, "/g/a.txt").await;
    assert_eq!(after.blob_store, "default");
    assert_eq!(served(&dev, &after).await, b"alpha, replaced");
    assert_eq!(
        blobs_in(&world, "other").await,
        0,
        "the new blob was discarded"
    );
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn an_asset_deleted_during_the_move_leaves_nothing_in_the_target() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let (holder, task) = blocked_move(&world, &dev, "/g/a.txt").await;

    assert!(store::delete(&dev, "/g/a.txt").await.unwrap());
    let outcome = finish(holder, task).await;

    assert_eq!(outcome, Outcome::Changed);
    assert!(
        store::find(dev.db(), "dev", "/g/a.txt")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_source_blob_that_is_gone_is_a_race_only_when_the_asset_moved_on() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let (a, b) = (row(&dev, "/g/a.txt").await, row(&dev, "/g/b.txt").await);
    let (stale, intact) = (Snapshot::from(&a), Snapshot::from(&b));
    let default = world.content.blob_stores().get("default").await.unwrap();
    store::delete(&dev, "/g/a.txt").await.unwrap();
    default.delete(a.blob_id.into()).await.unwrap();
    default.delete(b.blob_id.into()).await.unwrap();

    let gone = move_asset(&world.content, stale, "other", &options()).await;
    assert_eq!(gone, Outcome::Changed);
    let lost = move_asset(&world.content, intact, "other", &options()).await;
    assert!(matches!(lost, Outcome::Failed(_)), "{lost:?}");
    assert_eq!(row(&dev, "/g/b.txt").await, b);
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_target_store_deleted_before_the_commit_discards_the_new_blob() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let before = row(&dev, "/g/a.txt").await;
    let (holder, task) = blocked_move(&world, &dev, "/g/a.txt").await;

    holder
        .execute_unprepared("DELETE FROM blob_store WHERE name = 'other'")
        .await
        .unwrap();
    let outcome = finish(holder, task).await;

    assert_eq!(outcome, Outcome::Changed);
    assert_eq!(row(&dev, "/g/a.txt").await, before);
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_corrupt_source_is_reported_and_nothing_is_left_behind() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let before = row(&dev, "/g/a.txt").await;
    std::fs::write(blob_file(&world, "default", before.blob_id), b"alpho").unwrap();

    let outcome = move_to(&world, &dev, "/g/a.txt", "other").await;

    assert_eq!(outcome, Outcome::Corrupt);
    assert_eq!(row(&dev, "/g/a.txt").await, before);
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn an_error_before_the_commit_leaves_the_asset_on_the_old_blob_and_no_leaks() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let before = row(&dev, "/g/a.txt").await;
    world
        .db
        .conn()
        .execute_unprepared(
            "CREATE FUNCTION refuse_move() RETURNS trigger LANGUAGE plpgsql AS $$ \
             BEGIN RAISE EXCEPTION 'move refused'; END $$; \
             CREATE TRIGGER refuse_move BEFORE UPDATE OF blob_id ON asset \
             FOR EACH ROW EXECUTE FUNCTION refuse_move()",
        )
        .await
        .unwrap();

    let outcome = move_to(&world, &dev, "/g/a.txt", "other").await;

    let Outcome::Failed(message) = outcome else {
        panic!("the move did not fail: {outcome:?}");
    };
    assert!(message.contains("move refused"), "{message}");
    assert_eq!(row(&dev, "/g/a.txt").await, before);
    assert!(blob_file(&world, "default", before.blob_id).is_file());
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(
        hold_rows(&world).await,
        0,
        "no hold survives the failed move"
    );
}

#[tokio::test]
async fn a_new_blob_that_lost_its_hold_abandons_the_move() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let before = row(&dev, "/g/a.txt").await;
    world
        .db
        .conn()
        .execute_unprepared(
            "CREATE FUNCTION keep_hold() RETURNS trigger LANGUAGE plpgsql AS $$ \
             BEGIN RETURN NULL; END $$; \
             CREATE TRIGGER keep_hold BEFORE DELETE ON blob_hold \
             FOR EACH ROW WHEN (OLD.reason = 'tiering') EXECUTE FUNCTION keep_hold()",
        )
        .await
        .unwrap();

    let outcome = move_to(&world, &dev, "/g/a.txt", "other").await;

    let Outcome::Failed(message) = outcome else {
        panic!("the move did not fail: {outcome:?}");
    };
    assert!(message.contains("lost its hold"), "{message}");
    assert_eq!(row(&dev, "/g/a.txt").await, before);
    assert_eq!(blobs_in(&world, "other").await, 0);
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM blob_hold WHERE reason = 'retired'"
        )
        .await,
        0,
        "the retired hold was rolled back with the rest"
    );
}

#[tokio::test]
async fn a_hold_that_already_guards_the_old_blob_is_extended_and_never_shortened() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let (lapsed, long) = (row(&dev, "/g/a.txt").await, row(&dev, "/g/b.txt").await);
    holds::hold(
        world.db.conn(),
        "default",
        lapsed.blob_id.into(),
        Duration::from_secs(600),
    )
    .await
    .unwrap();
    world
        .db
        .conn()
        .execute_unprepared(&format!(
            "UPDATE blob_hold SET held_until = now() - interval '1 minute' WHERE blob_id = '{}'",
            lapsed.blob_id
        ))
        .await
        .unwrap();
    holds::hold(
        world.db.conn(),
        "default",
        long.blob_id.into(),
        Duration::from_secs(3 * 3600),
    )
    .await
    .unwrap();

    new_blob(move_to(&world, &dev, "/g/a.txt", "other").await);
    new_blob(move_to(&world, &dev, "/g/b.txt", "other").await);

    let kept = |blob: &Asset, from: &str, to: &str| {
        format!(
            "SELECT count(*) AS n FROM blob_hold WHERE blob_id = '{}' AND reason = 'transfer' \
             AND held_until > now() + interval '{from}' AND held_until < now() + interval '{to}'",
            blob.blob_id
        )
    };
    assert_eq!(
        number(&world, &kept(&lapsed, "55 minutes", "65 minutes")).await,
        1,
        "a lapsed hold is lengthened to the drain and keeps its reason"
    );
    assert_eq!(
        number(&world, &kept(&long, "170 minutes", "190 minutes")).await,
        1,
        "a longer hold is not shortened"
    );
}

#[tokio::test]
async fn an_equal_blob_is_duplicated_in_the_target_when_the_run_already_moved_one() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"same bytes").await;
    seed(&dev, "/g/b.txt", b"same bytes").await;
    seed(&dev, "/g/c.txt", b"not the same").await;
    let first = new_blob(move_to(&world, &dev, "/g/a.txt", "other").await);

    let second_options = Options {
        duplicate_of: Some(first),
        ..options()
    };
    let snapshot = Snapshot::from(&row(&dev, "/g/b.txt").await);
    let second = new_blob(move_asset(&world.content, snapshot, "other", &second_options).await);

    assert_ne!(first, second);
    assert_eq!(blobs_in(&world, "other").await, 2);
    for path in ["/g/a.txt", "/g/b.txt"] {
        let moved = row(&dev, path).await;
        assert_eq!(moved.blob_store, "other");
        assert_eq!(served(&dev, &moved).await, b"same bytes");
    }

    let wrong = Snapshot::from(&row(&dev, "/g/c.txt").await);
    let refused = move_asset(&world.content, wrong, "other", &second_options).await;
    assert!(matches!(refused, Outcome::Failed(_)), "{refused:?}");
    assert_eq!(row(&dev, "/g/c.txt").await.blob_store, "default");
    assert_eq!(blobs_in(&world, "other").await, 2);
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM blob_hold WHERE blob_store = 'other'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn an_asset_moves_to_an_s3_store_and_back() {
    let server = S3Server::start().await;
    let world = world().await;
    let settings = S3Settings {
        bucket: S3Bucket {
            region: "DEFAULT".to_owned(),
            name: "cold".to_owned(),
            prefix: String::new(),
            expiration: None,
        },
        connection: S3Connection {
            endpoint: Some(server.endpoint()),
            force_path_style: true,
            ..Default::default()
        },
        security: S3Security {
            access_key_id: Some(ACCESS_KEY.to_owned()),
            secret_access_key: Some(SecretBox::from_key([7; 32]).seal(SECRET_KEY)),
            ..Default::default()
        },
        ..Default::default()
    };
    astrum_blob::config::insert(
        world.db.conn(),
        &StoreConfig {
            name: "cold".to_owned(),
            settings: Settings::S3(Box::new(settings.clone())),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    world
        .content
        .blob_stores()
        .open_s3("cold", &settings)
        .await
        .unwrap()
        .ensure_bucket()
        .await
        .unwrap();
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;

    new_blob(move_to(&world, &dev, "/g/a.txt", "cold").await);
    let cold = row(&dev, "/g/a.txt").await;
    assert_eq!(cold.blob_store, "cold");
    assert_eq!(served(&dev, &cold).await, b"alpha");
    assert_eq!(server.objects("cold").len(), 1);

    new_blob(move_to(&world, &dev, "/g/a.txt", "default").await);
    let back = row(&dev, "/g/a.txt").await;
    assert_eq!(back.blob_store, "default");
    assert_eq!(served(&dev, &back).await, b"alpha");
}
