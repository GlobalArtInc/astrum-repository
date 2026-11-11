use std::time::{Duration, Instant};

use astrum_blob::{Written, content_of};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::transfer::{Conflict, PROMOTED};
use astrum_content::{ContentError, Ctx, holds};
use astrum_repo::config::WritePolicy;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, Statement, TransactionTrait};
use tokio::task::JoinHandle;

mod support;
use support::*;

const GATE: i64 = 7_700_113;
const MAGIC: usize = 1234;

type Item = (PutAsset, Written);
type Run = JoinHandle<Result<(), ContentError>>;

fn component(version: &str) -> ComponentKey {
    ComponentKey {
        version: version.to_owned(),
        ..key()
    }
}

async fn staged(target: &Ctx, path: &str, version: &str, body: Vec<u8>) -> Item {
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
    (
        PutAsset::hosted(target, path).component(component(version)),
        written,
    )
}

async fn install_gate(world: &World, definition: &str) {
    world
        .db
        .conn()
        .execute_unprepared(&format!(
            "CREATE FUNCTION usage_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN \
             PERFORM pg_advisory_xact_lock({GATE}); \
             IF TG_OP = 'DELETE' THEN RETURN OLD; END IF; RETURN NEW; END $$; \
             CREATE TRIGGER usage_gate {definition} EXECUTE FUNCTION usage_gate()"
        ))
        .await
        .unwrap();
}

async fn remove_gate(world: &World, table: &str) {
    world
        .db
        .conn()
        .execute_unprepared(&format!(
            "DROP TRIGGER usage_gate ON {table}; DROP FUNCTION usage_gate()"
        ))
        .await
        .unwrap();
}

async fn close_gate(world: &World) -> DatabaseTransaction {
    let gate = world.db.conn().begin().await.unwrap();
    gate.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({GATE})"))
        .await
        .unwrap();
    gate
}

