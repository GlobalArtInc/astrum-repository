use std::collections::HashSet;
use std::time::Duration;

use astrum_content::Component;
use astrum_content::inventory::{Inventory, LicenseFact, PackageRef};
use astrum_policy::{license, subject};
use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, Statement,
    TransactionTrait,
};

use super::{Claimed, EXTRACTOR, State};
use crate::PACKAGE_LOCK;

const MAX_ERROR_CHARS: usize = 1000;
const MAX_LICENSE_BYTES: usize = 1024;

pub async fn claim(
    db: &DatabaseConnection,
    batch: i64,
    lease: Duration,
) -> Result<Vec<Claimed>, DbErr> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE scan_queue SET attempts = attempts + 1, \
               next_attempt_at = now() + make_interval(secs => $1) \
             WHERE component_id IN (\
               SELECT component_id FROM scan_queue WHERE next_attempt_at <= now() \
               ORDER BY next_attempt_at, component_id LIMIT $2 FOR UPDATE SKIP LOCKED) \
             RETURNING component_id, attempts, queued_at",
            [lease.as_secs_f64().into(), batch.into()],
        ))
        .await?;
    let mut claimed = Vec::with_capacity(rows.len());
    for row in rows {
        claimed.push(Claimed {
            component_id: row.try_get("", "component_id")?,
            attempts: row.try_get("", "attempts")?,
            queued_at: row.try_get("", "queued_at")?,
        });
    }
    claimed.sort_by_key(|claimed| claimed.component_id);
    Ok(claimed)
}

pub async fn queue_component(db: &DatabaseConnection, component_id: i64) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO scan_queue (component_id, reason) \
         SELECT id, 'gate' FROM component WHERE id = $1 \
         ON CONFLICT (component_id) DO NOTHING",
        [component_id.into()],
    ))
    .await?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim {
    Taken(Claimed),
    NotDue,
    Held,
    Gone,
}

pub async fn claim_component(
    db: &DatabaseConnection,
    component_id: i64,
    lease: Duration,
) -> Result<Claim, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE scan_queue SET attempts = attempts + 1, \
               next_attempt_at = now() + make_interval(secs => $2) \
             WHERE component_id IN (\
               SELECT component_id FROM scan_queue \
               WHERE component_id = $1 AND next_attempt_at <= now() FOR UPDATE SKIP LOCKED) \
             RETURNING component_id, attempts, queued_at",
            [component_id.into(), lease.as_secs_f64().into()],
        ))
        .await?;
    if let Some(row) = row {
        return Ok(Claim::Taken(Claimed {
            component_id: row.try_get("", "component_id")?,
            attempts: row.try_get("", "attempts")?,
            queued_at: row.try_get("", "queued_at")?,
        }));
    }
    let waiting = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT next_attempt_at > now() AS waiting FROM scan_queue WHERE component_id = $1",
            [component_id.into()],
        ))
        .await?;
    match waiting {
        None => Ok(Claim::Gone),
        Some(row) => match row.try_get::<bool>("", "waiting")? {
            true => Ok(Claim::NotDue),
            false => Ok(Claim::Held),
        },
    }
}

pub async fn renew(
    db: &DatabaseConnection,
    items: &[Claimed],
    lease: Duration,
) -> Result<(), DbErr> {
    if items.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = items.iter().map(|item| item.component_id).collect();
    let attempts: Vec<i32> = items.iter().map(|item| item.attempts).collect();
    let queued: Vec<DateTime<Utc>> = items.iter().map(|item| item.queued_at).collect();
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE scan_queue q SET next_attempt_at = now() + make_interval(secs => $4) \
         FROM unnest($1::bigint[], $2::int[], $3::timestamptz[]) \
           AS held(component_id, attempts, queued_at) \
         WHERE q.component_id = held.component_id AND q.attempts = held.attempts \
           AND q.queued_at = held.queued_at",
        [
            ids.into(),
            attempts.into(),
            queued.into(),
            lease.as_secs_f64().into(),
        ],
    ))
    .await?;
    Ok(())
}

pub async fn retry_later(
    db: &DatabaseConnection,
    item: Claimed,
    delay: Duration,
) -> Result<(), DbErr> {
    let moved = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE scan_queue SET next_attempt_at = now() + make_interval(secs => $2) \
             WHERE component_id = $1 AND attempts = $3 AND queued_at = $4",
            [
                item.component_id.into(),
                delay.as_secs_f64().into(),
                item.attempts.into(),
                item.queued_at.into(),
            ],
        ))
        .await?;
    if moved.rows_affected() == 0 {
        requeue_changed(db, item).await?;
    }
    Ok(())
}

