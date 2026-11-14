use astrum_content::store::{self, ComponentKey};
use astrum_content::transfer::{Conflict, Status};
use astrum_repo::config::WritePolicy;
use sea_orm::sqlx::{self, Row};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

mod support;
use support::*;

async fn watch(world: &World) {
    world
        .db
        .conn()
        .execute_unprepared("INSERT INTO notification_channel (id, name, type) VALUES (gen_random_uuid(), 'watch', 'webhook')")
        .await
        .unwrap();
}

async fn actions(world: &World, repository: &str) -> String {
    world
        .db
        .conn()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT coalesce(string_agg(action, ',' ORDER BY action, id), '') AS list FROM notification_change WHERE repository = $1",
            [repository.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "list")
        .unwrap()
}

async fn count(world: &World, repository: &str, action: &str) -> i64 {
    number(
        world,
        &format!(
            "SELECT count(*) AS n FROM notification_change WHERE repository = '{repository}' AND action = '{action}'"
        ),
    )
    .await
}

async fn promoted(world: &World) -> (astrum_content::Ctx, astrum_content::Ctx) {
    let dev = repository(world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    seed(&dev, "/g/c.txt", b"charlie").await;
    let bundle = bundle_of(&dev).await;
    let outcome = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.assets.len(), 3);
    (dev, target)
}

#[tokio::test]
async fn a_promoted_component_raises_promoted_notices_and_no_asset_updates() {
    let world = world().await;
    watch(&world).await;
    let _ = promoted(&world).await;
    assert!(count(&world, "target", "promoted").await >= 1);
    assert_eq!(count(&world, "target", "updated").await, 0);
    assert_eq!(count(&world, "target", "published").await, 0);
    assert_eq!(
        actions(&world, "dev").await,
        "published,updated,updated,updated"
    );
}

#[tokio::test]
async fn replacing_assets_of_an_existing_component_by_promotion_is_promoted_not_updated() {
    let world = world().await;
    watch(&world).await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    for path in ["/g/a.txt", "/g/b.txt", "/g/c.txt"] {
        seed(&target, path, b"older").await;
    }
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    seed(&dev, "/g/c.txt", b"charlie").await;
    let bundle = bundle_of(&dev).await;
    let outcome = run(&dev, &target, &bundle, Conflict::Replace, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(count(&world, "target", "published").await, 1);
    assert_eq!(count(&world, "target", "updated").await, 3);
    assert!(count(&world, "target", "promoted").await >= 1);
}

#[tokio::test]
async fn replacing_a_file_by_hand_is_still_updated() {
    let world = world().await;
    watch(&world).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&target, "/g/a.txt", b"older").await;
    seed(&target, "/g/a.txt", b"newer").await;
    assert_eq!(actions(&world, "target").await, "published,updated,updated");
}

#[tokio::test]
async fn an_upload_after_a_promotion_publishes_and_no_connection_keeps_the_setting() {
    let world = world().await;
    watch(&world).await;
    let (_, target) = promoted(&world).await;
    let pool = world.db.conn().get_postgres_connection_pool();
    let mut held = Vec::new();
    for _ in 0..pool.options().get_max_connections() {
        held.push(pool.acquire().await.unwrap());
    }
    for connection in &mut held {
        let row = sqlx::query(
            "SELECT current_setting('astrum.action', true) AS action, current_setting('astrum.initiator', true) AS initiator",
        )
        .fetch_one(&mut **connection)
        .await
        .unwrap();
        for column in ["action", "initiator"] {
            let value: Option<String> = row.get(column);
            assert_eq!(value.unwrap_or_default(), "", "{column} leaked");
        }
    }
    drop(held);
    store::put(
        &target,
        store::PutAsset::hosted(&target, "/h/a.txt").component(ComponentKey {
            name: "fresh".to_owned(),
            ..key()
        }),
        astrum_blob::content_of(b"fresh".as_slice()),
    )
    .await
    .unwrap();
    assert!(count(&world, "target", "promoted").await >= 1);
    assert_eq!(count(&world, "target", "published").await, 1);
    assert_eq!(count(&world, "target", "updated").await, 1);
}

#[tokio::test]
async fn the_deletion_of_a_moved_source_is_still_a_deleted_notice() {
    let world = world().await;
    watch(&world).await;
    let (dev, _) = promoted(&world).await;
    let component = row(&dev, "/g/a.txt").await.component_id.unwrap();
    assert!(
        store::delete_component(&dev.content, component)
            .await
            .unwrap()
    );
    assert_eq!(
        actions(&world, "dev").await,
        "deleted,published,updated,updated,updated"
    );
    assert_eq!(count(&world, "target", "deleted").await, 0);
    assert_eq!(count(&world, "target", "updated").await, 0);
}

#[tokio::test]
async fn a_component_emptied_by_a_promotion_is_reported_as_deleted_not_promoted() {
    let world = world().await;
    watch(&world).await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&target, "/g/a.txt", b"older").await;
    store::put(
        &dev,
        store::PutAsset::hosted(&dev, "/g/a.txt").component(ComponentKey {
            version: "2".to_owned(),
            ..key()
        }),
        astrum_blob::content_of(b"newer".as_slice()),
    )
    .await
    .unwrap();
    let bundle = bundle_of(&dev).await;
    let outcome = run(&dev, &target, &bundle, Conflict::Replace, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(count(&world, "target", "deleted").await, 1);
    assert_eq!(count(&world, "target", "updated").await, 1);
    assert!(count(&world, "target", "promoted").await >= 1);
}

#[tokio::test]
async fn an_unknown_action_in_the_setting_is_never_taken_as_a_notice_action() {
    let world = world().await;
    watch(&world).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let blobs = target.content.blob_stores().get("default").await.unwrap();
    let written = blobs
        .put(astrum_blob::content_of(b"evil".as_slice()))
        .await
        .unwrap();
    astrum_content::holds::hold(
        target.db(),
        &written.store,
        written.id,
        std::time::Duration::from_secs(600),
    )
    .await
    .unwrap();
    let put = store::PutAsset::hosted(&target, "/g/a.txt").component(key());
    store::record_many(&target, vec![(put, written)], Conflict::Fail, "evil")
        .await
        .unwrap();
    assert_eq!(actions(&world, "target").await, "published,updated");
}
