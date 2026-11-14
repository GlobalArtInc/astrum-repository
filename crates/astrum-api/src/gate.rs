use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use astrum_content::{Asset, ContentError, Ctx, Gate};
use astrum_db::audit::{self, Event};
use astrum_policy::cache::{Cache, Snapshot as Policies};
use astrum_policy::policy::{Mode, Policy, Unscanned};
use astrum_policy::refusal;
use astrum_policy::subject;
use astrum_policy::verdict::{State, clean};
use astrum_scan::inline;
use astrum_scan::worker::ScanError;
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use serde_json::{Value, json};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};

use crate::routes::policies::POLICY_DOMAIN;
use crate::{auth, routes};

mod docker;

const INLINE_BUDGET: Duration = Duration::from_secs(2);
const MARK_BUDGET: Duration = Duration::from_millis(500);
const INLINE_PERMITS: usize = 8;
const AUDIT_WINDOW: Duration = Duration::from_secs(3600);
const AUDIT_CAP: usize = 10_000;
const NOTICE_WINDOW: Duration = Duration::from_secs(24 * 3600);
const NOTICE_CAP: usize = 10_000;
const NOTICE_PERMITS: usize = 2;
const NOTICE_BACKLOG: usize = 1_000;
const NOTICE_LOG_EVERY: Duration = Duration::from_secs(10);
const NOTICE_BUDGET: Duration = Duration::from_secs(5);
const NOTICE_ACTION: &str = "policy.blocked";
pub const BUCKETS: [f64; 14] = [
    0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 2.5, 5.0,
];

const UNAVAILABLE: &str = "the content policy could not be checked; try again";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Blocked,
    WouldBlock,
    Unguarded,
    InlinePass,
    InlineBlocked,
    InlineUnscanned,
    InlineTimeout,
    InlineError,
    MarkFailed,
    NoticeFailed,
    Error,
}

impl Outcome {
    pub const ALL: [Outcome; 12] = [
        Outcome::Pass,
        Outcome::Blocked,
        Outcome::WouldBlock,
        Outcome::Unguarded,
        Outcome::InlinePass,
        Outcome::InlineBlocked,
        Outcome::InlineUnscanned,
        Outcome::InlineTimeout,
        Outcome::InlineError,
        Outcome::MarkFailed,
        Outcome::NoticeFailed,
        Outcome::Error,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Blocked => "blocked",
            Outcome::WouldBlock => "would_block",
            Outcome::Unguarded => "unguarded",
            Outcome::InlinePass => "inline_scan_pass",
            Outcome::InlineBlocked => "inline_scan_blocked",
            Outcome::InlineUnscanned => "inline_scan_unscanned",
            Outcome::InlineTimeout => "inline_scan_timeout",
            Outcome::InlineError => "inline_scan_error",
            Outcome::MarkFailed => "mark_failed",
            Outcome::NoticeFailed => "notice_failed",
            Outcome::Error => "error",
        }
    }
}

fn permits_for(pool: Option<u32>) -> usize {
    match pool {
        Some(pool) => (pool as usize / 2).clamp(1, INLINE_PERMITS),
        None => INLINE_PERMITS,
    }
}

fn pool_size(db: &DatabaseConnection) -> Option<u32> {
    match db {
        DatabaseConnection::SqlxPostgresPoolConnection(_) => Some(
            db.get_postgres_connection_pool()
                .options()
                .get_max_connections(),
        ),
        _ => None,
    }
}

