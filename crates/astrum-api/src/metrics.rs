use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use astrum_blob::quota;
use astrum_db::setting;
use astrum_db::system::{PROMETHEUS_KEY, Prometheus};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::routes::blobstores::measure_all;
use crate::routes::support::health;
use crate::{ApiError, AppState, Caller};

pub const PATH: &str = "/service/metrics/prometheus";
pub const CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";
const BUCKETS: [f64; 13] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0,
];
const MAX_REPOSITORIES: usize = 1000;
const COLLECTION_BUDGET: Duration = Duration::from_secs(8);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REVISION: Option<&str> = option_env!("ASTRUM_GIT_REVISION");

#[derive(Default)]
pub struct Registry {
    counts: Mutex<Counts>,
}

#[derive(Default)]
struct Counts {
    requests: BTreeMap<(&'static str, &'static str, &'static str), u64>,
    repositories: HashMap<(String, &'static str, &'static str), u64>,
    names: BTreeSet<String>,
    durations: BTreeMap<&'static str, Histogram>,
}

#[derive(Clone, Default)]
struct Histogram {
    buckets: [u64; BUCKETS.len()],
    sum: f64,
    count: u64,
}

impl Histogram {
    fn observe(&mut self, seconds: f64) {
        for (bucket, bound) in self.buckets.iter_mut().zip(BUCKETS) {
            if seconds <= bound {
                *bucket += 1;
            }
        }
        self.sum += seconds;
        self.count += 1;
    }
}

fn method_label(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::PUT => "PUT",
        Method::POST => "POST",
        Method::DELETE => "DELETE",
        Method::PATCH => "PATCH",
        Method::OPTIONS => "OPTIONS",
        _ => "OTHER",
    }
}

fn status_label(status: StatusCode) -> &'static str {
    match status.as_u16() / 100 {
        1 => "1xx",
        2 => "2xx",
        3 => "3xx",
        4 => "4xx",
        _ => "5xx",
    }
}

fn classify(path: &str) -> (&'static str, Option<&str>) {
    let repository = path
        .strip_prefix("/repository/")
        .or_else(|| path.strip_prefix("/content/repositories/"))
        .or_else(|| path.strip_prefix("/content/groups/"));
    if let Some(rest) = repository {
        let name = rest.split('/').next().filter(|name| !name.is_empty());
        return ("repository", name);
    }
    let area = if path.starts_with("/service/rest/internal/") {
        "internal"
    } else if path.starts_with("/service/rest/") {
        "rest"
    } else if path.starts_with("/service/") {
        "service"
    } else if path == "/mcp"
        || path.starts_with("/mcp/")
        || path.starts_with("/oauth/")
        || path.starts_with("/.well-known/")
    {
        "mcp"
    } else if path == "/v2" || path.starts_with("/v2/") {
        "docker"
    } else {
        "web"
    };
    (area, None)
}

impl Registry {
    fn record(&self, path: &str, method: &'static str, status: StatusCode, elapsed: Duration) {
        let (area, repository) = classify(path);
        let status = status_label(status);
        let mut counts = self.counts.lock().expect("never poisoned");
        *counts.requests.entry((area, method, status)).or_default() += 1;
        counts
            .durations
            .entry(area)
            .or_default()
            .observe(elapsed.as_secs_f64());
        if let Some(name) = repository {
            let name = name.to_lowercase();
            if counts.names.contains(&name) || counts.names.len() < MAX_REPOSITORIES {
                counts.names.insert(name.clone());
                *counts
                    .repositories
                    .entry((name, method, status))
                    .or_default() += 1;
            }
        }
    }
}

pub async fn track(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = method_label(request.method());
    let path = request.uri().path().to_owned();
    let response = next.run(request).await;
    state
        .metrics()
        .record(&path, method, response.status(), started.elapsed());
    response
}

pub fn routes() -> Router<AppState> {
    Router::new().route(PATH, get(scrape))
}

async fn scrape(State(state): State<AppState>, caller: Caller) -> Result<Response, ApiError> {
    let settings = setting::get::<Prometheus>(state.db().conn(), PROMETHEUS_KEY)
        .await?
        .filter(|settings| settings.enabled)
        .ok_or_else(|| ApiError::not_found("Prometheus metrics are not enabled"))?;
    caller.require(health::READ)?;
    let text = render(&state, &settings).await;
    Ok(([(header::CONTENT_TYPE, CONTENT_TYPE)], text).into_response())
}

#[derive(Default)]
struct Exposition {
    text: String,
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

impl Exposition {
    fn family(&mut self, name: &str, kind: &str, help: &str) {
        let _ = writeln!(self.text, "# HELP {name} {help}");
        let _ = writeln!(self.text, "# TYPE {name} {kind}");
    }

