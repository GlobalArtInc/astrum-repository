use std::collections::HashMap;

use astrum_db::audit::Event;
use astrum_tasks::notification::{CONDITIONS, FAILURE};
use astrum_tasks::schedule::{FrequencyRequest, Schedule, public_time};
use astrum_tasks::store::{self, NewTask, Requested, Runs, Stopped, StoreError, Task};
use astrum_tasks::{Problem, Problems, TaskKind, Tasks, validate};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::error::{ApiError, FieldError};
use crate::{AppState, Caller};

pub const TASKS: &str = "/service/rest/v1/tasks";
pub const DOMAIN: &str = "tasks";
pub const READ: &str = "astrum:tasks:read";
pub const CREATE: &str = "astrum:tasks:create";
pub const UPDATE: &str = "astrum:tasks:update";
pub const DELETE: &str = "astrum:tasks:delete";
pub const START: &str = "astrum:tasks:start";
pub const STOP: &str = "astrum:tasks:stop";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(TASKS, get(list).post(create))
        .route("/service/rest/v1/tasks/templates", get(templates))
        .route("/service/rest/v1/tasks/templates/{kind}", get(template_of))
        .route(
            "/service/rest/v1/tasks/{id}",
            get(read).put(update).delete(remove),
        )
        .route("/service/rest/v1/tasks/{id}/run", post(run))
        .route("/service/rest/v1/tasks/{id}/stop", post(stop))
}

pub fn problems(problems: Vec<Problem>) -> ApiError {
    ApiError::Invalid(
        problems
            .into_iter()
            .map(|problem| FieldError {
                id: problem.id,
                message: problem.message,
            })
            .collect(),
    )
}

pub fn status(code: StatusCode, message: impl Into<String>) -> Response {
    (code, Json(json!({"id": "*", "message": message.into()}))).into_response()
}

fn not_found(id: &str) -> ApiError {
    ApiError::not_found(format!("there is no task with the id {id}"))
}

pub fn parse_id(id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|_| not_found(id))
}

pub async fn existing(state: &AppState, id: &str) -> Result<Task, ApiError> {
    let parsed = parse_id(id)?;
    store::find(state.db().conn(), parsed)
        .await?
        .ok_or_else(|| not_found(id))
}

fn sentence(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

pub fn type_name(tasks: &Tasks, kind: &str) -> String {
    tasks
        .kind(kind)
        .map(|kind| kind.name().to_owned())
        .unwrap_or_else(|| kind.to_owned())
}

pub fn document(tasks: &Tasks, task: &Task, runs: &Runs) -> Value {
    let schedule = store::schedule_of(task);
    let finished = runs.last_finished.as_ref();
    let result = finished
        .and_then(|run| store::RunState::parse(&run.state))
        .and_then(store::RunState::public_result);
    let state = match (runs.running(), &schedule.kind) {
        (true, _) => "RUNNING",
        (false, astrum_tasks::schedule::Kind::Once)
            if finished.is_some() && task.next_run_at.is_none() =>
        {
            "DONE"
        }
        _ => "WAITING",
    };
    let message = tasks
        .kind(&task.kind)
        .map(|kind| sentence(&kind.describe(&store::properties_of(task))));
    let start = match schedule.kind {
        astrum_tasks::schedule::Kind::Manual => None,
        _ => schedule.start.map(public_time),
    };
    json!({
        "id": task.id,
        "name": task.name,
        "type": task.kind,
        "typeName": type_name(tasks, &task.kind),
        "message": message,
        "currentState": state,
        "lastRunResult": result,
        "nextRun": task.next_run_at.map(|at| public_time(at.with_timezone(&Utc))),
        "lastRun": runs.last_started().map(public_time),
        "schedule": schedule.kind.public_name(),
        "properties": store::properties_of(task),
        "enabled": task.enabled,
        "alertEmail": task.alert_email,
        "notificationCondition": task.notification_condition,
        "startDate": start,
        "recurringDays": match schedule.days.is_empty() {
            true => Value::Null,
            false => json!(schedule.days),
        },
        "cronExpression": schedule.cron.as_ref().map(ToString::to_string),
        "timeZoneOffset": schedule.public_offset(),
    })
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "type")]
    kind: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let tasks = store::list(state.db().conn())
        .await?
        .into_iter()
        .filter(|task| query.kind.as_deref().is_none_or(|kind| task.kind == kind))
        .collect::<Vec<_>>();
    let ids: Vec<Uuid> = tasks.iter().map(|task| task.id).collect();
    let runs = store::runs_of(state.db().conn(), &ids).await?;
    let items: Vec<Value> = tasks
        .iter()
        .map(|task| {
            document(
                state.tasks(),
                task,
                &runs.get(&task.id).cloned().unwrap_or_default(),
            )
        })
        .collect();
    Ok(Json(json!({"items": items, "continuationToken": null})))
}

