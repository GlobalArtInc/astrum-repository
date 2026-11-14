use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use astrum_blob::{Written, content_of};
use astrum_content::import::{Described, Imported};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::transfer::{
    AssetStatus, Bundle, BundleAsset, Conflict, GenericTransfer, LocalSource, MAX_COMPONENTS,
    PROMOTED, Status, Terms, Transfer,
};
use astrum_content::{ContentError, Ctx, holds};
use astrum_repo::config::WritePolicy;
use futures_util::future::BoxFuture;
use sea_orm::{ConnectionTrait, TransactionTrait};

mod support;
use support::*;

type Item = (PutAsset, Written);

async fn staged(target: &Ctx, path: &str, version: &str, body: &'static str) -> Item {
    let component = ComponentKey {
        version: version.to_owned(),
        ..key()
    };
    staged_in(target, path, component, body).await
}

async fn staged_in(target: &Ctx, path: &str, component: ComponentKey, body: &'static str) -> Item {
    let store = target.content.blob_stores().get("default").await.unwrap();
    let written = store.put(content_of(body)).await.unwrap();
    holds::hold(
        target.db(),
        &written.store,
        written.id,
        Duration::from_secs(600),
    )
    .await
    .unwrap();
    (PutAsset::hosted(target, path).component(component), written)
}

fn describe_split(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(async move {
        let version = file.path.split('/').nth(2).unwrap_or_default().to_owned();
        let mut put = file.put(file.path.clone());
        put.component = Some(ComponentKey { version, ..key() });
        Ok(Some(Described::source(put)))
    })
}

fn only_a(_: &Ctx, path: &str, _: &BundleAsset) -> bool {
    path.ends_with("/a.txt")
}

static DESCRIBED: AtomicUsize = AtomicUsize::new(0);

fn describe_counting(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    DESCRIBED.fetch_add(1, Ordering::SeqCst);
    describe_split(file)
}

static DENIED_CALLS: AtomicUsize = AtomicUsize::new(0);

fn counting(_: &Ctx, _: &str, _: &BundleAsset) -> bool {
    DENIED_CALLS.fetch_add(1, Ordering::SeqCst);
    true
}

fn with_hook(hook: astrum_content::transfer::ReplaceFn) -> GenericTransfer {
    GenericTransfer {
        replaceable: Some(hook),
        carried: &[],
        bundle_skip: None,
        ..transfer()
    }
}

async fn receive(
    transfer: &GenericTransfer,
    source: &Ctx,
    target: &Ctx,
    bundle: &Bundle,
    conflict: Conflict,
) -> Result<astrum_content::transfer::Outcome, ContentError> {
    transfer
        .receive(
            target,
            bundle,
            &LocalSource::new(source.clone()),
            Terms::new(conflict, PROMOTED),
        )
        .await
}