fn inline_pass(state: &str) -> Outcome {
    match state == State::Unscanned.as_str() {
        true => Outcome::InlineUnscanned,
        false => Outcome::InlinePass,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub results: [u64; Outcome::ALL.len()],
    pub buckets: [u64; BUCKETS.len()],
    pub sum: f64,
    pub count: u64,
}

#[derive(Default)]
pub struct Metrics {
    results: [AtomicU64; Outcome::ALL.len()],
    durations: Mutex<Snapshot>,
}

impl Metrics {
    fn count(&self, outcome: Outcome) {
        self.results[outcome as usize].fetch_add(1, Ordering::Relaxed);
    }

    fn observe(&self, elapsed: Duration) {
        let seconds = elapsed.as_secs_f64();
        let mut durations = self
            .durations
            .lock()
            .expect("the gate duration lock is never poisoned");
        for (bucket, bound) in durations.buckets.iter_mut().zip(BUCKETS) {
            if seconds <= bound {
                *bucket += 1;
            }
        }
        durations.sum += seconds;
        durations.count += 1;
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self
            .durations
            .lock()
            .expect("the gate duration lock is never poisoned")
            .clone();
        for (slot, counter) in snapshot.results.iter_mut().zip(&self.results) {
            *slot = counter.load(Ordering::Relaxed);
        }
        snapshot
    }
}

struct Dedupe {
    window: Duration,
    cap: usize,
    seen: Mutex<HashMap<(String, String), Instant>>,
}

impl Dedupe {
    fn new(window: Duration, cap: usize) -> Self {
        Self {
            window,
            cap,
            seen: Mutex::new(HashMap::new()),
        }
    }

    fn first(&self, repository: &str, subject: &str, now: Instant) -> bool {
        let key = (repository.to_owned(), subject.to_owned());
        let mut seen = self
            .seen
            .lock()
            .expect("the audit dedupe lock is never poisoned");
        if let Some(at) = seen.get(&key)
            && now.saturating_duration_since(*at) < self.window
        {
            return false;
        }
        if seen.len() >= self.cap && !seen.contains_key(&key) {
            seen.retain(|_, at| now.saturating_duration_since(*at) < self.window);
            if seen.len() >= self.cap {
                let oldest = seen
                    .iter()
                    .min_by_key(|(_, at)| **at)
                    .map(|(key, _)| key.clone());
                if let Some(oldest) = oldest {
                    seen.remove(&oldest);
                }
            }
        }
        seen.insert(key, now);
        true
    }

    fn forget(&self, repository: &str, subject: &str) {
        self.seen
            .lock()
            .expect("the audit dedupe lock is never poisoned")
            .remove(&(repository.to_owned(), subject.to_owned()));
    }
}

struct LogThrottle {
    every: Duration,
    last: Mutex<Option<Instant>>,
    suppressed: AtomicU64,
}

impl LogThrottle {
    fn new(every: Duration) -> Self {
        Self {
            every,
            last: Mutex::new(None),
            suppressed: AtomicU64::new(0),
        }
    }

    fn pass(&self, now: Instant) -> Option<u64> {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.is_some_and(|at| now.saturating_duration_since(at) < self.every) {
            self.suppressed.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        *last = Some(now);
        Some(self.suppressed.swap(0, Ordering::Relaxed))
    }
}

struct Pending(Arc<AtomicUsize>);

impl Drop for Pending {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

struct Held {
    policy: String,
    revision: i64,
    state: String,
    blocked: bool,
    reasons: Value,
}

struct Read {
    policy: Option<String>,
    verdict: Option<Held>,
}

async fn read(
    db: &DatabaseConnection,
    repository: &str,
    subject: &str,
) -> Result<Option<Read>, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT r.attributes #>> '{policy,name}' AS current, v.policy, v.policy_revision, \
                    v.state, v.blocked, v.reasons \
             FROM repository r \
             LEFT JOIN policy_verdict v ON v.repository = r.name AND v.subject = $2 \
             WHERE r.name = $1",
            [repository.into(), subject.into()],
        ))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let verdict = match row.try_get::<Option<String>>("", "policy")? {
        Some(policy) => Some(Held {
            policy,
            revision: row.try_get("", "policy_revision")?,
            state: row.try_get("", "state")?,
            blocked: row.try_get("", "blocked")?,
            reasons: row.try_get("", "reasons")?,
        }),
        None => None,
    };
    Ok(Some(Read {
        policy: row.try_get("", "current")?,
        verdict,
    }))
}

