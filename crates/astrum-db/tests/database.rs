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

    let conn = db.conn();
    let from_cooldown = MIGRATIONS
        .all()
        .unwrap()
        .iter()
        .filter(|migration| migration.stamp >= 1_794_710_000_000)
        .count();
    let cooldown_exception = "INSERT INTO policy_exception (id, rule, name, reason, created_by) \
         VALUES (gen_random_uuid(), 'cooldown', 'left', 'audited', 'tester')";
    conn.execute_unprepared(cooldown_exception).await.unwrap();
    assert_eq!(
        MIGRATIONS.revert(conn, from_cooldown).await.unwrap().len(),
        from_cooldown
    );
    let left = conn
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM policy_exception".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(left.try_get::<i64>("", "n").unwrap(), 0);
    assert!(conn.execute_unprepared(cooldown_exception).await.is_err());
    assert_eq!(db.migrate().await.unwrap().len(), from_cooldown);
    conn.execute_unprepared(cooldown_exception).await.unwrap();
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
    let all = MIGRATIONS.all().unwrap();
    let since = all.len()
        - all
            .iter()
            .position(|migration| {
                migration
                    .name
                    .starts_with("DropGlobalartFromTheDefaultTitle")
            })
            .unwrap();
    for (stored, expected) in [
        ("GlobalArt Astrum Repository", "Astrum Repository"),
        ("Our repository", "Our repository"),
    ] {
        MIGRATIONS.revert(conn, since).await.unwrap();
        setting::put(
            conn,
            astrum_db::system::UI_SETTINGS_KEY,
            &serde_json::json!({"title": stored, "debugAllowed": true}),
        )
        .await
        .unwrap();
        assert_eq!(db.migrate().await.unwrap().len(), since);
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

#[tokio::test]
async fn policy_dirty_rows_follow_their_repository() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    conn.execute_unprepared(
        "INSERT INTO repository (name, format, type) VALUES ('releases', 'maven2', 'hosted')",
    )
    .await
    .unwrap();
    conn.execute_unprepared(
        "INSERT INTO policy_dirty (repository, subject) VALUES ('releases', 'c:1')",
    )
    .await
    .unwrap();
    let orphan = conn
        .execute_unprepared(
            "INSERT INTO policy_dirty (repository, subject) VALUES ('missing', 'c:2')",
        )
        .await;
    assert!(orphan.is_err());
    conn.execute_unprepared("DELETE FROM repository WHERE name = 'releases'")
        .await
        .unwrap();
    let left = conn
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM policy_dirty".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(left.try_get::<i64>("", "n").unwrap(), 0);
}

async fn number(db: &astrum_db::Database, sql: &str) -> i64 {
    db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
async fn policies_with_the_license_rule_on_are_marked_stale_when_the_license_migration_runs() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let all = MIGRATIONS.all().unwrap();
    let since = all.len()
        - all
            .iter()
            .position(|migration| migration.name.starts_with("JudgeLicenceChanges"))
            .unwrap();
    MIGRATIONS.revert(conn, since).await.unwrap();
    conn.execute_unprepared(
        "INSERT INTO repository (name, format, type, attributes) \
           VALUES ('one', 'npm', 'hosted', '{\"policy\":{\"name\":\"lic\"}}'); \
         INSERT INTO content_policy (name, licenses) VALUES \
           ('lic', '{\"mode\":\"block\",\"list\":\"deny\",\"ids\":[\"GPL-3.0-only\"]}'), \
           ('watch', '{\"mode\":\"report\"}'), ('quiet', '{\"mode\":\"off\"}'); \
         INSERT INTO content_policy (name) VALUES ('plain'); \
         INSERT INTO policy_verdict (repository, subject, policy, blocked, state, policy_revision) \
           VALUES ('one', 'c:1', 'lic', false, 'pass', 1); \
         INSERT INTO revision (name, value) VALUES ('policies', 7)",
    )
    .await
    .unwrap();

    assert_eq!(db.migrate().await.unwrap().len(), since);

    for (policy, revision) in [("lic", 2), ("watch", 2), ("quiet", 1), ("plain", 1)] {
        assert_eq!(
            number(
                &db,
                &format!("SELECT revision AS n FROM content_policy WHERE name = '{policy}'")
            )
            .await,
            revision,
            "{policy}"
        );
    }
    assert_eq!(
        number(
            &db,
            "SELECT value AS n FROM revision WHERE name = 'policies'"
        )
        .await,
        8
    );
    assert_eq!(
        number(
            &db,
            "SELECT count(*) AS n FROM policy_verdict v JOIN content_policy p ON p.name = v.policy \
             WHERE v.policy_revision <> p.revision"
        )
        .await,
        1
    );

    MIGRATIONS.revert(conn, since).await.unwrap();
    conn.execute_unprepared("DELETE FROM content_policy WHERE name IN ('lic', 'watch')")
        .await
        .unwrap();
    assert_eq!(db.migrate().await.unwrap().len(), since);
    assert_eq!(
        number(
            &db,
            "SELECT value AS n FROM revision WHERE name = 'policies'"
        )
        .await,
        8
    );
}
