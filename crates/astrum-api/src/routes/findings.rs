use std::collections::{BTreeMap, HashMap};

use astrum_content::ids;
use astrum_policy::cache::Snapshot;
use astrum_policy::subject::{self, COMPONENT_PREFIX};
use astrum_policy::verdict::{self, Finding, Target};
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, SecondsFormat, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement,
    Value as SqlValue,
};
use serde_json::{Value, json};

use crate::routes::components::{Visibility, visibility, visible_component};
use crate::{ApiError, AppState, Caller, REST};

pub const SELECTOR_ROW_CAP: usize = 20_000;
pub const PAGE: usize = 50;
pub const MAX_PAGE: usize = 200;
pub const ADVISORY_COMPONENTS: usize = 100;
pub const EXAMINE_CAP: usize = 5_000;
pub const BATCH_MIN: usize = 500;
pub const COMPONENT_FINDINGS: usize = 1_000;
pub const REFERENCES: usize = 50;
pub const AFFECTED: usize = 50;
pub const RANGES: usize = 20;
pub const VERSIONS: usize = 100;
pub const EVENTS: usize = 100;
const ADVISORY_ID_LENGTH: usize = 256;
const SEVERITIES: [&str; 5] = ["critical", "high", "medium", "low", "unknown"];

const HITS: &str = "FROM component c \
    JOIN component_scan cs ON cs.component_id = c.id \
    JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
    JOIN sca_match m ON m.package_id = ip.package_id \
    JOIN advisory a ON a.id = m.advisory_id AND a.withdrawn_at IS NULL";

const RANK: &str = "(CASE a.severity WHEN 'critical' THEN 4 WHEN 'high' THEN 3 \
    WHEN 'medium' THEN 2 WHEN 'low' THEN 1 ELSE 0 END)";

const HAS_ASSET: &str =
    "EXISTS (SELECT 1 FROM asset s WHERE s.component_id = c.id AND s.repository = c.repository)";

pub(crate) const PATHS: &str = "ARRAY(SELECT s.path FROM asset s \
    WHERE s.component_id = c.id AND s.repository = c.repository)";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{REST}/v1/components/{{id}}/security"),
            get(component),
        )
        .route(&format!("{REST}/v1/security/findings"), get(findings))
        .route(
            &format!("{REST}/v1/security/advisories/{{id}}"),
            get(advisory),
        )
        .route(&format!("{REST}/v1/security/summary"), get(summary))
}

fn blocked_now(component: &str) -> String {
    format!(
        "EXISTS (SELECT 1 FROM repository r2 \
           JOIN content_policy cp2 ON cp2.name = r2.attributes #>> '{{policy,name}}' \
           JOIN policy_verdict pv2 ON pv2.repository = r2.name \
             AND pv2.subject = '{COMPONENT_PREFIX}' || {component}::text \
             AND pv2.policy = cp2.name AND pv2.policy_revision = cp2.revision \
           WHERE r2.name = c.repository AND pv2.blocked)"
    )
}

pub(crate) fn visible(full: &str, selected: &str) -> String {
    format!("((c.repository = ANY({full}) AND {HAS_ASSET}) OR c.repository = ANY({selected}))")
}

#[derive(Default)]
pub(crate) struct Params(Vec<SqlValue>);

impl Params {
    pub(crate) fn bind(&mut self, value: impl Into<SqlValue>) -> String {
        self.0.push(value.into());
        format!("${}", self.0.len())
    }
}

pub(crate) fn statement(sql: String, params: Params) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, params.0)
}

