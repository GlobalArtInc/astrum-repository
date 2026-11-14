use std::time::Duration;

use sea_orm::{ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement, Value};
use uuid::Uuid;

use crate::error::{BACKOFF_BASE, BACKOFF_CAP};
pub use crate::records::{
    RemoteRow, ReplicationRow, Sent, failed, forget_sent, learned, own_instance, record_sent,
    remote, remote_ok, remotes_due, replication, sent, succeeded, suspend,
};

pub const MAX_ATTEMPTS: i32 = 12;

const EXPIRED: &str = "UPDATE replication_item SET \
     attempts = attempts + 1, \
     state = CASE WHEN attempts + 1 >= $1 THEN 'dead' ELSE 'pending' END, \
     last_error = CASE WHEN attempts + 1 >= $1 THEN 'lease expired' ELSE last_error END, \
     not_before = CASE WHEN attempts + 1 >= $1 THEN not_before ELSE now() + make_interval( \
         secs => least($3::double precision, \
             $2::double precision * power(2, least(attempts, 16)) * (0.8 + 0.4 * random()))) END, \
     leased_until = NULL \
     WHERE state = 'leased' AND leased_until < now()";

const LEASE: &str = "UPDATE replication_item SET state = 'leased', \
     leased_until = now() + make_interval(secs => $2::double precision) \
     WHERE id IN ( \
         SELECT i.id FROM replication_item i \
         JOIN replication r ON r.id = i.replication_id \
         JOIN replication_remote m ON m.id = r.remote_id \
         WHERE r.remote_id = $1 AND r.enabled \
         AND (m.suspended_until IS NULL OR m.suspended_until <= now()) \
         AND i.state = 'pending' AND i.not_before <= now() \
         ORDER BY i.not_before, i.id \
         FOR UPDATE OF i SKIP LOCKED \
         LIMIT $3) \
     RETURNING id, replication_id, namespace, name, version, generation, attempts";

const DUE_COUNT: &str = "SELECT count(*) AS n FROM ( \
     SELECT 1 FROM replication_item i JOIN replication r ON r.id = i.replication_id \
     WHERE r.remote_id = $1 AND r.enabled AND i.state = 'pending' AND i.not_before <= now() \
     LIMIT $2) due";

const RENEW: &str = "UPDATE replication_item \
     SET leased_until = now() + make_interval(secs => $2::double precision) \
     WHERE id = ANY($1::bigint[]) AND state = 'leased'";

const FINISH: &str = "DELETE FROM replication_item WHERE id = $1 AND generation = $2";

const AGAIN: &str = "UPDATE replication_item \
     SET state = 'pending', leased_until = NULL, attempts = 0, last_error = NULL \
     WHERE id = $1 AND state = 'leased'";

const RELEASE: &str = "UPDATE replication_item SET state = 'pending', leased_until = NULL \
     WHERE id = ANY($1::bigint[]) AND state = 'leased'";

const DEFER: &str = "UPDATE replication_item SET state = 'pending', leased_until = NULL, \
     not_before = greatest(not_before, now() + make_interval(secs => $2::double precision)) \
     WHERE id = $1 AND state = 'leased'";

const FAIL: &str = "UPDATE replication_item SET \
     state = CASE WHEN generation <> $2 THEN 'pending' \
                  WHEN $3 OR attempts + 1 >= $4 THEN 'dead' ELSE 'pending' END, \
     attempts = CASE WHEN generation <> $2 THEN 0 ELSE attempts + 1 END, \
     not_before = CASE WHEN generation <> $2 THEN not_before \
                       ELSE now() + make_interval(secs => $5::double precision) END, \
     reason = CASE WHEN generation <> $2 THEN reason ELSE 'retry' END, \
     last_error = CASE WHEN generation <> $2 THEN last_error ELSE $6 END, leased_until = NULL \
     WHERE id = $1 AND state = 'leased' \
     RETURNING state, attempts";

