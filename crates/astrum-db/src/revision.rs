use std::collections::BTreeMap;

use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};

pub const SECURITY: &str = "security";
pub const BLOB_STORES: &str = "blobstores";

pub async fn bump(db: &impl ConnectionTrait, name: &str) -> Result<i64, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO revision (name, value) VALUES ($1, 1) \
             ON CONFLICT (name) DO UPDATE SET value = revision.value + 1 RETURNING value",
            [name.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("revision {name}")))?;
    row.try_get("", "value")
}

pub async fn current(db: &impl ConnectionTrait, name: &str) -> Result<i64, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT value FROM revision WHERE name = $1",
            [name.into()],
        ))
        .await?;
    match row {
        Some(row) => row.try_get("", "value"),
        None => Ok(0),
    }
}

pub async fn all(db: &impl ConnectionTrait) -> Result<Revisions, DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT name, value FROM revision",
        ))
        .await?;
    let mut values = BTreeMap::new();
    for row in rows {
        values.insert(row.try_get("", "name")?, row.try_get("", "value")?);
    }
    Ok(Revisions(values))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Revisions(BTreeMap<String, i64>);

impl Revisions {
    pub fn of(&self, name: &str) -> i64 {
        self.0.get(name).copied().unwrap_or_default()
    }
}