async fn waiting_pid(world: &World, gated: bool) -> i64 {
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
        if let Some(row) = rows.first() {
            return row.try_get::<i64>("", "n").unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "no backend ever waited (gated: {gated})"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn query_of(world: &World, pid: i64) -> String {
    world
        .db
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT query AS q FROM pg_stat_activity WHERE pid = {pid}"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "q")
        .unwrap()
}

fn transfer(target: &Ctx, items: Vec<Item>, conflict: Conflict) -> Run {
    let target = target.clone();
    tokio::spawn(async move {
        store::record_many(&target, items, conflict, PROMOTED)
            .await
            .map(|_| ())
    })
}

fn put(target: &Ctx, path: &str, version: Option<&str>, body: &'static [u8]) -> Run {
    let target = target.clone();
    let path = path.to_owned();
    let version = version.map(str::to_owned);
    tokio::spawn(async move {
        let mut asset = PutAsset::hosted(&target, path);
        if let Some(version) = version {
            asset = asset.component(component(&version));
        }
        store::put(&target, asset, content_of(body))
            .await
            .map(|_| ())
    })
}

async fn finish(run: Run) -> Result<(), ContentError> {
    tokio::time::timeout(Duration::from_secs(30), run)
        .await
        .expect("a writer never finished")
        .unwrap()
}

fn assert_no_deadlock(results: &[&Result<(), ContentError>]) {
    for result in results {
        let text = format!("{result:?}");
        assert!(!text.contains("deadlock"), "{text}");
    }
}

async fn assert_usage_agrees(world: &World, repository: &str) {
    let counted = number(
        world,
        &format!("SELECT assets AS n FROM repository_usage WHERE repository = '{repository}'"),
    )
    .await;
    let bytes = number(
        world,
        &format!("SELECT bytes AS n FROM repository_usage WHERE repository = '{repository}'"),
    )
    .await;
    let rows = number(
        world,
        &format!("SELECT count(*) AS n FROM asset WHERE repository = '{repository}'"),
    )
    .await;
    let sum = number(
        world,
        &format!(
            "SELECT coalesce(sum(size), 0)::bigint AS n FROM asset WHERE repository = '{repository}'"
        ),
    )
    .await;
    assert_eq!((counted, bytes), (rows, sum));
}

#[tokio::test]
async fn a_put_and_a_transfer_that_writes_the_same_path_later_do_not_deadlock() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    install_gate(
        &world,
        &format!(
            "BEFORE INSERT ON asset FOR EACH ROW WHEN (NEW.path = '/g/p2' AND NEW.size = {MAGIC})"
        ),
    )
    .await;
    let items = vec![
        staged(&target, "/g/p1", "1", b"one".to_vec()).await,
        staged(&target, "/g/p2", "1", vec![b'x'; MAGIC]).await,
    ];
    let gate = close_gate(&world).await;
    let first = transfer(&target, items, Conflict::Fail);
    waiting_pid(&world, true).await;
    let second = put(&target, "/g/p2", None, b"two");
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (first, second) = (finish(first).await, finish(second).await);
    assert_no_deadlock(&[&first, &second]);
    first.unwrap();
    assert!(
        matches!(second, Ok(()) | Err(ContentError::Conflict(_))),
        "{second:?}"
    );
    assert_usage_agrees(&world, "target").await;
    remove_gate(&world, "asset").await;
}

#[tokio::test]
async fn two_transfers_that_overlap_on_a_later_path_do_not_deadlock() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    install_gate(
        &world,
        &format!(
            "BEFORE INSERT ON asset FOR EACH ROW WHEN (NEW.path = '/g/p2' AND NEW.size = {MAGIC})"
        ),
    )
    .await;
    let first = vec![
        staged(&target, "/g/p1", "1", b"one".to_vec()).await,
        staged(&target, "/g/p2", "1", vec![b'x'; MAGIC]).await,
    ];
    let second = vec![
        staged(&target, "/g/p2", "2", b"two".to_vec()).await,
        staged(&target, "/g/p3", "2", b"three".to_vec()).await,
    ];
    let gate = close_gate(&world).await;
    let first = transfer(&target, first, Conflict::Fail);
    waiting_pid(&world, true).await;
    let second = transfer(&target, second, Conflict::Replace);
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (first, second) = (finish(first).await, finish(second).await);
    assert_no_deadlock(&[&first, &second]);
    first.unwrap();
    assert!(
        matches!(second, Ok(()) | Err(ContentError::Conflict(_))),
        "{second:?}"
    );
    assert_usage_agrees(&world, "target").await;
    remove_gate(&world, "asset").await;
}

#[tokio::test]
async fn records_that_move_assets_between_two_components_in_opposite_directions_do_not_deadlock() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    for (path, version) in [("/a1", "a"), ("/b1", "b"), ("/x", "a"), ("/y", "b")] {
        store::put(
            &target,
            PutAsset::hosted(&target, path).component(component(version)),
            content_of(path.as_bytes().to_vec()),
        )
        .await
        .unwrap();
    }
    install_gate(
        &world,
        "AFTER UPDATE ON component FOR EACH ROW WHEN (NEW.version = 'b')",
    )
    .await;
    let gate = close_gate(&world).await;
    let first = put(&target, "/x", Some("b"), b"moved to b");
    waiting_pid(&world, true).await;
    let second = put(&target, "/y", Some("a"), b"moved to a");
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (first, second) = (finish(first).await, finish(second).await);
    assert_no_deadlock(&[&first, &second]);
    first.unwrap();
    second.unwrap();
    assert_eq!(target_rows(&world, "target").await, (2, 4));
    let placed = number(
        &world,
        "SELECT count(*) AS n FROM asset a JOIN component c ON c.id = a.component_id \
         WHERE (a.path = '/x' AND c.version = 'b') OR (a.path = '/y' AND c.version = 'a')",
    )
    .await;
    assert_eq!(placed, 2);
    assert_usage_agrees(&world, "target").await;
    remove_gate(&world, "component").await;
}