#[tokio::test]
async fn assets_of_two_components_arrive_together_or_not_at_all() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/2/b.txt", b"bravo").await;
    seed(&dev, "/g/1/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    let outcome = receive(
        &transfer_of(describe_split),
        &dev,
        &target,
        &bundle,
        Conflict::Fail,
    )
    .await
    .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.component_ids.len(), 2);
    assert_eq!(outcome.component_id, outcome.component_ids.first().copied());
    let first = store::component_by_id(target.db(), outcome.component_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.version, "1");
    assert_eq!(target_rows(&world, "target").await, (2, 2));
    assert_eq!(hold_rows(&world).await, 0);

    let busy = repository(&world, "busy", "default", WritePolicy::Allow).await;
    let taken = staged(&busy, "/g/2/taken.txt", "9", "older").await;
    store::record_many(&busy, vec![taken], Conflict::Fail, PROMOTED)
        .await
        .unwrap();
    let before = blobs_in(&world, "default").await;
    let items = vec![
        staged(&busy, "/g/1/a.txt", "1", "alpha").await,
        staged(&busy, "/g/2/taken.txt", "2", "newer").await,
    ];
    let error = store::record_many(&busy, items, Conflict::Fail, PROMOTED)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(target_rows(&world, "busy").await, (1, 1));
    assert_eq!(blobs_in(&world, "default").await, before);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transfers_creating_the_same_components_in_opposite_orders_never_deadlock() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let rounds = 50;
    let mut assets = 0;
    for round in 0..rounds {
        let (x, y) = (format!("x{round}"), format!("y{round}"));
        let conflicting = round % 2 == 1;
        let shared = format!("/r{round}/a1");
        let second = match conflicting {
            true => shared.clone(),
            false => format!("/r{round}/b2"),
        };
        let first = vec![
            staged(&target, &shared, &x, "first").await,
            staged(&target, &format!("/r{round}/a2"), &y, "first").await,
        ];
        let opposite = vec![
            staged(&target, &format!("/r{round}/b1"), &y, "second").await,
            staged(&target, &second, &x, "second").await,
        ];
        let before = blobs_in(&world, "default").await;
        let (a, b) = (target.clone(), target.clone());
        let (a, b) = tokio::join!(
            tokio::spawn(
                async move { store::record_many(&a, first, Conflict::Fail, PROMOTED).await }
            ),
            tokio::spawn(async move {
                store::record_many(&b, opposite, Conflict::Fail, PROMOTED).await
            }),
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        match conflicting {
            false => {
                assert!(a.is_ok() && b.is_ok(), "round {round}: {a:?} and {b:?}");
                assert_eq!(blobs_in(&world, "default").await, before);
                assets += 4;
            }
            true => {
                let (won, lost) = match (a.is_ok(), b.is_ok()) {
                    (true, false) => (a, b),
                    (false, true) => (b, a),
                    _ => panic!("round {round}: {a:?} and {b:?}"),
                };
                assert_eq!(won.unwrap().component_ids.len(), 2);
                let error = lost.unwrap_err();
                assert!(matches!(error, ContentError::Conflict(_)), "{error}");
                assert_eq!(blobs_in(&world, "default").await, before - 2);
                assets += 2;
            }
        }
    }
    assert_eq!(target_rows(&world, "target").await, (2 * rounds, assets));
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(unnamed_blobs(&world).await, 0);
}

#[tokio::test]
async fn more_components_than_the_limit_are_refused_before_anything_is_written() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let baseline = blobs_in(&world, "default").await;
    let mut items = Vec::new();
    for index in 0..=MAX_COMPONENTS {
        items.push(staged(&target, &format!("/c/{index}"), &index.to_string(), "body").await);
    }
    let error = store::record_many(&target, items, Conflict::Fail, PROMOTED)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(blobs_in(&world, "default").await, baseline);

    let mut limit = Vec::new();
    for index in 0..MAX_COMPONENTS {
        limit.push(staged(&target, &format!("/c/{index}"), &index.to_string(), "body").await);
    }
    store::record_many(&target, limit, Conflict::Fail, PROMOTED)
        .await
        .unwrap();
    assert_eq!(
        target_rows(&world, "target").await,
        (MAX_COMPONENTS as i64, MAX_COMPONENTS as i64)
    );
}