#[derive(Debug, thiserror::Error)]
enum Unchecked {
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
    #[error("the policies could not be read: {0}")]
    Policies(#[from] astrum_policy::Error),
    #[error("the repository of the asset is gone")]
    Vanished,
}

struct Refusal {
    policy: String,
    state: String,
    error: ContentError,
}

enum Judgement {
    Unguarded,
    Allowed(Outcome),
    Refused(Outcome, Refusal),
}

enum Ended {
    Judged,
    Failed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Finish {
    Judged,
    Failed,
}

type Finished = watch::Receiver<Option<Finish>>;

#[derive(Default)]
struct Flights {
    open: Mutex<HashMap<i64, Finished>>,
}

enum Role {
    Lead(Lead),
    Follow(Finished),
}

struct Lead {
    flights: Arc<Flights>,
    component_id: i64,
    sender: watch::Sender<Option<Finish>>,
    finish: Option<Finish>,
    permit: Option<OwnedSemaphorePermit>,
}

impl Flights {
    fn locked(&self) -> MutexGuard<'_, HashMap<i64, Finished>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn join_or_lead(self: &Arc<Self>, component_id: i64) -> Role {
        let mut open = self.locked();
        if let Some(finished) = open.get(&component_id) {
            return Role::Follow(finished.clone());
        }
        let (sender, finished) = watch::channel(None);
        open.insert(component_id, finished);
        Role::Lead(Lead {
            flights: self.clone(),
            component_id,
            sender,
            finish: None,
            permit: None,
        })
    }

    fn len(&self) -> usize {
        self.locked().len()
    }
}

impl Drop for Lead {
    fn drop(&mut self) {
        self.flights.locked().remove(&self.component_id);
        self.sender
            .send_replace(Some(self.finish.unwrap_or(Finish::Failed)));
    }
}

struct Scanned {
    ended: Ended,
    leader: bool,
}

pub struct PolicyGate {
    cache: Arc<Cache>,
    metrics: Arc<Metrics>,
    budget: Duration,
    permits: Arc<Semaphore>,
    flights: Arc<Flights>,
    audited: Dedupe,
    noticed: Arc<Dedupe>,
    notice_permits: Arc<Semaphore>,
    notice_backlog: Arc<AtomicUsize>,
    notice_logs: Arc<LogThrottle>,
}

impl PolicyGate {
    pub fn new(cache: Arc<Cache>, metrics: Arc<Metrics>) -> Self {
        Self {
            cache,
            metrics,
            budget: INLINE_BUDGET,
            permits: Arc::new(Semaphore::new(INLINE_PERMITS)),
            flights: Arc::default(),
            audited: Dedupe::new(AUDIT_WINDOW, AUDIT_CAP),
            noticed: Arc::new(Dedupe::new(NOTICE_WINDOW, NOTICE_CAP)),
            notice_permits: Arc::new(Semaphore::new(NOTICE_PERMITS)),
            notice_backlog: Arc::default(),
            notice_logs: Arc::new(LogThrottle::new(NOTICE_LOG_EVERY)),
        }
    }

    pub fn with_budget(mut self, budget: Duration) -> Self {
        self.budget = budget;
        self
    }

    pub fn with_permits(mut self, permits: usize) -> Self {
        self.permits = Arc::new(Semaphore::new(permits.max(1)));
        self
    }

    pub fn sized_for(self, db: &DatabaseConnection) -> Self {
        self.with_permits(permits_for(pool_size(db)))
    }

    pub fn available_permits(&self) -> usize {
        self.permits.available_permits()
    }

    pub fn notices_in_flight(&self) -> usize {
        self.notice_backlog.load(Ordering::Relaxed)
    }

    pub fn flights_open(&self) -> usize {
        self.flights.len()
    }