pub async fn runs_of(state: &AppState, task: &Task) -> Result<Runs, ApiError> {
    Ok(store::runs_of(state.db().conn(), &[task.id])
        .await?
        .remove(&task.id)
        .unwrap_or_default())
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let task = existing(&state, &id).await?;
    let runs = runs_of(&state, &task).await?;
    Ok(Json(document(state.tasks(), &task, &runs)))
}

pub fn template(kind: &dyn TaskKind) -> Value {
    let fields = kind.fields();
    let properties: Map<String, Value> = fields
        .iter()
        .map(|field| {
            (
                field.id.to_owned(),
                Value::String(field.initial.unwrap_or_default().to_owned()),
            )
        })
        .collect();
    json!({
        "type": kind.id(),
        "name": kind.name(),
        "enabled": true,
        "alertEmail": null,
        "notificationCondition": FAILURE,
        "frequency": {
            "schedule": "manual",
            "startDate": null,
            "timeZoneOffset": null,
            "recurringDays": null,
            "cronExpression": null,
        },
        "properties": properties,
        "formFields": fields.iter().map(|field| field.template()).collect::<Vec<_>>(),
        "concurrentRun": false,
    })
}

pub async fn creatable(state: &AppState) -> Result<Vec<std::sync::Arc<dyn TaskKind>>, ApiError> {
    let tasks = store::list(state.db().conn()).await?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for task in &tasks {
        *counts.entry(task.kind.clone()).or_default() += 1;
    }
    Ok(state
        .tasks()
        .kinds()
        .into_iter()
        .filter(|kind| !kind.singleton() || !counts.contains_key(kind.id()))
        .collect())
}

async fn templates(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let kinds = creatable(&state).await?;
    Ok(Json(Value::Array(
        kinds.iter().map(|kind| template(kind.as_ref())).collect(),
    )))
}

