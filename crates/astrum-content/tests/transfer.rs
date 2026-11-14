use std::os::unix::fs::MetadataExt;
use std::time::Duration;

use astrum_blob::content_of;
use astrum_content::store::{self, PutAsset};
use astrum_content::transfer::{AssetStatus, Conflict, PROMOTED, Status};
use astrum_content::{ContentError, holds};
use astrum_repo::config::WritePolicy;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

mod support;
use support::*;

#[tokio::test]
async fn a_component_is_copied_by_link_when_the_stores_match_and_streamed_when_they_differ() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let linked = repository(&world, "linked", "default", WritePolicy::Allow).await;
    let streamed = repository(&world, "streamed", "other", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo!").await;
    let bundle = bundle_of(&dev).await;

    let outcome = run(&dev, &linked, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!((outcome.bytes_copied, outcome.bytes_reused), (11, 0));
    assert!(outcome.assets.iter().all(|a| a.status == AssetStatus::New));
    let component = outcome.component_id.unwrap();
    assert_eq!(
        store::assets_of(linked.db(), component)
            .await
            .unwrap()
            .len(),
        2
    );
    let (source, copy) = (row(&dev, "/g/a.txt").await, row(&linked, "/g/a.txt").await);
    assert_ne!(source.blob_id, copy.blob_id);
    let (a, b) = (
        std::fs::metadata(blob_file(&world, "default", source.blob_id)).unwrap(),
        std::fs::metadata(blob_file(&world, "default", copy.blob_id)).unwrap(),
    );
    assert_eq!((a.ino(), a.nlink()), (b.ino(), 2), "the same store links");

    run(&dev, &streamed, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    let far = row(&streamed, "/g/a.txt").await;
    assert_eq!(far.sha256, source.sha256);
    assert_eq!(far.blob_store, "other");
    let metadata = std::fs::metadata(blob_file(&world, "other", far.blob_id)).unwrap();
    assert_eq!(metadata.nlink(), 1, "another store gets its own bytes");
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_streamed_blob_that_does_not_match_its_description_leaves_nothing_behind() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "other", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let bundle = bundle_of(&dev).await;
    let second = row(&dev, "/g/b.txt").await;
    std::fs::write(blob_file(&world, "default", second.blob_id), b"brave").unwrap();

    let error = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_eq!(
        blobs_in(&world, "other").await,
        0,
        "the first copy was discarded too"
    );
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_failure_midway_leaves_no_rows_no_blobs_and_no_holds() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let bundle = bundle_of(&dev).await;
    world
        .db
        .conn()
        .execute_unprepared("DELETE FROM asset WHERE repository = 'dev' AND path = '/g/b.txt'")
        .await
        .unwrap();
    let before = blobs_in(&world, "default").await;

    let error = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::NotFound(_)), "{error}");
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_eq!(blobs_in(&world, "default").await, before);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn an_unchanged_rerun_reports_reuse_and_writes_nothing() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    let first = row(&target, "/g/a.txt").await;
    let blobs = blobs_in(&world, "default").await;

    let again = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(again.status, Status::Unchanged);
    assert_eq!((again.bytes_copied, again.bytes_reused), (0, 5));
    assert_eq!(again.assets[0].status, AssetStatus::Unchanged);
    assert_eq!(row(&target, "/g/a.txt").await.blob_id, first.blob_id);
    assert_eq!(
        blobs_in(&world, "default").await,
        blobs,
        "the spare copy was discarded"
    );
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_conflict_fails_skips_or_replaces_as_asked() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let bundle = bundle_of(&dev).await;
    seed(&target, "/g/a.txt", b"older").await;
    let older = row(&target, "/g/a.txt").await;
    let blobs = blobs_in(&world, "default").await;

    let error = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    let skipped = run(&dev, &target, &bundle, Conflict::Skip, false)
        .await
        .unwrap();
    assert_eq!(skipped.status, Status::Skipped);
    assert_eq!(skipped.bytes_copied, 0);
    assert!(
        store::find(target.db(), "target", "/g/b.txt")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(row(&target, "/g/a.txt").await.blob_id, older.blob_id);
    assert_eq!(
        blobs_in(&world, "default").await,
        blobs,
        "refused copies are discarded"
    );

    let replaced = run(&dev, &target, &bundle, Conflict::Replace, false)
        .await
        .unwrap();
    assert_eq!(replaced.status, Status::Copied);
    let statuses: Vec<_> = replaced.assets.iter().map(|a| a.status).collect();
    assert_eq!(statuses, [AssetStatus::Replaced, AssetStatus::New]);
    assert_eq!(
        row(&target, "/g/a.txt").await.sha256,
        row(&dev, "/g/a.txt").await.sha256
    );
    assert_eq!(
        blobs_in(&world, "default").await,
        blobs + 1,
        "the replaced blob went, the new one came"
    );
}

#[tokio::test]
async fn allow_once_and_deny_are_never_overridden() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let once = repository(&world, "once", "default", WritePolicy::AllowOnce).await;
    let sealed = repository(&world, "sealed", "default", WritePolicy::Deny).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    seed(&once, "/g/a.txt", b"older").await;
    let older = row(&once, "/g/a.txt").await;

    let error = run(&dev, &once, &bundle, Conflict::Replace, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(row(&once, "/g/a.txt").await.blob_id, older.blob_id);
    let error = run(&dev, &sealed, &bundle, Conflict::Replace, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(target_rows(&world, "sealed").await.1, 0);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_hold_taken_by_a_racing_sweep_makes_the_transfer_fail() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let store = world.content.blob_stores().get("default").await.unwrap();
    let written = store.put(content_of("late")).await.unwrap();
    holds::hold(
        target.db(),
        &written.store,
        written.id,
        Duration::from_secs(600),
    )
    .await
    .unwrap();
    let sweep = world.db.conn().begin().await.unwrap();
    sweep
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM blob_hold WHERE blob_id = $1",
            [written.id.uuid().into()],
        ))
        .await
        .unwrap();

    let ctx = target.clone();
    let item = (
        PutAsset::hosted(&target, "/late.txt").component(key()),
        written.clone(),
    );
    let transfer = tokio::spawn(async move {
        store::record_many(&ctx, vec![item], Conflict::Fail, PROMOTED).await
    });
    wait_for_lock(&world).await;
    sweep.commit().await.unwrap();

    let error = transfer.await.unwrap().unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(target_rows(&world, "target").await.1, 0);
    assert!(
        !store.exists(written.id).await.unwrap(),
        "the blob was discarded"
    );
}

