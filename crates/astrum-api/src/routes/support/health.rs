use std::collections::BTreeMap;
use std::future::Future;
use std::path::Path;
use std::time::{Duration, Instant};

use astrum_blob::quota;
use astrum_db::read_only;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Value, json};

use crate::routes::blobstores::measure_all;
use crate::routes::internal::ui::UI;
use crate::{ApiError, AppState, Caller};

pub const READ: &str = "astrum:metrics:read";
pub const MIN_FREE_SPACE: u64 = 1024 * 1024 * 1024;
const SCHEDULER_SILENCE: chrono::Duration = chrono::Duration::seconds(60);
const CHECK_TIMEOUT: Duration = Duration::from_secs(10);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/service/rest/v1/status/check", get(checks))
        .route(&format!("{UI}/status-check"), get(checks))
}

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub healthy: bool,
    pub message: Option<String>,
    pub error: Option<String>,
    pub details: Option<Value>,
    pub time: i64,
    pub duration: i64,
    pub timestamp: String,
}

#[derive(Clone)]
struct Outcome {
    healthy: bool,
    message: String,
    details: Option<Value>,
}

fn good(message: impl Into<String>) -> Outcome {
    Outcome {
        healthy: true,
        message: message.into(),
        details: None,
    }
}

fn bad(message: impl Into<String>) -> Outcome {
    Outcome {
        healthy: false,
        message: message.into(),
        details: None,
    }
}