pub async fn give_up(db: &DatabaseConnection, item: Claimed, message: &str) -> Result<(), DbErr> {
    let message: String = message.chars().take(MAX_ERROR_CHARS).collect();
    let txn = db.begin().await?;
    if !begin_write(&txn, item.component_id).await? {
        return txn.commit().await;
    }
    if !owns(&txn, item).await? {
        return txn.commit().await;
    }
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO component_scan (component_id, repository, state, error) \
         SELECT id, repository, $2, $3 FROM component WHERE id = $1 \
         ON CONFLICT (component_id) DO UPDATE SET state = EXCLUDED.state, error = EXCLUDED.error, \
           inventory_id = NULL, digest = NULL, scanned_at = now()",
        [
            item.component_id.into(),
            State::Failed.as_str().into(),
            message.into(),
        ],
    ))
    .await?;
    mark_dirty(&txn, item.component_id).await?;
    dequeue(&txn, item).await?;
    txn.commit().await
}

pub async fn record_unsupported(
    db: &DatabaseConnection,
    item: Claimed,
    component: &Component,
) -> Result<bool, DbErr> {
    let txn = db.begin().await?;
    if !begin_write(&txn, component.id).await? {
        txn.commit().await?;
        return Ok(false);
    }
    if !owns(&txn, item).await? {
        txn.commit().await?;
        return Ok(false);
    }
    upsert_scan(&txn, component, None, State::Unsupported).await?;
    mark_dirty(&txn, component.id).await?;
    dequeue(&txn, item).await?;
    txn.commit().await?;
    Ok(true)
}

pub async fn record_inventory(
    db: &DatabaseConnection,
    item: Claimed,
    component: &Component,
    inventory: &Inventory,
    state: State,
) -> Result<bool, DbErr> {
    let packages = distinct(&inventory.packages);
    let txn = db.begin().await?;
    txn.execute_unprepared(&format!(
        "SELECT pg_advisory_xact_lock_shared({PACKAGE_LOCK})"
    ))
    .await?;
    if !begin_write(&txn, component.id).await? {
        txn.commit().await?;
        return Ok(false);
    }
    if !owns(&txn, item).await? {
        txn.commit().await?;
        return Ok(false);
    }
    let inventory_id = upsert_inventory(&txn, &inventory.key).await?;
    let package_ids = lock_packages(&txn, &packages).await?;
    replace_links(&txn, inventory_id, &package_ids).await?;
    replace_license(&txn, component.id, inventory.license.as_ref()).await?;
    upsert_scan(&txn, component, Some(inventory_id), state).await?;
    mark_dirty(&txn, component.id).await?;
    dequeue(&txn, item).await?;
    txn.commit().await?;
    Ok(true)
}

async fn begin_write(txn: &DatabaseTransaction, component_id: i64) -> Result<bool, DbErr> {
    let repository = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS found FROM repository r JOIN component c ON c.repository = r.name \
             WHERE c.id = $1 FOR KEY SHARE OF r",
            [component_id.into()],
        ))
        .await?;
    if repository.is_none() {
        return Ok(false);
    }
    let component = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS found FROM component WHERE id = $1 FOR KEY SHARE",
            [component_id.into()],
        ))
        .await?;
    Ok(component.is_some())
}

async fn owns(txn: &DatabaseTransaction, item: Claimed) -> Result<bool, DbErr> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT attempts, queued_at FROM scan_queue WHERE component_id = $1 FOR UPDATE",
            [item.component_id.into()],
        ))
        .await?;
    let held: Option<(i32, DateTime<Utc>)> = match row {
        Some(row) => Some((row.try_get("", "attempts")?, row.try_get("", "queued_at")?)),
        None => None,
    };
    if held == Some((item.attempts, item.queued_at)) {
        return Ok(true);
    }
    requeue_changed(txn, item).await?;
    Ok(false)
}

fn distinct(packages: &[PackageRef]) -> Vec<&PackageRef> {
    let mut seen = HashSet::new();
    let mut kept: Vec<&PackageRef> = packages
        .iter()
        .filter(|package| seen.insert((&package.ecosystem, &package.name, &package.version)))
        .collect();
    kept.sort_by(|a, b| {
        (&a.ecosystem, &a.name, &a.version).cmp(&(&b.ecosystem, &b.name, &b.version))
    });
    kept
}

async fn upsert_inventory(txn: &DatabaseTransaction, key: &str) -> Result<i64, DbErr> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO sca_inventory (key, source, extractor) VALUES ($1, 'metadata', $2) \
             ON CONFLICT (key) DO UPDATE SET source = EXCLUDED.source, \
               extractor = EXCLUDED.extractor, extracted_at = now() \
             RETURNING id",
            [key.into(), EXTRACTOR.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the inventory upsert returned no row".to_owned()))?;
    row.try_get("", "id")
}

