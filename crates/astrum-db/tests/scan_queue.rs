use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};

async fn count(conn: &impl ConnectionTrait, table: &str) -> i64 {
    conn.query_one(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT count(*) AS n FROM {table}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

async fn run(conn: &impl ConnectionTrait, sql: &str) {
    conn.execute_unprepared(sql).await.unwrap();
}

async fn setup() -> (TestDatabase, astrum_db::Database) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    run(
        conn,
        "INSERT INTO repository (name, format, type, attributes) VALUES \
         ('guarded', 'maven2', 'hosted', '{\"policy\": {\"name\": \"strict\"}}'), \
         ('open', 'maven2', 'hosted', '{}')",
    )
    .await;
    (test, db)
}

async fn component(conn: &impl ConnectionTrait, repository: &str, name: &str) -> i64 {
    conn.query_one(Statement::from_string(
        DbBackend::Postgres,
        format!(
            "INSERT INTO component (repository, format, name, version) \
             VALUES ('{repository}', 'maven2', '{name}', '1') RETURNING id"
        ),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "id")
    .unwrap()
}

async fn asset(conn: &impl ConnectionTrait, component_id: i64) {
    run(
        conn,
        &format!(
            "INSERT INTO asset (repository, format, path, component_id, blob_store, blob_id, size, \
             content_type, sha1, sha256, sha512, md5) \
             SELECT repository, format, '/' || name, id, 'default', gen_random_uuid(), 1, 't', '', '', '', '' \
             FROM component WHERE id = {component_id}"
        ),
    )
    .await;
}

async fn queued(conn: &DatabaseConnection, component_id: i64) -> i64 {
    conn.query_one(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT count(*) AS n FROM scan_queue WHERE component_id = {component_id}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

async fn queued_at(conn: &DatabaseConnection) -> chrono::DateTime<chrono::Utc> {
    conn.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT queued_at FROM scan_queue".to_owned(),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "queued_at")
    .unwrap()
}

#[tokio::test]
async fn a_component_is_queued_once_and_only_in_a_repository_with_a_policy() {
    let (_test, db) = setup().await;
    let conn = db.conn();
    let guarded = component(conn, "guarded", "a").await;
    let open = component(conn, "open", "b").await;
    assert_eq!(queued(conn, guarded).await, 1);
    assert_eq!(queued(conn, open).await, 0);
    let reason: String = conn
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT reason FROM scan_queue".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "reason")
        .unwrap();
    assert_eq!(reason, "component");
}

#[tokio::test]
async fn a_changed_blob_queues_its_component_again_but_a_touched_asset_does_not() {
    let (_test, db) = setup().await;
    let conn = db.conn();
    let guarded = component(conn, "guarded", "a").await;
    let open = component(conn, "open", "b").await;
    asset(conn, guarded).await;
    asset(conn, open).await;
    run(conn, "DELETE FROM scan_queue").await;

    run(conn, "UPDATE asset SET size = 2, download_count = 5").await;
    assert_eq!(count(conn, "scan_queue").await, 0);

    run(conn, "UPDATE asset SET blob_id = gen_random_uuid()").await;
    assert_eq!(queued(conn, guarded).await, 1);
    assert_eq!(queued(conn, open).await, 0);
    let first = queued_at(conn).await;
    run(conn, "UPDATE asset SET blob_id = gen_random_uuid()").await;
    assert_eq!(count(conn, "scan_queue").await, 1);
    assert!(queued_at(conn).await > first);

    run(conn, "DELETE FROM scan_queue").await;
    run(conn, "UPDATE asset SET blob_store = 'cold'").await;
    assert_eq!(queued(conn, guarded).await, 1);
}

#[tokio::test]
async fn nothing_is_queued_while_a_blob_is_being_tiered() {
    let (_test, db) = setup().await;
    let conn = db.conn();
    let first = component(conn, "guarded", "a").await;
    asset(conn, first).await;
    run(conn, "DELETE FROM scan_queue").await;

    let txn = conn.begin().await.unwrap();
    run(&txn, "SELECT set_config('astrum.action', 'tiered', true)").await;
    run(&txn, "UPDATE asset SET blob_store = 'cold'").await;
    let second = component(&txn, "guarded", "b").await;
    txn.commit().await.unwrap();
    assert_eq!(count(conn, "scan_queue").await, 0);

    run(conn, "UPDATE asset SET blob_id = gen_random_uuid()").await;
    assert_eq!(queued(conn, first).await, 1);
    assert_eq!(queued(conn, second).await, 0);
}