async fn checks(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<BTreeMap<String, Check>>, ApiError> {
    caller.require(READ)?;
    Ok(axum::Json(run(&state).await))
}

pub async fn run(state: &AppState) -> BTreeMap<String, Check> {
    let (cpus, disk, stores, database, writes, admin, role, logs, scheduler) = tokio::join!(
        timed(async { vec![cpus()] }),
        timed(async { vec![disk(state.data_dir()).await] }),
        timed(blob_stores(state)),
        timed(async { vec![database(state).await] }),
        timed(async { vec![writable(state).await] }),
        timed(async { vec![admin_password(state.data_dir())] }),
        timed(async { vec![default_role(state)] }),
        timed(async { vec![log_files(state)] }),
        timed(async { vec![scheduler(state)] }),
    );
    let names: [&[&str]; 9] = [
        &["Available CPUs"],
        &["Available Disk Space"],
        &["Blob Stores Quota", "Blob Stores Ready"],
        &["Database"],
        &["Database Writes"],
        &["Default Admin Credentials"],
        &["Default Role Realm"],
        &["Log Files"],
        &["Scheduler"],
    ];
    let results = [
        cpus, disk, stores, database, writes, admin, role, logs, scheduler,
    ];
    let mut checks = BTreeMap::new();
    for (names, results) in names.into_iter().zip(results) {
        for (name, check) in names.iter().zip(results) {
            checks.insert((*name).to_owned(), check);
        }
    }
    checks
}

async fn timed(check: impl Future<Output = Vec<Outcome>>) -> Vec<Check> {
    let started = Instant::now();
    let now = Utc::now();
    let outcomes = match tokio::time::timeout(CHECK_TIMEOUT, check).await {
        Ok(outcomes) => outcomes,
        Err(_) => vec![bad("The check did not finish in time"); 2],
    };
    let duration = started.elapsed().as_millis() as i64;
    outcomes
        .into_iter()
        .map(|outcome| Check {
            healthy: outcome.healthy,
            message: Some(outcome.message),
            error: None,
            details: outcome.details,
            time: now.timestamp_millis(),
            duration,
            timestamp: now.to_rfc3339_opts(SecondsFormat::Millis, true),
        })
        .collect()
}

fn cpus() -> Outcome {
    let count = std::thread::available_parallelism().map_or(1, usize::from);
    good(format!("The host gives the server {count} CPU cores."))
}

pub fn space(path: &Path) -> Option<(u64, u64)> {
    let stats = rustix::fs::statvfs(path).ok()?;
    Some((
        stats.f_blocks.saturating_mul(stats.f_frsize),
        stats.f_bavail.saturating_mul(stats.f_frsize),
    ))
}

async fn disk(data_dir: &Path) -> Outcome {
    let dir = data_dir.to_owned();
    let measured = tokio::task::spawn_blocking(move || space(&dir))
        .await
        .ok()
        .flatten();
    let shown = data_dir.display();
    match measured {
        Some((total, free)) => {
            let text = format!(
                "{} of {} free in the data directory {shown}.",
                quota::human_size(free),
                quota::human_size(total)
            );
            let mut outcome = match free >= MIN_FREE_SPACE {
                true => good(text),
                false => bad(format!(
                    "{text} Keep at least {} free.",
                    quota::human_size(MIN_FREE_SPACE)
                )),
            };
            outcome.details = Some(json!({ "totalBytes": total, "freeBytes": free }));
            outcome
        }
        None => bad(format!(
            "The free space of the data directory {shown} cannot be measured."
        )),
    }
}

async fn blob_stores(state: &AppState) -> Vec<Outcome> {
    let measured = match measure_all(state).await {
        Ok(measured) => measured,
        Err(_) => {
            let failed = bad("The blob stores cannot be read from the database.");
            return vec![failed.clone(), failed];
        }
    };
    let total = measured.len();
    let mut violations = Vec::new();
    let mut unavailable = Vec::new();
    for (config, measured) in &measured {
        let status = quota::status(
            &config.name,
            config.soft_quota.as_ref(),
            measured.totals,
            measured.available,
        );
        if status.violation {
            violations.push(json!({ "blobStore": config.name, "message": status.message }));
        }
        if measured.unavailable {
            unavailable.push(json!({ "blobStore": config.name }));
        }
    }
    let quota = Outcome {
        healthy: violations.is_empty(),
        message: format!(
            "{}/{total} blob stores violate their quota.",
            violations.len()
        ),
        details: (!violations.is_empty()).then_some(Value::Array(violations)),
    };
    let ready = Outcome {
        healthy: unavailable.is_empty(),
        message: format!("{}/{total} blob stores are unavailable.", unavailable.len()),
        details: (!unavailable.is_empty()).then_some(Value::Array(unavailable)),
    };
    vec![quota, ready]
}

async fn database(state: &AppState) -> Outcome {
    match state.db().ping().await {
        Ok(()) => good("PostgreSQL answers."),
        Err(error) => bad(format!("PostgreSQL does not answer: {error}")),
    }
}

async fn writable(state: &AppState) -> Outcome {
    let db = state.db();
    match db.accepts_writes().await {
        Ok(true) => {}
        Ok(false) => {
            return bad("PostgreSQL accepts no writes (a standby or a read-only transaction).");
        }
        Err(error) => return bad(format!("PostgreSQL does not answer: {error}")),
    }
    match read_only::state(db.conn()).await {
        Ok(mode) if mode.frozen => bad(format!(
            "Read-only mode is on: {}",
            mode.reason.as_deref().unwrap_or("no reason given")
        )),
        Ok(_) => good("The database accepts writes."),
        Err(error) => bad(format!("The read-only mode cannot be read: {error}")),
    }
}

fn admin_password(data_dir: &Path) -> Outcome {
    match data_dir.join("admin.password").exists() {
        true => bad(
            "The generated password of the admin user is still in admin.password in the data \
             directory. Sign in, change the password, and the file is removed.",
        ),
        false => good("The generated admin password was changed."),
    }
}

fn default_role(state: &AppState) -> Outcome {
    let model = state.security().model();
    if !model.realm_active(astrum_security::realm::DEFAULT_ROLE) {
        return good("The Default Role realm is not in use.");
    }
    match model.default_role.as_deref() {
        None | Some("") => bad("The Default Role realm is active but no role is chosen."),
        Some(role) if !model.roles.contains_key(role) => bad(format!(
            "The Default Role realm grants the role {role}, which does not exist."
        )),
        Some(role) => good(format!("The Default Role realm grants the role {role}.")),
    }
}

fn log_files(state: &AppState) -> Outcome {
    match state.logging().files() {
        Some(files) => good(format!("Logs are written to {}.", files.dir().display())),
        None => bad("Logs go to the standard error only; the log directory cannot be written."),
    }
}

fn scheduler(state: &AppState) -> Outcome {
    match state.tasks().last_tick() {
        Some(at) if Utc::now() - at <= SCHEDULER_SILENCE => {
            good("The task scheduler looks for due tasks.")
        }
        Some(at) => bad(format!(
            "The task scheduler last looked for due tasks at {}.",
            at.to_rfc3339_opts(SecondsFormat::Secs, true)
        )),
        None => bad("The task scheduler is not running on this node."),
    }
}
