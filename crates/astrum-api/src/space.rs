use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use astrum_blob::quota;
use astrum_db::entity::repository_usage;
use astrum_db::setting;
use astrum_db::system::{SPACE_ALERT_KEY, SPACE_ALERT_STATE_KEY, SpaceAlert};
use astrum_notify::alerts::{self, Finding, Problem};
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::forecast::{self, Forecast};
use crate::routes::blobstores::{Measured, measure_all};
use crate::{ApiError, AppState};

pub const LOCK_KEY: i64 = i64::from_be_bytes(*b"astrspac");
pub const GIGABYTE: u64 = 1_000_000_000;
const TICK: Duration = Duration::from_secs(60);
const LOW_SPACE: &str = "lowSpace";
const QUOTA: &str = "quota";
const UNAVAILABLE: &str = "unavailable";
const REPOSITORY: &str = "repository";
const REPOSITORY_WARN: &str = "repositoryWarn";
const REPOSITORY_LIMIT: &str = "repositoryLimit";
const FORECAST: &str = "forecast";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Watch {
    pub checked_at: Option<DateTime<Utc>>,
    pub problems: Vec<Tracked>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracked {
    pub store: String,
    pub problem: String,
    pub detail: String,
    pub since: DateTime<Utc>,
    pub notified_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RepositoryQuota {
    name: String,
    used: u64,
    limit: u64,
    warn_percent: u64,
}

struct Repositories {
    names: BTreeSet<String>,
    quotas: Vec<RepositoryQuota>,
}

type Measures = [(astrum_blob::config::StoreConfig, Measured)];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub alerted: Vec<Finding>,
    pub recovered: Vec<Finding>,
    pub channels: usize,
}

fn code(problem: &Problem) -> &'static str {
    match problem {
        Problem::LowSpace { .. } => LOW_SPACE,
        Problem::Quota { .. } => QUOTA,
        Problem::Unavailable => UNAVAILABLE,
        Problem::RepositoryWarn { .. } => REPOSITORY_WARN,
        Problem::RepositoryLimit { .. } => REPOSITORY_LIMIT,
        Problem::Forecast { .. } => FORECAST,
    }
}

fn slot(kind: &str) -> &str {
    match kind {
        REPOSITORY_WARN | REPOSITORY_LIMIT => REPOSITORY,
        other => other,
    }
}

fn is_repository(kind: &str) -> bool {
    slot(kind) == REPOSITORY
}

fn detail(problem: &Problem) -> String {
    match problem {
        Problem::LowSpace { available, minimum } => {
            format!("{available} free, less than {minimum}")
        }
        Problem::Quota { message } => format!("passed its soft quota: {message}"),
        Problem::Unavailable => "cannot be reached".to_owned(),
        Problem::RepositoryWarn {
            percent,
            used,
            limit,
        } => format!("repository is at {percent}% of its {limit} quota ({used} used)"),
        Problem::RepositoryLimit { used, limit } => {
            format!("repository reached its {limit} quota ({used} used)")
        }
        Problem::Forecast { days } => format!("will be full in {days} days at the current growth"),
    }
}

fn minimum(settings: &SpaceAlert) -> u64 {
    u64::from(settings.minimum_free_gigabytes).saturating_mul(GIGABYTE)
}

fn repository_finding(quota: &RepositoryQuota) -> Option<Finding> {
    let used = u128::from(quota.used);
    let limit = u128::from(quota.limit);
    let problem = if used >= limit {
        Problem::RepositoryLimit {
            used: quota::human_size(quota.used),
            limit: quota::human_size(quota.limit),
        }
    } else if used * 100 >= limit * u128::from(quota.warn_percent) {
        Problem::RepositoryWarn {
            percent: u32::try_from(used * 100 / limit).unwrap_or(u32::MAX),
            used: quota::human_size(quota.used),
            limit: quota::human_size(quota.limit),
        }
    } else {
        return None;
    };
    Some(Finding {
        store: quota.name.clone(),
        problem,
    })
}