fn time(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn clean_text(value: Option<&str>) -> Option<String> {
    value.map(verdict::clean)
}

fn text_array(value: Option<Vec<String>>) -> Vec<String> {
    value.unwrap_or_default()
}

pub(crate) struct Scope {
    visibility: Visibility,
    pub(crate) full: Vec<String>,
    pub(crate) selected: Vec<String>,
}

impl Scope {
    pub(crate) async fn open(
        state: &AppState,
        caller: &Caller,
        only: Option<&str>,
    ) -> Result<Self, ApiError> {
        if caller.subject().is_none() {
            return Err(ApiError::Unauthorized);
        }
        let visibility = visibility(state, caller, only).await?;
        let full = visibility.full.iter().cloned().collect();
        let selected = visibility.selected.keys().cloned().collect();
        Ok(Self {
            visibility,
            full,
            selected,
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.full.is_empty() && self.selected.is_empty()
    }

    pub(crate) fn admits(&self, repository: &str, paths: Option<&[String]>) -> bool {
        self.visibility.full.contains(repository)
            || paths.is_some_and(|paths| {
                paths
                    .iter()
                    .any(|path| self.visibility.sees(repository, path))
            })
    }
}

#[derive(Default)]
struct Filters {
    severity: Option<String>,
    advisory: Option<String>,
    advisory_id: Option<String>,
    blocked: Option<bool>,
}

struct Cursor {
    rank: i32,
    component: i64,
    advisory: String,
}

impl Cursor {
    fn token(&self, state: &AppState) -> String {
        let sealed = state.secrets().seal(&format!(
            "after:{}:{}:{}",
            self.rank, self.component, self.advisory
        ));
        URL_SAFE_NO_PAD.encode(sealed)
    }

    fn parse(state: &AppState, token: &str) -> Result<Self, ApiError> {
        let invalid = || ApiError::BadRequest("the continuation token is not valid".to_owned());
        let sealed = URL_SAFE_NO_PAD
            .decode(token)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or_else(invalid)?;
        let text = state.secrets().open(&sealed).map_err(|_| invalid())?;
        let rest = text.strip_prefix("after:").ok_or_else(invalid)?;
        let mut parts = rest.splitn(3, ':');
        let rank: i32 = parts
            .next()
            .and_then(|part| part.parse().ok())
            .filter(|rank| (0..=4).contains(rank))
            .ok_or_else(invalid)?;
        let component: i64 = parts
            .next()
            .and_then(|part| part.parse().ok())
            .ok_or_else(invalid)?;
        let advisory = parts
            .next()
            .filter(|part| !part.is_empty() && part.len() <= ADVISORY_ID_LENGTH)
            .ok_or_else(invalid)?
            .to_owned();
        Ok(Self {
            rank,
            component,
            advisory,
        })
    }
}

struct Row {
    component_id: i64,
    advisory: String,
    rank: i32,
    repository: String,
    format: String,
    namespace: String,
    name: String,
    version: String,
    aliases: Vec<String>,
    severity: String,
    score: Option<f64>,
    malicious: bool,
    fixed: Option<String>,
    policy: Option<String>,
    blocked: bool,
    paths: Option<Vec<String>>,
}

impl Row {
    fn read(row: &QueryResult) -> Result<Self, DbErr> {
        Ok(Self {
            component_id: row.try_get("", "component_id")?,
            advisory: row.try_get("", "advisory_id")?,
            rank: row.try_get("", "rank")?,
            repository: row.try_get("", "repository")?,
            format: row.try_get("", "format")?,
            namespace: row.try_get("", "namespace")?,
            name: row.try_get("", "name")?,
            version: row.try_get("", "version")?,
            aliases: row.try_get("", "aliases")?,
            severity: row.try_get("", "severity")?,
            score: row.try_get("", "score")?,
            malicious: row.try_get("", "malicious")?,
            fixed: row.try_get("", "fixed")?,
            policy: row.try_get("", "policy")?,
            blocked: row.try_get("", "blocked")?,
            paths: row.try_get("", "paths")?,
        })
    }

    fn cursor(&self) -> Cursor {
        Cursor {
            rank: self.rank,
            component: self.component_id,
            advisory: self.advisory.clone(),
        }
    }

    fn excepted(&self, snapshot: &Snapshot, now: DateTime<Utc>) -> bool {
        let Some(policy) = &self.policy else {
            return false;
        };
        let target = Target {
            format: self.format.clone(),
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            digest: None,
        };
        let finding = Finding {
            advisory: self.advisory.clone(),
            aliases: self.aliases.clone(),
            severity: verdict::parse_severity(&self.severity),
            malicious: self.malicious,
            package: String::new(),
            fixed: self.fixed.clone(),
        };
        verdict::exception_for(&snapshot.exceptions, policy, &target, &finding, now).is_some()
    }
}

async fn fetch(
    db: &DatabaseConnection,
    scope: &Scope,
    filters: &Filters,
    after: Option<&Cursor>,
    limit: usize,
) -> Result<Vec<Row>, ApiError> {
    let mut params = Params::default();
    let full = params.bind(scope.full.clone());
    let selected = params.bind(scope.selected.clone());
    let mut conditions = vec![visible(&full, &selected)];
    if let Some(severity) = &filters.severity {
        let severity = params.bind(severity.clone());
        conditions.push(format!("a.severity = {severity}"));
    }
    if let Some(id) = &filters.advisory_id {
        let id = params.bind(id.clone());
        conditions.push(format!("a.id = {id}"));
    }
    if let Some(wanted) = &filters.advisory {
        let wanted = params.bind(wanted.clone());
        conditions.push(format!(
            "(lower(a.id) = lower({wanted}) OR EXISTS (SELECT 1 FROM unnest(a.aliases) AS alias(value) \
             WHERE lower(alias.value) = lower({wanted})))"
        ));
    }
    if let Some(blocked) = filters.blocked {
        let exists = blocked_now("c.id");
        conditions.push(if blocked {
            exists
        } else {
            format!("NOT {exists}")
        });
    }
    if let Some(after) = after {
        let rank = params.bind(after.rank);
        let component = params.bind(after.component);
        let advisory = params.bind(after.advisory.clone());
        conditions.push(format!(
            "({RANK} < {rank} OR ({RANK} = {rank} AND (c.id > {component} \
             OR (c.id = {component} AND a.id > {advisory}))))"
        ));
    }
    let size = params.bind(limit as i64);
    let sql = format!(
        "SELECT h.component_id, h.advisory_id, h.rank, h.fixed, c.repository, c.format, \
                c.namespace, c.name, c.version, a.aliases, a.severity, a.score::float8 AS score, \
                a.malicious, r.attributes #>> '{{policy,name}}' AS policy, \
                {blocked} AS blocked, \
                CASE WHEN c.repository = ANY({selected}) THEN {PATHS} END AS paths \
         FROM (SELECT c.id AS component_id, a.id AS advisory_id, {RANK} AS rank, \
                      min(m.fixed) AS fixed \
               {HITS} WHERE {where_clause} \
               GROUP BY c.id, a.id \
               ORDER BY {RANK} DESC, c.id, a.id LIMIT {size}) h \
         JOIN component c ON c.id = h.component_id \
         JOIN advisory a ON a.id = h.advisory_id \
         JOIN repository r ON r.name = c.repository \
         ORDER BY h.rank DESC, h.component_id, h.advisory_id",
        blocked = blocked_now("c.id"),
        where_clause = conditions.join(" AND "),
    );
    let rows = db.query_all(statement(sql, params)).await?;
    Ok(rows.iter().map(Row::read).collect::<Result<Vec<_>, _>>()?)
}

async fn page(
    state: &AppState,
    scope: &Scope,
    filters: &Filters,
    after: Option<Cursor>,
    limit: usize,
) -> Result<(Vec<Row>, Option<Cursor>), ApiError> {
    let mut items: Vec<Row> = Vec::new();
    if scope.is_empty() {
        return Ok((items, None));
    }
    let db = state.db().conn();
    let cap = state.examine_cap();
    let batch = (limit + 1).max(BATCH_MIN);
    let mut cursor = after;
    let mut examined = 0usize;
    loop {
        let rows = fetch(db, scope, filters, cursor.as_ref(), batch).await?;
        let fetched = rows.len();
        let mut last = None;
        for (index, row) in rows.into_iter().enumerate() {
            let key = row.cursor();
            if scope.admits(&row.repository, row.paths.as_deref()) {
                if items.len() == limit {
                    let next = items.last().map(Row::cursor);
                    return Ok((items, next));
                }
                items.push(row);
            }
            last = Some(key);
            examined += 1;
            let exhausted = fetched < batch && index + 1 == fetched;
            if examined >= cap && !exhausted {
                return Ok((items, last));
            }
        }
        if fetched < batch {
            return Ok((items, None));
        }
        cursor = last;
    }
}

async fn snapshot(state: &AppState) -> Result<std::sync::Arc<Snapshot>, ApiError> {
    state
        .policy_cache()
        .get(state.db().conn())
        .await
        .map_err(ApiError::internal)
}

fn severity_param(params: &HashMap<String, String>) -> Result<Option<String>, ApiError> {
    match params.get("severity") {
        None => Ok(None),
        Some(value) if SEVERITIES.contains(&value.as_str()) => Ok(Some(value.clone())),
        Some(_) => Err(ApiError::invalid(
            "severity",
            "must be critical, high, medium, low or unknown",
        )),
    }
}

fn blocked_param(params: &HashMap<String, String>) -> Result<Option<bool>, ApiError> {
    match params.get("blocked").map(String::as_str) {
        None => Ok(None),
        Some("true") => Ok(Some(true)),
        Some("false") => Ok(Some(false)),
        Some(_) => Err(ApiError::invalid("blocked", "must be true or false")),
    }
}

fn advisory_param(params: &HashMap<String, String>) -> Result<Option<String>, ApiError> {
    match params.get("advisory") {
        None => Ok(None),
        Some(value) if value.is_empty() || value.len() > ADVISORY_ID_LENGTH => Err(
            ApiError::invalid("advisory", "must be an advisory id or alias"),
        ),
        Some(value) => Ok(Some(value.clone())),
    }
}

fn limit_param(params: &HashMap<String, String>) -> Result<usize, ApiError> {
    match params.get("limit") {
        None => Ok(PAGE),
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|limit| (1..=MAX_PAGE).contains(limit))
            .ok_or_else(|| ApiError::invalid("limit", format!("must be between 1 and {MAX_PAGE}"))),
    }
}

pub(crate) fn repository_param(params: &HashMap<String, String>) -> Result<Option<&str>, ApiError> {
    match params.get("repository") {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => {
            Err(ApiError::invalid("repository", "must not be empty"))
        }
        Some(value) => Ok(Some(value.as_str())),
    }
}

async fn findings(
    State(state): State<AppState>,
    caller: Caller,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let repository = repository_param(&params)?;
    let filters = Filters {
        severity: severity_param(&params)?,
        advisory: advisory_param(&params)?,
        advisory_id: None,
        blocked: blocked_param(&params)?,
    };
    let limit = limit_param(&params)?;
    let after = params
        .get("continuationToken")
        .filter(|token| !token.is_empty())
        .map(|token| Cursor::parse(&state, token))
        .transpose()?;
    let scope = Scope::open(&state, &caller, repository).await?;
    let (rows, next) = page(&state, &scope, &filters, after, limit).await?;
    let items = if rows.is_empty() {
        Vec::new()
    } else {
        let snapshot = snapshot(&state).await?;
        let now = Utc::now();
        rows.iter()
            .map(|row| {
                json!({
                    "componentId": ids::encode(&row.repository, row.component_id),
                    "repository": row.repository,
                    "format": row.format,
                    "namespace": row.namespace,
                    "name": row.name,
                    "version": row.version,
                    "advisory": verdict::clean(&row.advisory),
                    "severity": row.severity,
                    "score": row.score,
                    "fixed": clean_text(row.fixed.as_deref()),
                    "malicious": row.malicious,
                    "excepted": row.excepted(&snapshot, now),
                    "blocked": row.blocked,
                })
            })
            .collect()
    };
    Ok(Json(json!({
        "items": items,
        "continuationToken": next.map(|cursor| cursor.token(&state)),
    })))
}

async fn component(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let (_, found, _) = visible_component(&state, &caller, &id).await?;
    let db = state.db().conn();
    let scan = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT cs.state, cs.scanned_at, \
                    CASE WHEN cs.inventory_id IS NULL THEN NULL ELSE \
                      (SELECT count(*) FROM sca_inventory_package ip \
                       WHERE ip.inventory_id = cs.inventory_id) END AS packages \
             FROM component_scan cs WHERE cs.component_id = $1",
            [found.id.into()],
        ))
        .await?;
    let scan = match scan {
        Some(row) => {
            let at: DateTime<Utc> = row.try_get("", "scanned_at")?;
            let packages: Option<i64> = row.try_get("", "packages")?;
            json!({
                "state": row.try_get::<String>("", "state")?,
                "scannedAt": time(at),
                "packages": packages,
            })
        }
        None => {
            let queued = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT EXISTS (SELECT 1 FROM scan_queue WHERE component_id = $1) AS queued",
                    [found.id.into()],
                ))
                .await?
                .map(|row| row.try_get::<bool>("", "queued"))
                .transpose()?
                .unwrap_or(false);
            json!({
                "state": if queued { "queued" } else { "unscanned" },
                "scannedAt": Value::Null,
                "packages": Value::Null,
            })
        }
    };
    let current = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT r.attributes #>> '{policy,name}' AS policy, cp.revision, \
                    pv.blocked, pv.state, pv.critical, pv.high, pv.medium, pv.low, pv.unknown, \
                    pv.max_severity, pv.reasons, pv.computed_at \
             FROM repository r \
             LEFT JOIN content_policy cp ON cp.name = r.attributes #>> '{policy,name}' \
             LEFT JOIN policy_verdict pv ON pv.repository = r.name AND pv.subject = $2 \
               AND pv.policy = cp.name AND pv.policy_revision = cp.revision \
             WHERE r.name = $1",
            [
                found.repository.clone().into(),
                subject::component(found.id).into(),
            ],
        ))
        .await?;
    let (policy, verdict) = match current {
        None => (Value::Null, Value::Null),
        Some(row) => {
            let name: Option<String> = row.try_get("", "policy")?;
            let revision: Option<i64> = row.try_get("", "revision")?;
            let state: Option<String> = row.try_get("", "state")?;
            let policy = match (&name, revision) {
                (Some(name), revision) => json!({"name": name, "revision": revision}),
                (None, _) => Value::Null,
            };
            let verdict = match (&name, state) {
                (Some(name), Some(state)) => {
                    let at: DateTime<Utc> = row.try_get("", "computed_at")?;
                    json!({
                        "policy": name,
                        "revision": revision,
                        "state": state,
                        "blocked": row.try_get::<bool>("", "blocked")?,
                        "counts": {
                            "critical": row.try_get::<i32>("", "critical")?,
                            "high": row.try_get::<i32>("", "high")?,
                            "medium": row.try_get::<i32>("", "medium")?,
                            "low": row.try_get::<i32>("", "low")?,
                            "unknown": row.try_get::<i32>("", "unknown")?,
                        },
                        "maxSeverity": row.try_get::<Option<String>>("", "max_severity")?,
                        "reasons": row.try_get::<Value>("", "reasons")?,
                        "computedAt": time(at),
                    })
                }
                _ => Value::Null,
            };
            (policy, verdict)
        }
    };
    let policy_name = policy
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "SELECT a.id, a.aliases, a.severity, a.score::float8 AS score, a.summary, \
                        a.malicious, min(m.fixed) AS fixed \
                 FROM component_scan cs \
                 JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
                 JOIN sca_match m ON m.package_id = ip.package_id \
                 JOIN advisory a ON a.id = m.advisory_id AND a.withdrawn_at IS NULL \
                 WHERE cs.component_id = $1 \
                 GROUP BY a.id ORDER BY {RANK} DESC, a.id LIMIT $2"
            ),
            [found.id.into(), ((COMPONENT_FINDINGS + 1) as i64).into()],
        ))
        .await?;
    let truncated = rows.len() > COMPONENT_FINDINGS;
    let mut vulnerabilities = Vec::new();
    if !rows.is_empty() {
        let snapshot = snapshot(&state).await?;
        let now = Utc::now();
        let target = Target {
            format: found.format.clone(),
            namespace: found.namespace.clone(),
            name: found.name.clone(),
            version: found.version.clone(),
            digest: None,
        };
        for row in rows.iter().take(COMPONENT_FINDINGS) {
            let advisory: String = row.try_get("", "id")?;
            let aliases: Vec<String> = row.try_get("", "aliases")?;
            let severity: String = row.try_get("", "severity")?;
            let malicious: bool = row.try_get("", "malicious")?;
            let fixed: Option<String> = row.try_get("", "fixed")?;
            let excepted = policy_name.as_deref().is_some_and(|policy| {
                let finding = Finding {
                    advisory: advisory.clone(),
                    aliases: aliases.clone(),
                    severity: verdict::parse_severity(&severity),
                    malicious,
                    package: String::new(),
                    fixed: fixed.clone(),
                };
                verdict::exception_for(&snapshot.exceptions, policy, &target, &finding, now)
                    .is_some()
            });
            let summary: String = row.try_get("", "summary")?;
            vulnerabilities.push(json!({
                "id": verdict::clean(&advisory),
                "aliases": aliases.iter().map(|alias| verdict::clean(alias)).collect::<Vec<_>>(),
                "severity": severity,
                "score": row.try_get::<Option<f64>>("", "score")?,
                "summary": verdict::clean(&summary),
                "fixed": clean_text(fixed.as_deref()),
                "malicious": malicious,
                "excepted": excepted,
            }));
        }
    }
    Ok(Json(json!({
        "scan": scan,
        "policy": policy,
        "verdict": verdict,
        "vulnerabilities": vulnerabilities,
        "truncated": truncated,
        "license": Value::Null,
    })))
}