#[tokio::test]
async fn the_last_two_assets_of_a_component_leave_it_deleted_after_parallel_removal() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    for path in ["/c/a1", "/c/a2"] {
        store::put(
            &target,
            PutAsset::hosted(&target, path).component(component("1")),
            content_of(path.as_bytes().to_vec()),
        )
        .await
        .unwrap();
    }
    let first_asset = store::find(target.db(), "target", "/c/a1")
        .await
        .unwrap()
        .unwrap();
    let second_asset = store::find(target.db(), "target", "/c/a2")
        .await
        .unwrap()
        .unwrap();
    install_gate(
        &world,
        "AFTER DELETE ON asset FOR EACH ROW WHEN (OLD.path = '/c/a1')",
    )
    .await;
    let gate = close_gate(&world).await;
    let content = target.content.clone();
    let first = tokio::spawn(async move { store::delete_asset(&content, first_asset).await });
    waiting_pid(&world, true).await;
    let probe = world.db.conn().begin().await.unwrap();
    let refused = probe
        .execute_unprepared(
            "SELECT 1 FROM repository_usage WHERE repository = 'target' FOR UPDATE NOWAIT",
        )
        .await;
    probe.rollback().await.unwrap();
    assert!(
        refused.is_err_and(|error| error.to_string().contains("could not obtain lock")),
        "the usage row was not locked before the delete"
    );
    let content = target.content.clone();
    let second = tokio::spawn(async move { store::delete_asset(&content, second_asset).await });
    let blocked = waiting_pid(&world, false).await;
    assert!(query_of(&world, blocked).await.contains("component"));
    gate.commit().await.unwrap();
    let (first, second) = (finish(first).await, finish(second).await);
    assert_no_deadlock(&[&first, &second]);
    first.unwrap();
    second.unwrap();
    assert_eq!(target_rows(&world, "target").await, (0, 0));
    assert_usage_agrees(&world, "target").await;
    remove_gate(&world, "asset").await;
}

#[tokio::test]
async fn deleting_a_repository_during_a_gated_put_waits_for_the_put_and_takes_the_counter_with_it()
{
    let world = world().await;
    let doomed = repository(&world, "doomed", "default", WritePolicy::Allow).await;
    install_gate(
        &world,
        &format!("BEFORE INSERT ON asset FOR EACH ROW WHEN (NEW.size = {MAGIC})"),
    )
    .await;
    let gate = close_gate(&world).await;
    let writer = tokio::spawn({
        let doomed = doomed.clone();
        async move {
            store::put(
                &doomed,
                PutAsset::hosted(&doomed, "/g/p"),
                content_of(vec![b'x'; MAGIC]),
            )
            .await
            .map(|_| ())
        }
    });
    waiting_pid(&world, true).await;
    let conn = world.db.conn().clone();
    let deleter = tokio::spawn(async move {
        conn.execute_unprepared("DELETE FROM repository WHERE name = 'doomed'")
            .await
            .map(|_| ())
            .map_err(|error| ContentError::internal(anyhow::anyhow!("{error}")))
    });
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (writer, deleter) = (finish(writer).await, finish(deleter).await);
    assert_no_deadlock(&[&writer, &deleter]);
    writer.unwrap();
    deleter.unwrap();
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM repository WHERE name = 'doomed'"
        )
        .await,
        0
    );
    assert_eq!(
        number(
            &world,
            "SELECT count(*) AS n FROM repository_usage WHERE repository = 'doomed'"
        )
        .await,
        0
    );
    assert_eq!(target_rows(&world, "doomed").await, (0, 0));
    remove_gate(&world, "asset").await;
}
fn internal(error: sea_orm::DbErr) -> ContentError {
    ContentError::internal(anyhow::anyhow!("{error}"))
}

async fn seeded(world: &World) -> Ctx {
    let doomed = repository(world, "doomed", "default", WritePolicy::Allow).await;
    for path in ["/seed/a", "/seed/b"] {
        store::put(
            &doomed,
            PutAsset::hosted(&doomed, path).component(component("1")),
            content_of(path.as_bytes().to_vec()),
        )
        .await
        .unwrap();
    }
    doomed
}

