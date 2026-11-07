use std::collections::HashSet;
use std::time::Duration;

use astrum_blob::BlobId;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, FromQueryResult, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::ContentError;

pub const TRANSFER: &str = "transfer";

const HOLD: &str = "INSERT INTO blob_hold (blob_store, blob_id, reason, held_until) \
     VALUES ($1, $2, $3, now() + make_interval(secs => $4)) \
     ON CONFLICT (blob_store, blob_id) DO NOTHING";

const EXTEND: &str = "UPDATE blob_hold \
     SET held_until = GREATEST(held_until, now() + make_interval(secs => $3)) \
     WHERE blob_store = $1 AND blob_id = $2 AND held_until > now()";

const RELEASE: &str = "DELETE FROM blob_hold WHERE blob_store = $1 AND blob_id = $2";

const HELD: &str = "SELECT blob_id FROM blob_hold WHERE blob_store = ANY($1) AND blob_id = ANY($2)";

const TAKE_EXPIRED: &str = "DELETE FROM blob_hold WHERE (blob_store, blob_id) IN ( \
     SELECT blob_store, blob_id FROM blob_hold WHERE held_until <= now() \
     ORDER BY held_until LIMIT $1 FOR UPDATE SKIP LOCKED) \
     RETURNING blob_store, blob_id";

const NAMED: &str = "SELECT DISTINCT blob_id FROM asset WHERE blob_id = ANY($1)";

#[derive(Debug, FromQueryResult)]
struct Reference {
    blob_id: Uuid,
}

#[derive(Debug, FromQueryResult)]
struct Taken {
    blob_store: String,
    blob_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum HoldError {
    #[error("the blob {id} of the blob store \"{store}\" is held already")]
    Exists { store: String, id: BlobId },
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),
}

impl From<HoldError> for ContentError {
    fn from(error: HoldError) -> Self {
        match error {
            HoldError::Exists { .. } => ContentError::Conflict(error.to_string()),
            HoldError::Database(error) => error.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expired {
    pub store: String,
    pub id: BlobId,
    pub named: bool,
}

pub async fn hold(
    db: &impl ConnectionTrait,
    store: &str,
    id: BlobId,
    ttl: Duration,
) -> Result<(), HoldError> {
    let inserted = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            HOLD,
            [
                store.into(),
                id.uuid().into(),
                TRANSFER.into(),
                ttl.as_secs_f64().into(),
            ],
        ))
        .await?;
    match inserted.rows_affected() {
        0 => Err(HoldError::Exists {
            store: store.to_owned(),
            id,
        }),
        _ => Ok(()),
    }
}

pub async fn extend(
    db: &impl ConnectionTrait,
    store: &str,
    id: BlobId,
    ttl: Duration,
) -> Result<bool, ContentError> {
    let extended = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            EXTEND,
            [store.into(), id.uuid().into(), ttl.as_secs_f64().into()],
        ))
        .await?;
    Ok(extended.rows_affected() > 0)
}

pub async fn release(
    db: &impl ConnectionTrait,
    store: &str,
    id: BlobId,
) -> Result<bool, ContentError> {
    let released = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            RELEASE,
            [store.into(), id.uuid().into()],
        ))
        .await?;
    Ok(released.rows_affected() > 0)
}

pub async fn held(
    db: &impl ConnectionTrait,
    stores: &[String],
    ids: &[Uuid],
) -> Result<HashSet<Uuid>, ContentError> {
    Ok(Reference::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        HELD,
        [stores.to_vec().into(), ids.to_vec().into()],
    ))
    .all(db)
    .await?
    .into_iter()
    .map(|found| found.blob_id)
    .collect())
}

pub async fn take_expired(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<Expired>, ContentError> {
    let txn = db.begin().await?;
    let taken = Taken::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        TAKE_EXPIRED,
        [(limit as i64).into()],
    ))
    .all(&txn)
    .await?;
    let named: HashSet<Uuid> = match taken.is_empty() {
        true => HashSet::new(),
        false => Reference::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            NAMED,
            [taken
                .iter()
                .map(|row| row.blob_id)
                .collect::<Vec<_>>()
                .into()],
        ))
        .all(&txn)
        .await?
        .into_iter()
        .map(|row| row.blob_id)
        .collect(),
    };
    txn.commit().await?;
    Ok(taken
        .into_iter()
        .map(|row| Expired {
            named: named.contains(&row.blob_id),
            store: row.blob_store,
            id: row.blob_id.into(),
        })
        .collect())
}
