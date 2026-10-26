use std::collections::BTreeMap;

use axum::Router;
use axum::extract::State;
use axum::routing::get;
use chrono::{SecondsFormat, Utc};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement};
use serde_json::{Map, Value, json};

use crate::routes::blobstores::measure_all;
use crate::routes::support::health::space;
use crate::{ApiError, AppState, Caller};

pub const READ: &str = "astrum:atlas:read";
pub const PATH: &str = "/service/rest/atlas/system-information";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REVISION: Option<&str> = option_env!("ASTRUM_GIT_REVISION");

pub fn routes() -> Router<AppState> {
    Router::new().route(PATH, get(read))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Value>, ApiError> {
    caller.require(READ)?;
    Ok(axum::Json(collect(&state).await))
}

pub async fn collect(state: &AppState) -> Value {
    let mut report = Map::new();
    report.insert("astrum-status".into(), status());
    report.insert("astrum-node".into(), node(state));
    report.insert("astrum-configuration".into(), directories(state));
    report.insert("astrum-properties".into(), properties(state));
    report.insert("system-time".into(), time());
    report.insert("system-runtime".into(), runtime(state));
    report.insert("system-os".into(), os());
    report.insert("system-database".into(), database(state).await);
    report.insert("system-blobstores".into(), blob_stores(state).await);
    report.insert("system-filestores".into(), filestores(state));
    Value::Object(report)
}

fn status() -> Value {
    json!({
        "version": VERSION,
        "buildRevision": REVISION.filter(|revision| !revision.is_empty()).unwrap_or("unknown"),
        "rustVersion": env!("CARGO_PKG_RUST_VERSION"),
    })
}

fn node(state: &AppState) -> Value {
    json!({
        "node-id": state.logging().node(),
        "process-id": std::process::id(),
        "task-node": state.tasks().node(),
    })
}

fn directories(state: &AppState) -> Value {
    let data = state.data_dir();
    let mut entries = json!({
        "dataDirectory": data.display().to_string(),
        "temporaryDirectory": data.join("tmp").display().to_string(),
        "blobsDirectory": state.blob_stores().blobs_dir().display().to_string(),
    });
    if let Some(files) = state.logging().files() {
        entries["logDirectory"] = json!(files.dir().display().to_string());
    }
    entries
}

fn properties(state: &AppState) -> Value {
    let mut entries: BTreeMap<String, Value> = state
        .configuration()
        .iter()
        .map(|(key, value)| (key.clone(), json!(value)))
        .collect();
    entries.insert("log.filter".into(), json!(state.logging().filter()));
    json!(entries)
}

fn time() -> Value {
    let now = Utc::now();
    json!({
        "current": now.timestamp_millis(),
        "timezone": std::env::var("TZ").unwrap_or_else(|_| "UTC".to_owned()),
        "iso8601": now.to_rfc3339_opts(SecondsFormat::Millis, true),
    })
}

fn runtime(state: &AppState) -> Value {
    let status = proc_fields("/proc/self/status");
    let kib = |key: &str| {
        status
            .get(key)
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|value| value * 1024)
    };
    let started = state.started();
    json!({
        "availableProcessors": std::thread::available_parallelism().map_or(1, usize::from),
        "residentMemory": kib("VmRSS"),
        "peakResidentMemory": kib("VmHWM"),
        "virtualMemory": kib("VmSize"),
        "threads": status.get("Threads").and_then(|value| value.parse::<u64>().ok()),
        "startTime": started.to_rfc3339_opts(SecondsFormat::Secs, true),
        "uptimeSeconds": (Utc::now() - started).num_seconds(),
    })
}

fn os() -> Value {
    let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let pretty = release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|name| name.trim_matches('"').to_owned());
    let kernel = |name: &str| {
        std::fs::read_to_string(format!("/proc/sys/kernel/{name}"))
            .ok()
            .map(|value| value.trim().to_owned())
    };
    let memory = proc_fields("/proc/meminfo");
    let kib = |key: &str| {
        memory
            .get(key)
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|value| value * 1024)
    };
    let load = std::fs::read_to_string("/proc/loadavg").ok().map(|value| {
        value
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    });
    json!({
        "name": std::env::consts::OS,
        "distribution": pretty,
        "kernel": kernel("osrelease"),
        "architecture": std::env::consts::ARCH,
        "hostname": kernel("hostname"),
        "totalMemory": kib("MemTotal"),
        "availableMemory": kib("MemAvailable"),
        "loadAverage": load,
    })
}

fn proc_fields(path: &str) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

#[derive(FromQueryResult)]
struct DatabaseFacts {
    version: String,
    name: String,
    size: i64,
    standby: bool,
}

async fn database(state: &AppState) -> Value {
    let db = state.db();
    let facts = DatabaseFacts::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        "SELECT current_setting('server_version') AS version, current_database() AS name, \
         pg_database_size(current_database()) AS size, pg_is_in_recovery() AS standby",
    ))
    .one(db.conn())
    .await;
    let migrations = db
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS applied FROM migration",
        ))
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<i64>("", "applied").ok());
    match facts {
        Ok(Some(facts)) => json!({
            "product": "PostgreSQL",
            "version": facts.version,
            "database": facts.name,
            "schema": db.schema(),
            "sizeBytes": facts.size,
            "standby": facts.standby,
            "appliedMigrations": migrations,
        }),
        Ok(None) => json!({ "product": "PostgreSQL", "schema": db.schema() }),
        Err(error) => json!({
            "product": "PostgreSQL",
            "schema": db.schema(),
            "error": error.to_string(),
        }),
    }
}

async fn blob_stores(state: &AppState) -> Value {
    let Ok(measured) = measure_all(state).await else {
        return json!({});
    };
    let mut stores = Map::new();
    for (config, measured) in measured {
        stores.insert(
            config.name.clone(),
            json!({
                "type": config.kind().as_str(),
                "available": !measured.unavailable,
                "blobCount": measured.totals.blob_count,
                "totalSizeBytes": measured.totals.total_bytes,
                "availableSpaceBytes": measured.available,
                "softQuota": config.soft_quota,
            }),
        );
    }
    Value::Object(stores)
}

fn filestores(state: &AppState) -> Value {
    let mut stores = Map::new();
    let data = state.data_dir();
    if let Some((total, free)) = space(data) {
        stores.insert(
            "dataDirectory".into(),
            json!({
                "path": data.display().to_string(),
                "totalSpace": total,
                "usableSpace": free,
            }),
        );
    }
    Value::Object(stores)
}