fn findings(
    settings: &SpaceAlert,
    measured: &Measures,
    repositories: &[RepositoryQuota],
    forecasts: &BTreeMap<String, Forecast>,
) -> Vec<Finding> {
    let minimum = minimum(settings);
    let mut found = Vec::new();
    for (config, measured) in measured {
        let store = config.name.clone();
        if measured.unavailable {
            if settings.unavailable {
                found.push(Finding {
                    store,
                    problem: Problem::Unavailable,
                });
            }
            continue;
        }
        if let Some(available) = measured.available
            && available < minimum
        {
            found.push(Finding {
                store: store.clone(),
                problem: Problem::LowSpace {
                    available: quota::human_size(available),
                    minimum: quota::human_size(minimum),
                },
            });
        }
        if settings.quota {
            let status = quota::status(
                &config.name,
                config.soft_quota.as_ref(),
                measured.totals,
                measured.available,
            );
            if status.violation {
                found.push(Finding {
                    store: store.clone(),
                    problem: Problem::Quota {
                        message: status.message,
                    },
                });
            }
        }
        if settings.forecast_days > 0
            && let Some(days) = forecasts
                .get(&config.name)
                .and_then(|forecast| forecast.days_until_full)
            && days < f64::from(settings.forecast_days)
        {
            found.push(Finding {
                store,
                problem: Problem::Forecast {
                    days: days.ceil() as u64,
                },
            });
        }
    }
    found.extend(repositories.iter().filter_map(repository_finding));
    found
}

fn recovery(tracked: &Tracked, measured: &Measures) -> Problem {
    match tracked.problem.as_str() {
        LOW_SPACE => {
            let available = measured
                .iter()
                .find(|(config, _)| config.name == tracked.store)
                .and_then(|(_, measured)| measured.available)
                .map(quota::human_size)
                .unwrap_or_else(|| "?".to_owned());
            Problem::LowSpace {
                available,
                minimum: String::new(),
            }
        }
        QUOTA => Problem::Quota {
            message: String::new(),
        },
        REPOSITORY_WARN => Problem::RepositoryWarn {
            percent: 0,
            used: String::new(),
            limit: String::new(),
        },
        REPOSITORY_LIMIT => Problem::RepositoryLimit {
            used: String::new(),
            limit: String::new(),
        },
        FORECAST => Problem::Forecast { days: 0 },
        _ => Problem::Unavailable,
    }
}

pub async fn settings(state: &AppState) -> Result<Option<SpaceAlert>, ApiError> {
    Ok(
        setting::get::<SpaceAlert>(state.db().conn(), SPACE_ALERT_KEY)
            .await?
            .filter(|settings| settings.enabled),
    )
}

pub async fn watch(state: &AppState) -> Result<Watch, ApiError> {
    Ok(
        setting::get::<Watch>(state.db().conn(), SPACE_ALERT_STATE_KEY)
            .await?
            .unwrap_or_default(),
    )
}

pub async fn forget(state: &AppState) -> Result<(), ApiError> {
    setting::delete(state.db().conn(), SPACE_ALERT_STATE_KEY).await?;
    Ok(())
}

pub async fn recheck_soon(state: &AppState) -> Result<(), ApiError> {
    let mut watch = watch(state).await?;
    if watch.checked_at.is_some() {
        watch.checked_at = None;
        setting::put(state.db().conn(), SPACE_ALERT_STATE_KEY, &watch).await?;
    }
    Ok(())
}

pub async fn tracked(state: &AppState) -> Vec<(String, String)> {
    match settings(state).await {
        Ok(Some(_)) => {}
        _ => return Vec::new(),
    }
    watch(state)
        .await
        .map(|watch| {
            watch
                .problems
                .into_iter()
                .filter(|tracked| !is_repository(&tracked.problem))
                .map(|tracked| (tracked.store, tracked.problem))
                .collect()
        })
        .unwrap_or_default()
}

