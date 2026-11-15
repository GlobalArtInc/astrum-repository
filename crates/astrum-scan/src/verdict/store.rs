use std::collections::{BTreeMap, HashMap};
use std::future::Future;

use astrum_db::revision;
use astrum_policy::cache::{Cache, Snapshot};
use astrum_policy::policy::Mode;
use astrum_policy::subject::{self, COMPONENT_PREFIX, IMAGE_PREFIX};
use astrum_policy::verdict::{self, Finding, Input, LicenseFact, Scan, Target, Verdict};
use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, Statement,
    TransactionTrait,
};

use super::Config;
use super::metrics::STATES;

mod oci;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Judged {
    pub taken: u64,
    pub written: [u64; STATES.len()],
    pub dropped: u64,
    pub malformed: u64,
    pub busy: bool,
}

#[derive(Clone)]
struct Component {
    id: i64,
    repository: String,
    target: Target,
}

struct Row {
    repository: String,
    subject: String,
    policy: String,
    component_id: Option<i64>,
    verdict: Verdict,
}

fn custom(error: impl std::fmt::Display) -> DbErr {
    DbErr::Custom(error.to_string())
}

pub async fn judge_batch(
    db: &DatabaseConnection,
    cache: &Cache,
    config: &Config,
) -> Result<Judged, DbErr> {
    judge_batch_with(db, cache, config, || async {}).await
}

pub async fn judge_batch_with<F, Fut>(
    db: &DatabaseConnection,
    cache: &Cache,
    config: &Config,
    midway: F,
) -> Result<Judged, DbErr>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    judge(db, cache, config, None, midway).await
}

pub async fn judge_subject(
    db: &DatabaseConnection,
    cache: &Cache,
    config: &Config,
    repository: &str,
    subject: &str,
) -> Result<Judged, DbErr> {
    judge(db, cache, config, Some((repository, subject)), || async {}).await
}