    async fn snapshot_for(
        &self,
        db: &DatabaseConnection,
        name: &str,
    ) -> Result<Arc<Policies>, Unchecked> {
        let snapshot = self.cache.get(db).await?;
        match snapshot.policies.contains_key(name) {
            true => Ok(snapshot),
            false => Ok(self.cache.fresh(db).await?),
        }
    }

    async fn judge(
        &self,
        ctx: &Ctx,
        asset: &Asset,
        component_id: i64,
    ) -> Result<Judgement, Unchecked> {
        let db = ctx.db();
        let held = read(db, &asset.repository, &subject::component(component_id))
            .await?
            .ok_or(Unchecked::Vanished)?;
        let Some(name) = held.policy else {
            return Ok(Judgement::Unguarded);
        };
        let snapshot = self.snapshot_for(db, &name).await?;
        let Some(stored) = snapshot.policies.get(&name) else {
            return Ok(unreadable(&name));
        };
        let Ok(policy) = &stored.policy else {
            return Ok(unreadable(&name));
        };
        let matching = held
            .verdict
            .filter(|verdict| verdict.policy == name && verdict.revision == stored.revision);
        match policy.vulnerabilities.mode {
            Mode::Off => Ok(Judgement::Allowed(Outcome::Pass)),
            Mode::Report => {
                let would = matching.is_some_and(|verdict| verdict.state == State::Report.as_str());
                Ok(Judgement::Allowed(match would {
                    true => Outcome::WouldBlock,
                    false => Outcome::Pass,
                }))
            }
            Mode::Block => match matching {
                Some(verdict) => Ok(from_verdict(
                    &name,
                    &verdict,
                    Outcome::Pass,
                    Outcome::Blocked,
                )),
                None => {
                    self.inline(ctx, asset, component_id, policy, stored.revision)
                        .await
                }
            },
        }
    }

    async fn scan(&self, ctx: &Ctx, repository: &str, component_id: i64) -> Scanned {
        match self.flights.join_or_lead(component_id) {
            Role::Follow(finished) => Scanned {
                ended: self.follow(finished, repository, component_id).await,
                leader: false,
            },
            Role::Lead(lead) => Scanned {
                ended: self.lead(ctx, lead, repository, component_id).await,
                leader: true,
            },
        }
    }

    async fn follow(&self, mut finished: Finished, repository: &str, component_id: i64) -> Ended {
        let waited =
            tokio::time::timeout(self.budget, finished.wait_for(|finish| finish.is_some())).await;
        let finish = match waited {
            Ok(Ok(finish)) => *finish,
            Ok(Err(_)) => None,
            Err(_) => {
                tracing::warn!(%repository, component = component_id, budget_ms = self.budget.as_millis() as u64, "the scan another download started did not finish in time, so the policy for unscanned components applies");
                return Ended::TimedOut;
            }
        };
        match finish {
            Some(Finish::Judged) => Ended::Judged,
            _ => Ended::Failed,
        }
    }

    async fn lead(&self, ctx: &Ctx, lead: Lead, repository: &str, component_id: i64) -> Ended {
        let permits = self.permits.clone();
        let content = ctx.content.clone();
        let security = ctx.security.clone();
        let cache = self.cache.clone();
        let name = repository.to_owned();
        let work = async move {
            let mut lead = lead;
            lead.permit = Some(permits.acquire_owned().await.map_err(|error| {
                inline::InlineError::Scan(ScanError::Stopped(format!(
                    "the scan permits are closed: {error}"
                )))
            })?);
            let task = tokio::spawn(async move {
                let mut lead = lead;
                let result =
                    inline::judge_component(&content, &security, &cache, &name, component_id).await;
                lead.finish = Some(match result {
                    Ok(()) => Finish::Judged,
                    Err(_) => Finish::Failed,
                });
                result
            });
            task.await.map_err(|error| {
                inline::InlineError::Scan(ScanError::Stopped(format!(
                    "the scan task stopped: {error}"
                )))
            })?
        };
        match tokio::time::timeout(self.budget, work).await {
            Ok(Ok(())) => Ended::Judged,
            Ok(Err(inline::InlineError::NotDue)) => {
                tracing::debug!(%repository, component = component_id, "the scan of the component is not due, so the policy for unscanned components applies");
                Ended::Failed
            }
            Ok(Err(error)) => {
                tracing::warn!(%repository, component = component_id, %error, "the scan at download failed, so the policy for unscanned components applies");
                Ended::Failed
            }
            Err(_) => {
                tracing::warn!(%repository, component = component_id, budget_ms = self.budget.as_millis() as u64, permits_available = self.permits.available_permits(), "the scan at download did not finish in time or found no free slot, so the policy for unscanned components applies");
                Ended::TimedOut
            }
        }
    }

