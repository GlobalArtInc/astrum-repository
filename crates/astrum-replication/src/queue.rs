use std::time::Duration;

use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, FromQueryResult, Statement, Value,
};
use uuid::Uuid;

pub const MAX_ATTEMPTS: i32 = 12;

const EXPIRED: &str = "UPDATE replication_item SET state = 'pending', leased_until = NULL \
     WHERE state = 'leased' AND leased_until < now()";

const REMOTES_DUE: &str = "SELECT m.id, m.url, m.token, m.use_trust_store, m.allow_private_network, \
     m.max_concurrency, m.max_bytes_per_second, m.remote_instance_id \
     FROM replication_remote m \
     WHERE EXISTS (SELECT 1 FROM replication r \
         JOIN replication_item i ON i.replication_id = r.id \
         WHERE r.remote_id = m.id AND i.state = 'pending' AND i.not_before <= now()) \
     ORDER BY m.id LIMIT $1";

const LEASE: &str = "UPDATE replication_item SET state = 'leased', \
     leased_until = now() + make_interval(secs => $2::double precision) \
     WHERE id IN ( \
         SELECT i.id FROM replication_item i \
         JOIN replication r ON r.id = i.replication_id \
         WHERE r.remote_id = $1 AND i.state = 'pending' AND i.not_before <= now() \
         ORDER BY i.not_before, i.id \
         FOR UPDATE OF i SKIP LOCKED \
         LIMIT $3) \
     RETURNING id, replication_id, namespace, name, version, generation, attempts";

const RENEW: &str = "UPDATE replication_item \
     SET leased_until = now() + make_interval(secs => $2::double precision) \
     WHERE id = $1 AND state = 'leased'";

const FINISH: &str = "DELETE FROM replication_item WHERE id = $1 AND generation = $2";

const AGAIN: &str = "UPDATE replication_item \
     SET state = 'pending', leased_until = NULL, attempts = 0, last_error = NULL \
     WHERE id = $1 AND state = 'leased'";

const RELEASE: &str = "UPDATE replication_item SET state = 'pending', leased_until = NULL WHERE id = $1 AND state = 'leased'";

const FAIL: &str = "UPDATE replication_item SET \
     state = CASE WHEN generation <> $2 THEN 'pending' \
                  WHEN $3 OR attempts + 1 >= $4 THEN 'dead' ELSE 'pending' END, \
     attempts = CASE WHEN generation <> $2 THEN 0 ELSE attempts + 1 END, \
     not_before = CASE WHEN generation <> $2 THEN not_before \
                       ELSE now() + make_interval(secs => $5::double precision) END, \
     reason = CASE WHEN generation <> $2 THEN reason ELSE 'retry' END, \
     last_error = $6, leased_until = NULL \
     WHERE id = $1 AND state = 'leased' \
     RETURNING state, attempts";

const REPLICATION: &str = "SELECT id, source_repository, remote_id, remote_repository, enabled, \
     name_filter, version_filter, propagate_deletes FROM replication WHERE id = $1";

const SUCCEEDED: &str = "UPDATE replication SET last_success_at = now() WHERE id = $1";

const FAILED: &str =
    "UPDATE replication SET last_failure_at = now(), last_error = $2 WHERE id = $1";

const LEARNED: &str = "UPDATE replication_remote \
     SET remote_instance_id = $2, remote_version = $3, checked_at = now() \
     WHERE id = $1 AND url = $4";

const SENT: &str = "SELECT fingerprint, origin FROM replication_sent \
     WHERE replication_id = $1 AND namespace = $2 AND name = $3 AND version = $4";

const RECORD_SENT: &str = "INSERT INTO replication_sent \
     (replication_id, namespace, name, version, fingerprint, origin) \
     VALUES ($1, $2, $3, $4, $5, $6) \
     ON CONFLICT (replication_id, namespace, name, version) \
     DO UPDATE SET fingerprint = EXCLUDED.fingerprint, origin = EXCLUDED.origin, sent_at = now()";

const FORGET_SENT: &str = "DELETE FROM replication_sent \
     WHERE replication_id = $1 AND namespace = $2 AND name = $3 AND version = $4";

const RECORD_INSTANCE: &str = "INSERT INTO setting (key, value, updated_at) \
     VALUES ($1, to_jsonb($2::text), now()) ON CONFLICT (key) DO NOTHING";

const INSTANCE_KEY: &str = "replication.instanceId";