async fn judge<F, Fut>(
    db: &DatabaseConnection,
    cache: &Cache,
    config: &Config,
    scope: Option<(&str, &str)>,
    midway: F,
) -> Result<Judged, DbErr>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let txn = db.begin().await?;
    let candidates = peek_repositories(&txn, config.batch, scope).await?;
    let contended = !candidates.is_empty();
    let policies = lock_repositories(&txn, &candidates).await?;
    if policies.is_empty() {
        txn.commit().await?;
        return Ok(Judged {
            busy: contended,
            ..Judged::default()
        });
    }
    let names: Vec<String> = policies.keys().cloned().collect();
    let marks = take_marks(&txn, &names, config.batch, scope).await?;
    let mut judged = Judged {
        taken: marks.len() as u64,
        busy: contended && marks.is_empty(),
        ..Judged::default()
    };
    if marks.is_empty() {
        txn.commit().await?;
        return Ok(judged);
    }
    let mut wanted: BTreeMap<i64, Vec<(String, String)>> = BTreeMap::new();
    let mut vanished: Vec<(String, String)> = Vec::new();
    let mut images: Vec<(String, String)> = Vec::new();
    for (repository, subject) in marks {
        if let Some(digest) = subject::image_digest(&subject) {
            images.push((repository, digest.to_owned()));
            continue;
        }
        match subject
            .strip_prefix(COMPONENT_PREFIX)
            .and_then(|id| id.parse::<i64>().ok())
        {
            Some(id) => wanted.entry(id).or_default().push((repository, subject)),
            None => {
                judged.malformed += 1;
                tracing::warn!(%repository, %subject, "a policy mark names no component, so it is dropped");
                vanished.push((repository, subject));
            }
        }
    }
    let mut cleared: Vec<(String, String)> = vanished;
    let mut picked: Vec<(String, String, String)> = Vec::new();
    for (repository, digest) in images {
        match policies.get(&repository).cloned().flatten() {
            Some(policy) => picked.push((repository, digest, policy)),
            None => cleared.push((repository, subject::image(&digest))),
        }
    }
    let ids: Vec<i64> = wanted.keys().copied().collect();
    let mut components: HashMap<i64, Component> = lock_components(&txn, &ids)
        .await?
        .into_iter()
        .map(|component| (component.id, component))
        .collect();
    let mut judging: Vec<(Component, String, String, String)> = Vec::new();
    for (id, marks) in wanted {
        let component = components.remove(&id);
        for (repository, subject) in marks {
            let present = component
                .as_ref()
                .is_some_and(|component| component.repository == repository);
            let policy = policies.get(&repository).cloned().flatten();
            match (present, policy) {
                (true, Some(policy)) => {
                    let held = component.clone().expect("a present component is held");
                    judging.push((held, subject, repository, policy));
                }
                _ => cleared.push((repository, subject)),
            }
        }
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut basis: Option<i64> = None;
    if !judging.is_empty() || !picked.is_empty() {
        let snapshot = cache.fresh(&txn).await.map_err(custom)?;
        basis = Some(snapshot.revision);
        let sound = |policy: &String| matches!(snapshot.policies.get(policy), Some(stored) if stored.policy.is_ok());
        let readable: Vec<i64> = judging
            .iter()
            .filter(|(_, _, _, policy)| sound(policy))
            .map(|(component, ..)| component.id)
            .collect();
        let licensed: Vec<i64> = judging
            .iter()
            .filter(|(_, _, _, policy)| {
                matches!(snapshot.policies.get(policy), Some(stored)
                    if stored.policy.as_ref().is_ok_and(|content| content.licenses.mode != Mode::Off))
            })
            .map(|(component, ..)| component.id)
            .collect();
        let scans = load_scans(&txn, &readable).await?;
        let findings = load_findings(&txn, &readable).await?;
        let licenses = load_licenses(&txn, &licensed).await?;
        let readable_images: Vec<oci::Key> = picked
            .iter()
            .filter(|(_, _, policy)| sound(policy))
            .map(|(repository, digest, _)| (repository.clone(), digest.clone()))
            .collect();
        let facts = oci::load(&txn, &readable_images).await?;
        let now = database_time(&txn).await?;
        for (component, subject, repository, policy) in judging {
            let verdict = evaluate_for(
                &snapshot,
                &policy,
                &component.target,
                Basis {
                    scan: scans.get(&component.id).copied().unwrap_or(Scan::Missing),
                    findings: findings
                        .get(&component.id)
                        .map(Vec::as_slice)
                        .unwrap_or_default(),
                    license: licenses.get(&component.id),
                    evaluate: verdict::evaluate,
                },
                now,
            );
            match verdict {
                Some(verdict) => rows.push(Row {
                    repository,
                    subject,
                    policy,
                    component_id: Some(component.id),
                    verdict,
                }),
                None => cleared.push((repository, subject)),
            }
        }
        for (repository, digest, policy) in picked {
            let subject = subject::image(&digest);
            let known = facts.get(&(repository.clone(), digest.clone()));
            let target = oci::target_of(&digest, known);
            let verdict = evaluate_for(
                &snapshot,
                &policy,
                &target,
                Basis {
                    scan: known.map_or(Scan::Missing, |known| known.scan),
                    findings: known.map_or(&[][..], |known| known.findings.as_slice()),
                    license: None,
                    evaluate: verdict::evaluate_vulnerabilities,
                },
                now,
            );
            match verdict {
                Some(verdict) => rows.push(Row {
                    repository,
                    subject,
                    policy,
                    component_id: None,
                    verdict,
                }),
                None => cleared.push((repository, subject)),
            }
        }
        rows.sort_by(|a, b| (&a.repository, &a.subject).cmp(&(&b.repository, &b.subject)));
        for row in &rows {
            let slot = STATES
                .iter()
                .position(|state| *state == row.verdict.state)
                .ok_or_else(|| {
                    custom(format!(
                        "the verdict state {} is not counted",
                        row.verdict.state.as_str()
                    ))
                })?;
            judged.written[slot] += 1;
        }
        write_rows(&txn, &rows, now).await?;
    }
    judged.dropped = delete_verdicts(&txn, &cleared).await?;
    midway().await;
    if let Some(basis) = basis
        && !rows.is_empty()
        && revision::current(&txn, revision::POLICIES).await? != basis
    {
        let again = requeue(&txn, &rows).await?;
        tracing::debug!(
            again,
            "the policies changed while a batch was judged, so its subjects are marked again"
        );
    }
    txn.commit().await?;
    Ok(judged)
}

struct Basis<'a> {
    scan: Scan,
    findings: &'a [Finding],
    license: Option<&'a LicenseFact>,
    evaluate: fn(&Input) -> Option<Verdict>,
}