    async fn queue_mark(&self, db: &DatabaseConnection, repository: &str, component_id: i64) {
        let failure =
            match tokio::time::timeout(MARK_BUDGET, inline::mark(db, repository, component_id))
                .await
            {
                Ok(Ok(())) => return,
                Ok(Err(error)) => error.to_string(),
                Err(_) => format!("it was not written within {} ms", MARK_BUDGET.as_millis()),
            };
        self.metrics.count(Outcome::MarkFailed);
        tracing::error!(%repository, component = component_id, error = %failure, "a component without a verdict was not queued for the workers");
    }

    async fn inline(
        &self,
        ctx: &Ctx,
        asset: &Asset,
        component_id: i64,
        policy: &Policy,
        revision: i64,
    ) -> Result<Judgement, Unchecked> {
        let db = ctx.db();
        let Scanned { mut ended, leader } = self.scan(ctx, &asset.repository, component_id).await;
        if let Ended::Judged = ended {
            let current = self.cache.fresh(db).await?;
            let fresh = current
                .policies
                .get(&policy.name)
                .filter(|stored| stored.policy.is_ok());
            let verdict = read(db, &asset.repository, &subject::component(component_id))
                .await?
                .ok_or(Unchecked::Vanished)?
                .verdict;
            match (fresh, verdict) {
                (Some(stored), Some(verdict))
                    if verdict.policy == policy.name && verdict.revision == stored.revision =>
                {
                    return Ok(from_verdict(
                        &policy.name,
                        &verdict,
                        inline_pass(&verdict.state),
                        Outcome::InlineBlocked,
                    ));
                }
                _ => {
                    tracing::warn!(repository = %asset.repository, component = component_id, revision, "the verdict stored by the scan at download does not fit the current policy, so the policy for unscanned components applies");
                    ended = Ended::Failed;
                }
            }
        }
        if leader {
            self.queue_mark(db, &asset.repository, component_id).await;
        }
        let outcome = match ended {
            Ended::TimedOut => Outcome::InlineTimeout,
            _ => Outcome::InlineError,
        };
        Ok(match policy.vulnerabilities.unscanned {
            Unscanned::Allow => Judgement::Allowed(outcome),
            Unscanned::Block => Judgement::Refused(
                outcome,
                Refusal {
                    policy: policy.name.clone(),
                    state: State::Unscanned.as_str().to_owned(),
                    error: ContentError::blocked_by(&clean(&policy.name), refusal::UNSCANNED),
                },
            ),
        })
    }

