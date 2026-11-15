use astrum_tasks::notification::FAILURE;
use astrum_tasks::store::{self, NewTask, Requested, Stopped, StoreError};
use astrum_tasks::{Properties, Schedule, Task};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, FromQueryResult, Statement,
    TransactionTrait,
};
use uuid::Uuid;

use crate::reconcile::{ID, PROPERTY};

pub const NAME_PREFIX: &str = "replication-reconcile-";
pub const ALLOWED_MINUTES: [i32; 13] = [5, 10, 15, 20, 30, 60, 120, 180, 240, 360, 480, 720, 1440];
pub const DEFAULT_MINUTES: i32 = 360;
const TRIGGER: &str = "manual";
const SYSTEM: &str = "*SYSTEM";
const BACKFILL_LOCK: i64 = i64::from_be_bytes(*b"astrrcnb");

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
pub struct Snapshot {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub reconcile_minutes: i32,
}

impl Snapshot {
    pub async fn locked(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<Self>, DbErr> {
        Self::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, name, enabled, reconcile_minutes FROM replication WHERE id = $1 FOR UPDATE",
            [id.into()],
        ))
        .one(db)
        .await
    }
}

pub fn nearest_allowed(minutes: i32) -> i32 {
    ALLOWED_MINUTES
        .iter()
        .rev()
        .find(|allowed| **allowed <= minutes)
        .copied()
        .unwrap_or(ALLOWED_MINUTES[0])
}

fn spread(replication: Uuid) -> u64 {
    let bytes = replication.as_bytes();
    let mut head = [0u8; 8];
    head.copy_from_slice(&bytes[..8]);
    u64::from_be_bytes(head)
}

fn expression(minutes: i32, replication: Uuid) -> String {
    let seed = spread(replication);
    if minutes < 60 {
        let step = u64::try_from(minutes).unwrap_or(5).max(1);
        return format!("0 {}/{step} * * * ?", seed % step);
    }
    let minute = seed % 60;
    let hours = minutes / 60;
    match hours >= 24 {
        true => format!("0 {minute} 0 * * ?"),
        false => format!("0 {minute} 0/{hours} * * ?"),
    }
}

pub fn schedule_for(minutes: i32, replication: Uuid) -> Result<Schedule, DbErr> {
    Schedule::cron(&expression(nearest_allowed(minutes), replication)).map_err(DbErr::Custom)
}

fn failed(error: StoreError) -> DbErr {
    match error {
        StoreError::Database(error) => error,
        other => DbErr::Custom(other.to_string()),
    }
}

async fn find(db: &impl ConnectionTrait, replication: Uuid) -> Result<Option<Task>, DbErr> {
    Task::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT * FROM task WHERE type = $1 AND properties->>$2 = $3",
        [ID.into(), PROPERTY.into(), replication.to_string().into()],
    ))
    .one(db)
    .await
}

async fn free_name(
    db: &impl ConnectionTrait,
    snapshot: &Snapshot,
    own: Option<Uuid>,
) -> Result<String, DbErr> {
    let wanted = format!("{NAME_PREFIX}{}", snapshot.name);
    match store::find_by_name(db, &wanted).await? {
        Some(taken) if Some(taken.id) != own => Ok(format!("{wanted}-{}", snapshot.id)),
        _ => Ok(wanted),
    }
}

fn properties(replication: Uuid) -> Properties {
    Properties::from([(PROPERTY.to_owned(), replication.to_string())])
}

async fn start(db: &impl ConnectionTrait, task: Uuid, initiator: &str) -> Result<(), DbErr> {
    match store::request_run(db, task, TRIGGER, initiator).await? {
        Requested::Queued(_) | Requested::AlreadyActive => Ok(()),
    }
}

async fn make_task(db: &impl ConnectionTrait, snapshot: &Snapshot) -> Result<Task, DbErr> {
    store::create(
        db,
        NewTask {
            kind: ID.to_owned(),
            name: free_name(db, snapshot, None).await?,
            enabled: true,
            schedule: schedule_for(snapshot.reconcile_minutes, snapshot.id)?,
            alert_email: None,
            notification_condition: FAILURE.to_owned(),
            properties: properties(snapshot.id),
        },
    )
    .await
    .map_err(failed)
}

async fn make(
    db: &impl ConnectionTrait,
    snapshot: &Snapshot,
    initiator: &str,
) -> Result<(), DbErr> {
    let task = make_task(db, snapshot).await?;
    if snapshot.enabled {
        start(db, task.id, initiator).await?;
    }
    Ok(())
}

pub async fn run_now(
    db: &impl ConnectionTrait,
    snapshot: &Snapshot,
    initiator: &str,
) -> Result<Requested, DbErr> {
    let task = match find(db, snapshot.id).await? {
        Some(task) => task,
        None => make_task(db, snapshot).await?,
    };
    store::request_run(db, task.id, TRIGGER, initiator).await
}