fn evaluate_for(
    snapshot: &Snapshot,
    policy: &str,
    target: &Target,
    basis: Basis,
    now: DateTime<Utc>,
) -> Option<Verdict> {
    match snapshot.policies.get(policy) {
        None => Some(verdict::failed(
            0,
            format!("the content policy \"{policy}\" does not exist"),
        )),
        Some(stored) => match &stored.policy {
            Err(reason) => Some(verdict::failed(
                stored.revision,
                format!("the content policy \"{policy}\" cannot be read: {reason}"),
            )),
            Ok(content) => (basis.evaluate)(&Input {
                policy: content,
                exceptions: &snapshot.exceptions,
                target,
                scan: basis.scan,
                findings: basis.findings,
                license: basis.license,
                now,
            }),
        },
    }
}

async fn requeue(txn: &DatabaseTransaction, rows: &[Row]) -> Result<u64, DbErr> {
    let repositories: Vec<String> = rows.iter().map(|row| row.repository.clone()).collect();
    let subjects: Vec<String> = rows.iter().map(|row| row.subject.clone()).collect();
    let ids: Vec<i64> = rows
        .iter()
        .map(|row| row.component_id.unwrap_or_default())
        .collect();
    Ok(txn
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO policy_dirty (repository, subject) \
             SELECT repository, subject FROM unnest($1::text[], $2::text[], $3::bigint[]) \
               AS again(repository, subject, component_id) \
             ORDER BY (subject LIKE 'oci:%'), component_id, subject, repository \
             ON CONFLICT (repository, subject) DO NOTHING",
            [repositories.into(), subjects.into(), ids.into()],
        ))
        .await?
        .rows_affected())
}

async fn database_time(txn: &DatabaseTransaction) -> Result<DateTime<Utc>, DbErr> {
    txn.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT clock_timestamp() AS at",
    ))
    .await?
    .ok_or_else(|| DbErr::RecordNotFound("the clock query returned no row".to_owned()))?
    .try_get("", "at")
}

async fn peek_repositories(
    txn: &DatabaseTransaction,
    limit: i64,
    scope: Option<(&str, &str)>,
) -> Result<Vec<String>, DbErr> {
    let statement = match scope {
        None => Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "SELECT DISTINCT repository FROM (\
                   SELECT repository FROM policy_dirty \
                   WHERE subject LIKE '{COMPONENT_PREFIX}%' OR subject LIKE '{IMAGE_PREFIX}%' \
                   ORDER BY queued_at LIMIT $1) marks ORDER BY repository"
            ),
            [limit.into()],
        ),
        Some((repository, subject)) => Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT repository FROM policy_dirty WHERE repository = $1 AND subject = $2",
            [repository.into(), subject.into()],
        ),
    };
    let rows = txn.query_all(statement).await?;
    rows.iter()
        .map(|row| row.try_get("", "repository"))
        .collect()
}

async fn lock_repositories(
    txn: &DatabaseTransaction,
    names: &[String],
) -> Result<BTreeMap<String, Option<String>>, DbErr> {
    if names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT name, attributes #>> '{policy,name}' AS policy FROM repository \
             WHERE name = ANY($1) ORDER BY name FOR KEY SHARE",
            [names.to_vec().into()],
        ))
        .await?;
    let mut locked = BTreeMap::new();
    for row in rows {
        locked.insert(row.try_get("", "name")?, row.try_get("", "policy")?);
    }
    Ok(locked)
}

async fn take_marks(
    txn: &DatabaseTransaction,
    repositories: &[String],
    limit: i64,
    scope: Option<(&str, &str)>,
) -> Result<Vec<(String, String)>, DbErr> {
    let statement = match scope {
        None => Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "DELETE FROM policy_dirty WHERE (repository, subject) IN (\
                   SELECT repository, subject FROM policy_dirty \
                   WHERE repository = ANY($1) \
                     AND (subject LIKE '{COMPONENT_PREFIX}%' OR subject LIKE '{IMAGE_PREFIX}%') \
                   ORDER BY queued_at LIMIT $2 FOR UPDATE SKIP LOCKED) \
                 RETURNING repository, subject"
            ),
            [repositories.to_vec().into(), limit.into()],
        ),
        Some((repository, subject)) => Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM policy_dirty WHERE (repository, subject) IN (\
               SELECT repository, subject FROM policy_dirty \
               WHERE repository = $1 AND subject = $2 FOR UPDATE SKIP LOCKED) \
             RETURNING repository, subject",
            [repository.into(), subject.into()],
        ),
    };
    let rows = txn.query_all(statement).await?;
    rows.iter()
        .map(|row| Ok((row.try_get("", "repository")?, row.try_get("", "subject")?)))
        .collect()
}

