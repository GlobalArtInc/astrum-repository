use astrum_db::revision;
use astrum_policy::subject::COMPONENT_PREFIX;
use std::time::Instant;

use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement, TransactionTrait,
};

use super::{Config, Swept};

mod oci;

const SWEEP_LOCK: &str = "SELECT pg_try_advisory_xact_lock(hashtext('policy.sweep')) AS locked";

#[derive(Clone, Debug, Default)]
pub struct State {
    revision: Option<i64>,
    expired_until: Option<DateTime<Utc>>,
    orphans_at: Option<Instant>,
    orphans_after: (String, String),
}

pub async fn sweep(
    db: &DatabaseConnection,
    config: &Config,
    state: &mut State,
) -> Result<Swept, DbErr> {
    let guard = db.begin().await?;
    let locked: bool = guard
        .query_one(Statement::from_string(DbBackend::Postgres, SWEEP_LOCK))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the sweep lock query returned no row".to_owned()))?
        .try_get("", "locked")?;
    if !locked {
        guard.rollback().await?;
        return Ok(Swept::Busy);
    }
    let clock = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT clock_timestamp() AS at, \
               COALESCE((SELECT value FROM revision WHERE name = $1), 0) AS revision",
            [revision::POLICIES.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the sweep clock query returned no row".to_owned()))?;
    let now: DateTime<Utc> = clock.try_get("", "at")?;
    let current: i64 = clock.try_get("", "revision")?;
    let mut marked = 0u64;
    let from = state.expired_until.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    marked += mark_expired(db, config.chunk, from, now).await?;
    if state
        .orphans_at
        .is_none_or(|at| at.elapsed() >= config.orphan_every)
    {
        let (next, _) = oci::drop_orphans(db, &state.orphans_after).await?;
        state.orphans_after = next;
        state.orphans_at = Some(Instant::now());
    }
    state.expired_until = Some(now);
    if state.revision != Some(current) {
        marked += mark_stale(db, config.chunk).await?;
        marked += oci::mark_stale(db, config.chunk).await?;
        marked += mark_with_findings(db, config.chunk).await?;
        state.revision = Some(current);
    }
    guard.rollback().await?;
    Ok(Swept::Done { marked })
}

struct Page {
    found: i64,
    queued: i64,
    last_repository: Option<String>,
    last_subject: Option<String>,
}

fn page(row: Option<QueryResult>) -> Result<Page, DbErr> {
    let row =
        row.ok_or_else(|| DbErr::RecordNotFound("a sweep page returned no row".to_owned()))?;
    Ok(Page {
        found: row.try_get("", "n")?,
        queued: row.try_get("", "queued")?,
        last_repository: row.try_get("", "last_repository")?,
        last_subject: row.try_get("", "last_subject")?,
    })
}

async fn mark_expired(
    db: &DatabaseConnection,
    chunk: i64,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<u64, DbErr> {
    let mut marked = 0u64;
    loop {
        let Page { found, queued, .. } = page(
            db.query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "WITH page AS (\
                   SELECT DISTINCT v.repository, v.subject, v.component_id FROM policy_exception e \
                   JOIN policy_verdict v ON v.reasons <> '[]'::jsonb \
                     AND v.computed_at < e.expires_at \
                     AND v.reasons @> jsonb_build_array(\
                       jsonb_build_object('kind', 'exception', 'id', e.id::text)) \
                   WHERE e.expires_at > $1 AND e.expires_at <= $2 \
                     AND NOT EXISTS (SELECT 1 FROM policy_dirty d \
                                     WHERE d.repository = v.repository AND d.subject = v.subject) \
                   ORDER BY v.repository, v.subject LIMIT $3), \
                 live AS (\
                   SELECT name FROM repository WHERE name IN (SELECT repository FROM page) \
                   ORDER BY name FOR KEY SHARE), \
                 queued AS (\
                   INSERT INTO policy_dirty (repository, subject) \
                   SELECT p.repository, p.subject FROM page p JOIN live l ON l.name = p.repository \
                   ORDER BY (p.subject LIKE 'oci:%'), p.component_id, p.subject, p.repository ON CONFLICT (repository, subject) DO NOTHING RETURNING 1) \
                 SELECT count(*) AS n, (SELECT count(*) FROM queued) AS queued, NULL::text AS last_repository, NULL::text AS last_subject FROM page",
                [from.into(), to.into(), chunk.into()],
            ))
            .await?,
        )?;
        marked += queued as u64;
        if found < chunk {
            return Ok(marked);
        }
    }
}

