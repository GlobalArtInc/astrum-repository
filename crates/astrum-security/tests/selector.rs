use astrum_db::entity::content_selector;
use astrum_security::selector::{Columns, Selector};
use astrum_security::{Security, bootstrap};
use astrum_testkit::TestDatabase;
use sea_orm::{ActiveValue, ConnectionTrait, DbBackend, EntityTrait, Statement};

const SELECTORS: &[&str] = &[
    "format == \"maven2\"",
    "format != \"maven2\"",
    "path =^ \"/org/astrum/\"",
    "path =~ \"/org/.*\\.jar\"",
    "path =~ \"(?i)/ORG/.*\"",
    "path =~ \"[[:alpha:]/]+\"",
    "path =~ \"/(com|org)/[^/]+/\\d+\\.\\d+/.*\"",
    "format == \"maven2\" and path =^ \"/org/\" or format == \"npm\"",
    "format == \"maven2\" && (path =^ \"/com/\" || path =~ \".*\\.pom\")",
    "\"npm\" == format",
    "format == path",
    "\"a\" == \"a\"",
];

const SAMPLES: &[(&str, &str)] = &[
    ("maven2", "/org/astrum/core/1.0/core-1.0.jar"),
    ("maven2", "/org/astrum/core/1.0/core-1.0.pom"),
    ("maven2", "/com/example/2.1/x.jar"),
    ("npm", "/@scope/pkg/-/pkg-1.0.0.tgz"),
    ("raw", "/ORG/thing"),
    ("raw", "org"),
    ("raw", "raw"),
    ("raw", ""),
];

#[tokio::test]
async fn sql_fragments_agree_with_the_evaluator() {
    let test = TestDatabase::create().await;
    let db = test.connect().await;
    let columns = Columns::new("a.format", "a.path");
    for expression in SELECTORS {
        let selector = Selector::parse(expression).unwrap();
        let fragment = selector.to_sql(&columns, 3);
        let sql = format!(
            "SELECT count(*) AS n FROM (SELECT $1::text AS format, $2::text AS path) AS a WHERE {}",
            fragment.sql
        );
        for (format, path) in SAMPLES {
            let mut values: Vec<sea_orm::Value> = vec![(*format).into(), (*path).into()];
            values.extend(fragment.values.iter().map(|value| value.as_str().into()));
            let row = db
                .conn()
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &sql,
                    values,
                ))
                .await
                .unwrap_or_else(|error| panic!("{expression}: {error}"))
                .unwrap();
            let count: i64 = row.try_get("", "n").unwrap();
            assert_eq!(
                count == 1,
                selector.matches(format, path),
                "{expression} on ({format}, {path})"
            );
        }
    }
}

#[tokio::test]
async fn the_model_holds_parsed_selectors() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    bootstrap(db.conn(), data.path()).await.unwrap();
    let now = chrono::Utc::now().fixed_offset();
    for (name, expression) in [
        ("team-a", "format == \"raw\" && path =^ \"/team-a/\""),
        ("broken", "format = \"raw\""),
    ] {
        content_selector::Entity::insert(content_selector::ActiveModel {
            name: ActiveValue::Set(name.to_owned()),
            kind: ActiveValue::Set("csel".to_owned()),
            description: ActiveValue::Set(String::new()),
            expression: ActiveValue::Set(expression.to_owned()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        })
        .exec(db.conn())
        .await
        .unwrap();
    }
    let security = Security::load(db.conn().clone()).await.unwrap();
    let model = security.model();
    let team = &model.selectors["team-a"];
    assert!(team.selector.matches("raw", "/team-a/file"));
    assert!(!team.selector.matches("raw", "/team-b/file"));
    assert!(
        !model.selectors.contains_key("broken"),
        "a selector that does not parse grants nothing"
    );
}
