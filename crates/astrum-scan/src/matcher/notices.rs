use std::collections::{BTreeSet, HashMap};

use astrum_policy::exception::{self, Exception};
use astrum_policy::policy::{self, Mode, Policy};
use astrum_policy::verdict::{self, Finding, Target};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement};
use serde_json::json;

use super::store::Change;

pub const NOTICE_ROWS: i64 = 2_000;
const ACTION: &str = "vulnerability.found";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queued {
    pub queued: u64,
    pub cut: bool,
}

struct Hit {
    component_id: i64,
    format: String,
    namespace: String,
    name: String,
    version: String,
    digest: Option<String>,
    policy: String,
    finding: Finding,
}

fn custom(error: impl std::fmt::Display) -> DbErr {
    DbErr::Custom(error.to_string())
}

pub async fn queue(
    txn: &DatabaseTransaction,
    added: &[Change],
    limit: i64,
) -> Result<Queued, DbErr> {
    if added.is_empty() {
        return Ok(Queued::default());
    }
    let mut hits = hits(txn, added, limit + 1).await?;
    let cut = hits.len() as i64 > limit;
    hits.truncate(limit as usize);
    if cut {
        tracing::warn!(
            limit,
            "new matches reach more components than a batch notifies about, so the rest are not announced"
        );
    }
    if hits.is_empty() {
        return Ok(Queued { queued: 0, cut });
    }
    let names: BTreeSet<&str> = hits.iter().map(|hit| hit.policy.as_str()).collect();
    let mut policies: HashMap<String, Policy> = HashMap::new();
    for name in names {
        match policy::find(txn, name).await {
            Ok(Some(found)) if found.vulnerabilities.mode != Mode::Off => {
                policies.insert(name.to_owned(), found);
            }
            Ok(_) => {}
            Err(astrum_policy::Error::Database(error)) => return Err(error),
            Err(error) => {
                tracing::warn!(policy = name, %error, "a content policy cannot be read, so its new matches are not announced");
            }
        }
    }
    let exceptions: Vec<Exception> = exception::list_readable(txn)
        .await
        .map_err(custom)?
        .exceptions;
    let now = database_time(txn).await?;
    let mut seen: BTreeSet<(i64, String)> = BTreeSet::new();
    let mut components: Vec<i64> = Vec::new();
    let mut details: Vec<String> = Vec::new();
    for hit in hits {
        let Some(policy) = policies.get(&hit.policy) else {
            continue;
        };
        let Some(threshold) = policy.vulnerabilities.notify else {
            continue;
        };
        if verdict::rank(hit.finding.severity) < verdict::rank(threshold) {
            continue;
        }
        let target = Target {
            format: hit.format.clone(),
            namespace: hit.namespace.clone(),
            name: hit.name.clone(),
            version: hit.version.clone(),
            digest: hit.digest.clone(),
        };
        if verdict::exception_for(&exceptions, &policy.name, &target, &hit.finding, now).is_some() {
            continue;
        }
        if !seen.insert((hit.component_id, hit.finding.advisory.clone())) {
            continue;
        }
        components.push(hit.component_id);
        details.push(
            json!({
                "policy": verdict::clean(&policy.name),
                "advisory": verdict::clean(&hit.finding.advisory),
                "severity": verdict::severity_name(hit.finding.severity),
            })
            .to_string(),
        );
    }
    if components.is_empty() {
        return Ok(Queued { queued: 0, cut });
    }
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT notification_capture_events($1, $2::bigint[], $3::text[]) AS n",
            [ACTION.into(), components.clone().into(), details.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the capture returned no row".to_owned()))?;
    let queued: i64 = row.try_get("", "n")?;
    Ok(Queued {
        queued: queued.max(0) as u64,
        cut,
    })
}

async fn hits(txn: &DatabaseTransaction, added: &[Change], limit: i64) -> Result<Vec<Hit>, DbErr> {
    let packages: Vec<i64> = added.iter().map(|change| change.package_id).collect();
    let advisories: Vec<String> = added
        .iter()
        .map(|change| change.advisory_id.clone())
        .collect();
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT cs.component_id, c.format, c.namespace, c.name, c.version, cs.digest, \
                    r.attributes #>> '{policy,name}' AS policy, \
                    a.id AS advisory_id, a.aliases, a.severity, a.malicious, p.purl \
             FROM unnest($1::bigint[], $2::text[]) AS added(package_id, advisory_id) \
             JOIN sca_package p ON p.id = added.package_id \
             JOIN advisory a ON a.id = added.advisory_id AND a.withdrawn_at IS NULL \
             JOIN sca_inventory_package ip ON ip.package_id = added.package_id \
             JOIN component_scan cs ON cs.inventory_id = ip.inventory_id \
             JOIN component c ON c.id = cs.component_id \
             JOIN repository r ON r.name = c.repository \
             WHERE r.attributes #>> '{policy,name}' IS NOT NULL \
             LIMIT $3",
            [packages.into(), advisories.into(), limit.into()],
        ))
        .await?;
    let mut hits = Vec::with_capacity(rows.len());
    for row in rows {
        let severity: String = row.try_get("", "severity")?;
        hits.push(Hit {
            component_id: row.try_get("", "component_id")?,
            format: row.try_get("", "format")?,
            namespace: row.try_get("", "namespace")?,
            name: row.try_get("", "name")?,
            version: row.try_get("", "version")?,
            digest: row.try_get("", "digest")?,
            policy: row.try_get("", "policy")?,
            finding: Finding {
                advisory: row.try_get("", "advisory_id")?,
                aliases: row.try_get("", "aliases")?,
                severity: verdict::parse_severity(&severity),
                malicious: row.try_get("", "malicious")?,
                package: row.try_get("", "purl")?,
                fixed: None,
            },
        });
    }
    Ok(hits)
}

async fn database_time(txn: &DatabaseTransaction) -> Result<chrono::DateTime<chrono::Utc>, DbErr> {
    txn.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT clock_timestamp() AS at",
    ))
    .await?
    .ok_or_else(|| DbErr::RecordNotFound("the clock query returned no row".to_owned()))?
    .try_get("", "at")
}