    fn sample(&mut self, name: &str, labels: &[(&str, &str)], value: impl Display) {
        self.text.push_str(name);
        if !labels.is_empty() {
            self.text.push('{');
            for (index, (label, value)) in labels.iter().enumerate() {
                if index > 0 {
                    self.text.push(',');
                }
                let _ = write!(self.text, "{label}=\"{}\"", escape(value));
            }
            self.text.push('}');
        }
        let _ = writeln!(self.text, " {value}");
    }
}

fn kind(config: &astrum_blob::config::StoreConfig) -> String {
    config.kind().as_str().to_lowercase()
}

fn flag(value: bool) -> u8 {
    u8::from(value)
}

fn seconds(at: chrono::DateTime<chrono::Utc>) -> f64 {
    at.timestamp_millis() as f64 / 1000.0
}

struct Collected {
    stores: Option<
        Vec<(
            astrum_blob::config::StoreConfig,
            crate::routes::blobstores::Measured,
        )>,
    >,
    checks: BTreeMap<String, health::Check>,
    repositories: Option<Vec<astrum_repo::config::Repository>>,
    tasks: Option<Vec<(astrum_tasks::store::Task, astrum_tasks::store::Runs)>>,
    pending: Option<i64>,
    disk: Option<(u64, u64)>,
    alerts: Vec<(String, String)>,
}

async fn tasks(
    state: &AppState,
) -> Option<Vec<(astrum_tasks::store::Task, astrum_tasks::store::Runs)>> {
    let db = state.db().conn();
    let tasks = astrum_tasks::store::list(db).await.ok()?;
    let ids: Vec<uuid::Uuid> = tasks.iter().map(|task| task.id).collect();
    let mut runs = astrum_tasks::store::runs_of(db, &ids).await.ok()?;
    Some(
        tasks
            .into_iter()
            .map(|task| {
                let run = runs.remove(&task.id).unwrap_or_default();
                (task, run)
            })
            .collect(),
    )
}

async fn pending(state: &AppState) -> Option<i64> {
    let row = state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS pending FROM notification_delivery",
        ))
        .await
        .ok()??;
    row.try_get::<i64>("", "pending").ok()
}

async fn disk(state: &AppState) -> Option<(u64, u64)> {
    let dir = state.data_dir().to_owned();
    tokio::task::spawn_blocking(move || health::space(&dir))
        .await
        .ok()
        .flatten()
}

async fn collect(state: &AppState) -> Collected {
    let (stores, checks, repositories, tasks, pending, disk, alerts) = tokio::join!(
        async { measure_all(state).await.ok() },
        health::run(state),
        async { astrum_repo::store::list(state.db().conn()).await.ok() },
        tasks(state),
        pending(state),
        disk(state),
        crate::space::tracked(state),
    );
    Collected {
        stores,
        checks,
        repositories,
        tasks,
        pending,
        disk,
        alerts,
    }
}