fn statement(sql: &str, values: Vec<Value>) -> Statement {
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

#[derive(Clone, FromQueryResult)]
pub struct RemoteRow {
    pub id: Uuid,
    pub url: String,
    pub token: String,
    pub use_trust_store: bool,
    pub allow_private_network: bool,
    pub max_concurrency: i32,
    pub max_bytes_per_second: Option<i64>,
    pub remote_instance_id: Option<Uuid>,
}

#[derive(Clone, Debug, FromQueryResult)]
pub struct ReplicationRow {
    pub id: Uuid,
    pub source_repository: String,
    pub remote_id: Uuid,
    pub remote_repository: String,
    pub enabled: bool,
    pub name_filter: Option<String>,
    pub version_filter: Option<String>,
    pub propagate_deletes: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
pub struct Sent {
    pub fingerprint: String,
    pub origin: String,
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

pub async fn requeue_expired(db: &DatabaseConnection) -> Result<u64, DbErr> {
    Ok(db
        .execute(statement(EXPIRED, Vec::new()))
        .await?
        .rows_affected())
}

pub async fn remotes_due(db: &DatabaseConnection, limit: i64) -> Result<Vec<RemoteRow>, DbErr> {
    RemoteRow::find_by_statement(statement(REMOTES_DUE, vec![limit.into()]))
        .all(db)
        .await
}

pub async fn lease(
    db: &DatabaseConnection,
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

pub async fn renew(db: &DatabaseConnection, id: i64, lease: Duration) -> Result<bool, DbErr> {
    let renewed = db
        .execute(statement(
            RENEW,
            vec![id.into(), lease.as_secs_f64().into()],
        ))
        .await?;
    Ok(renewed.rows_affected() == 1)
}

pub async fn finish(db: &DatabaseConnection, item: &Item) -> Result<Finished, DbErr> {
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

pub async fn release(db: &DatabaseConnection, id: i64) -> Result<(), DbErr> {
    db.execute(statement(RELEASE, vec![id.into()])).await?;
    Ok(())
}

pub async fn fail(
    db: &DatabaseConnection,
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

pub async fn replication(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<ReplicationRow>, DbErr> {
    ReplicationRow::find_by_statement(statement(REPLICATION, vec![id.into()]))
        .one(db)
        .await
}

pub async fn succeeded(db: &DatabaseConnection, replication: Uuid) -> Result<(), DbErr> {
    db.execute(statement(SUCCEEDED, vec![replication.into()]))
        .await?;
    Ok(())
}

pub async fn failed(db: &DatabaseConnection, replication: Uuid, error: &str) -> Result<(), DbErr> {
    db.execute(statement(
        FAILED,
        vec![replication.into(), error.to_owned().into()],
    ))
    .await?;
    Ok(())
}

pub async fn learned(
    db: &DatabaseConnection,
    remote: &RemoteRow,
    instance: Uuid,
    version: &str,
) -> Result<(), DbErr> {
    db.execute(statement(
        LEARNED,
        vec![
            remote.id.into(),
            instance.into(),
            version.to_owned().into(),
            remote.url.clone().into(),
        ],
    ))
    .await?;
    Ok(())
}

fn key(item: &Item) -> Vec<Value> {
    vec![
        item.replication_id.into(),
        item.namespace.clone().into(),
        item.name.clone().into(),
        item.version.clone().into(),
    ]
}

pub async fn sent(db: &DatabaseConnection, item: &Item) -> Result<Option<Sent>, DbErr> {
    Sent::find_by_statement(statement(SENT, key(item)))
        .one(db)
        .await
}

pub async fn record_sent(
    db: &DatabaseConnection,
    item: &Item,
    fingerprint: &str,
    origin: &str,
) -> Result<(), DbErr> {
    let mut values = key(item);
    values.push(fingerprint.to_owned().into());
    values.push(origin.to_owned().into());
    db.execute(statement(RECORD_SENT, values)).await?;
    Ok(())
}

pub async fn forget_sent(db: &DatabaseConnection, item: &Item) -> Result<(), DbErr> {
    db.execute(statement(FORGET_SENT, key(item))).await?;
    Ok(())
}

pub async fn own_instance(db: &DatabaseConnection) -> Result<Uuid, DbErr> {
    db.execute(statement(
        RECORD_INSTANCE,
        vec![INSTANCE_KEY.into(), Uuid::new_v4().to_string().into()],
    ))
    .await?;
    let stored: Option<String> = astrum_db::setting::get(db, INSTANCE_KEY)
        .await
        .map_err(|error| DbErr::Custom(error.to_string()))?;
    let stored = stored.ok_or_else(|| {
        DbErr::Custom("the setting replication.instanceId was not stored".to_owned())
    })?;
    Uuid::parse_str(&stored)
        .map_err(|_| DbErr::Custom("the setting replication.instanceId is not a UUID".to_owned()))
}