#[tokio::test]
async fn the_component_limit_stops_the_transfer_at_the_first_asset_over_it() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    for index in 0..MAX_COMPONENTS + 2 {
        store::put(
            &dev,
            PutAsset::hosted(&dev, format!("/g/{index}/a.txt")).component(key()),
            content_of(index.to_string().into_bytes()),
        )
        .await
        .unwrap();
    }
    let bundle = bundle_of(&dev).await;
    assert_eq!(bundle.assets.len(), MAX_COMPONENTS + 2);
    let baseline = blobs_in(&world, "default").await;
    let error = receive(
        &transfer_of(describe_counting),
        &dev,
        &target,
        &bundle,
        Conflict::Fail,
    )
    .await
    .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(DESCRIBED.load(Ordering::SeqCst), MAX_COMPONENTS + 1);
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_eq!(blobs_in(&world, "default").await, baseline);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn a_format_may_allow_replacing_a_path_under_allow_once_and_only_then() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let once = repository(&world, "once", "default", WritePolicy::AllowOnce).await;
    let both = repository(&world, "both", "default", WritePolicy::AllowOnce).await;
    let sealed = repository(&world, "sealed", "default", WritePolicy::Deny).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let bundle = bundle_of(&dev).await;
    seed(&once, "/g/a.txt", b"older").await;
    seed(&both, "/g/a.txt", b"older").await;
    seed(&both, "/g/b.txt", b"older").await;
    let hooked = with_hook(only_a);

    let error = receive(&hooked, &dev, &once, &bundle, Conflict::Fail)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    let error = receive(&transfer(), &dev, &once, &bundle, Conflict::Replace)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(target_rows(&world, "once").await.1, 1);

    let outcome = receive(&hooked, &dev, &once, &bundle, Conflict::Replace)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    let status_of = |path: &str| {
        outcome
            .assets
            .iter()
            .find(|asset| asset.path == path)
            .map(|asset| asset.status)
    };
    assert_eq!(status_of("/g/a.txt"), Some(AssetStatus::Replaced));
    assert_eq!(status_of("/g/b.txt"), Some(AssetStatus::New));
    assert_eq!(
        row(&once, "/g/a.txt").await.sha256,
        row(&dev, "/g/a.txt").await.sha256
    );

    let older = row(&both, "/g/b.txt").await;
    let error = receive(&hooked, &dev, &both, &bundle, Conflict::Replace)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(row(&both, "/g/b.txt").await.blob_id, older.blob_id);
    assert_ne!(
        row(&both, "/g/a.txt").await.sha256,
        row(&dev, "/g/a.txt").await.sha256
    );

    let counting = with_hook(counting);
    let error = receive(&counting, &dev, &sealed, &bundle, Conflict::Replace)
        .await
        .unwrap_err();
    assert!(matches!(error, ContentError::BadRequest(_)), "{error}");
    assert_eq!(DENIED_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(hold_rows(&world).await, 0);
}