async fn lock_components(txn: &DatabaseTransaction, ids: &[i64]) -> Result<Vec<Component>, DbErr> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT c.id, c.repository, c.format, c.namespace, c.name, c.version, cs.digest \
             FROM component c LEFT JOIN component_scan cs ON cs.component_id = c.id \
             WHERE c.id = ANY($1) ORDER BY c.id FOR KEY SHARE OF c",
            [ids.to_vec().into()],
        ))
        .await?;
    rows.iter()
        .map(|row| {
            Ok(Component {
                id: row.try_get("", "id")?,
                repository: row.try_get("", "repository")?,
                target: Target {
                    format: row.try_get("", "format")?,
                    namespace: row.try_get("", "namespace")?,
                    name: row.try_get("", "name")?,
                    version: row.try_get("", "version")?,
                    digest: row.try_get("", "digest")?,
                    peers: Vec::new(),
                },
            })
        })
        .collect()
}

async fn load_scans(txn: &DatabaseTransaction, ids: &[i64]) -> Result<HashMap<i64, Scan>, DbErr> {
    let mut scans = HashMap::new();
    if ids.is_empty() {
        return Ok(scans);
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT cs.component_id, cs.state, cs.inventory_id IS NULL AS detached, \
               EXISTS (SELECT 1 FROM sca_inventory_package ip \
                       JOIN sca_package p ON p.id = ip.package_id \
                       WHERE ip.inventory_id = cs.inventory_id AND p.matched_at IS NULL) AS pending \
             FROM component_scan cs WHERE cs.component_id = ANY($1)",
            [ids.to_vec().into()],
        ))
        .await?;
    for row in rows {
        let state: String = row.try_get("", "state")?;
        let detached: bool = row.try_get("", "detached")?;
        let pending: bool = row.try_get("", "pending")?;
        let scan = match Scan::parse(Some(&state)) {
            Scan::Scanned if detached => Scan::Unrecognised,
            Scan::Scanned if pending => Scan::Matching,
            other => other,
        };
        scans.insert(row.try_get("", "component_id")?, scan);
    }
    Ok(scans)
}

async fn load_findings(
    txn: &DatabaseTransaction,
    ids: &[i64],
) -> Result<HashMap<i64, Vec<Finding>>, DbErr> {
    let mut found: HashMap<i64, Vec<Finding>> = HashMap::new();
    if ids.is_empty() {
        return Ok(found);
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT cs.component_id, a.id AS advisory_id, a.aliases, a.severity, a.malicious, \
                    m.fixed, p.ecosystem, p.name, p.version \
             FROM component_scan cs \
             JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
             JOIN sca_match m ON m.package_id = ip.package_id \
             JOIN advisory a ON a.id = m.advisory_id AND a.withdrawn_at IS NULL \
             JOIN sca_package p ON p.id = ip.package_id \
             WHERE cs.component_id = ANY($1) \
             ORDER BY cs.component_id, a.id, p.id",
            [ids.to_vec().into()],
        ))
        .await?;
    for row in rows {
        let severity: String = row.try_get("", "severity")?;
        let (ecosystem, name, version): (String, String, String) = (
            row.try_get("", "ecosystem")?,
            row.try_get("", "name")?,
            row.try_get("", "version")?,
        );
        found
            .entry(row.try_get("", "component_id")?)
            .or_default()
            .push(Finding {
                advisory: row.try_get("", "advisory_id")?,
                aliases: row.try_get("", "aliases")?,
                severity: verdict::parse_severity(&severity),
                malicious: row.try_get("", "malicious")?,
                package: format!("{ecosystem}/{name}@{version}"),
                fixed: row.try_get("", "fixed")?,
            });
    }
    Ok(found)
}

async fn load_licenses(
    txn: &DatabaseTransaction,
    ids: &[i64],
) -> Result<HashMap<i64, LicenseFact>, DbErr> {
    let mut found = HashMap::new();
    if ids.is_empty() {
        return Ok(found);
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT component_id, state, expression, ids FROM component_license \
             WHERE component_id = ANY($1)",
            [ids.to_vec().into()],
        ))
        .await?;
    for row in rows {
        let state: String = row.try_get("", "state")?;
        found.insert(
            row.try_get("", "component_id")?,
            LicenseFact::stored(
                &state,
                row.try_get("", "expression")?,
                row.try_get("", "ids")?,
            ),
        );
    }
    Ok(found)
}

