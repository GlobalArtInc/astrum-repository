use std::os::unix::fs::MetadataExt;
use std::time::Duration;

use astrum_blob::content_of;
use astrum_content::store::{self, PutAsset};
use astrum_content::transfer::{
    AssetStatus, Conflict, LocalSource, PROMOTED, Status, Terms, Transfer,
};
use astrum_content::{ContentError, holds};
use astrum_repo::config::WritePolicy;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};

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

fn terms(conflict: Conflict) -> Terms {
    Terms::new(conflict, PROMOTED)
}

#[tokio::test]
async fn an_asset_that_keeps_its_component_is_recorded_beside_its_companion() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/lib.1.nupkg", b"package").await;
    seed(&dev, "/g/lib.1.snupkg", b"symbols").await;
    let bundle = bundle_of(&dev).await;

    let outcome = transfer_of(describe_companion)
        .receive(
            &target,
            &bundle,
            &LocalSource::new(dev.clone()),
            terms(Conflict::Fail),
        )
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.component_ids.len(), 1);
    let assets = store::assets_of(target.db(), outcome.component_id.unwrap())
        .await
        .unwrap();
    let mut paths: Vec<&str> = assets.iter().map(|asset| asset.path.as_str()).collect();
    paths.sort_unstable();
    assert_eq!(paths, ["/g/lib.1.nupkg", "/g/lib.1.snupkg"]);
    assert_eq!(target_rows(&world, "target").await, (1, 2));
}

#[tokio::test]
async fn the_after_hook_gets_what_was_copied_or_replaced_and_a_failure_becomes_a_warning() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/1/a.txt", b"alpha").await;
    seed(&dev, "/g/2/b.txt", b"bravo").await;
    let bundle = bundle_of(&dev).await;
    let format = Recording::new(describe_by_version, None);

    let first = receive_with(&format, &dev, &target, &bundle, Conflict::Fail)
        .await
        .unwrap();
    assert!(first.warnings.is_empty());
    let json = serde_json::to_value(&first).unwrap();
    assert!(!json.as_object().unwrap().contains_key("warnings"));
    let calls = format.calls();
    assert_eq!(calls.len(), 1);
    let mut written: Vec<(&str, Vec<&str>)> = calls[0]
        .components
        .iter()
        .map(|found| {
            let paths = found.paths.iter().map(String::as_str).collect();
            (found.version.as_str(), paths)
        })
        .collect();
    written.sort_unstable();
    assert_eq!(
        written,
        [("1", vec!["/g/1/a.txt"]), ("2", vec!["/g/2/b.txt"])]
    );

    let again = receive_with(&format, &dev, &target, &bundle, Conflict::Fail)
        .await
        .unwrap();
    assert_eq!(again.status, Status::Unchanged);
    assert_eq!(format.calls().len(), 1, "nothing was written");

    seed(&dev, "/g/2/b.txt", b"bravo, again").await;
    let changed = bundle_of(&dev).await;
    let skipped = receive_with(&format, &dev, &target, &changed, Conflict::Skip)
        .await
        .unwrap();
    assert_eq!(skipped.status, Status::Skipped);
    assert_eq!(
        format.calls().len(),
        1,
        "a skipped transfer promotes nothing"
    );

    receive_with(&format, &dev, &target, &changed, Conflict::Replace)
        .await
        .unwrap();
    let calls = format.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[1].components.len(),
        1,
        "the unchanged component is left out"
    );
    assert_eq!(calls[1].components[0].version, "2");
    assert_eq!(calls[1].components[0].paths, ["/g/2/b.txt"]);

    let failing = Recording::new(describe_by_version, Some("the index is locked"));
    let other = repository(&world, "other", "default", WritePolicy::Allow).await;
    let outcome = receive_with(&failing, &dev, &other, &changed, Conflict::Fail)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.warnings.len(), 1);
    assert!(outcome.warnings[0].contains("the index is locked"));
    assert_eq!(failing.calls().len(), 1);
    assert_eq!(target_rows(&world, "other").await, (2, 2));
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_conflict_the_policy_refuses_calls_no_hook() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let once = repository(&world, "once", "default", WritePolicy::AllowOnce).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    seed(&once, "/g/a.txt", b"older").await;
    let format = Recording::new(describe, None);

    for conflict in [Conflict::Fail, Conflict::Replace] {
        let error = receive_with(&format, &dev, &once, &bundle, conflict)
            .await
            .unwrap_err();
        assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    }
    assert!(format.calls().is_empty());
}

#[tokio::test]
async fn a_hook_in_the_old_function_form_still_runs() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use futures_util::future::BoxFuture;

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    fn count(_: &astrum_content::Ctx) -> BoxFuture<'_, Result<(), ContentError>> {
        Box::pin(async {
            CALLS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }

    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    transfer()
        .after_fn(count)
        .receive(
            &target,
            &bundle,
            &LocalSource::new(dev.clone()),
            terms(Conflict::Fail),
        )
        .await
        .unwrap();
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn the_description_may_switch_the_content_check_off_but_never_loosen_the_policy() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/picture.png", b"this is plain text").await;
    let bundle = bundle_of(&dev).await;
    let strict = strict_repository(&world, "strict").await;
    let source = LocalSource::new(dev.clone());

    let error = transfer_of(describe)
        .receive(&strict, &bundle, &source, terms(Conflict::Fail))
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(target_rows(&world, "strict").await, (0, 0));
    transfer_of(describe_unchecked)
        .receive(&strict, &bundle, &source, terms(Conflict::Fail))
        .await
        .unwrap();
    assert_eq!(target_rows(&world, "strict").await, (1, 1));

    let once = repository(&world, "once", "default", WritePolicy::AllowOnce).await;
    seed(&once, "/g/picture.png", b"older").await;
    let error = transfer_of(describe_loosening)
        .receive(&once, &bundle, &source, terms(Conflict::Replace))
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
}

