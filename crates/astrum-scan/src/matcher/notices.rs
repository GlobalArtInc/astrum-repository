use std::collections::{BTreeSet, HashMap};

use astrum_policy::exception::{self, Exception};
use astrum_policy::policy::Policy;
use astrum_policy::verdict::{self, Finding, Target};
use sea_orm::{
    ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement, TransactionTrait,
};
use serde_json::json;

use super::store::Change;

pub const NOTICE_ROWS: i64 = 2_000;
pub const SCAN_ROWS: i64 = NOTICE_ROWS * 10;
const PAGE: i64 = 1_000;
const ACTION: &str = "vulnerability.found";

const RANK: &str = "(CASE {} WHEN 'critical' THEN 4 WHEN 'high' THEN 3 \
    WHEN 'medium' THEN 2 WHEN 'low' THEN 1 ELSE 0 END)";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queued {
    pub queued: u64,
    pub cut: bool,
}

#[derive(Clone, Copy)]
pub enum Source<'a> {
    Added(&'a [Change]),
    Component(i64),
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

pub async fn announce(
    txn: &DatabaseTransaction,
    source: Source<'_>,
    limit: i64,
) -> Result<Queued, DbErr> {
    let nested = txn.begin().await?;
    match queue(&nested, source, limit).await {
        Ok(queued) => {
            nested.commit().await?;
            Ok(queued)
        }
        Err(error) => {
            nested.rollback().await?;
            tracing::warn!(%error, "new findings were not announced to the notification channels; the scan and the matches are kept");
            Ok(Queued::default())
        }
    }
}

pub async fn queue(
    txn: &DatabaseTransaction,
    source: Source<'_>,
    limit: i64,
) -> Result<Queued, DbErr> {
    queue_within(txn, source, limit, SCAN_ROWS).await
}

pub async fn queue_within(
    txn: &DatabaseTransaction,
    source: Source<'_>,
    limit: i64,
    examine: i64,
) -> Result<Queued, DbErr> {
    if matches!(source, Source::Added(added) if added.is_empty()) {
        return Ok(Queued::default());
    }
    let mut policies: HashMap<String, Policy> = HashMap::new();
    let mut exceptions: Option<Vec<Exception>> = None;
    let mut now: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut components: Vec<i64> = Vec::new();
    let mut details: Vec<String> = Vec::new();
    let mut after: Option<(i64, String)> = None;
    let mut cut = false;
    let mut examined = 0i64;
    'pages: loop {
        let hits = hits(txn, source, after.as_ref(), PAGE).await?;
        let full = hits.len() as i64 == PAGE;
        if let Some(hit) = hits.last() {
            after = Some((hit.component_id, hit.finding.advisory.clone()));
        }
        if hits.is_empty() {
            break;
        }
        let missing: BTreeSet<&str> = hits
            .iter()
            .map(|hit| hit.policy.as_str())
            .filter(|name| !policies.contains_key(*name))
            .collect();
        for name in missing {
            match astrum_policy::policy::find(txn, name).await {
                Ok(Some(found)) => {
                    policies.insert(name.to_owned(), found);
                }
                Ok(None) => {}
                Err(astrum_policy::Error::Database(error)) => return Err(error),
                Err(error) => {
                    tracing::warn!(policy = name, %error, "a content policy cannot be read, so its new matches are not announced");
                }
            }
        }
        if exceptions.is_none() {
            exceptions = Some(
                exception::list_readable(txn)
                    .await
                    .map_err(custom)?
                    .exceptions,
            );
            now = Some(database_time(txn).await?);
        }
        let (exceptions, now) = (
            exceptions.as_deref().unwrap_or_default(),
            now.unwrap_or_default(),
        );
        for hit in hits {
            if examined == examine {
                cut = true;
                break 'pages;
            }
            examined += 1;
            let Some(policy) = policies.get(&hit.policy) else {
                continue;
            };
            let target = Target {
                format: hit.format.clone(),
                namespace: hit.namespace.clone(),
                name: hit.name.clone(),
                version: hit.version.clone(),
                digest: hit.digest.clone(),
                peers: Vec::new(),
            };
            if verdict::exception_for(exceptions, &policy.name, &target, &hit.finding, now)
                .is_some()
            {
                continue;
            }
            if components.len() as i64 == limit {
                cut = true;
                break 'pages;
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
        if !full {
            break;
        }
    }
    if cut {
        tracing::warn!(
            limit,
            examine,
            "new matches reach more components than a batch announces or looks at, so the rest are not announced"
        );
    }
    if components.is_empty() {
        return Ok(Queued { queued: 0, cut });
    }
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT notification_capture_events($1, $2::bigint[], $3::text[]) AS n",
            [ACTION.into(), components.into(), details.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the capture returned no row".to_owned()))?;
    let queued: i64 = row.try_get("", "n")?;
    Ok(Queued {
        queued: queued.max(0) as u64,
        cut,
    })
}

async fn hits(
    txn: &DatabaseTransaction,
    source: Source<'_>,
    after: Option<&(i64, String)>,
    page: i64,
) -> Result<Vec<Hit>, DbErr> {
    let advisory_rank = RANK.replace("{}", "a.severity");
    let notify_rank = RANK.replace("{}", "cp.vulnerabilities->>'notify'");
    let (pairs, mut values): (&str, Vec<sea_orm::Value>) = match source {
        Source::Added(added) => (
            "unnest($1::bigint[], $2::text[]) AS added(package_id, advisory_id) \
             JOIN sca_inventory_package ip ON ip.package_id = added.package_id \
             JOIN component_scan cs ON cs.inventory_id = ip.inventory_id",
            vec![
                added
                    .iter()
                    .map(|change| change.package_id)
                    .collect::<Vec<i64>>()
                    .into(),
                added
                    .iter()
                    .map(|change| change.advisory_id.clone())
                    .collect::<Vec<String>>()
                    .into(),
            ],
        ),
        Source::Component(component_id) => (
            "sca_match added \
             JOIN sca_inventory_package ip ON ip.package_id = added.package_id \
             JOIN component_scan cs ON cs.inventory_id = ip.inventory_id AND cs.component_id = $1",
            vec![component_id.into()],
        ),
    };
    let first = values.len() as i32 + 1;
    values.push(after.map_or(0, |(component, _)| *component).into());
    values.push(
        after
            .map_or_else(String::new, |(_, advisory)| advisory.clone())
            .into(),
    );
    values.push(page.into());
    let sql = format!(
        "SELECT DISTINCT ON (cs.component_id, a.id) cs.component_id, c.format, c.namespace, c.name, \
                c.version, cs.digest, cp.name AS policy, a.id AS advisory_id, a.aliases, a.severity, \
                a.malicious, p.purl \
         FROM {pairs} \
         JOIN sca_package p ON p.id = added.package_id \
         JOIN advisory a ON a.id = added.advisory_id AND a.withdrawn_at IS NULL \
         JOIN component c ON c.id = cs.component_id \
         JOIN repository r ON r.name = c.repository \
         JOIN content_policy cp ON cp.name = r.attributes #>> '{{policy,name}}' \
         WHERE COALESCE(cp.vulnerabilities->>'mode', 'off') <> 'off' \
           AND cp.vulnerabilities->>'notify' IS NOT NULL \
           AND {advisory_rank} >= {notify_rank} \
           AND (cs.component_id, a.id) > (${first}::bigint, ${second}::text) \
         ORDER BY cs.component_id, a.id, p.id \
         LIMIT ${third}",
        second = first + 1,
        third = first + 2,
    );
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            values,
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