    fn notify(&self, ctx: &Ctx, asset: &Asset, component_id: i64, refusal: &Refusal) {
        let subject = subject::component(component_id);
        if !self
            .noticed
            .first(&asset.repository, &subject, Instant::now())
        {
            return;
        }
        if self.notice_backlog.fetch_add(1, Ordering::Relaxed) >= NOTICE_BACKLOG {
            self.notice_backlog.fetch_sub(1, Ordering::Relaxed);
            self.noticed.forget(&asset.repository, &subject);
            self.metrics.count(Outcome::NoticeFailed);
            if let Some(suppressed) = self.notice_logs.pass(Instant::now()) {
                tracing::warn!(repository = %asset.repository, component = component_id, backlog = NOTICE_BACKLOG, suppressed, "a refusal was not announced because too many announcements are waiting to be written");
            }
            return;
        }
        let pending = Pending(self.notice_backlog.clone());
        let permits = self.notice_permits.clone();
        let db = ctx.db().clone();
        let metrics = self.metrics.clone();
        let noticed = self.noticed.clone();
        let logs = self.notice_logs.clone();
        let repository = asset.repository.clone();
        let detail = json!({ "policy": clean(&refusal.policy) }).to_string();
        tokio::spawn(async move {
            let _pending = pending;
            let work = async {
                let _permit = permits
                    .acquire()
                    .await
                    .map_err(|error| format!("the announcement slots are closed: {error}"))?;
                enqueue_notice(&db, component_id, &detail)
                    .await
                    .map_err(|error| error.to_string())
            };
            let failure = match tokio::time::timeout(NOTICE_BUDGET, work).await {
                Ok(Ok(_)) => return,
                Ok(Err(error)) => error,
                Err(_) => format!("it was not written within {} s", NOTICE_BUDGET.as_secs()),
            };
            noticed.forget(&repository, &subject);
            metrics.count(Outcome::NoticeFailed);
            if let Some(suppressed) = logs.pass(Instant::now()) {
                tracing::error!(%repository, component = component_id, error = %failure, suppressed, "a refusal was not announced to the notification channels");
            }
        });
    }

    async fn audit(&self, ctx: &Ctx, asset: &Asset, component_id: i64, refusal: &Refusal) {
        let subject = subject::component(component_id);
        if !self
            .audited
            .first(&asset.repository, &subject, Instant::now())
        {
            return;
        }
        let attributes = json!({
            "repository": asset.repository,
            "componentId": component_id,
            "policy": refusal.policy,
            "state": refusal.state,
        });
        self.record(ctx, &asset.repository, &subject, &asset.path, attributes)
            .await;
    }

    async fn record(
        &self,
        ctx: &Ctx,
        repository: &str,
        subject: &str,
        context: &str,
        mut attributes: Value,
    ) {
        if let Some(client) = routes::mcp::client() {
            attributes["via"] = json!("mcp");
            attributes["client"] = json!(client);
        }
        let initiator = match auth::client_address().filter(|address| !address.is_empty()) {
            Some(address) => format!("{}/{address}", ctx.uploader()),
            None => ctx.uploader(),
        };
        let event = Event::new(POLICY_DOMAIN, "blocked", context)
            .by(initiator)
            .with(attributes);
        if let Err(error) = audit::record(ctx.db(), event).await {
            self.audited.forget(repository, subject);
            tracing::error!(%error, "a refused download was not recorded in the audit log");
        }
    }
}

async fn enqueue_notice(
    db: &DatabaseConnection,
    component_id: i64,
    detail: &str,
) -> Result<u64, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "WITH claim AS (\
               INSERT INTO policy_blocked_notice (component_id, noticed_at) \
               SELECT $1::bigint, now() \
               WHERE EXISTS (SELECT 1 FROM notification_channel ch \
                             WHERE ch.enabled AND $3::text = ANY (ch.events)) \
               ON CONFLICT (component_id) DO UPDATE SET noticed_at = now() \
                 WHERE policy_blocked_notice.noticed_at <= now() - make_interval(secs => $4::float8) \
               RETURNING component_id) \
             SELECT COALESCE(sum(notification_capture_events($3::text, ARRAY[claim.component_id], \
                                                             ARRAY[$2::text])), 0)::bigint AS n \
             FROM claim",
            [
                component_id.into(),
                detail.into(),
                NOTICE_ACTION.into(),
                NOTICE_WINDOW.as_secs_f64().into(),
            ],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the notice claim returned no row".to_owned()))?;
    let queued: i64 = row.try_get("", "n")?;
    Ok(queued.max(0) as u64)
}