async fn lock_packages(
    txn: &DatabaseTransaction,
    packages: &[&PackageRef],
) -> Result<Vec<i64>, DbErr> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let column = |pick: fn(&PackageRef) -> &str| -> Vec<String> {
        packages
            .iter()
            .map(|package| pick(package).to_owned())
            .collect()
    };
    let values = || {
        [
            column(|package| &package.ecosystem).into(),
            column(|package| &package.name).into(),
            column(|package| &package.version).into(),
            column(|package| &package.purl).into(),
        ]
    };
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO sca_package (ecosystem, name, version, purl) \
         SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[]) \
           AS wanted(ecosystem, name, version, purl) ORDER BY 1, 2, 3 \
         ON CONFLICT (ecosystem, name, version) DO NOTHING",
        values(),
    ))
    .await?;
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT p.id FROM sca_package p \
             JOIN unnest($1::text[], $2::text[], $3::text[]) AS wanted(ecosystem, name, version) \
               ON p.ecosystem = wanted.ecosystem AND p.name = wanted.name \
               AND p.version = wanted.version \
             ORDER BY p.id FOR UPDATE OF p",
            [
                column(|package| &package.ecosystem).into(),
                column(|package| &package.name).into(),
                column(|package| &package.version).into(),
            ],
        ))
        .await?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        ids.push(row.try_get("", "id")?);
    }
    if ids.len() != packages.len() {
        return Err(DbErr::Custom(format!(
            "{} of {} packages are missing after their insert",
            packages.len() - ids.len(),
            packages.len()
        )));
    }
    Ok(ids)
}

async fn replace_links(
    txn: &DatabaseTransaction,
    inventory_id: i64,
    package_ids: &[i64],
) -> Result<(), DbErr> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM sca_inventory_package WHERE inventory_id = $1",
        [inventory_id.into()],
    ))
    .await?;
    if package_ids.is_empty() {
        return Ok(());
    }
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO sca_inventory_package (inventory_id, package_id) \
         SELECT $1, linked FROM unnest($2::bigint[]) AS linked ORDER BY linked \
         ON CONFLICT (inventory_id, package_id) DO NOTHING",
        [inventory_id.into(), package_ids.to_vec().into()],
    ))
    .await?;
    Ok(())
}

async fn replace_license(
    txn: &DatabaseTransaction,
    component_id: i64,
    fact: Option<&LicenseFact>,
) -> Result<(), DbErr> {
    let Some(fact) = fact else {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM component_license WHERE component_id = $1",
            [component_id.into()],
        ))
        .await?;
        return Ok(());
    };
    let declared = capped(&fact.declared);
    let normalised = license::normalise(declared);
    let expression = match normalised.state {
        license::State::Spdx | license::State::Mapped => Some(normalised.text),
        license::State::Unknown | license::State::None => None,
    };
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO component_license (component_id, declared, source, expression, ids, state) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (component_id) DO UPDATE SET declared = EXCLUDED.declared, \
           source = EXCLUDED.source, expression = EXCLUDED.expression, ids = EXCLUDED.ids, \
           state = EXCLUDED.state",
        [
            component_id.into(),
            declared.into(),
            fact.source.as_str().into(),
            expression.into(),
            normalised.ids.into(),
            normalised.state.as_str().into(),
        ],
    ))
    .await?;
    Ok(())
}

fn capped(text: &str) -> &str {
    let mut end = text.len().min(MAX_LICENSE_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

async fn upsert_scan(
    txn: &DatabaseTransaction,
    component: &Component,
    inventory_id: Option<i64>,
    state: State,
) -> Result<(), DbErr> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO component_scan (component_id, repository, inventory_id, state) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (component_id) DO UPDATE SET repository = EXCLUDED.repository, \
           inventory_id = EXCLUDED.inventory_id, digest = NULL, state = EXCLUDED.state, \
           error = NULL, scanned_at = now()",
        [
            component.id.into(),
            component.repository.clone().into(),
            inventory_id.into(),
            state.as_str().into(),
        ],
    ))
    .await?;
    Ok(())
}

async fn mark_dirty(txn: &DatabaseTransaction, component_id: i64) -> Result<(), DbErr> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO policy_dirty (repository, subject) \
         SELECT repository, $2 FROM component WHERE id = $1 \
         ON CONFLICT (repository, subject) DO NOTHING",
        [component_id.into(), subject::component(component_id).into()],
    ))
    .await?;
    Ok(())
}

async fn dequeue(txn: &DatabaseTransaction, item: Claimed) -> Result<(), DbErr> {
    let deleted = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM scan_queue WHERE component_id = $1 AND attempts = $2 AND queued_at = $3",
            [
                item.component_id.into(),
                item.attempts.into(),
                item.queued_at.into(),
            ],
        ))
        .await?;
    if deleted.rows_affected() == 0 {
        requeue_changed(txn, item).await?;
    }
    Ok(())
}

async fn requeue_changed(db: &impl ConnectionTrait, item: Claimed) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE scan_queue SET attempts = 0, next_attempt_at = now() \
         WHERE component_id = $1 AND attempts = $2 AND queued_at <> $3",
        [
            item.component_id.into(),
            item.attempts.into(),
            item.queued_at.into(),
        ],
    ))
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