#[tokio::test]
async fn a_dry_run_reports_the_plan_and_writes_nothing() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/a.txt.sha1", b"sum").await;
    let bundle = bundle_of(&dev).await;
    let blobs = blobs_in(&world, "default").await;

    let planned = run(&dev, &target, &bundle, Conflict::Fail, true)
        .await
        .unwrap();
    assert_eq!(planned.status, Status::Copied);
    assert_eq!(
        planned.assets.len(),
        1,
        "derived files are not part of the plan"
    );
    assert_eq!(planned.bytes_copied, 5);
    assert_eq!(target_rows(&world, "target").await.1, 0);
    assert_eq!(blobs_in(&world, "default").await, blobs);
    assert_eq!(hold_rows(&world).await, 0);

    seed(&target, "/g/a.txt", b"older").await;
    let clash = run(&dev, &target, &bundle, Conflict::Fail, true)
        .await
        .unwrap();
    assert_eq!(clash.status, Status::Conflict);
    assert_eq!(clash.assets[0].status, AssetStatus::Conflict);
}

#[tokio::test]
async fn a_path_the_format_refuses_stops_the_transfer_before_any_copy() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.bad", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    let blobs = blobs_in(&world, "default").await;

    let error = run(&dev, &target, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(blobs_in(&world, "default").await, blobs);
}