async fn inversion(
    world: &World,
    start: impl FnOnce() -> Run,
) -> (Result<(), ContentError>, Result<(), ContentError>) {
    let holder = world.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT 1 FROM repository WHERE name = 'doomed' FOR UPDATE")
        .await
        .unwrap();
    let writer = start();
    waiting_pid(world, false).await;
    let deleter: Run = tokio::spawn(async move {
        match holder
            .execute_unprepared("DELETE FROM component WHERE repository = 'doomed'")
            .await
        {
            Ok(_) => holder.commit().await.map_err(internal),
            Err(error) => {
                let _ = holder.rollback().await;
                Err(internal(error))
            }
        }
    });
    (finish(writer).await, finish(deleter).await)
}

#[tokio::test]
async fn a_put_into_an_existing_component_does_not_deadlock_with_a_repository_delete() {
    let world = world().await;
    let doomed = seeded(&world).await;
    install_gate(
        &world,
        "AFTER UPDATE ON component FOR EACH ROW WHEN (NEW.version = '1')",
    )
    .await;
    let gate = close_gate(&world).await;
    let writer = put(&doomed, "/new", Some("1"), b"new");
    waiting_pid(&world, true).await;
    let conn = world.db.conn().clone();
    let deleter: Run = tokio::spawn(async move {
        conn.execute_unprepared("DELETE FROM repository WHERE name = 'doomed'")
            .await
            .map(|_| ())
            .map_err(internal)
    });
    waiting_pid(&world, false).await;
    gate.commit().await.unwrap();
    let (writer, deleter) = (finish(writer).await, finish(deleter).await);
    assert_no_deadlock(&[&writer, &deleter]);
}

#[tokio::test]
async fn a_put_takes_the_repository_lock_before_any_component() {
    let world = world().await;
    let doomed = seeded(&world).await;
    let (writer, deleter) = inversion(&world, || put(&doomed, "/new", Some("1"), b"new")).await;
    assert_no_deadlock(&[&writer, &deleter]);
}

#[tokio::test]
async fn a_transfer_takes_the_repository_lock_before_any_component() {
    let world = world().await;
    let doomed = seeded(&world).await;
    let items = vec![staged(&doomed, "/n1", "1", b"one".to_vec()).await];
    let (writer, deleter) = inversion(&world, || transfer(&doomed, items, Conflict::Fail)).await;
    assert_no_deadlock(&[&writer, &deleter]);
}

#[tokio::test]
async fn delete_asset_takes_the_repository_lock_before_its_component() {
    let world = world().await;
    let doomed = seeded(&world).await;
    let found = store::find(doomed.db(), "doomed", "/seed/a")
        .await
        .unwrap()
        .unwrap();
    let content = doomed.content.clone();
    let (writer, deleter) = inversion(&world, move || {
        tokio::spawn(async move { store::delete_asset(&content, found).await })
    })
    .await;
    assert_no_deadlock(&[&writer, &deleter]);
}

#[tokio::test]
async fn delete_component_takes_the_repository_lock_before_its_component() {
    let world = world().await;
    let doomed = seeded(&world).await;
    let found = store::find(doomed.db(), "doomed", "/seed/a")
        .await
        .unwrap()
        .unwrap();
    let content = doomed.content.clone();
    let id = found.component_id.unwrap();
    let (writer, deleter) = inversion(&world, move || {
        tokio::spawn(async move { store::delete_component(&content, id).await.map(|_| ()) })
    })
    .await;
    assert_no_deadlock(&[&writer, &deleter]);
}

#[tokio::test]
async fn deleting_a_stale_asset_removes_the_blob_of_the_row_it_deleted() {
    let world = world().await;
    let target = repository(&world, "target", "default", WritePolicy::Allow).await;
    store::put(
        &target,
        PutAsset::hosted(&target, "/o/p"),
        content_of(b"first".to_vec()),
    )
    .await
    .unwrap();
    let stale = store::find(target.db(), "target", "/o/p")
        .await
        .unwrap()
        .unwrap();
    store::put(
        &target,
        PutAsset::hosted(&target, "/o/p"),
        content_of(b"second".to_vec()),
    )
    .await
    .unwrap();
    store::delete_asset(&target.content, stale).await.unwrap();
    let store = target.content.blob_stores().get("default").await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let left = loop {
        let listed = store.list(None, 5000).await.unwrap().len();
        if listed == 0 || Instant::now() > deadline {
            break listed;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(left, 0, "an orphan blob stays after a stale delete");
}
