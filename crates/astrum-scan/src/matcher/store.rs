use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::future::Future;

use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, Statement,
    TransactionTrait,
};

use super::Config;
use super::evaluate::{self, Candidate, Evaluation, Found};
use crate::version::Ecosystem;

const IMPORT_LOCK: &str = "SELECT pg_advisory_xact_lock_shared(hashtext('advisories.import'))";
const WRITE_CHUNK: usize = 5000;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Change {
    pub package_id: i64,
    pub advisory_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marked {
    pub package_id: i64,
    pub token: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Batch {
    pub matched: u64,
    pub added: Vec<Change>,
    pub removed: Vec<Change>,
    pub updated: Vec<Change>,
    pub marked: Vec<Marked>,
    pub skipped_advisories: u64,
    pub unresolved_packages: u64,
    pub truncated_packages: u64,
}

struct Pending {
    id: i64,
    ecosystem: String,
    name: String,
    version: String,
}

pub async fn match_batch(db: &DatabaseConnection, config: &Config) -> Result<Batch, DbErr> {
    match_batch_with(db, config, || async {}).await
}

pub async fn match_batch_with<F, Fut>(
    db: &DatabaseConnection,
    config: &Config,
    midway: F,
) -> Result<Batch, DbErr>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    run(db, config, None, midway).await
}

pub async fn match_component(
    db: &DatabaseConnection,
    config: &Config,
    component_id: i64,
) -> Result<Batch, DbErr> {
    run(db, config, Some(component_id), || async {}).await
}

async fn run<F, Fut>(
    db: &DatabaseConnection,
    config: &Config,
    component_id: Option<i64>,
    midway: F,
) -> Result<Batch, DbErr>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let txn = db.begin().await?;
    txn.execute_unprepared(IMPORT_LOCK).await?;
    let pending = lock_unmatched(&txn, config.batch, component_id).await?;
    if pending.is_empty() {
        txn.commit().await?;
        return Ok(Batch::default());
    }
    let mut batch = Batch {
        matched: pending.len() as u64,
        ..Batch::default()
    };
    let mut resolved = Vec::with_capacity(pending.len());
    for package in &pending {
        match evaluate::resolve(&package.ecosystem, &package.version) {
            Some(ecosystem) => resolved.push((package, ecosystem)),
            None => batch.unresolved_packages += 1,
        }
    }
    let keys: BTreeSet<(&str, &str)> = resolved
        .iter()
        .map(|(package, _)| (package.ecosystem.as_str(), package.name.as_str()))
        .collect();
    let mut candidates = fetch_candidates(&txn, &keys, config.cap).await?;
    let cut = cap_candidates(&txn, &mut candidates, config.cap).await?;
    batch.truncated_packages = resolved
        .iter()
        .filter(|(package, _)| cut.contains(&(package.ecosystem.clone(), package.name.clone())))
        .count() as u64;
    let jobs: Vec<Job> = resolved
        .iter()
        .map(|(package, ecosystem)| Job {
            id: package.id,
            ecosystem: *ecosystem,
            version: package.version.clone(),
            key: (package.ecosystem.clone(), package.name.clone()),
        })
        .collect();
    let evaluated = tokio::task::spawn_blocking(move || {
        jobs.into_iter()
            .map(|job| {
                let evaluation = match candidates.get(&job.key) {
                    Some(found) => evaluate::evaluate(job.ecosystem, &job.version, found),
                    None => Evaluation::default(),
                };
                (job.id, evaluation)
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|error| DbErr::Custom(format!("the advisory evaluation task failed: {error}")))?;
    let mut current: BTreeMap<i64, Vec<Found>> = pending
        .iter()
        .map(|package| (package.id, Vec::new()))
        .collect();
    for (id, evaluation) in evaluated {
        batch.skipped_advisories += evaluation.skipped;
        current.insert(id, evaluation.found);
    }
    midway().await;
    let ids: Vec<i64> = pending.iter().map(|package| package.id).collect();
    let stored = stored_matches(&txn, &ids).await?;
    let mut upsert: Vec<(i64, &Found)> = Vec::new();
    let mut delete: Vec<(i64, &str)> = Vec::new();
    for (package_id, found) in &current {
        let before = stored.get(package_id);
        for item in found {
            let change = Change {
                package_id: *package_id,
                advisory_id: item.advisory_id.clone(),
            };
            match before.and_then(|rows| rows.get(&item.advisory_id)) {
                None => {
                    upsert.push((*package_id, item));
                    batch.added.push(change);
                }
                Some(fixed) if *fixed != item.fixed => {
                    upsert.push((*package_id, item));
                    batch.updated.push(change);
                }
                Some(_) => {}
            }
        }
        if let Some(rows) = before {
            let kept: BTreeSet<&str> = found.iter().map(|item| item.advisory_id.as_str()).collect();
            for advisory_id in rows.keys().filter(|id| !kept.contains(id.as_str())) {
                delete.push((*package_id, advisory_id));
                batch.removed.push(Change {
                    package_id: *package_id,
                    advisory_id: advisory_id.clone(),
                });
            }
        }
    }
    for chunk in delete.chunks(WRITE_CHUNK) {
        delete_matches(&txn, chunk).await?;
    }
    for chunk in upsert.chunks(WRITE_CHUNK) {
        upsert_matches(&txn, chunk).await?;
    }
    batch.marked = finish(&txn, &ids).await?;
    txn.commit().await?;
    Ok(batch)
}

struct Job {
    id: i64,
    ecosystem: Ecosystem,
    version: String,
    key: (String, String),
}

async fn lock_unmatched(
    txn: &DatabaseTransaction,
    limit: i64,
    component_id: Option<i64>,
) -> Result<Vec<Pending>, DbErr> {
    let statement = match component_id {
        None => Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, ecosystem, name, version FROM sca_package WHERE matched_at IS NULL \
             ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED",
            [limit.into()],
        ),
        Some(component_id) => Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, ecosystem, name, version FROM sca_package WHERE matched_at IS NULL \
               AND id IN (SELECT ip.package_id FROM component_scan cs \
                          JOIN sca_inventory_package ip ON ip.inventory_id = cs.inventory_id \
                          WHERE cs.component_id = $2) \
             ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED",
            [limit.into(), component_id.into()],
        ),
    };
    let rows = txn.query_all(statement).await?;
    let mut pending = Vec::with_capacity(rows.len());
    for row in rows {
        pending.push(Pending {
            id: row.try_get("", "id")?,
            ecosystem: row.try_get("", "ecosystem")?,
            name: row.try_get("", "name")?,
            version: row.try_get("", "version")?,
        });
    }
    Ok(pending)
}

async fn fetch_candidates(
    txn: &DatabaseTransaction,
    keys: &BTreeSet<(&str, &str)>,
    cap: usize,
) -> Result<HashMap<(String, String), Vec<Candidate>>, DbErr> {
    let mut found: HashMap<(String, String), Vec<Candidate>> = HashMap::new();
    if keys.is_empty() {
        return Ok(found);
    }
    let ecosystems: Vec<String> = keys.iter().map(|key| key.0.to_owned()).collect();
    let names: Vec<String> = keys.iter().map(|key| key.1.to_owned()).collect();
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT k.ecosystem, k.name, x.advisory_id, x.ranges, x.versions \
             FROM unnest($1::text[], $2::text[]) AS k(ecosystem, name) \
             CROSS JOIN LATERAL (\
               SELECT f.advisory_id, f.ranges, f.versions FROM advisory_affected f \
               JOIN advisory a ON a.id = f.advisory_id \
               WHERE f.ecosystem = k.ecosystem AND f.name = k.name AND a.withdrawn_at IS NULL \
               ORDER BY f.advisory_id LIMIT $3) x",
            [ecosystems.into(), names.into(), (cap as i64 + 1).into()],
        ))
        .await?;
    for row in rows {
        let key = (row.try_get("", "ecosystem")?, row.try_get("", "name")?);
        found.entry(key).or_default().push(Candidate {
            advisory_id: row.try_get("", "advisory_id")?,
            ranges: row.try_get("", "ranges")?,
            versions: row.try_get("", "versions")?,
        });
    }
    Ok(found)
}

