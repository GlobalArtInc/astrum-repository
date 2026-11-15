use std::time::Duration;

use astrum_blob::{Written, content_of};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::transfer::{Bundle, Conflict, LocalSource, PROMOTED, Status, Terms, Transfer};
use astrum_content::{ContentError, Ctx, holds};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use sea_orm::{ConnectionTrait, TransactionTrait};

mod support;
use support::*;

type Forge = fn(&mut Bundle);

async fn staged(
    target: &Ctx,
    path: &str,
    version: &str,
    body: &'static str,
) -> (PutAsset, Written) {
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
    let component = ComponentKey {
        version: version.to_owned(),
        ..key()
    };
    (PutAsset::hosted(target, path).component(component), written)
}

async fn race_rounds(world: &World, name: &str, policy: WritePolicy, one_component: bool) {
    let target = repository(world, name, "default", policy).await;
    for round in 0..30 {
        let version = round.to_string();
        let other = match one_component {
            true => version.clone(),
            false => format!("{round}b"),
        };
        let path = format!("/g/{version}/a.txt");
        let first = staged(&target, &path, &version, "first").await;
        let second = staged(&target, &path, &other, "second").await;
        let before = blobs_in(world, "default").await;
        let (a, b) = (target.clone(), target.clone());
        let (a, b) = tokio::join!(
            tokio::spawn(async move {
                store::record_many(&a, vec![first], Conflict::Fail, PROMOTED).await
            }),
            tokio::spawn(async move {
                store::record_many(&b, vec![second], Conflict::Fail, PROMOTED).await
            }),
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        let (won, lost) = match (a.is_ok(), b.is_ok()) {
            (true, false) => (a, b),
            (false, true) => (b, a),
            _ => panic!("{name} round {round}: {a:?} and {b:?}"),
        };
        assert_eq!(won.unwrap().status, Status::Copied);
        let error = lost.unwrap_err();
        assert!(matches!(error, ContentError::Conflict(_)), "{error}");
        assert_eq!(blobs_in(world, "default").await, before - 1);
    }
    assert_eq!(hold_rows(world).await, 0);
    assert_eq!(settled_unnamed_blobs(world).await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_transfers_of_one_new_component_and_path_let_exactly_one_through() {
    let world = world().await;
    race_rounds(&world, "open", WritePolicy::Allow, true).await;
    race_rounds(&world, "once", WritePolicy::AllowOnce, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_transfers_of_one_new_path_into_different_components_let_exactly_one_through() {
    let world = world().await;
    race_rounds(&world, "open", WritePolicy::Allow, false).await;
    race_rounds(&world, "once", WritePolicy::AllowOnce, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rewritten_and_extra_blobs_stay_held_until_the_commit_and_leave_no_orphans() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    let blocker = world.db.conn().begin().await.unwrap();
    blocker
        .execute_unprepared(
            "INSERT INTO component (repository, format, namespace, name, version, kind, attributes, created_at, updated_at) \
             VALUES ('target', 'raw', 'g', 'lib', '1', '', '{}', now(), now())",
        )
        .await
        .unwrap();
    let (source, copy) = (dev.clone(), target.clone());
    let running = tokio::spawn(async move {
        transfer_of(describe_rewritten)
            .receive(
                &copy,
                &bundle,
                &LocalSource::new(source),
                Terms::new(Conflict::Fail, PROMOTED),
            )
            .await
    });
    wait_for_lock(&world).await;
    assert_eq!(hold_rows(&world).await, 2, "the fresh blobs are held");
    assert_eq!(unnamed_blobs(&world).await, 2, "the original is gone");
    blocker.rollback().await.unwrap();

    let outcome = running.await.unwrap().unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(target_rows(&world, "target").await, (1, 2));
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(settled_unnamed_blobs(&world).await, 0);
    assert_eq!(blobs_in(&world, "default").await, 3);
}

#[tokio::test]
async fn a_forged_bundle_is_refused_before_anything_is_copied() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    let group = repository_of(
        &world,
        "group",
        "default",
        WritePolicy::Allow,
        RepositoryType::Group,
    )
    .await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    let bundle = bundle_of(&dev).await;
    let blobs = blobs_in(&world, "default").await;
    let forgeries: [(&str, Forge); 5] = [
        ("dot dot", |b| b.assets[0].path = "/g/../a.txt".to_owned()),
        ("twice", |b| b.assets.push(b.assets[0].clone())),
        ("digest", |b| b.assets[0].sha256 = "z".repeat(64)),
        ("format", |b| b.format = "maven2".to_owned()),
        ("slash", |b| b.assets[0].path = "/g/a.txt/".to_owned()),
    ];
    for (name, forge) in forgeries {
        let mut forged = bundle.clone();
        forge(&mut forged);
        let error = run(&dev, &target, &forged, Conflict::Fail, false)
            .await
            .unwrap_err();
        assert!(
            matches!(error, ContentError::BadRequest(_)),
            "{name}: {error}"
        );
    }
    let error = run(&dev, &group, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    assert!(
        matches!(error, ContentError::BadRequest(_)),
        "group: {error}"
    );
    assert_eq!(blobs_in(&world, "default").await, blobs);
    assert_eq!(hold_rows(&world).await, 0);
}