pub fn status(watch: &Watch) -> String {
    let Some(checked) = watch.checked_at else {
        return "The blob stores have not been checked yet.".to_owned();
    };
    let mut text = format!(
        "Last checked at {}.",
        checked.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    );
    if watch.problems.is_empty() {
        text.push_str("\nNo blob store needs attention.");
    }
    for tracked in &watch.problems {
        text.push_str(&format!(
            "\n{}: {} (since {})",
            tracked.store,
            tracked.detail,
            tracked
                .since
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ));
    }
    text
}

async fn repositories(state: &AppState) -> Result<Repositories, ApiError> {
    let db = state.db().conn();
    let repositories = astrum_repo::store::list(db)
        .await
        .map_err(ApiError::internal)?;
    let used: BTreeMap<String, i64> = repository_usage::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.repository, row.bytes))
        .collect();
    let names = repositories
        .iter()
        .map(|repository| repository.name.clone())
        .collect();
    let quotas = repositories
        .into_iter()
        .filter_map(|repository| {
            let quota = repository.attributes.storage.as_ref()?.quota.as_ref()?;
            let limit = u64::try_from(quota.limit_bytes?)
                .ok()
                .filter(|limit| *limit > 0)?;
            let warn_percent = quota
                .warn_percent
                .and_then(|percent| u64::try_from(percent).ok())
                .unwrap_or(astrum_repo::config::DEFAULT_WARN_PERCENT as u64);
            let used = u64::try_from(*used.get(&repository.name)?).unwrap_or(0);
            Some(RepositoryQuota {
                name: repository.name,
                used,
                limit,
                warn_percent,
            })
        })
        .collect();
    Ok(Repositories { names, quotas })
}

async fn deliver(
    state: &AppState,
    settings: &SpaceAlert,
    event: &str,
    findings: &[Finding],
) -> Result<usize, ApiError> {
    if findings.is_empty() {
        return Ok(0);
    }
    let wanted: BTreeSet<String> = settings
        .channels
        .iter()
        .map(|name| name.to_lowercase())
        .collect();
    let base_url = state.notifier().base_url().await;
    let db = state.db().conn();
    let mut sent = 0;
    for channel in astrum_notify::store::list(db).await? {
        if !channel.enabled || !wanted.contains(&channel.name.to_lowercase()) {
            continue;
        }
        let text = alerts::render(&channel, event, findings, base_url.as_deref());
        state.notifier().enqueue(db, &channel, event, text).await?;
        sent += 1;
    }
    Ok(sent)
}

