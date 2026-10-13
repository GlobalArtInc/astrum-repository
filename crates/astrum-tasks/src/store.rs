use std::collections::HashMap;

use astrum_db::entity::{task, task_log, task_run};
use chrono::{DateTime, Utc};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, QueryOrder, QuerySelect, SqlErr, Statement,
};
use serde_json::Value;
use uuid::Uuid;

use crate::kind::Properties;
use crate::schedule::Schedule;

pub type Task = task::Model;
pub type TaskRun = task_run::Model;
pub type LogLine = task_log::Model;

pub const KEPT_RUNS: i64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Waiting,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl RunState {
    pub fn as_str(self) -> &'static str {
        match self {
            RunState::Waiting => "waiting",
            RunState::Running => "running",
            RunState::Done => "done",
            RunState::Failed => "failed",
            RunState::Cancelled => "cancelled",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "waiting" => RunState::Waiting,
            "running" => RunState::Running,
            "done" => RunState::Done,
            "failed" => RunState::Failed,
            "cancelled" => RunState::Cancelled,
            _ => return None,
        })
    }

    pub fn finished(self) -> bool {
        matches!(
            self,
            RunState::Done | RunState::Failed | RunState::Cancelled
        )
    }

    pub fn public_result(self) -> Option<&'static str> {
        match self {
            RunState::Done => Some("OK"),
            RunState::Failed => Some("FAILED"),
            RunState::Cancelled => Some("CANCELED"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewTask {
    pub kind: String,
    pub name: String,
    pub enabled: bool,
    pub schedule: Schedule,
    pub alert_email: Option<String>,
    pub notification_condition: String,
    pub properties: Properties,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("a task named \"{0}\" exists already")]
    Duplicate(String),
    #[error(transparent)]
    Database(#[from] DbErr),
}

pub fn properties_of(task: &Task) -> Properties {
    match &task.properties {
        Value::Object(map) => map
            .iter()
            .map(|(key, value)| {
                let text = match value {
                    Value::String(text) => text.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                (key.clone(), text)
            })
            .collect(),
        _ => Properties::new(),
    }
}

pub fn schedule_of(task: &Task) -> Schedule {
    Schedule::from_json(&task.frequency)
}

pub fn next_run(enabled: bool, schedule: &Schedule, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match enabled {
        true => schedule.next_after(now),
        false => None,
    }
}

fn duplicate(error: DbErr, name: &str) -> StoreError {
    match error.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => StoreError::Duplicate(name.to_owned()),
        _ => StoreError::Database(error),
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Task>, DbErr> {
    task::Entity::find()
        .order_by_asc(Expr::expr(Func::lower(Expr::col(task::Column::Name))))
        .all(db)
        .await
}

pub async fn find(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<Task>, DbErr> {
    task::Entity::find_by_id(id).one(db).await
}

pub async fn find_by_name(db: &impl ConnectionTrait, name: &str) -> Result<Option<Task>, DbErr> {
    task::Entity::find()
        .filter(Expr::expr(Func::lower(Expr::col(task::Column::Name))).eq(name.to_lowercase()))
        .one(db)
        .await
}

pub async fn count_of_kind(db: &impl ConnectionTrait, kind: &str) -> Result<u64, DbErr> {
    use sea_orm::PaginatorTrait;
    task::Entity::find()
        .filter(task::Column::Kind.eq(kind))
        .count(db)
        .await
}

pub async fn create(db: &impl ConnectionTrait, new: NewTask) -> Result<Task, StoreError> {
    let now = Utc::now();
    let row = task::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        kind: ActiveValue::Set(new.kind),
        name: ActiveValue::Set(new.name.clone()),
        enabled: ActiveValue::Set(new.enabled),
        frequency: ActiveValue::Set(new.schedule.to_json()),
        alert_email: ActiveValue::Set(new.alert_email),
        notification_condition: ActiveValue::Set(new.notification_condition),
        properties: ActiveValue::Set(serde_json::to_value(&new.properties).unwrap_or_default()),
        next_run_at: ActiveValue::Set(
            next_run(new.enabled, &new.schedule, now).map(|next| next.fixed_offset()),
        ),
        created_at: ActiveValue::Set(now.fixed_offset()),
        updated_at: ActiveValue::Set(now.fixed_offset()),
    };
    task::Entity::insert(row)
        .exec_with_returning(db)
        .await
        .map_err(|error| duplicate(error, &new.name))
}

pub async fn update(db: &impl ConnectionTrait, id: Uuid, new: NewTask) -> Result<Task, StoreError> {
    let now = Utc::now();
    let row = task::ActiveModel {
        id: ActiveValue::Unchanged(id),
        name: ActiveValue::Set(new.name.clone()),
        enabled: ActiveValue::Set(new.enabled),
        frequency: ActiveValue::Set(new.schedule.to_json()),
        alert_email: ActiveValue::Set(new.alert_email),
        notification_condition: ActiveValue::Set(new.notification_condition),
        properties: ActiveValue::Set(serde_json::to_value(&new.properties).unwrap_or_default()),
        next_run_at: ActiveValue::Set(
            next_run(new.enabled, &new.schedule, now).map(|next| next.fixed_offset()),
        ),
        updated_at: ActiveValue::Set(now.fixed_offset()),
        ..Default::default()
    };
    task::Entity::update(row)
        .exec(db)
        .await
        .map_err(|error| duplicate(error, &new.name))
}

pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<bool, DbErr> {
    Ok(task::Entity::delete_by_id(id).exec(db).await?.rows_affected > 0)
}

#[derive(Clone, Debug, Default)]
pub struct Runs {
    pub active: Option<TaskRun>,
    pub last_finished: Option<TaskRun>,
}

impl Runs {
    pub fn last_started(&self) -> Option<DateTime<Utc>> {
        self.active
            .as_ref()
            .and_then(|run| run.started_at)
            .or_else(|| self.last_finished.as_ref().and_then(|run| run.started_at))
            .map(|at| at.with_timezone(&Utc))
    }

    pub fn running(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|run| run.state == RunState::Running.as_str())
    }

    pub fn queued(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|run| run.state == RunState::Waiting.as_str())
    }
}

pub async fn runs_of(
    db: &impl ConnectionTrait,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, Runs>, DbErr> {
    let mut found: HashMap<Uuid, Runs> = HashMap::new();
    if ids.is_empty() {
        return Ok(found);
    }
    let active = TaskRun::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT * FROM task_run WHERE task_id = ANY($1) AND state IN ('waiting', 'running')",
        [ids.to_vec().into()],
    ))
    .all(db)
    .await?;
    for run in active {
        let task = run.task_id;
        found.entry(task).or_default().active = Some(run);
    }
    let finished = TaskRun::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT DISTINCT ON (task_id) * FROM task_run \
         WHERE task_id = ANY($1) AND state IN ('done', 'failed', 'cancelled') \
         ORDER BY task_id, id DESC",
        [ids.to_vec().into()],
    ))
    .all(db)
    .await?;
    for run in finished {
        let task = run.task_id;
        found.entry(task).or_default().last_finished = Some(run);
    }
    Ok(found)
}