async fn template_of(
    State(state): State<AppState>,
    caller: Caller,
    Path(kind): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let found = state
        .tasks()
        .kind(&kind)
        .ok_or_else(|| ApiError::not_found(format!("there is no task type {kind}")))?;
    Ok(Json(template(found.as_ref())))
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequest {
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub alert_email: Option<Value>,
    #[serde(default)]
    pub notification_condition: Option<String>,
    #[serde(default)]
    pub frequency: Option<FrequencyRequest>,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
}

async fn prepare(
    state: &AppState,
    kind: &dyn TaskKind,
    request: &TaskRequest,
    stored: Option<&Task>,
) -> Result<NewTask, ApiError> {
    let mut problems = Problems::default();
    let name = match (&request.name, stored) {
        (Some(name), _) => name.trim().to_owned(),
        (None, Some(stored)) => stored.name.clone(),
        (None, None) => String::new(),
    };
    if name.is_empty() {
        problems.add("name", "give the task a name");
    } else if name.chars().count() > 200 || name.chars().any(char::is_control) {
        problems.add(
            "name",
            "must be at most 200 characters without control characters",
        );
    }
    let schedule = match (&request.frequency, stored) {
        (Some(frequency), _) => Schedule::from_request(frequency, Utc::now(), &mut problems),
        (None, Some(stored)) => Some(store::schedule_of(stored)),
        (None, None) => Some(Schedule::manual()),
    };
    let alert_email = match (&request.alert_email, stored) {
        (Some(Value::String(text)), _) if !text.trim().is_empty() => {
            let text = text.trim().to_owned();
            if !validate::email(&text) {
                problems.add("alertEmail", format!("{text} is not an email address"));
            }
            Some(text)
        }
        (Some(Value::String(_) | Value::Null), _) => None,
        (Some(_), _) => {
            problems.add("alertEmail", "must be an email address");
            None
        }
        (None, Some(stored)) => stored.alert_email.clone(),
        (None, None) => None,
    };
    let condition = match (&request.notification_condition, stored) {
        (Some(condition), _) => condition.trim().to_uppercase(),
        (None, Some(stored)) => stored.notification_condition.clone(),
        (None, None) => FAILURE.to_owned(),
    };
    if !CONDITIONS.contains(&condition.as_str()) {
        problems.add("notificationCondition", "use FAILURE or SUCCESS_FAILURE");
    }
    let choices = state.tasks().choices().await.map_err(ApiError::internal)?;
    let stored_properties = stored.map(store::properties_of);
    let properties = validate::properties(
        kind,
        request.properties.as_ref(),
        stored_properties.as_ref(),
        &choices,
        &mut problems,
    );
    problems.finish().map_err(self::problems)?;
    Ok(NewTask {
        kind: kind.id().to_owned(),
        name,
        enabled: request
            .enabled
            .unwrap_or_else(|| stored.is_none_or(|stored| stored.enabled)),
        schedule: schedule.unwrap_or_else(Schedule::manual),
        alert_email,
        notification_condition: condition,
        properties,
    })
}

fn duplicate(error: StoreError) -> Result<Response, ApiError> {
    match error {
        StoreError::Duplicate(name) => Ok(status(
            StatusCode::CONFLICT,
            format!("a task named \"{name}\" exists already"),
        )),
        StoreError::Database(error) => Err(error.into()),
    }
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<TaskRequest>, ApiError>,
) -> Result<Response, ApiError> {
    caller.require(CREATE)?;
    let crate::Json(request) = body?;
    let Some(kind_id) = request
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|kind| !kind.is_empty())
    else {
        return Err(ApiError::invalid(
            "type",
            "say which type of task to create",
        ));
    };
    let kind = state
        .tasks()
        .kind(kind_id)
        .ok_or_else(|| ApiError::invalid("type", format!("{kind_id} is not a task type")))?;
    if kind.singleton() && store::count_of_kind(state.db().conn(), kind.id()).await? > 0 {
        return Err(ApiError::invalid(
            "type",
            format!("a task of the type {kind_id} exists already; there can be only one"),
        ));
    }
    let new = prepare(&state, kind.as_ref(), &request, None).await?;
    let task = match store::create(state.db().conn(), new).await {
        Ok(task) => task,
        Err(error) => return duplicate(error),
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "created", &task.name)
                .with(json!({"type": task.kind, "id": task.id})),
        )
        .await;
    state.tasks().wake();
    Ok((
        StatusCode::CREATED,
        Json(document(state.tasks(), &task, &Runs::default())),
    )
        .into_response())
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Result<crate::Json<TaskRequest>, ApiError>,
) -> Result<Response, ApiError> {
    caller.require(UPDATE)?;
    let crate::Json(request) = body?;
    let stored = existing(&state, &id).await?;
    if request
        .kind
        .as_deref()
        .is_some_and(|kind| !kind.trim().is_empty() && kind.trim() != stored.kind)
    {
        return Err(ApiError::invalid(
            "type",
            "the type of a task cannot change",
        ));
    }
    let kind = state.tasks().kind(&stored.kind).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "this server cannot run tasks of the type {}, so it cannot change them",
            stored.kind
        ))
    })?;
    let new = prepare(&state, kind.as_ref(), &request, Some(&stored)).await?;
    let task = match store::update(state.db().conn(), stored.id, new).await {
        Ok(task) => task,
        Err(error) => return duplicate(error),
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &task.name)
                .with(json!({"type": task.kind, "id": task.id})),
        )
        .await;
    state.tasks().wake();
    let runs = runs_of(&state, &task).await?;
    Ok(Json(document(state.tasks(), &task, &runs)).into_response())
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let task = existing(&state, &id).await?;
    if let Stopped::Signalled(run) = store::request_stop(state.db().conn(), task.id).await? {
        state.tasks().signal_stop(run);
    }
    store::delete(state.db().conn(), task.id).await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "deleted", &task.name)
                .with(json!({"type": task.kind, "id": task.id})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn run(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    caller.require(START)?;
    let task = existing(&state, &id).await?;
    if !task.enabled {
        return Ok(status(
            StatusCode::METHOD_NOT_ALLOWED,
            format!("the task \"{}\" is disabled", task.name),
        ));
    }
    match store::request_run(state.db().conn(), task.id, "manual", &caller.initiator()).await? {
        Requested::Queued(run) => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "started", &task.name)
                        .with(json!({"run": run, "id": task.id})),
                )
                .await;
            state.tasks().wake();
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Requested::AlreadyActive => Ok(status(
            StatusCode::CONFLICT,
            format!(
                "the task \"{}\" is already running or about to run",
                task.name
            ),
        )),
    }
}

async fn stop(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    caller.require(STOP)?;
    let task = existing(&state, &id).await?;
    let run = match store::request_stop(state.db().conn(), task.id).await? {
        Stopped::NotRunning => {
            return Ok(status(
                StatusCode::CONFLICT,
                format!("the task \"{}\" is not running", task.name),
            ));
        }
        Stopped::Dequeued(run) => run,
        Stopped::Signalled(run) => {
            state.tasks().signal_stop(run);
            run
        }
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "stopped", &task.name).with(json!({"run": run, "id": task.id})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