async fn wait_for_sleep(world: &World) {
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while number(
        world,
        "SELECT count(*) AS n FROM pg_stat_activity WHERE wait_event = 'PgSleep' AND datname = current_database()",
    )
    .await
        == 0
    {
        assert!(std::time::Instant::now() < deadline, "nothing ever slept");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_creation_waiting_on_a_key_never_closes_a_cycle_with_the_id_ordered_lock() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let conn = world.db.conn();
    conn.execute_unprepared("CREATE SEQUENCE slow_once")
        .await
        .unwrap();
    conn.execute_unprepared(
        "CREATE FUNCTION slow_second() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.version = '2' AND nextval('slow_once') = 1 THEN PERFORM pg_sleep(2); END IF; RETURN NEW; END $$",
    )
    .await
    .unwrap();
    conn.execute_unprepared(
        "CREATE TRIGGER slow_second BEFORE INSERT ON component FOR EACH ROW EXECUTE FUNCTION slow_second()",
    )
    .await
    .unwrap();
    let outsider = conn.begin().await.unwrap();
    outsider
        .execute_unprepared("INSERT INTO component (repository, format, namespace, name, version, kind, attributes, created_at, updated_at) VALUES ('target', 'raw', 'g', 'lib', '1', '', '{}', now(), now())")
        .await
        .unwrap();
    let first = vec![
        staged(&target, "/x/1", "1", "x1").await,
        staged(&target, "/x/2", "2", "x2").await,
    ];
    let second = vec![
        staged(&target, "/y/1", "1", "y1").await,
        staged(&target, "/y/2", "2", "y2").await,
    ];
    let a = target.clone();
    let first =
        tokio::spawn(async move { store::record_many(&a, first, Conflict::Fail, PROMOTED).await });
    wait_for_lock(&world).await;
    outsider.commit().await.unwrap();
    wait_for_sleep(&world).await;
    let b = target.clone();
    let second =
        tokio::spawn(async move { store::record_many(&b, second, Conflict::Fail, PROMOTED).await });
    let (first, second) = (first.await.unwrap(), second.await.unwrap());
    assert!(first.is_ok() && second.is_ok(), "{first:?} and {second:?}");
    assert_eq!(target_rows(&world, "target").await, (2, 4));
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(unnamed_blobs(&world).await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_component_deleted_while_the_transfer_waits_for_its_lock_is_a_conflict() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&target, "/c/a1", b"seed").await;
    let component = number(
        &world,
        "SELECT id AS n FROM component WHERE repository = 'target'",
    )
    .await;
    let outsider = world.db.conn().begin().await.unwrap();
    outsider
        .execute_unprepared(&format!(
            "SELECT id FROM component WHERE id = {component} FOR UPDATE"
        ))
        .await
        .unwrap();
    let item = staged(&target, "/c/a2", "1", "newer").await;
    let before = blobs_in(&world, "default").await;
    let t = target.clone();
    let handle =
        tokio::spawn(
            async move { store::record_many(&t, vec![item], Conflict::Fail, PROMOTED).await },
        );
    wait_for_lock(&world).await;
    outsider
        .execute_unprepared(&format!(
            "DELETE FROM asset WHERE component_id = {component}; DELETE FROM component WHERE id = {component}"
        ))
        .await
        .unwrap();
    outsider.commit().await.unwrap();
    let error = handle.await.unwrap().unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_eq!(blobs_in(&world, "default").await, before - 1);
    assert_eq!(hold_rows(&world).await, 0);
}

#[tokio::test]
async fn components_whose_assets_are_all_unchanged_are_neither_created_nor_touched() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    world
        .db
        .conn()
        .execute_unprepared("INSERT INTO notification_channel (id, name, type) VALUES (gen_random_uuid(), 'watch', 'webhook')")
        .await
        .unwrap();
    let named = |name: &str| ComponentKey {
        name: name.to_owned(),
        ..key()
    };
    let original = ComponentKey {
        kind: "original".to_owned(),
        attributes: serde_json::json!({"a": 1}),
        ..named("known")
    };
    for (path, component, body) in [
        ("/p/1", named("holder"), "same"),
        ("/k/1", original.clone(), "kept"),
    ] {
        let (put, written) = staged_in(&target, path, component, body).await;
        store::record_many(&target, vec![(put, written)], Conflict::Fail, PROMOTED)
            .await
            .unwrap();
    }
    let stamp = "SELECT (extract(epoch from updated_at) * 1000000)::bigint AS n FROM component WHERE name = 'known'";
    let before = number(&world, stamp).await;
    let notices = number(
        &world,
        "SELECT count(*) AS n FROM notification_change WHERE name = 'known'",
    )
    .await;
    let changed = ComponentKey {
        kind: "changed".to_owned(),
        attributes: serde_json::json!({"b": 2}),
        ..named("known")
    };
    let items = vec![
        staged_in(&target, "/p/1", named("phantom"), "same").await,
        staged_in(&target, "/p/2", named("fresh"), "fresh").await,
        staged_in(&target, "/k/1", changed, "kept").await,
    ];
    let outcome = store::record_many(&target, items, Conflict::Fail, PROMOTED)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(outcome.component_ids.len(), 2);
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM component WHERE name = 'phantom'"
        )
        .await,
        0
    );
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM notification_change WHERE name = 'phantom'"
        )
        .await,
        0
    );
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM notification_change WHERE name = 'known'"
        )
        .await,
        notices
    );
    assert_eq!(number(&world, stamp).await, before);
    assert_eq!(
        number(&world, "SELECT count(*) AS n FROM component WHERE name = 'known' AND kind = 'original' AND attributes = '{\"a\": 1}'::jsonb").await,
        1
    );
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(unnamed_blobs(&world).await, 0);
}
