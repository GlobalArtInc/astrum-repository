use astrum_db::entity::component;
use sea_orm::{DatabaseConnection, DbBackend, DbErr, FromQueryResult, Statement, Value};

pub type Key = (String, String, String);

const FIRST: &str = "SELECT c.* FROM component c WHERE c.repository = $1 \
     ORDER BY c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\" LIMIT $2";

const AFTER: &str = "SELECT c.* FROM component c WHERE c.repository = $1 \
     AND (c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\") \
         > ($2::text COLLATE \"C\", $3::text COLLATE \"C\", $4::text COLLATE \"C\") \
     ORDER BY c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\" LIMIT $5";

const ABSENT: &str = "SELECT k.namespace, k.name, k.version \
     FROM unnest($2::text[], $3::text[], $4::text[]) AS k(namespace, name, version) \
     WHERE NOT EXISTS (SELECT 1 FROM component c WHERE c.repository = $1 \
         AND c.namespace = k.namespace AND c.name = k.name AND c.version = k.version)";

#[derive(FromQueryResult)]
struct Row {
    namespace: String,
    name: String,
    version: String,
}

fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

pub fn key_of(found: &component::Model) -> Key {
    (
        found.namespace.clone(),
        found.name.clone(),
        found.version.clone(),
    )
}

pub async fn page(
    db: &DatabaseConnection,
    repository: &str,
    after: Option<&Key>,
    limit: u32,
) -> Result<Vec<component::Model>, DbErr> {
    let statement = match after {
        None => statement(FIRST, vec![repository.into(), i64::from(limit).into()]),
        Some((namespace, name, version)) => statement(
            AFTER,
            vec![
                repository.into(),
                namespace.clone().into(),
                name.clone().into(),
                version.clone().into(),
                i64::from(limit).into(),
            ],
        ),
    };
    component::Model::find_by_statement(statement).all(db).await
}

pub async fn absent(
    db: &DatabaseConnection,
    repository: &str,
    keys: &[Key],
) -> Result<Vec<Key>, DbErr> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let column = |pick: fn(&Key) -> &String| -> Vec<String> {
        keys.iter().map(|key| pick(key).clone()).collect()
    };
    let rows = Row::find_by_statement(statement(
        ABSENT,
        vec![
            repository.into(),
            column(|key| &key.0).into(),
            column(|key| &key.1).into(),
            column(|key| &key.2).into(),
        ],
    ))
    .all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.namespace, row.name, row.version))
        .collect())
}