pub(crate) fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
pub struct Item {
    pub id: i64,
    pub replication_id: Uuid,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub generation: i64,
    pub attempts: i32,
}

#[derive(FromQueryResult)]
struct Counted {
    n: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finished {
    Deleted,
    Again,
    Gone,
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
pub struct Failed {
    pub state: String,
    pub attempts: i32,
}

pub async fn requeue_expired(db: &impl ConnectionTrait) -> Result<u64, DbErr> {
    let values = vec![
        MAX_ATTEMPTS.into(),
        BACKOFF_BASE.as_secs_f64().into(),
        BACKOFF_CAP.as_secs_f64().into(),
    ];
    Ok(db
        .execute(statement(EXPIRED, values))
        .await?
        .rows_affected())
}

pub async fn lease(
    db: &impl ConnectionTrait,
    remote: Uuid,
    lease: Duration,
    limit: i64,
) -> Result<Vec<Item>, DbErr> {
    Item::find_by_statement(statement(
        LEASE,
        vec![remote.into(), lease.as_secs_f64().into(), limit.into()],
    ))
    .all(db)
    .await
}

pub async fn lease_share(
    db: &impl ConnectionTrait,
    remote: Uuid,
    lease: Duration,
    limit: i64,
    workers: i64,
) -> Result<Vec<Item>, DbErr> {
    let workers = workers.max(1);
    let limit = limit.max(1);
    let counted = Counted::find_by_statement(statement(
        DUE_COUNT,
        vec![remote.into(), limit.saturating_mul(workers).into()],
    ))
    .one(db)
    .await?
    .map_or(0, |counted| counted.n);
    let share = ((counted + workers - 1) / workers).clamp(1, limit);
    self::lease(db, remote, lease, share).await
}

pub async fn renew(db: &impl ConnectionTrait, id: i64, lease: Duration) -> Result<bool, DbErr> {
    Ok(renew_many(db, &[id], lease).await? == 1)
}

pub async fn renew_many(
    db: &impl ConnectionTrait,
    ids: &[i64],
    lease: Duration,
) -> Result<u64, DbErr> {
    let renewed = db
        .execute(statement(
            RENEW,
            vec![ids.to_vec().into(), lease.as_secs_f64().into()],
        ))
        .await?;
    Ok(renewed.rows_affected())
}

pub async fn finish(db: &impl ConnectionTrait, item: &Item) -> Result<Finished, DbErr> {
    let deleted = db
        .execute(statement(
            FINISH,
            vec![item.id.into(), item.generation.into()],
        ))
        .await?;
    if deleted.rows_affected() == 1 {
        return Ok(Finished::Deleted);
    }
    let again = db.execute(statement(AGAIN, vec![item.id.into()])).await?;
    Ok(match again.rows_affected() {
        1 => Finished::Again,
        _ => Finished::Gone,
    })
}

pub async fn release(db: &impl ConnectionTrait, id: i64) -> Result<(), DbErr> {
    release_many(db, &[id]).await
}

pub async fn release_many(db: &impl ConnectionTrait, ids: &[i64]) -> Result<(), DbErr> {
    if ids.is_empty() {
        return Ok(());
    }
    db.execute(statement(RELEASE, vec![ids.to_vec().into()]))
        .await?;
    Ok(())
}

pub async fn defer(db: &impl ConnectionTrait, id: i64, delay: Duration) -> Result<(), DbErr> {
    db.execute(statement(
        DEFER,
        vec![id.into(), delay.as_secs_f64().into()],
    ))
    .await?;
    Ok(())
}

pub async fn fail(
    db: &impl ConnectionTrait,
    item: &Item,
    permanent: bool,
    delay: Duration,
    error: &str,
) -> Result<Option<Failed>, DbErr> {
    Failed::find_by_statement(statement(
        FAIL,
        vec![
            item.id.into(),
            item.generation.into(),
            permanent.into(),
            MAX_ATTEMPTS.into(),
            delay.as_secs_f64().into(),
            error.to_owned().into(),
        ],
    ))
    .one(db)
    .await
}
