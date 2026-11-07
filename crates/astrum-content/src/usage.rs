use astrum_db::entity::repository_usage;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter,
    QuerySelect, Statement, TransactionTrait,
};

use crate::ContentError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub assets: i64,
    pub bytes: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reconciled {
    pub usage: Usage,
    pub assets_drift: i64,
    pub bytes_drift: i64,
}

impl Reconciled {
    pub fn corrected(&self) -> bool {
        self.assets_drift != 0 || self.bytes_drift != 0
    }
}

pub async fn current(
    db: &impl ConnectionTrait,
    repository: &str,
) -> Result<Option<Usage>, ContentError> {
    let row = repository_usage::Entity::find_by_id(repository)
        .one(db)
        .await?;
    Ok(row.map(|row| Usage {
        assets: row.assets,
        bytes: row.bytes,
    }))
}

pub async fn reconcile(
    db: &DatabaseConnection,
    repository: &str,
) -> Result<Option<Reconciled>, ContentError> {
    let txn = db.begin().await?;
    let Some(held) = repository_usage::Entity::find()
        .filter(repository_usage::Column::Repository.eq(repository))
        .lock_exclusive()
        .one(&txn)
        .await?
    else {
        txn.rollback().await?;
        return Ok(None);
    };
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*)::bigint AS assets, coalesce(sum(size), 0)::bigint AS bytes \
             FROM asset WHERE repository = $1",
            [repository.into()],
        ))
        .await?
        .ok_or_else(|| ContentError::internal(anyhow::anyhow!("an aggregate returned no row")))?;
    let actual = Usage {
        assets: row.try_get("", "assets")?,
        bytes: row.try_get("", "bytes")?,
    };
    let reconciled = Reconciled {
        usage: actual,
        assets_drift: actual.assets - held.assets,
        bytes_drift: actual.bytes - held.bytes,
    };
    if reconciled.corrected() {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE repository_usage SET assets = $2, bytes = $3, updated_at = now() \
             WHERE repository = $1",
            [repository.into(), actual.assets.into(), actual.bytes.into()],
        ))
        .await?;
    }
    txn.commit().await?;
    if reconciled.corrected() {
        tracing::warn!(
            repository,
            assets_drift = reconciled.assets_drift,
            bytes_drift = reconciled.bytes_drift,
            "the usage counter of a repository did not match its assets and was corrected"
        );
    }
    Ok(Some(reconciled))
}