pub async fn created(
    db: &impl ConnectionTrait,
    snapshot: &Snapshot,
    initiator: &str,
) -> Result<(), DbErr> {
    make(db, snapshot, initiator).await
}

pub async fn updated(
    db: &impl ConnectionTrait,
    before: &Snapshot,
    after: &Snapshot,
    initiator: &str,
) -> Result<(), DbErr> {
    let Some(task) = find(db, after.id).await? else {
        return Ok(());
    };
    let renamed = before.name != after.name;
    let rescheduled = before.reconcile_minutes != after.reconcile_minutes;
    if renamed || rescheduled {
        let schedule = match rescheduled {
            true => schedule_for(after.reconcile_minutes, after.id)?,
            false => store::schedule_of(&task),
        };
        let name = match renamed {
            true => free_name(db, after, Some(task.id)).await?,
            false => task.name.clone(),
        };
        store::update(
            db,
            task.id,
            NewTask {
                kind: task.kind.clone(),
                name,
                enabled: task.enabled,
                schedule,
                alert_email: task.alert_email.clone(),
                notification_condition: task.notification_condition.clone(),
                properties: store::properties_of(&task),
            },
        )
        .await
        .map_err(failed)?;
    }
    if !before.enabled && after.enabled && task.enabled {
        start(db, task.id, initiator).await?;
    }
    Ok(())
}

pub async fn removed(db: &impl ConnectionTrait, replication: Uuid) -> Result<Option<i64>, DbErr> {
    if Snapshot::locked(db, replication).await?.is_none() {
        return Ok(None);
    }
    let Some(task) = find(db, replication).await? else {
        return Ok(None);
    };
    let stopped = store::request_stop(db, task.id).await?;
    store::delete(db, task.id).await?;
    Ok(match stopped {
        Stopped::Signalled(run) => Some(run),
        Stopped::Dequeued(_) | Stopped::NotRunning => None,
    })
}

pub async fn backfill(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let txn = db.begin().await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT pg_advisory_xact_lock($1)",
        [BACKFILL_LOCK.into()],
    ))
    .await?;
    let lacking = Snapshot::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT r.id, r.name, r.enabled, r.reconcile_minutes FROM replication r \
         WHERE NOT EXISTS (SELECT 1 FROM task t WHERE t.type = $1 \
             AND t.properties ->> $2 = r.id::text) \
         ORDER BY r.created_at, r.id FOR UPDATE OF r",
        [ID.into(), PROPERTY.into()],
    ))
    .all(&txn)
    .await?;
    for snapshot in &lacking {
        make(&txn, snapshot, SYSTEM).await?;
    }
    txn.commit().await?;
    Ok(lacking.len() as u64)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn a_stored_value_maps_to_the_nearest_allowed_one_not_above_it() {
        for allowed in ALLOWED_MINUTES {
            assert_eq!(nearest_allowed(allowed), allowed);
        }
        assert_eq!(nearest_allowed(5), 5);
        assert_eq!(nearest_allowed(7), 5);
        assert_eq!(nearest_allowed(45), 30);
        assert_eq!(nearest_allowed(100), 60);
        assert_eq!(nearest_allowed(1000), 720);
        assert_eq!(nearest_allowed(i32::MAX), 1440);
    }

    #[test]
    fn every_allowed_period_fires_at_exactly_its_interval_from_an_offset_of_the_id() {
        let id = Uuid::from_u128(0x0123_4567_89ab_cdef_0011_2233_4455_6677);
        let seed = spread(id);
        assert_eq!(expression(30, id), format!("0 {}/30 * * * ?", seed % 30));
        assert_eq!(expression(360, id), format!("0 {} 0/6 * * ?", seed % 60));
        assert_eq!(expression(1440, id), format!("0 {} 0 * * ?", seed % 60));
        let from = Utc.with_ymd_and_hms(2026, 11, 14, 0, 0, 0).unwrap();
        for minutes in ALLOWED_MINUTES {
            let schedule = schedule_for(minutes, id).unwrap();
            let mut at = from;
            let mut gaps = Vec::new();
            for _ in 0..6 {
                let next = schedule.next_after(at).unwrap();
                gaps.push((next - at).num_minutes());
                at = next;
            }
            assert!(
                gaps[1..].iter().all(|gap| *gap == i64::from(minutes)),
                "{minutes}: {gaps:?}"
            );
        }
    }

    #[test]
    fn replications_spread_over_the_minutes_of_the_hour() {
        let minutes: std::collections::HashSet<u64> = (0..200u128)
            .map(|number| spread(Uuid::from_u128(number << 64)) % 60)
            .collect();
        assert!(minutes.len() > 30);
    }
}