async fn cap_candidates(
    txn: &DatabaseTransaction,
    candidates: &mut HashMap<(String, String), Vec<Candidate>>,
    cap: usize,
) -> Result<HashSet<(String, String)>, DbErr> {
    let mut cut = HashSet::new();
    for (key, found) in candidates.iter_mut() {
        if found.len() <= cap {
            continue;
        }
        found.truncate(cap);
        let row = txn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT count(*) AS n FROM advisory_affected f \
                 JOIN advisory a ON a.id = f.advisory_id \
                 WHERE f.ecosystem = $1 AND f.name = $2 AND a.withdrawn_at IS NULL",
                [key.0.clone().into(), key.1.clone().into()],
            ))
            .await?
            .ok_or_else(|| {
                DbErr::RecordNotFound("the advisory count returned no row".to_owned())
            })?;
        let total: i64 = row.try_get("", "n")?;
        tracing::warn!(
            ecosystem = %key.0,
            name = %key.1,
            cap,
            cut = total.saturating_sub(cap as i64),
            "a package has more advisories than the cap, so the rest are not matched"
        );
        cut.insert(key.clone());
    }
    Ok(cut)
}

async fn stored_matches(
    txn: &DatabaseTransaction,
    ids: &[i64],
) -> Result<BTreeMap<i64, BTreeMap<String, Option<String>>>, DbErr> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT package_id, advisory_id, fixed FROM sca_match WHERE package_id = ANY($1)",
            [ids.to_vec().into()],
        ))
        .await?;
    let mut stored: BTreeMap<i64, BTreeMap<String, Option<String>>> = BTreeMap::new();
    for row in rows {
        stored
            .entry(row.try_get("", "package_id")?)
            .or_default()
            .insert(row.try_get("", "advisory_id")?, row.try_get("", "fixed")?);
    }
    Ok(stored)
}