fn unreadable(policy: &str) -> Judgement {
    Judgement::Refused(
        Outcome::Blocked,
        Refusal {
            policy: policy.to_owned(),
            state: State::Error.as_str().to_owned(),
            error: ContentError::blocked_by(&clean(policy), refusal::UNREADABLE),
        },
    )
}

fn from_verdict(policy: &str, verdict: &Held, pass: Outcome, blocked: Outcome) -> Judgement {
    match refusal::blocks(&verdict.state, verdict.blocked) {
        false => Judgement::Allowed(pass),
        true => Judgement::Refused(
            blocked,
            Refusal {
                policy: policy.to_owned(),
                state: clean(&verdict.state),
                error: ContentError::blocked_by(
                    &clean(policy),
                    &refusal::details(&verdict.reasons),
                ),
            },
        ),
    }
}

#[async_trait]
impl Gate for PolicyGate {
    async fn check(&self, ctx: &Ctx, asset: &Asset) -> Result<(), ContentError> {
        let Some(component_id) = asset.component_id else {
            return Ok(());
        };
        if docker::judged_by_digest(ctx)
            || (asset.repository == ctx.name() && ctx.repository.policy_name().is_none())
        {
            return Ok(());
        }
        let started = Instant::now();
        let judged = self.judge(ctx, asset, component_id).await;
        if let Err(Unchecked::Vanished) = &judged {
            return Err(ContentError::not_found(asset.path.clone()));
        }
        self.metrics.observe(started.elapsed());
        match judged {
            Ok(Judgement::Unguarded) => {
                self.metrics.count(Outcome::Unguarded);
                Ok(())
            }
            Ok(Judgement::Allowed(outcome)) => {
                self.metrics.count(outcome);
                Ok(())
            }
            Ok(Judgement::Refused(outcome, refusal)) => {
                self.metrics.count(outcome);
                self.audit(ctx, asset, component_id, &refusal).await;
                self.notify(ctx, asset, component_id, &refusal);
                Err(refusal.error)
            }
            Err(error) => {
                self.metrics.count(Outcome::Error);
                tracing::error!(repository = %asset.repository, path = %asset.path, %error, "the download was refused because its content policy could not be checked");
                Err(ContentError::Unavailable(UNAVAILABLE.to_owned()))
            }
        }
    }

