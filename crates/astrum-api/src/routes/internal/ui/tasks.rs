use astrum_tasks::store::{self, Runs, TaskRun};
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{UI, time};
use crate::error::ApiError;
use crate::routes::tasks::{READ, creatable, existing, template, type_name};
use crate::{AppState, Caller};

const LOG_PAGE: u64 = 500;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/tasks"), get(list))
        .route(&format!("{UI}/tasks/types"), get(types))
        .route(&format!("{UI}/tasks/choices"), get(choices))
        .route(&format!("{UI}/tasks/{{id}}"), get(read))
        .route(&format!("{UI}/tasks/{{id}}/runs"), get(runs))
        .route(&format!("{UI}/tasks/{{id}}/runs/{{run}}/log"), get(log))
}

pub fn run_view(run: &TaskRun) -> Value {
    let duration = match (run.started_at, run.finished_at) {
        (Some(started), Some(finished)) => Some((finished - started).num_milliseconds()),
        (Some(started), None) => Some((Utc::now().fixed_offset() - started).num_milliseconds()),
        _ => None,
    };
    json!({
        "id": run.id,
        "state": run.state,
        "trigger": run.trigger,
        "initiator": run.initiator,
        "node": run.node,
        "message": run.message,
        "counts": run.counts,
        "cancelRequested": run.cancel_requested,
        "queuedAt": time(run.queued_at),
        "startedAt": run.started_at.map(time),
        "finishedAt": run.finished_at.map(time),
        "durationMs": duration,
    })
}

fn view(state: &AppState, task: &store::Task, runs: &Runs) -> Value {
    let status = match (runs.running(), runs.queued()) {
        (true, _) => "running",
        (false, true) => "queued",
        _ => "waiting",
    };
    json!({
        "id": task.id,
        "name": task.name,
        "type": task.kind,
        "typeName": type_name(state.tasks(), &task.kind),
        "known": state.tasks().kind(&task.kind).is_some(),
        "enabled": task.enabled,
        "frequency": task.frequency,
        "properties": store::properties_of(task),
        "alertEmail": task.alert_email,
        "notificationCondition": task.notification_condition,
        "nextRun": task.next_run_at.map(time),
        "state": status,
        "current": runs.active.as_ref().map(run_view),
        "lastRun": runs.last_finished.as_ref().map(run_view),
        "createdAt": time(task.created_at),
        "updatedAt": time(task.updated_at),
    })
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let tasks = store::list(state.db().conn()).await?;
    let ids: Vec<Uuid> = tasks.iter().map(|task| task.id).collect();
    let runs = store::runs_of(state.db().conn(), &ids).await?;
    Ok(Json(Value::Array(
        tasks
            .iter()
            .map(|task| {
                view(
                    &state,
                    task,
                    &runs.get(&task.id).cloned().unwrap_or_default(),
                )
            })
            .collect(),
    )))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let task = existing(&state, &id).await?;
    let runs = crate::routes::tasks::runs_of(&state, &task).await?;
    let mut document = view(&state, &task, &runs);
    if let (Value::Object(map), Some(kind)) = (&mut document, state.tasks().kind(&task.kind)) {
        map.insert("template".to_owned(), template(kind.as_ref()));
    }
    Ok(Json(document))
}

async fn types(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let creatable: Vec<&'static str> = creatable(&state)
        .await?
        .iter()
        .map(|kind| kind.id())
        .collect();
    Ok(Json(Value::Array(
        state
            .tasks()
            .kinds()
            .iter()
            .map(|kind| {
                let mut document = template(kind.as_ref());
                if let Value::Object(map) = &mut document {
                    map.insert(
                        "creatable".to_owned(),
                        creatable.contains(&kind.id()).into(),
                    );
                    map.insert("singleton".to_owned(), kind.singleton().into());
                }
                document
            })
            .collect(),
    )))
}

async fn choices(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let choices = state.tasks().choices().await.map_err(ApiError::internal)?;
    Ok(Json(json!({
        "repositories": choices
            .repositories
            .iter()
            .map(|(name, format, kind)| json!({"name": name, "format": format, "type": kind}))
            .collect::<Vec<_>>(),
        "blobStores": choices
            .blob_stores
            .iter()
            .map(|(name, group)| json!({"name": name, "group": group}))
            .collect::<Vec<_>>(),
    })))
}

async fn runs(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let task = existing(&state, &id).await?;
    let history = store::history(state.db().conn(), task.id).await?;
    Ok(Json(Value::Array(history.iter().map(run_view).collect())))
}

#[derive(Deserialize)]
struct LogQuery {
    after: Option<i32>,
}

async fn log(
    State(state): State<AppState>,
    caller: Caller,
    Path((id, run)): Path<(String, i64)>,
    Query(query): Query<LogQuery>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let task = existing(&state, &id).await?;
    let found = store::run(state.db().conn(), run)
        .await?
        .filter(|found| found.task_id == task.id)
        .ok_or_else(|| ApiError::not_found(format!("the task has no run {run}")))?;
    let lines = store::log(state.db().conn(), run, query.after.unwrap_or(0), LOG_PAGE).await?;
    let finished = store::RunState::parse(&found.state).is_some_and(store::RunState::finished);
    Ok(Json(json!({
        "run": run_view(&found),
        "finished": finished,
        "more": lines.len() as u64 == LOG_PAGE,
        "lines": lines
            .iter()
            .map(|line| json!({
                "line": line.line,
                "at": time(line.at),
                "level": line.level,
                "message": line.message,
            }))
            .collect::<Vec<_>>(),
    })))
}
