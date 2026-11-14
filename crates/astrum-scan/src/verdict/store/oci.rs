use std::collections::HashMap;

use astrum_policy::verdict::{self, Finding, Peer, Scan, Target};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement};

pub(super) type Key = (String, String);

const PEERS: i64 = 50;
const INVENTORY: &str = "'oci:' || w.repository || ':' || w.digest";

pub(super) struct Facts {
    pub scan: Scan,
    pub findings: Vec<Finding>,
    pub peers: Vec<Peer>,
}

pub(super) fn target_of(digest: &str, facts: Option<&Facts>) -> Target {
    Target {
        format: "docker".to_owned(),
        digest: Some(digest.to_owned()),
        peers: facts.map(|facts| facts.peers.clone()).unwrap_or_default(),
        ..Target::default()
    }
}

fn columns(images: &[Key]) -> (Vec<String>, Vec<String>) {
    (
        images.iter().map(|image| image.0.clone()).collect(),
        images.iter().map(|image| image.1.clone()).collect(),
    )
}

pub(super) async fn load(
    txn: &DatabaseTransaction,
    images: &[Key],
) -> Result<HashMap<Key, Facts>, DbErr> {
    let mut facts = HashMap::new();
    if images.is_empty() {
        return Ok(facts);
    }
    let (repositories, digests) = columns(images);
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "SELECT w.repository, w.digest, i.complete, \
                   EXISTS (SELECT 1 FROM sca_inventory_package ip \
                           JOIN sca_package p ON p.id = ip.package_id \
                           WHERE ip.inventory_id = i.id AND p.matched_at IS NULL) AS pending \
                 FROM unnest($1::text[], $2::text[]) AS w(repository, digest) \
                 JOIN sca_inventory i ON i.key = {INVENTORY}"
            ),
            [repositories.clone().into(), digests.clone().into()],
        ))
        .await?;
    for row in rows {
        let complete: bool = row.try_get("", "complete")?;
        let pending: bool = row.try_get("", "pending")?;
        let scan = match (complete, pending) {
            (false, _) => Scan::Incomplete,
            (true, true) => Scan::Matching,
            (true, false) => Scan::Scanned,
        };
        facts.insert(
            (row.try_get("", "repository")?, row.try_get("", "digest")?),
            Facts {
                scan,
                findings: Vec::new(),
                peers: Vec::new(),
            },
        );
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "SELECT w.repository, w.digest, a.id AS advisory_id, a.aliases, a.severity, \
                   a.malicious, m.fixed, p.ecosystem, p.name, p.version \
                 FROM unnest($1::text[], $2::text[]) AS w(repository, digest) \
                 JOIN sca_inventory i ON i.key = {INVENTORY} \
                 JOIN sca_inventory_package ip ON ip.inventory_id = i.id \
                 JOIN sca_match m ON m.package_id = ip.package_id \
                 JOIN advisory a ON a.id = m.advisory_id AND a.withdrawn_at IS NULL \
                 JOIN sca_package p ON p.id = ip.package_id \
                 ORDER BY w.repository, w.digest, a.id, p.id"
            ),
            [repositories.clone().into(), digests.clone().into()],
        ))
        .await?;
    for row in rows {
        let key = (row.try_get("", "repository")?, row.try_get("", "digest")?);
        let severity: String = row.try_get("", "severity")?;
        let (ecosystem, name, version): (String, String, String) = (
            row.try_get("", "ecosystem")?,
            row.try_get("", "name")?,
            row.try_get("", "version")?,
        );
        if let Some(known) = facts.get_mut(&key) {
            known.findings.push(Finding {
                advisory: row.try_get("", "advisory_id")?,
                aliases: row.try_get("", "aliases")?,
                severity: verdict::parse_severity(&severity),
                malicious: row.try_get("", "malicious")?,
                package: format!("{ecosystem}/{name}@{version}"),
                fixed: row.try_get("", "fixed")?,
            });
        }
    }
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT repository, digest, namespace, name, version FROM (\
               SELECT w.repository, w.digest, c.namespace, c.name, c.version, \
                 row_number() OVER (PARTITION BY w.repository, w.digest \
                                    ORDER BY c.name, c.version, c.id) AS place \
               FROM unnest($1::text[], $2::text[]) AS w(repository, digest) \
               JOIN component_scan cs ON cs.repository = w.repository AND cs.digest = w.digest \
               JOIN component c ON c.id = cs.component_id) named \
             WHERE place <= $3 ORDER BY repository, digest, place",
            [repositories.into(), digests.into(), PEERS.into()],
        ))
        .await?;
    for row in rows {
        let key = (row.try_get("", "repository")?, row.try_get("", "digest")?);
        let peer = Peer {
            namespace: row.try_get("", "namespace")?,
            name: row.try_get("", "name")?,
            version: row.try_get("", "version")?,
        };
        facts
            .entry(key)
            .or_insert_with(|| Facts {
                scan: Scan::Missing,
                findings: Vec::new(),
                peers: Vec::new(),
            })
            .peers
            .push(peer);
    }
    Ok(facts)
}
