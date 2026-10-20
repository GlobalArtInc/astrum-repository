use astrum_db::{LEDGER, MIGRATIONS, read_only, setting};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde::{Deserialize, Serialize};

async fn tables(db: &astrum_db::Database) -> Vec<String> {
    db.conn()
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT table_name::text AS name FROM information_schema.tables WHERE table_schema = $1 ORDER BY 1",
            [db.schema().into()],
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get::<String>("", "name").unwrap())
        .collect()
}

#[tokio::test]
async fn migrations_build_the_schema_once() {
    let test = TestDatabase::create().await;
    let db = test.connect().await;
    assert_eq!(db.schema(), "astrum");

    let applied = db.migrate().await.unwrap();
    let carried: Vec<String> = MIGRATIONS
        .all()
        .unwrap()
        .into_iter()
        .map(|migration| migration.name)
        .collect();
    assert_eq!(applied, carried);
    assert!(applied[0].starts_with("CreateSettings"), "{applied:?}");

    assert!(db.migrate().await.unwrap().is_empty());
    let tables = tables(&db).await;
    assert!(tables.contains(&LEDGER.to_owned()), "{tables:?}");
    assert!(tables.contains(&"setting".to_owned()), "{tables:?}");
}

#[tokio::test]
async fn every_migration_can_be_taken_back() {
    for migration in MIGRATIONS.all().unwrap() {
        assert!(migration.down.is_some(), "{}", migration.name);
    }
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let count = MIGRATIONS.all().unwrap().len();
    let reverted = MIGRATIONS.revert(db.conn(), count).await.unwrap();
    assert_eq!(reverted.len(), count);
    let left: Vec<String> = tables(&db)
        .await
        .into_iter()
        .filter(|table| !table.starts_with(LEDGER))
        .collect();
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(db.migrate().await.unwrap().len(), count);
}

#[tokio::test]
async fn two_starts_at_once_share_the_work() {
    let test = TestDatabase::create().await;
    let mut config = test.config();
    config.max_connections = astrum_db::MIN_CONNECTIONS;
    let (first, second) = tokio::join!(
        astrum_db::Database::connect(&config),
        astrum_db::Database::connect(&config)
    );
    let (first, second) = (first.unwrap(), second.unwrap());
    let (first, second) = tokio::join!(first.migrate(), second.migrate());
    let total = first.unwrap().len() + second.unwrap().len();
    assert_eq!(total, MIGRATIONS.all().unwrap().len());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BaseUrl {
    url: String,
    force: bool,
}

#[tokio::test]
async fn settings_keep_what_was_written() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();

    assert_eq!(
        setting::get::<BaseUrl>(conn, "base_url").await.unwrap(),
        None
    );

    let first = BaseUrl {
        url: "https://astrum.example".to_owned(),
        force: false,
    };
    setting::put(conn, "base_url", &first).await.unwrap();
    assert_eq!(setting::get(conn, "base_url").await.unwrap(), Some(first));

    let second = BaseUrl {
        url: "https://repo.example".to_owned(),
        force: true,
    };
    setting::put(conn, "base_url", &second).await.unwrap();
    assert_eq!(setting::get(conn, "base_url").await.unwrap(), Some(second));

    assert!(setting::delete(conn, "base_url").await.unwrap());
    assert!(!setting::delete(conn, "base_url").await.unwrap());
}

#[tokio::test]
async fn a_setting_of_another_shape_is_reported_by_its_key() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    setting::put(db.conn(), "base_url", &42).await.unwrap();
    let error = setting::get::<BaseUrl>(db.conn(), "base_url")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("\"base_url\""), "{error}");
}

#[tokio::test]
async fn read_only_mode_is_off_until_frozen() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();

    assert_eq!(read_only::state(conn).await.unwrap(), Default::default());

    read_only::freeze(conn, "backup", false).await.unwrap();
    let state = read_only::state(conn).await.unwrap();
    assert!(state.frozen);
    assert_eq!(state.reason.as_deref(), Some("backup"));
    assert!(!state.system_initiated);

    read_only::release(conn).await.unwrap();
    assert!(!read_only::state(conn).await.unwrap().frozen);
}

#[tokio::test]
async fn a_read_only_database_is_noticed() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    assert!(db.accepts_writes().await.unwrap());
    db.ping().await.unwrap();

    db.conn()
        .execute_unprepared(&format!(
            "ALTER DATABASE \"{}\" SET default_transaction_read_only = on",
            test.name()
        ))
        .await
        .unwrap();
    db.close().await.unwrap();

    let db = test.connect().await;
    assert!(!db.accepts_writes().await.unwrap());
    db.ping().await.unwrap();
}

#[tokio::test]
async fn a_schema_of_another_name_is_created_on_connect() {
    let test = TestDatabase::create().await;
    let mut config = test.config();
    config.schema = "astrum_side".to_owned();
    let db = astrum_db::Database::connect(&config).await.unwrap();
    db.migrate().await.unwrap();
    assert_eq!(db.schema(), "astrum_side");
    assert!(tables(&db).await.contains(&"setting".to_owned()));
}

#[tokio::test]
async fn the_old_default_title_loses_globalart_and_own_titles_stay() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    for (stored, expected) in [
        ("GlobalArt Astrum Repository", "Astrum Repository"),
        ("Our repository", "Our repository"),
    ] {
        MIGRATIONS.revert(conn, 1).await.unwrap();
        setting::put(
            conn,
            astrum_db::system::UI_SETTINGS_KEY,
            &serde_json::json!({"title": stored, "debugAllowed": true}),
        )
        .await
        .unwrap();
        assert_eq!(db.migrate().await.unwrap().len(), 1);
        let ui: astrum_db::system::UiSettings =
            setting::get(conn, astrum_db::system::UI_SETTINGS_KEY)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(ui.title, expected);
        assert!(ui.debug_allowed);
    }
    assert_eq!(astrum_db::system::DEFAULT_TITLE, "Astrum Repository");
}