fn nested(containers: usize) -> Value {
    let mut value = json!(1);
    for _ in 0..containers {
        value = json!({"k": value});
    }
    value
}

#[tokio::test]
async fn declared_attributes_travel_in_the_bundle_and_nothing_else_does() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let attributes = json!({"note": {"a": [1, 2]}, "quota": 5, "replication": {"origin": "x"}});
    seed_in(&dev, "/g/a.txt", b"alpha", attributes).await;
    let sender = declaring(&["note", "replication"]);
    let bundle = bundle_with(&dev, &sender).await;
    assert_eq!(bundle.meta, json!({"note": {"a": [1, 2]}}));

    let mut forged = bundle.clone();
    forged.meta = json!({"note": {"a": 1}, "quota": 9, "replication": {"origin": "evil"}});
    let receiver = Recording::declaring(&["note"]);
    receive_with(&receiver, &dev, &target, &forged, Conflict::Fail)
        .await
        .unwrap();
    assert_eq!(
        component_attributes(&target).await,
        json!({"note": {"a": 1}})
    );

    let plain = repository(&world, "plain", "default", WritePolicy::Allow).await;
    run(&dev, &plain, &forged, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(component_attributes(&plain).await, json!({}));
}

#[tokio::test]
async fn meta_that_is_too_large_too_deep_or_misshapen_is_refused_before_anything_is_recorded() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    let receiver = Recording::declaring(&["note"]);
    let refused = [
        json!({"note": "x".repeat(70_000)}),
        json!({"note": nested(8)}),
        json!({"note": [1]}),
        json!([1]),
    ];
    for meta in refused {
        let mut crafted = bundle.clone();
        crafted.meta = meta;
        let error = receive_with(&receiver, &dev, &target, &crafted, Conflict::Fail)
            .await
            .unwrap_err();
        assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    }
    assert_eq!(target_rows(&world, "target").await, (0, 0));

    let mut edge = bundle.clone();
    edge.meta = json!({"note": nested(7)});
    receive_with(&receiver, &dev, &target, &edge, Conflict::Fail)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_changed_attribute_is_applied_in_place_without_writing_a_blob() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed_in(&dev, "/g/a.txt", b"alpha", json!({"note": {"v": 2}})).await;
    let bundle = bundle_with(&dev, &declaring(&["note"])).await;

    for (name, policy) in [
        ("open", WritePolicy::Allow),
        ("once", WritePolicy::AllowOnce),
    ] {
        let target = repository(&world, name, "default", policy).await;
        seed_in(
            &target,
            "/g/a.txt",
            b"alpha",
            json!({"note": {"v": 1}, "kept": true}),
        )
        .await;
        let before = blobs_in(&world, "default").await;
        let receiver = Recording::declaring(&["note"]);

        let preview = run_with(&receiver, &dev, &target, &bundle, true).await;
        assert_eq!(preview.status, Status::Copied);
        assert_eq!(component_attributes(&target).await["note"], json!({"v": 1}));
        assert!(receiver.calls().is_empty());

        let outcome = receive_with(&receiver, &dev, &target, &bundle, Conflict::Fail)
            .await
            .unwrap();
        assert_eq!(outcome.status, Status::Copied);
        assert!(
            outcome
                .assets
                .iter()
                .all(|a| a.status == AssetStatus::Unchanged)
        );
        assert_eq!(outcome.bytes_copied, 0);
        assert_eq!(
            component_attributes(&target).await,
            json!({"note": {"v": 2}, "kept": true})
        );
        assert_eq!(blobs_in(&world, "default").await, before);
        let calls = receiver.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].components.len(), 1);
        assert!(calls[0].components[0].paths.is_empty());

        let again = receive_with(&receiver, &dev, &target, &bundle, Conflict::Fail)
            .await
            .unwrap();
        assert_eq!(again.status, Status::Unchanged);
        assert_eq!(receiver.calls().len(), 1);
        assert_eq!(hold_rows(&world).await, 0);
    }
}

#[tokio::test]
async fn a_read_only_target_refuses_an_attribute_only_change() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "locked", "default", WritePolicy::Deny).await;
    seed_in(&dev, "/g/a.txt", b"alpha", json!({"note": {"v": 2}})).await;
    seed_in(&target, "/g/a.txt", b"alpha", json!({"note": {"v": 1}})).await;
    let bundle = bundle_with(&dev, &declaring(&["note"])).await;
    let receiver = Recording::declaring(&["note"]);

    let error = receive_with(&receiver, &dev, &target, &bundle, Conflict::Fail)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(component_attributes(&target).await["note"], json!({"v": 1}));
    assert!(receiver.calls().is_empty());
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn the_component_takes_its_attributes_from_the_primary_asset_whatever_the_bundle_order() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/lib.1.nupkg", b"package").await;
    seed(&dev, "/g/lib.1.snupkg", b"symbols").await;
    let mut bundle = bundle_of(&dev).await;
    bundle.assets.sort_by(|a, b| b.path.cmp(&a.path));
    assert_eq!(bundle.assets[0].path, "/g/lib.1.snupkg");

    transfer_of(describe_marking)
        .receive(
            &target,
            &bundle,
            &LocalSource::new(dev.clone()),
            terms(Conflict::Fail),
        )
        .await
        .unwrap();
    assert_eq!(
        component_attributes(&target).await,
        json!({"first": "/g/lib.1.nupkg"})
    );
}