async fn write_rows(
    txn: &DatabaseTransaction,
    rows: &[Row],
    now: DateTime<Utc>,
) -> Result<(), DbErr> {
    if rows.is_empty() {
        return Ok(());
    }
    let text = |pick: fn(&Row) -> String| -> Vec<String> { rows.iter().map(pick).collect() };
    let number = |pick: fn(&Row) -> i32| -> Vec<i32> { rows.iter().map(pick).collect() };
    let reasons = rows
        .iter()
        .map(|row| serde_json::to_string(&row.verdict.reasons))
        .collect::<Result<Vec<String>, _>>()
        .map_err(custom)?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, \
           critical, high, medium, low, unknown, max_severity, license_state, reasons, policy_revision, \
           computed_at) \
         SELECT repository, subject, policy, NULLIF(component_id, 0), blocked, state, crit, hi, med, lo, unk, \
           NULLIF(top, ''), NULLIF(licensed, ''), reasons::jsonb, revision, $16 \
         FROM unnest($1::text[], $2::text[], $3::text[], $4::bigint[], $5::bool[], $6::text[], \
           $7::int[], $8::int[], $9::int[], $10::int[], $11::int[], $12::text[], $13::text[], \
           $14::bigint[], $15::text[]) \
           AS v(repository, subject, policy, component_id, blocked, state, crit, hi, med, lo, unk, \
             top, reasons, revision, licensed) \
         ORDER BY 1, 2 \
         ON CONFLICT (repository, subject) DO UPDATE SET policy = EXCLUDED.policy, \
           component_id = EXCLUDED.component_id, blocked = EXCLUDED.blocked, state = EXCLUDED.state, \
           critical = EXCLUDED.critical, high = EXCLUDED.high, medium = EXCLUDED.medium, \
           low = EXCLUDED.low, unknown = EXCLUDED.unknown, max_severity = EXCLUDED.max_severity, \
           license_state = EXCLUDED.license_state, reasons = EXCLUDED.reasons, policy_revision = EXCLUDED.policy_revision, \
           computed_at = EXCLUDED.computed_at",
        [
            text(|row| row.repository.clone()).into(),
            text(|row| row.subject.clone()).into(),
            text(|row| row.policy.clone()).into(),
            rows.iter()
                .map(|row| row.component_id.unwrap_or_default())
                .collect::<Vec<i64>>()
                .into(),
            rows.iter()
                .map(|row| row.verdict.blocked)
                .collect::<Vec<bool>>()
                .into(),
            text(|row| row.verdict.state.as_str().to_owned()).into(),
            number(|row| row.verdict.counts.critical).into(),
            number(|row| row.verdict.counts.high).into(),
            number(|row| row.verdict.counts.medium).into(),
            number(|row| row.verdict.counts.low).into(),
            number(|row| row.verdict.counts.unknown).into(),
            text(|row| {
                row.verdict
                    .max_severity
                    .map(|severity| verdict::severity_name(severity).to_owned())
                    .unwrap_or_default()
            })
            .into(),
            reasons.into(),
            rows.iter()
                .map(|row| row.verdict.policy_revision)
                .collect::<Vec<i64>>()
                .into(),
            text(|row| {
                row.verdict
                    .license_state
                    .map(|state| state.as_str().to_owned())
                    .unwrap_or_default()
            })
            .into(),
            now.into(),
        ],
    ))
    .await?;
    Ok(())
}

async fn delete_verdicts(
    txn: &DatabaseTransaction,
    keys: &[(String, String)],
) -> Result<u64, DbErr> {
    if keys.is_empty() {
        return Ok(0);
    }
    let repositories: Vec<String> = keys.iter().map(|key| key.0.clone()).collect();
    let subjects: Vec<String> = keys.iter().map(|key| key.1.clone()).collect();
    Ok(txn
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM policy_verdict v USING unnest($1::text[], $2::text[]) AS gone(repository, subject) \
             WHERE v.repository = gone.repository AND v.subject = gone.subject",
            [repositories.into(), subjects.into()],
        ))
        .await?
        .rows_affected())
}
