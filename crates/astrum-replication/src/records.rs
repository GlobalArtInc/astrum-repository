use std::time::Duration;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ConnectionTrait, DbErr, FromQueryResult, Value};
use uuid::Uuid;

use crate::queue::{Item, statement};

const REMOTE_COLUMNS: &str = "m.id, m.url, m.token, m.use_trust_store, m.allow_private_network, \
     m.max_concurrency, m.max_bytes_per_second, m.remote_instance_id, m.updated_at, m.failures, \
     CASE WHEN m.suspended_until > now() \
         THEN extract(epoch FROM m.suspended_until - now())::float8 END AS suspended_for";

const REMOTES_DUE: &str = "FROM replication_remote m \
     WHERE (m.suspended_until IS NULL OR m.suspended_until <= now()) \
     AND EXISTS (SELECT 1 FROM replication r \
         JOIN replication_item i ON i.replication_id = r.id \
         WHERE r.remote_id = m.id AND r.enabled AND i.state = 'pending' AND i.not_before <= now()) \
     ORDER BY m.id LIMIT $1";

const REMOTE: &str = "FROM replication_remote m WHERE m.id = $1";

const REPLICATION: &str = "SELECT id, source_repository, remote_id, remote_repository, enabled, \
     name_filter, version_filter, propagate_deletes FROM replication WHERE id = $1";

const SUCCEEDED: &str =
    "UPDATE replication SET last_success_at = now(), last_error = NULL WHERE id = $1";

const SUSPEND: &str = "UPDATE replication_remote SET \
     failures = CASE WHEN suspended_until > now() THEN failures ELSE failures + 1 END, \
     suspended_until = greatest(coalesce(suspended_until, now()), \
         now() + make_interval(secs => $2::double precision)), \
     last_error = $3 WHERE id = $1 AND updated_at = $4";

const REMOTE_OK: &str = "UPDATE replication_remote \
     SET failures = 0, suspended_until = NULL, last_error = NULL \
     WHERE id = $1 AND (failures <> 0 OR suspended_until IS NOT NULL OR last_error IS NOT NULL)";

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
    pub updated_at: DateTimeWithTimeZone,
    pub failures: i32,
    pub suspended_for: Option<f64>,
}

impl RemoteRow {
    pub fn same_settings(&self, other: &RemoteRow) -> bool {
        self.url == other.url && self.token == other.token && self.updated_at == other.updated_at
    }
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

pub async fn remotes_due(db: &impl ConnectionTrait, limit: i64) -> Result<Vec<RemoteRow>, DbErr> {
    let sql = format!("SELECT {REMOTE_COLUMNS} {REMOTES_DUE}");
    RemoteRow::find_by_statement(statement(&sql, vec![limit.into()]))
        .all(db)
        .await
}

pub async fn remote(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<RemoteRow>, DbErr> {
    let sql = format!("SELECT {REMOTE_COLUMNS} {REMOTE}");
    RemoteRow::find_by_statement(statement(&sql, vec![id.into()]))
        .one(db)
        .await
}

pub async fn suspend(
    db: &impl ConnectionTrait,
    remote: &RemoteRow,
    delay: Duration,
    error: &str,
) -> Result<bool, DbErr> {
    let suspended = db
        .execute(statement(
            SUSPEND,
            vec![
                remote.id.into(),
                delay.as_secs_f64().into(),
                error.to_owned().into(),
                remote.updated_at.into(),
            ],
        ))
        .await?;
    Ok(suspended.rows_affected() == 1)
}

pub async fn remote_ok(db: &impl ConnectionTrait, remote: Uuid) -> Result<(), DbErr> {
    db.execute(statement(REMOTE_OK, vec![remote.into()]))
        .await?;
    Ok(())
}

pub async fn replication(
    db: &impl ConnectionTrait,
    id: Uuid,
) -> Result<Option<ReplicationRow>, DbErr> {
    ReplicationRow::find_by_statement(statement(REPLICATION, vec![id.into()]))
        .one(db)
        .await
}

pub async fn succeeded(db: &impl ConnectionTrait, replication: Uuid) -> Result<(), DbErr> {
    db.execute(statement(SUCCEEDED, vec![replication.into()]))
        .await?;
    Ok(())
}

pub async fn failed(
    db: &impl ConnectionTrait,
    replication: Uuid,
    error: &str,
) -> Result<(), DbErr> {
    db.execute(statement(
        FAILED,
        vec![replication.into(), error.to_owned().into()],
    ))
    .await?;
    Ok(())
}

pub async fn learned(
    db: &impl ConnectionTrait,
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

pub async fn sent(db: &impl ConnectionTrait, item: &Item) -> Result<Option<Sent>, DbErr> {
    Sent::find_by_statement(statement(SENT, key(item)))
        .one(db)
        .await
}

pub async fn record_sent(
    db: &impl ConnectionTrait,
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

pub async fn forget_sent(db: &impl ConnectionTrait, item: &Item) -> Result<(), DbErr> {
    db.execute(statement(FORGET_SENT, key(item))).await?;
    Ok(())
}

pub async fn own_instance(db: &impl ConnectionTrait) -> Result<Uuid, DbErr> {
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