    async fn check_digest(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
        reference: Option<&str>,
    ) -> Result<(), ContentError> {
        self.check_image(ctx, repository, digest, reference).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(state: &str, blocked: bool) -> Held {
        Held {
            policy: "p".to_owned(),
            revision: 1,
            state: state.to_owned(),
            blocked,
            reasons: json!([]),
        }
    }

    fn inline_outcome(verdict: &Held) -> Outcome {
        match from_verdict(
            "p",
            verdict,
            inline_pass(&verdict.state),
            Outcome::InlineBlocked,
        ) {
            Judgement::Allowed(outcome) | Judgement::Refused(outcome, _) => outcome,
            Judgement::Unguarded => unreachable!("a verdict is never unguarded"),
        }
    }

    #[test]
    fn the_scan_slots_stay_within_half_of_the_pool_and_eight_and_never_drop_below_one() {
        assert_eq!(permits_for(Some(20)), 8);
        assert_eq!(permits_for(Some(16)), 8);
        assert_eq!(permits_for(Some(200)), 8);
        assert_eq!(permits_for(Some(10)), 5);
        assert_eq!(permits_for(Some(4)), 2);
        assert_eq!(permits_for(Some(3)), 1);
        assert_eq!(permits_for(Some(1)), 1);
        assert_eq!(permits_for(Some(0)), 1);
        assert_eq!(permits_for(None), 8);
    }

    #[test]
    fn a_scan_at_download_that_left_the_component_unscanned_is_not_counted_as_a_pass() {
        assert_eq!(inline_outcome(&held("pass", false)), Outcome::InlinePass);
        assert_eq!(inline_outcome(&held("report", false)), Outcome::InlinePass);
        assert_eq!(
            inline_outcome(&held("unscanned", false)),
            Outcome::InlineUnscanned
        );
        assert_eq!(
            inline_outcome(&held("unscanned", true)),
            Outcome::InlineBlocked
        );
        assert_eq!(inline_outcome(&held("block", true)), Outcome::InlineBlocked);
    }

    #[tokio::test]
    async fn a_lead_that_ends_is_dropped_or_panics_leaves_no_flight_and_tells_its_followers() {
        let flights: Arc<Flights> = Arc::default();
        let finished = |mut finished: Finished| async move {
            *finished
                .wait_for(|finish| finish.is_some())
                .await
                .expect("the lead publishes before it goes")
        };

        let Role::Lead(mut lead) = flights.join_or_lead(7) else {
            panic!("the first caller leads");
        };
        let Role::Follow(follower) = flights.join_or_lead(7) else {
            panic!("the second caller follows");
        };
        assert!(matches!(flights.join_or_lead(8), Role::Lead(_)));
        assert_eq!(flights.len(), 1);
        lead.finish = Some(Finish::Judged);
        drop(lead);
        assert_eq!(flights.len(), 0);
        assert_eq!(finished(follower).await, Some(Finish::Judged));

        let Role::Lead(lead) = flights.join_or_lead(7) else {
            panic!("a flight that ended can be led again");
        };
        let Role::Follow(follower) = flights.join_or_lead(7) else {
            panic!("the second caller follows");
        };
        drop(lead);
        assert_eq!(flights.len(), 0);
        assert_eq!(finished(follower).await, Some(Finish::Failed));

        let Role::Lead(lead) = flights.join_or_lead(9) else {
            panic!("the first caller leads");
        };
        let Role::Follow(follower) = flights.join_or_lead(9) else {
            panic!("the second caller follows");
        };
        let task = tokio::spawn(async move {
            let _lead = lead;
            panic!("the scan panicked");
        });
        assert!(task.await.unwrap_err().is_panic());
        assert_eq!(flights.len(), 0);
        assert_eq!(finished(follower).await, Some(Finish::Failed));
    }

    #[test]
    fn a_log_line_comes_once_an_interval_and_counts_what_it_swallowed() {
        let throttle = LogThrottle::new(Duration::from_secs(10));
        let start = Instant::now();
        assert_eq!(throttle.pass(start), Some(0));
        assert_eq!(throttle.pass(start + Duration::from_secs(1)), None);
        assert_eq!(throttle.pass(start + Duration::from_secs(9)), None);
        assert_eq!(throttle.pass(start + Duration::from_secs(10)), Some(2));
        assert_eq!(throttle.pass(start + Duration::from_secs(11)), None);
    }

    #[test]
    fn a_subject_is_audited_once_a_window_and_the_map_stays_bounded() {
        let dedupe = Dedupe::new(Duration::from_secs(60), 2);
        let start = Instant::now();
        assert!(dedupe.first("r", "c:1", start));
        assert!(!dedupe.first("r", "c:1", start + Duration::from_secs(59)));
        assert!(dedupe.first("r", "oci:sha256:2", start + Duration::from_secs(1)));
        assert!(dedupe.first("other", "c:1", start + Duration::from_secs(2)));
        assert_eq!(dedupe.seen.lock().unwrap().len(), 2);
        assert!(!dedupe.first("other", "c:1", start + Duration::from_secs(3)));
        assert!(dedupe.first("r", "c:1", start + Duration::from_secs(61)));
        dedupe.forget("r", "c:1");
        assert!(dedupe.first("r", "c:1", start + Duration::from_secs(62)));
        assert!(dedupe.seen.lock().unwrap().len() <= 2);
    }
}