pub async fn history(db: &impl ConnectionTrait, task: Uuid) -> Result<Vec<TaskRun>, DbErr> {
    task_run::Entity::find()
        .filter(task_run::Column::TaskId.eq(task))
        .order_by_desc(task_run::Column::Id)
        .limit(KEPT_RUNS as u64)
        .all(db)
        .await
}

pub async fn run(db: &impl ConnectionTrait, id: i64) -> Result<Option<TaskRun>, DbErr> {
    task_run::Entity::find_by_id(id).one(db).await
}

pub async fn log(
    db: &impl ConnectionTrait,
    run: i64,
    after: i32,
    limit: u64,
) -> Result<Vec<LogLine>, DbErr> {
    task_log::Entity::find()
        .filter(task_log::Column::RunId.eq(run))
        .filter(task_log::Column::Line.gt(after))
        .order_by_asc(task_log::Column::Line)
        .limit(limit)
        .all(db)
        .await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requested {
    Queued(i64),
    AlreadyActive,
}

pub async fn request_run(
    db: &impl ConnectionTrait,
    task: Uuid,
    trigger: &str,
    initiator: &str,
) -> Result<Requested, DbErr> {
    #[derive(FromQueryResult)]
    struct Inserted {
        id: i64,
    }
    let inserted = Inserted::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO task_run (task_id, state, trigger, initiator) VALUES ($1, 'waiting', $2, $3) \
         ON CONFLICT (task_id) WHERE state IN ('waiting', 'running') DO NOTHING RETURNING id",
        [task.into(), trigger.into(), initiator.into()],
    ))
    .one(db)
    .await?;
    Ok(match inserted {
        Some(inserted) => Requested::Queued(inserted.id),
        None => Requested::AlreadyActive,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopped {
    Dequeued(i64),
    Signalled(i64),
    NotRunning,
}

pub async fn request_stop(db: &impl ConnectionTrait, task: Uuid) -> Result<Stopped, DbErr> {
    #[derive(FromQueryResult)]
    struct Changed {
        id: i64,
        state: String,
    }
    let changed = Changed::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_run SET cancel_requested = true, \
         state = CASE WHEN state = 'waiting' THEN 'cancelled' ELSE state END, \
         finished_at = CASE WHEN state = 'waiting' THEN now() ELSE finished_at END, \
         message = CASE WHEN state = 'waiting' THEN 'stopped before it started' ELSE message END \
         WHERE task_id = $1 AND state IN ('waiting', 'running') RETURNING id, state",
        [task.into()],
    ))
    .one(db)
    .await?;
    Ok(match changed {
        Some(changed) if changed.state == RunState::Cancelled.as_str() => {
            Stopped::Dequeued(changed.id)
        }
        Some(changed) => Stopped::Signalled(changed.id),
        None => Stopped::NotRunning,
    })
}

pub async fn prune(db: &impl ConnectionTrait, task: Uuid) -> Result<u64, DbErr> {
    let deleted = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM task_run WHERE task_id = $1 AND state IN ('done', 'failed', 'cancelled') \
             AND id NOT IN (SELECT id FROM task_run WHERE task_id = $1 ORDER BY id DESC LIMIT $2)",
            [task.into(), KEPT_RUNS.into()],
        ))
        .await?;
    Ok(deleted.rows_affected())
}
