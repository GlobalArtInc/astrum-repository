use std::collections::BTreeSet;
use std::time::Duration;

use astrum_blob::quota;
use astrum_db::setting;
use astrum_db::system::{SPACE_ALERT_KEY, SPACE_ALERT_STATE_KEY, SpaceAlert};
use astrum_notify::alerts::{self, Finding, Problem};
use chrono::{DateTime, Utc};
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::routes::blobstores::{Measured, measure_all};
use crate::{ApiError, AppState};

pub const LOCK_KEY: i64 = i64::from_be_bytes(*b"astrspac");
pub const GIGABYTE: u64 = 1_000_000_000;
const TICK: Duration = Duration::from_secs(60);
const LOW_SPACE: &str = "lowSpace";
const QUOTA: &str = "quota";
const UNAVAILABLE: &str = "unavailable";

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
    }
}

fn detail(problem: &Problem) -> String {
    match problem {
        Problem::LowSpace { available, minimum } => {
            format!("{available} free, less than {minimum}")
        }
        Problem::Quota { message } => format!("passed its soft quota: {message}"),
        Problem::Unavailable => "cannot be reached".to_owned(),
    }
}

fn minimum(settings: &SpaceAlert) -> u64 {
    u64::from(settings.minimum_free_gigabytes).saturating_mul(GIGABYTE)
}

fn findings(
    settings: &SpaceAlert,
    measured: &[(astrum_blob::config::StoreConfig, Measured)],
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
                    store,
                    problem: Problem::Quota {
                        message: status.message,
                    },
                });
            }
        }
    }
    found
}

fn recovery(
    tracked: &Tracked,
    measured: &[(astrum_blob::config::StoreConfig, Measured)],
) -> Problem {
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
    let current = findings(&settings, &measured);
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
            .find(|tracked| tracked.store == finding.store && tracked.problem == kind);
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
        if known.is_some() && settings.repeat_hours > 0 && now - tracked.notified_at >= repeat {
            outcome.alerted.push(finding.clone());
            tracked.notified_at = now;
        }
        tracked.detail = detail(&finding.problem);
        problems.push(tracked);
    }
    for tracked in &before.problems {
        let still = problems
            .iter()
            .any(|kept| kept.store == tracked.store && kept.problem == tracked.problem);
        if still || !existing.contains(tracked.store.as_str()) {
            continue;
        }
        if unreachable.contains(tracked.store.as_str()) && tracked.problem != UNAVAILABLE {
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