pub async fn render(state: &AppState, settings: &Prometheus) -> String {
    let mut out = Exposition::default();
    out.family(
        "astrum_build_info",
        "gauge",
        "Version of the running server; the value is always 1.",
    );
    out.sample(
        "astrum_build_info",
        &[
            ("version", VERSION),
            ("revision", REVISION.unwrap_or("unknown")),
        ],
        1,
    );
    out.family(
        "astrum_start_time_seconds",
        "gauge",
        "When this server process started, in seconds since the Unix epoch.",
    );
    out.sample(
        "astrum_start_time_seconds",
        &[],
        seconds(state.started_at().to_utc()),
    );

    let collected = tokio::time::timeout(COLLECTION_BUDGET, collect(state)).await;
    out.family(
        "astrum_metrics_collection_complete",
        "gauge",
        "1 when every measurement of this scrape finished in time, 0 when some were left out.",
    );
    out.sample(
        "astrum_metrics_collection_complete",
        &[],
        flag(collected.is_ok()),
    );

    let (requests, repositories_seen, durations) = {
        let counts = state.metrics().counts.lock().expect("never poisoned");
        (
            counts.requests.clone(),
            counts.repositories.clone(),
            counts.durations.clone(),
        )
    };
    out.family(
        "astrum_http_requests_total",
        "counter",
        "HTTP requests answered, by area of the server, method and status class.",
    );
    for ((area, method, status), count) in &requests {
        out.sample(
            "astrum_http_requests_total",
            &[("area", area), ("method", method), ("status", status)],
            count,
        );
    }
    out.family(
        "astrum_http_request_duration_seconds",
        "histogram",
        "Time until the response headers were ready, by area of the server.",
    );
    for (area, histogram) in &durations {
        for (bound, count) in BUCKETS.iter().zip(histogram.buckets) {
            out.sample(
                "astrum_http_request_duration_seconds_bucket",
                &[("area", area), ("le", &bound.to_string())],
                count,
            );
        }
        out.sample(
            "astrum_http_request_duration_seconds_bucket",
            &[("area", area), ("le", "+Inf")],
            histogram.count,
        );
        out.sample(
            "astrum_http_request_duration_seconds_sum",
            &[("area", area)],
            histogram.sum,
        );
        out.sample(
            "astrum_http_request_duration_seconds_count",
            &[("area", area)],
            histogram.count,
        );
    }

    let Ok(collected) = collected else {
        return out.text;
    };

    if settings.repositories
        && let Some(repositories) = &collected.repositories
    {
        let names: HashMap<String, &str> = repositories
            .iter()
            .map(|repository| (repository.name.to_lowercase(), repository.name.as_str()))
            .collect();
        let mut by_repository: BTreeMap<(&str, &str, &str), u64> = BTreeMap::new();
        for ((name, method, status), count) in &repositories_seen {
            if let Some(name) = names.get(name) {
                *by_repository.entry((name, method, status)).or_default() += count;
            }
        }
        out.family(
            "astrum_repository_requests_total",
            "counter",
            "HTTP requests to a repository, by method and status class.",
        );
        for ((repository, method, status), count) in &by_repository {
            out.sample(
                "astrum_repository_requests_total",
                &[
                    ("repository", repository),
                    ("method", method),
                    ("status", status),
                ],
                count,
            );
        }
        out.family(
            "astrum_repository_online",
            "gauge",
            "1 when a repository is online, 0 when it is offline.",
        );
        for repository in repositories {
            out.sample(
                "astrum_repository_online",
                &[
                    ("repository", &repository.name),
                    ("format", repository.format.id),
                    ("type", repository.kind.as_str()),
                ],
                flag(repository.online),
            );
        }
    }

    if let Some(stores) = &collected.stores {
        out.family(
            "astrum_blobstore_up",
            "gauge",
            "1 when a blob store can be opened and measured, 0 when it cannot.",
        );
        for (config, measured) in stores {
            out.sample(
                "astrum_blobstore_up",
                &[("blobstore", &config.name), ("type", &kind(config))],
                flag(!measured.unavailable),
            );
        }
        out.family(
            "astrum_blobstore_blobs",
            "gauge",
            "Blobs kept in a blob store.",
        );
        for (config, measured) in stores {
            out.sample(
                "astrum_blobstore_blobs",
                &[("blobstore", &config.name), ("type", &kind(config))],
                measured.totals.blob_count,
            );
        }
        out.family(
            "astrum_blobstore_size_bytes",
            "gauge",
            "Bytes of the blobs kept in a blob store.",
        );
        for (config, measured) in stores {
            out.sample(
                "astrum_blobstore_size_bytes",
                &[("blobstore", &config.name), ("type", &kind(config))],
                measured.totals.total_bytes,
            );
        }
        out.family(
            "astrum_blobstore_available_bytes",
            "gauge",
            "Free space left to a blob store, where the store can tell it.",
        );
        for (config, measured) in stores {
            if let Some(available) = measured.available {
                out.sample(
                    "astrum_blobstore_available_bytes",
                    &[("blobstore", &config.name), ("type", &kind(config))],
                    available,
                );
            }
        }
        out.family(
            "astrum_blobstore_quota_violation",
            "gauge",
            "1 when a blob store passed its soft quota, 0 when it is within it; only stores with a quota.",
        );
        for (config, measured) in stores {
            if config.soft_quota.is_some() {
                let status = quota::status(
                    &config.name,
                    config.soft_quota.as_ref(),
                    measured.totals,
                    measured.available,
                );
                out.sample(
                    "astrum_blobstore_quota_violation",
                    &[("blobstore", &config.name)],
                    flag(status.violation),
                );
            }
        }
    }

    out.family(
        "astrum_blobstore_space_alert",
        "gauge",
        "1 for every problem the blob store space alert reported and that still lasts.",
    );
    for (store, problem) in &collected.alerts {
        out.sample(
            "astrum_blobstore_space_alert",
            &[("blobstore", store), ("problem", problem)],
            1,
        );
    }

    out.family(
        "astrum_health_check_healthy",
        "gauge",
        "1 when a health check of Settings, Support, Metrics and Health passes, 0 when it fails.",
    );
    for (name, check) in &collected.checks {
        out.sample(
            "astrum_health_check_healthy",
            &[("check", name)],
            flag(check.healthy),
        );
    }

    if let Some((total, free)) = collected.disk {
        out.family(
            "astrum_data_directory_size_bytes",
            "gauge",
            "Size of the volume of the data directory.",
        );
        out.sample("astrum_data_directory_size_bytes", &[], total);
        out.family(
            "astrum_data_directory_free_bytes",
            "gauge",
            "Free space on the volume of the data directory.",
        );
        out.sample("astrum_data_directory_free_bytes", &[], free);
    }

    let pool = state.db().conn().get_postgres_connection_pool();
    let idle = pool.num_idle() as u64;
    out.family(
        "astrum_database_connections",
        "gauge",
        "Connections of this node to PostgreSQL, idle or in use.",
    );
    out.sample("astrum_database_connections", &[("state", "idle")], idle);
    out.sample(
        "astrum_database_connections",
        &[("state", "used")],
        u64::from(pool.size()).saturating_sub(idle),
    );

    if let Some(pending) = collected.pending {
        out.family(
            "astrum_notification_deliveries_pending",
            "gauge",
            "Notifications waiting to be sent or sent again.",
        );
        out.sample("astrum_notification_deliveries_pending", &[], pending);
    }

    if let Some(at) = state.tasks().last_tick() {
        out.family(
            "astrum_scheduler_last_tick_timestamp_seconds",
            "gauge",
            "When the task scheduler of this node last looked for due tasks.",
        );
        out.sample(
            "astrum_scheduler_last_tick_timestamp_seconds",
            &[],
            seconds(at),
        );
    }

    if let Some(tasks) = &collected.tasks {
        out.family(
            "astrum_task_running",
            "gauge",
            "1 while a task runs or waits to run, 0 otherwise.",
        );
        for (task, runs) in tasks {
            out.sample(
                "astrum_task_running",
                &[("task", &task.name), ("type", &task.kind)],
                flag(runs.running() || runs.queued()),
            );
        }
        out.family(
            "astrum_task_last_run_success",
            "gauge",
            "1 when the last finished run of a task succeeded, 0 when it failed or was cancelled.",
        );
        out.family(
            "astrum_task_last_run_timestamp_seconds",
            "gauge",
            "When the last finished run of a task ended.",
        );
        for (task, runs) in tasks {
            let Some(finished) = &runs.last_finished else {
                continue;
            };
            let labels = [("task", task.name.as_str()), ("type", task.kind.as_str())];
            out.sample(
                "astrum_task_last_run_success",
                &labels,
                flag(finished.state == astrum_tasks::store::RunState::Done.as_str()),
            );
            if let Some(at) = finished.finished_at {
                out.sample(
                    "astrum_task_last_run_timestamp_seconds",
                    &labels,
                    seconds(at.to_utc()),
                );
            }
        }
    }
    out.text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_sorted_into_areas() {
        assert_eq!(
            classify("/repository/docker-prod/v2/x"),
            ("repository", Some("docker-prod"))
        );
        assert_eq!(
            classify("/content/groups/public/a"),
            ("repository", Some("public"))
        );
        assert_eq!(classify("/repository/"), ("repository", None));
        assert_eq!(
            classify("/service/rest/internal/ui/user"),
            ("internal", None)
        );
        assert_eq!(classify("/service/rest/v1/status"), ("rest", None));
        assert_eq!(classify(PATH), ("service", None));
        assert_eq!(classify("/mcp"), ("mcp", None));
        assert_eq!(classify("/v2/"), ("docker", None));
        assert_eq!(classify("/browse"), ("web", None));
    }

    #[test]
    fn histograms_count_every_bucket_at_or_above_the_value() {
        let mut histogram = Histogram::default();
        histogram.observe(0.02);
        histogram.observe(100.0);
        assert_eq!(histogram.buckets[0], 0);
        assert_eq!(histogram.buckets[2], 1);
        assert_eq!(histogram.buckets[12], 1);
        assert_eq!(histogram.count, 2);
    }

    #[test]
    fn repository_names_are_bounded() {
        let registry = Registry::default();
        for index in 0..MAX_REPOSITORIES + 5 {
            registry.record(
                &format!("/repository/r{index}/x"),
                "GET",
                StatusCode::NOT_FOUND,
                Duration::from_millis(1),
            );
        }
        registry.record(
            "/repository/R0/x",
            "GET",
            StatusCode::OK,
            Duration::from_millis(1),
        );
        let counts = registry.counts.lock().unwrap();
        assert_eq!(counts.names.len(), MAX_REPOSITORIES);
        assert_eq!(counts.repositories[&("r0".to_owned(), "GET", "2xx")], 1);
        assert_eq!(
            counts.requests[&("repository", "GET", "4xx")],
            (MAX_REPOSITORIES + 5) as u64
        );
    }

    #[test]
    fn label_values_are_escaped() {
        let mut out = Exposition::default();
        out.sample("m", &[("a", "x\"y\\z\nw")], 1);
        assert_eq!(out.text, "m{a=\"x\\\"y\\\\z\\nw\"} 1\n");
    }
}
