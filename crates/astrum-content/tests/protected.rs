use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_blob::content_of;
use astrum_content::store::{self, Asset, PutAsset};
use astrum_content::transfer::{Conflict, Status};
use astrum_content::{ContentError, ContentRequest, Ctx, Format};
use astrum_repo::config::{Repository, WritePolicy};
use async_trait::async_trait;
use axum::response::Response;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, Statement, TransactionTrait};

mod support;
use support::*;

const GATE: i64 = 7_700_211;
const MAGIC: usize = 4321;

struct Guarded {
    armed: bool,
}

#[async_trait]
impl Format for Guarded {
    fn id(&self) -> &'static str {
        "raw"
    }

    async fn handle(&self, _: Ctx, _: ContentRequest) -> Result<Response, ContentError> {
        Err(ContentError::not_found("not served"))
    }

    async fn get(&self, _: &Ctx, _: &str) -> Result<Option<Asset>, ContentError> {
        Ok(None)
    }

    fn can_protect(&self, asset: &Asset) -> bool {
        self.armed && asset.path.starts_with("/locked/")
    }

    fn protected(&self, repository: &Repository, asset: &Asset) -> Option<String> {
        self.can_protect(asset)
            .then(|| format!("{} keeps {}", repository.name, asset.path))
    }
}

async fn guarded(armed: bool) -> (World, Ctx) {
    let world = world().await;
    world.content.register(Arc::new(Guarded { armed }));
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    (world, target)
}

async fn put(ctx: &Ctx, path: &str, body: &'static [u8]) -> Result<Asset, ContentError> {
    store::put(ctx, PutAsset::hosted(ctx, path), content_of(body)).await
}

fn refused(result: Result<impl std::fmt::Debug, ContentError>, path: &str) {
    match result {
        Err(ContentError::Conflict(reason)) => {
            assert_eq!(reason, format!("target keeps {path}"))
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn a_protected_asset_takes_the_same_content_again_but_never_other_content() {
    let (world, target) = guarded(true).await;
    put(&target, "/locked/a", b"one").await.unwrap();
    put(&target, "/locked/a", b"one").await.unwrap();
    refused(put(&target, "/locked/a", b"two").await, "/locked/a");
    assert_eq!(row(&target, "/locked/a").await.size, 3);
    put(&target, "/free/a", b"one").await.unwrap();
    put(&target, "/free/a", b"two").await.unwrap();
    assert_eq!(unnamed_blobs(&world).await, 0);
}

#[tokio::test]
async fn a_protected_asset_and_its_component_are_not_deleted() {
    let (world, target) = guarded(true).await;
    seed(&target, "/locked/a", b"one").await;
    seed(&target, "/free/b", b"two").await;
    let locked = row(&target, "/locked/a").await;
    refused(
        store::delete_asset(&target.content, locked.clone()).await,
        "/locked/a",
    );
    refused(store::delete(&target, "/locked/a").await, "/locked/a");
    refused(
        store::delete_component(&target.content, locked.component_id.unwrap()).await,
        "/locked/a",
    );
    assert_eq!(target_rows(&world, "target").await, (1, 2));
    store::delete(&target, "/free/b").await.unwrap();
    assert_eq!(target_rows(&world, "target").await, (1, 1));
}

#[tokio::test]
async fn a_format_that_protects_nothing_deletes_and_replaces_as_before() {
    let (world, target) = guarded(false).await;
    seed(&target, "/locked/a", b"one").await;
    seed(&target, "/locked/a", b"two").await;
    assert_eq!(row(&target, "/locked/a").await.size, 3);
    let found = row(&target, "/locked/a").await;
    store::delete_component(&target.content, found.component_id.unwrap())
        .await
        .unwrap();
    assert_eq!(target_rows(&world, "target").await, (0, 0));
}

#[tokio::test]
async fn a_transfer_that_replaces_a_protected_asset_is_refused_and_leaves_nothing() {
    let (world, target) = guarded(true).await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&target, "/locked/a", b"one").await;
    seed(&dev, "/locked/a", b"other").await;
    seed(&dev, "/locked/b", b"new").await;
    let bundle = bundle_of(&dev).await;
    let before = target_rows(&world, "target").await;
    let blobs = blobs_in(&world, "default").await;
    for dry_run in [true, false] {
        let result = run(&dev, &target, &bundle, Conflict::Replace, dry_run).await;
        refused(result, "/locked/a");
    }
    assert_eq!(target_rows(&world, "target").await, before);
    assert_eq!(hold_rows(&world).await, 0);
    assert_eq!(blobs_in(&world, "default").await, blobs);
    assert_eq!(row(&target, "/locked/a").await.size, 3);
    let same = run(&dev, &target, &bundle, Conflict::Skip, false)
        .await
        .unwrap();
    assert_eq!(same.status, Status::Skipped);
}

async fn close_gate(world: &World) -> DatabaseTransaction {
    let gate = world.db.conn().begin().await.unwrap();
    gate.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({GATE})"))
        .await
        .unwrap();
    gate
}

async fn waiting_pid(world: &World, gated: bool) {
    let sql = format!(
        "SELECT DISTINCT l.pid::bigint AS n FROM pg_locks l JOIN pg_stat_activity a ON a.pid = l.pid \
         WHERE NOT l.granted AND a.datname = current_database() AND (l.locktype = 'advisory') = {gated}"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let rows = world
            .db
            .conn()
            .query_all(Statement::from_string(DbBackend::Postgres, sql.clone()))
            .await
            .unwrap();
        if !rows.is_empty() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "no backend ever waited (gated: {gated})"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn the_check_sees_what_a_concurrent_writer_committed_under_the_usage_lock() {
    let (world, target) = guarded(true).await;
    world
        .db
        .conn()
        .execute_unprepared(&format!(
            "CREATE FUNCTION protected_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN \
             PERFORM pg_advisory_xact_lock({GATE}); RETURN NEW; END $$; \
             CREATE TRIGGER protected_gate BEFORE INSERT ON asset FOR EACH ROW \
             WHEN (NEW.path = '/locked/q' AND NEW.size = {MAGIC}) EXECUTE FUNCTION protected_gate()"
        ))
        .await
        .unwrap();
    let gate = close_gate(&world).await;
    let first = {
        let target = target.clone();
        tokio::spawn(async move { put_bytes(&target, "/locked/q", vec![b'x'; MAGIC]).await })
    };
    waiting_pid(&world, true).await;
    let second = {
        let target = target.clone();
        tokio::spawn(async move { put(&target, "/locked/q", b"other").await.map(|_| ()) })
    };
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (first, second) = (first.await.unwrap(), second.await.unwrap());
    first.unwrap();
    refused(second, "/locked/q");
    assert_eq!(row(&target, "/locked/q").await.size, MAGIC as i64);
    assert_eq!(unnamed_blobs(&world).await, 0);
    world
        .db
        .conn()
        .execute_unprepared("DROP TRIGGER protected_gate ON asset; DROP FUNCTION protected_gate()")
        .await
        .unwrap();
}

async fn put_bytes(ctx: &Ctx, path: &str, body: Vec<u8>) -> Result<(), ContentError> {
    store::put(ctx, PutAsset::hosted(ctx, path), content_of(body))
        .await
        .map(|_| ())
}