async fn delete_matches(txn: &DatabaseTransaction, rows: &[(i64, &str)]) -> Result<(), DbErr> {
    let packages: Vec<i64> = rows.iter().map(|row| row.0).collect();
    let advisories: Vec<String> = rows.iter().map(|row| row.1.to_owned()).collect();
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM sca_match m USING unnest($1::bigint[], $2::text[]) AS gone(package_id, advisory_id) \
         WHERE m.package_id = gone.package_id AND m.advisory_id = gone.advisory_id",
        [packages.into(), advisories.into()],
    ))
    .await?;
    Ok(())
}

async fn upsert_matches(txn: &DatabaseTransaction, rows: &[(i64, &Found)]) -> Result<(), DbErr> {
    let packages: Vec<i64> = rows.iter().map(|row| row.0).collect();
    let advisories: Vec<String> = rows.iter().map(|row| row.1.advisory_id.clone()).collect();
    let fixed: Vec<String> = rows
        .iter()
        .map(|row| row.1.fixed.clone().unwrap_or_default())
        .collect();
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO sca_match (package_id, advisory_id, fixed) \
         SELECT package_id, advisory_id, NULLIF(fixed, '') \
         FROM unnest($1::bigint[], $2::text[], $3::text[]) AS m(package_id, advisory_id, fixed) \
         ON CONFLICT (package_id, advisory_id) DO UPDATE SET fixed = EXCLUDED.fixed",
        [packages.into(), advisories.into(), fixed.into()],
    ))
    .await?;
    Ok(())
}

async fn finish(txn: &DatabaseTransaction, ids: &[i64]) -> Result<Vec<Marked>, DbErr> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE sca_package SET matched_at = now(), rejudge_at = clock_timestamp() \
             WHERE id = ANY($1) RETURNING id, rejudge_at",
            [ids.to_vec().into()],
        ))
        .await?;
    let mut marked = Vec::with_capacity(rows.len());
    for row in rows {
        marked.push(Marked {
            package_id: row.try_get("", "id")?,
            token: row.try_get("", "rejudge_at")?,
        });
    }
    marked.sort_by_key(|marked| marked.package_id);
    Ok(marked)
}