async fn visible_count(
    state: &AppState,
    scope: &Scope,
    advisory: &str,
) -> Result<(i64, bool), ApiError> {
    let db = state.db().conn();
    let mut total = 0i64;
    if !scope.full.is_empty() {
        let mut params = Params::default();
        let full = params.bind(scope.full.clone());
        let id = params.bind(advisory.to_owned());
        let row = db
            .query_one(statement(
                format!(
                    "SELECT count(DISTINCT c.id) AS n {HITS} \
                     WHERE a.id = {id} AND c.repository = ANY({full}) AND {HAS_ASSET}"
                ),
                params,
            ))
            .await?
            .ok_or_else(|| ApiError::internal(anyhow::anyhow!("a count returned no row")))?;
        total += row.try_get::<i64>("", "n")?;
    }
    let mut truncated = false;
    if !scope.selected.is_empty() {
        let cap = state.selector_row_cap();
        let mut params = Params::default();
        let selected = params.bind(scope.selected.clone());
        let id = params.bind(advisory.to_owned());
        let size = params.bind((cap + 1) as i64);
        let rows = db
            .query_all(statement(
                format!(
                    "SELECT c.id, c.repository, {PATHS} AS paths {HITS} \
                     WHERE a.id = {id} AND c.repository = ANY({selected}) \
                     GROUP BY c.id ORDER BY c.id LIMIT {size}"
                ),
                params,
            ))
            .await?;
        truncated = rows.len() > cap;
        for row in rows.iter().take(cap) {
            let repository: String = row.try_get("", "repository")?;
            let paths = text_array(row.try_get("", "paths")?);
            if scope.admits(&repository, Some(&paths)) {
                total += 1;
            }
        }
    }
    Ok((total, truncated))
}