pub async fn check(state: &AppState) -> Result<Option<Outcome>, ApiError> {
    let Some(settings) = settings(state).await? else {
        return Ok(None);
    };
    let now = Utc::now();
    let before = watch(state).await?;
    let measured = measure_all(state).await?;
    let mut forecasts = BTreeMap::new();
    let mut forecast_failed: BTreeSet<String> = BTreeSet::new();
    if settings.forecast_days > 0 {
        for (config, measures) in &measured {
            if measures.unavailable || config.group().is_some() {
                continue;
            }
            match forecast::of_store(state.db().conn(), config, measures).await {
                Ok(found) => {
                    forecasts.insert(config.name.clone(), found);
                }
                Err(error) => {
                    tracing::warn!(store = %config.name, %error, "the fill forecast of a blob store could not be computed");
                    forecast_failed.insert(config.name.clone());
                }
            }
        }
    }
    let repositories = match repositories(state).await {
        Ok(repositories) => Some(repositories),
        Err(error) => {
            tracing::warn!(%error, "the quotas of the repositories could not be read");
            None
        }
    };
    let current = findings(
        &settings,
        &measured,
        repositories
            .as_ref()
            .map(|repositories| repositories.quotas.as_slice())
            .unwrap_or_default(),
        &forecasts,
    );
    let unreachable: BTreeSet<&str> = measured
        .iter()
        .filter(|(_, measured)| measured.unavailable)
        .map(|(config, _)| config.name.as_str())
        .collect();
    let existing: BTreeSet<&str> = measured
        .iter()
        .map(|(config, _)| config.name.as_str())
        .collect();
    let repeat = chrono::Duration::hours(i64::from(settings.repeat_hours));
    let mut outcome = Outcome::default();
    let mut problems = Vec::new();
    for finding in current {
        let kind = code(&finding.problem);
        let known = before
            .problems
            .iter()
            .find(|tracked| tracked.store == finding.store && slot(&tracked.problem) == slot(kind));
        let mut tracked = match known {
            Some(tracked) => tracked.clone(),
            None => {
                outcome.alerted.push(finding.clone());
                Tracked {
                    store: finding.store.clone(),
                    problem: kind.to_owned(),
                    detail: String::new(),
                    since: now,
                    notified_at: now,
                }
            }
        };
        if known.is_some_and(|known| known.problem != kind) {
            outcome.alerted.push(finding.clone());
            tracked.problem = kind.to_owned();
            tracked.since = now;
            tracked.notified_at = now;
        } else if known.is_some()
            && settings.repeat_hours > 0
            && now - tracked.notified_at >= repeat
        {
            outcome.alerted.push(finding.clone());
            tracked.notified_at = now;
        }
        tracked.detail = detail(&finding.problem);
        problems.push(tracked);
    }
    for tracked in &before.problems {
        let still = problems.iter().any(|kept| {
            kept.store == tracked.store && slot(&kept.problem) == slot(&tracked.problem)
        });
        if still {
            continue;
        }
        if is_repository(&tracked.problem) {
            match &repositories {
                Some(repositories) if !repositories.names.contains(&tracked.store) => continue,
                Some(_) => {}
                None => {
                    problems.push(tracked.clone());
                    continue;
                }
            }
        } else if !existing.contains(tracked.store.as_str()) {
            continue;
        } else if (unreachable.contains(tracked.store.as_str()) && tracked.problem != UNAVAILABLE)
            || (tracked.problem == FORECAST && forecast_failed.contains(&tracked.store))
        {
            problems.push(tracked.clone());
            continue;
        }
        if settings.recovery {
            outcome.recovered.push(Finding {
                store: tracked.store.clone(),
                problem: recovery(tracked, &measured),
            });
        }
    }
    outcome.channels = deliver(state, &settings, alerts::ALERT, &outcome.alerted).await?;
    outcome.channels += deliver(state, &settings, alerts::RECOVERED, &outcome.recovered).await?;
    setting::put(
        state.db().conn(),
        SPACE_ALERT_STATE_KEY,
        &Watch {
            checked_at: Some(now),
            problems,
        },
    )
    .await?;
    Ok(Some(outcome))
}

struct Lock {
    connection: PoolConnection<Postgres>,
}

impl Lock {
    async fn try_take(state: &AppState) -> Result<Option<Self>, ApiError> {
        let mut connection = state
            .db()
            .conn()
            .get_postgres_connection_pool()
            .acquire()
            .await
            .map_err(ApiError::internal)?;
        connection.close_on_drop();
        let (taken,): (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(LOCK_KEY)
            .fetch_one(&mut *connection)
            .await
            .map_err(ApiError::internal)?;
        Ok(taken.then_some(Self { connection }))
    }

    async fn release(mut self) {
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(LOCK_KEY)
            .execute(&mut *self.connection)
            .await;
    }
}

async fn tick(state: &AppState) -> Result<(), ApiError> {
    let Some(settings) = settings(state).await? else {
        return Ok(());
    };
    let Some(lock) = Lock::try_take(state).await? else {
        return Ok(());
    };
    let result = async {
        let interval = chrono::Duration::minutes(i64::from(settings.interval_minutes.max(1)));
        let due = watch(state)
            .await?
            .checked_at
            .is_none_or(|at| Utc::now() - at >= interval);
        if due {
            check(state).await?;
        }
        Ok(())
    }
    .await;
    lock.release().await;
    result
}

pub fn spawn(state: AppState) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Err(error) = tick(&state).await {
                tracing::warn!(%error, "the free space of the blob stores could not be checked");
            }
            tokio::time::sleep(TICK).await;
        }
    })
}