async fn paged(
    db: &DatabaseConnection,
    chunk: i64,
    sql: &str,
    start: (&str, &str),
) -> Result<u64, DbErr> {
    let mut marked = 0u64;
    let mut after = (start.0.to_owned(), start.1.to_owned());
    loop {
        let Page {
            found,
            queued,
            last_repository,
            last_subject,
        } = page(
            db.query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql,
                [after.0.clone().into(), after.1.clone().into(), chunk.into()],
            ))
            .await?,
        )?;
        marked += queued as u64;
        match (found < chunk, last_repository, last_subject) {
            (false, Some(repository), Some(subject)) => after = (repository, subject),
            _ => return Ok(marked),
        }
    }
}

async fn mark_stale(db: &DatabaseConnection, chunk: i64) -> Result<u64, DbErr> {
    let sql = format!(
        "WITH page AS (\
           SELECT cs.repository, cs.component_id \
           FROM content_policy p \
           JOIN repository r ON r.attributes #>> '{{policy,name}}' = p.name \
           JOIN component_scan cs ON cs.repository = r.name \
           LEFT JOIN policy_verdict v \
             ON v.repository = cs.repository AND v.subject = '{COMPONENT_PREFIX}' || cs.component_id \
           WHERE (cs.repository, cs.component_id) > ($1, $2::bigint) \
             AND ((v.subject IS NULL AND ((p.vulnerabilities ->> 'mode') IS DISTINCT FROM 'off' \
                                       OR (p.licenses ->> 'mode') IS DISTINCT FROM 'off')) \
               OR (v.subject IS NOT NULL AND (v.policy_revision <> p.revision OR v.policy <> p.name))) \
           ORDER BY cs.repository, cs.component_id LIMIT $3), \
         live AS (\
           SELECT name FROM repository WHERE name IN (SELECT repository FROM page) \
           ORDER BY name FOR KEY SHARE), \
         queued AS (\
           INSERT INTO policy_dirty (repository, subject) \
           SELECT p.repository, '{COMPONENT_PREFIX}' || p.component_id \
           FROM page p JOIN live l ON l.name = p.repository \
           ORDER BY p.component_id ON CONFLICT (repository, subject) DO NOTHING RETURNING 1) \
         SELECT count(*) AS n, (SELECT count(*) FROM queued) AS queued, \
           (array_agg(repository ORDER BY repository DESC, component_id DESC))[1] AS last_repository, \
           (array_agg(component_id ORDER BY repository DESC, component_id DESC))[1]::text AS last_subject \
         FROM page"
    );
    paged(db, chunk, &sql, ("", "0")).await
}

async fn mark_with_findings(db: &DatabaseConnection, chunk: i64) -> Result<u64, DbErr> {
    paged(
        db,
        chunk,
        "WITH page AS (\
           SELECT v.repository, v.subject, v.component_id FROM policy_verdict v \
           WHERE (v.max_severity IS NOT NULL OR v.reasons @> '[{\"kind\":\"license\"}]'::jsonb) \
             AND (v.repository, v.subject) > ($1, $2) \
           ORDER BY v.repository, v.subject LIMIT $3), \
         live AS (\
           SELECT name FROM repository WHERE name IN (SELECT repository FROM page) \
           ORDER BY name FOR KEY SHARE), \
         queued AS (\
           INSERT INTO policy_dirty (repository, subject) \
           SELECT p.repository, p.subject FROM page p JOIN live l ON l.name = p.repository \
           ORDER BY (p.subject LIKE 'oci:%'), p.component_id, p.subject, p.repository ON CONFLICT (repository, subject) DO NOTHING RETURNING 1) \
         SELECT count(*) AS n, (SELECT count(*) FROM queued) AS queued, \
           (array_agg(repository ORDER BY repository DESC, subject DESC))[1] AS last_repository, \
           (array_agg(subject ORDER BY repository DESC, subject DESC))[1] AS last_subject FROM page",
        ("", ""),
    )
    .await
}