fn references(document: Option<Value>) -> (Vec<Value>, bool) {
    let Some(Value::Array(items)) = document else {
        return (Vec::new(), false);
    };
    let truncated = items.len() > REFERENCES;
    let kept = items
        .iter()
        .filter_map(|item| {
            let url = item.get("url")?.as_str()?;
            let plain = url.chars().all(|ch| ch.is_ascii_graphic());
            if !plain
                || !(url.starts_with("https://") || url.starts_with("http://"))
                || url.len() > 512
            {
                return None;
            }
            let kind = item.get("type").and_then(Value::as_str).unwrap_or_default();
            Some(json!({"type": verdict::clean(kind), "url": url}))
        })
        .take(REFERENCES)
        .collect();
    (kept, truncated)
}

async fn advisory(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let scope = Scope::open(&state, &caller, None).await?;
    if scope.is_empty() {
        return Err(ApiError::forbidden());
    }
    let db = state.db().conn();
    let missing = || ApiError::not_found("Advisory not found");
    if id.len() > ADVISORY_ID_LENGTH {
        return Err(missing());
    }
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, aliases, summary, severity, score::float8 AS score, malicious, \
                    published_at, modified_at, withdrawn_at, document #> '{references}' AS refs \
             FROM advisory WHERE id = $1",
            [id.clone().into()],
        ))
        .await?
        .ok_or_else(missing)?;
    let (references, references_truncated) = references(row.try_get("", "refs")?);
    let affected_rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT ecosystem, name, ranges, versions FROM advisory_affected \
             WHERE advisory_id = $1 ORDER BY ecosystem, name LIMIT $2",
            [id.clone().into(), ((AFFECTED + 1) as i64).into()],
        ))
        .await?;
    let mut affected = Vec::new();
    let mut affected_truncated = affected_rows.len() > AFFECTED;
    for entry in affected_rows.iter().take(AFFECTED) {
        let ranges = match entry.try_get::<Value>("", "ranges")? {
            Value::Array(items) => {
                affected_truncated |= items.len() > RANGES;
                items
                    .into_iter()
                    .take(RANGES)
                    .map(|mut range| {
                        if let Some(Value::Array(events)) = range.get_mut("events") {
                            affected_truncated |= events.len() > EVENTS;
                            events.truncate(EVENTS);
                        }
                        range
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        let versions: Vec<String> = entry.try_get("", "versions")?;
        affected_truncated |= versions.len() > VERSIONS;
        affected.push(json!({
            "ecosystem": entry.try_get::<String>("", "ecosystem")?,
            "name": entry.try_get::<String>("", "name")?,
            "ranges": ranges,
            "versions": versions.into_iter().take(VERSIONS).collect::<Vec<_>>(),
        }));
    }
    let filters = Filters {
        advisory_id: Some(id.clone()),
        ..Filters::default()
    };
    let (rows, next) = page(&state, &scope, &filters, None, ADVISORY_COMPONENTS).await?;
    let (count, count_truncated) = visible_count(&state, &scope, &id).await?;
    let components = if rows.is_empty() {
        Vec::new()
    } else {
        let snapshot = snapshot(&state).await?;
        let now = Utc::now();
        rows.iter()
            .map(|row| {
                json!({
                    "componentId": ids::encode(&row.repository, row.component_id),
                    "repository": row.repository,
                    "format": row.format,
                    "name": row.name,
                    "version": row.version,
                    "fixed": clean_text(row.fixed.as_deref()),
                    "excepted": row.excepted(&snapshot, now),
                    "blocked": row.blocked,
                })
            })
            .collect()
    };
    let aliases: Vec<String> = row.try_get("", "aliases")?;
    let summary: String = row.try_get("", "summary")?;
    let published: Option<DateTime<Utc>> = row.try_get("", "published_at")?;
    let modified: DateTime<Utc> = row.try_get("", "modified_at")?;
    let withdrawn: Option<DateTime<Utc>> = row.try_get("", "withdrawn_at")?;
    Ok(Json(json!({
        "id": verdict::clean(&id),
        "aliases": aliases.iter().map(|alias| verdict::clean(alias)).collect::<Vec<_>>(),
        "summary": verdict::clean(&summary),
        "severity": row.try_get::<String>("", "severity")?,
        "score": row.try_get::<Option<f64>>("", "score")?,
        "malicious": row.try_get::<bool>("", "malicious")?,
        "published": published.map(time),
        "modified": time(modified),
        "withdrawn": withdrawn.map(time),
        "references": references,
        "affected": affected,
        "truncated": references_truncated || affected_truncated,
        "components": components,
        "componentCount": count,
        "componentsTruncated": count_truncated || (next.is_some() && rows.len() < ADVISORY_COMPONENTS),
    })))
}

#[derive(Default)]
struct Tally {
    format: String,
    scanned: i64,
    blocked: i64,
    vulnerable: i64,
    counts: [i64; 5],
    truncated: bool,
}

impl Tally {
    fn json(&self, repository: &str) -> Value {
        json!({
            "repository": repository,
            "format": self.format,
            "critical": self.counts[0],
            "high": self.counts[1],
            "medium": self.counts[2],
            "low": self.counts[3],
            "unknown": self.counts[4],
            "blocked": self.blocked,
            "componentsScanned": self.scanned,
            "componentsVulnerable": self.vulnerable,
            "truncated": self.truncated,
        })
    }
}

const SEVERITY_COUNTS: &str = "count(*) FILTER (WHERE {s} = 'critical') AS critical, \
    count(*) FILTER (WHERE {s} = 'high') AS high, \
    count(*) FILTER (WHERE {s} = 'medium') AS medium, \
    count(*) FILTER (WHERE {s} = 'low') AS low, \
    count(*) FILTER (WHERE {s} NOT IN ('critical', 'high', 'medium', 'low')) AS unknown";

fn severity_counts(column: &str) -> String {
    SEVERITY_COUNTS.replace("{s}", column)
}

async fn full_tallies(
    state: &AppState,
    scope: &Scope,
    tallies: &mut BTreeMap<String, Tally>,
) -> Result<(), ApiError> {
    if scope.full.is_empty() {
        return Ok(());
    }
    let db = state.db().conn();
    let mut params = Params::default();
    let full = params.bind(scope.full.clone());
    let rows = db
        .query_all(statement(
            format!(
                "SELECT c.repository, min(c.format) AS format, \
                        count(*) FILTER (WHERE cs.state = 'scanned') AS scanned, \
                        count(*) FILTER (WHERE {blocked}) AS blocked \
                 FROM component c JOIN component_scan cs ON cs.component_id = c.id \
                 WHERE c.repository = ANY({full}) AND {HAS_ASSET} \
                 GROUP BY c.repository",
                blocked = blocked_now("c.id"),
            ),
            params,
        ))
        .await?;
    for row in &rows {
        let repository: String = row.try_get("", "repository")?;
        let tally = tallies.entry(repository).or_default();
        tally.format = row.try_get("", "format")?;
        tally.scanned = row.try_get("", "scanned")?;
        tally.blocked = row.try_get("", "blocked")?;
    }
    let mut params = Params::default();
    let full = params.bind(scope.full.clone());
    let rows = db
        .query_all(statement(
            format!(
                "SELECT p.repository, count(DISTINCT p.component_id) AS vulnerable, {counts} \
                 FROM (SELECT c.id AS component_id, c.repository, a.id AS advisory, a.severity \
                       {HITS} WHERE c.repository = ANY({full}) AND {HAS_ASSET} \
                       GROUP BY c.id, a.id) p \
                 GROUP BY p.repository",
                counts = severity_counts("p.severity"),
            ),
            params,
        ))
        .await?;
    for row in &rows {
        let repository: String = row.try_get("", "repository")?;
        let tally = tallies.entry(repository).or_default();
        tally.vulnerable = row.try_get("", "vulnerable")?;
        tally.counts = [
            row.try_get("", "critical")?,
            row.try_get("", "high")?,
            row.try_get("", "medium")?,
            row.try_get("", "low")?,
            row.try_get("", "unknown")?,
        ];
    }
    Ok(())
}

async fn selected_tally(
    state: &AppState,
    scope: &Scope,
    repository: &str,
) -> Result<Option<Tally>, ApiError> {
    let cap = state.selector_row_cap();
    let mut params = Params::default();
    let name = params.bind(repository.to_owned());
    let size = params.bind((cap + 1) as i64);
    let rows = state
        .db()
        .conn()
        .query_all(statement(
            format!(
                "SELECT c.id, c.repository, c.format, cs.state, {blocked} AS blocked, {PATHS} AS paths, \
                        f.critical, f.high, f.medium, f.low, f.unknown \
                 FROM component c JOIN component_scan cs ON cs.component_id = c.id \
                 LEFT JOIN LATERAL (SELECT {counts} \
                      FROM (SELECT DISTINCT a.id, a.severity \
                            FROM sca_inventory_package ip \
                            JOIN sca_match m ON m.package_id = ip.package_id \
                            JOIN advisory a ON a.id = m.advisory_id AND a.withdrawn_at IS NULL \
                            WHERE ip.inventory_id = cs.inventory_id) d) f ON true \
                 WHERE c.repository = {name} ORDER BY c.id LIMIT {size}",
                blocked = blocked_now("c.id"),
                counts = severity_counts("d.severity"),
            ),
            params,
        ))
        .await?;
    let mut tally = Tally {
        truncated: rows.len() > cap,
        ..Tally::default()
    };
    let mut admitted = 0usize;
    for row in rows.iter().take(cap) {
        let paths = text_array(row.try_get("", "paths")?);
        if !scope.admits(repository, Some(&paths)) {
            continue;
        }
        admitted += 1;
        tally.format = row.try_get("", "format")?;
        if row.try_get::<String>("", "state")? == "scanned" {
            tally.scanned += 1;
        }
        if row.try_get::<bool>("", "blocked")? {
            tally.blocked += 1;
        }
        let counts = [
            row.try_get::<i64>("", "critical")?,
            row.try_get::<i64>("", "high")?,
            row.try_get::<i64>("", "medium")?,
            row.try_get::<i64>("", "low")?,
            row.try_get::<i64>("", "unknown")?,
        ];
        if counts.iter().sum::<i64>() > 0 {
            tally.vulnerable += 1;
        }
        for (slot, count) in tally.counts.iter_mut().zip(counts) {
            *slot += count;
        }
    }
    Ok((admitted > 0).then_some(tally))
}

async fn summary(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    let scope = Scope::open(&state, &caller, None).await?;
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    full_tallies(&state, &scope, &mut tallies).await?;
    for repository in &scope.selected {
        if let Some(tally) = selected_tally(&state, &scope, repository).await? {
            tallies.insert(repository.clone(), tally);
        }
    }
    let mut totals = Tally::default();
    for tally in tallies.values() {
        totals.scanned += tally.scanned;
        totals.blocked += tally.blocked;
        totals.vulnerable += tally.vulnerable;
        totals.truncated |= tally.truncated;
        for (slot, count) in totals.counts.iter_mut().zip(tally.counts) {
            *slot += count;
        }
    }
    Ok(Json(json!({
        "generatedAt": time(Utc::now()),
        "totals": {
            "critical": totals.counts[0],
            "high": totals.counts[1],
            "medium": totals.counts[2],
            "low": totals.counts[3],
            "unknown": totals.counts[4],
            "blocked": totals.blocked,
            "componentsScanned": totals.scanned,
            "componentsVulnerable": totals.vulnerable,
            "truncated": totals.truncated,
        },
        "repositories": tallies
            .iter()
            .map(|(repository, tally)| tally.json(repository))
            .collect::<Vec<_>>(),
    })))
}
