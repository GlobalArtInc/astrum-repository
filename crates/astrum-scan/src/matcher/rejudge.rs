use astrum_policy::subject::COMPONENT_PREFIX;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};

use super::Config;
use super::store::Marked;

const SWEEP_LIMIT: i64 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rejudged {
    pub packages: u64,
    pub subjects: u64,
    pub failed: u64,
}

pub async fn fan_out(db: &DatabaseConnection, config: &Config, marked: &[Marked]) -> Rejudged {
    let mut done = Rejudged::default();
    let mut finished = Vec::with_capacity(marked.len());
    for item in marked {
        match fan_out_package(db, config.chunk, *item).await {
            Ok(subjects) => {
                finished.push(*item);
                done.packages += 1;
                done.subjects += subjects;
            }
            Err(error) => {
                done.failed += 1;
                tracing::warn!(
                    %error,
                    package = item.package_id,
                    "the components of a matched package were not all queued for judgement; the sweep repeats it"
                );
            }
        }
    }
    if let Err(error) = clear(db, &finished).await {
        done.failed += 1;
        tracing::warn!(%error, "the markers of queued packages were not cleared; the sweep repeats the queueing");
    }
    done
}

pub async fn sweep(db: &DatabaseConnection, config: &Config) -> Result<Rejudged, DbErr> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE sca_package SET rejudge_at = clock_timestamp() \
             WHERE id IN (\
               SELECT id FROM sca_package \
               WHERE rejudge_at IS NOT NULL AND rejudge_at < now() - make_interval(secs => $1) \
               ORDER BY id LIMIT $2 FOR UPDATE SKIP LOCKED) \
             RETURNING id, rejudge_at",
            [config.grace.as_secs_f64().into(), SWEEP_LIMIT.into()],
        ))
        .await?;
    let mut claimed = Vec::with_capacity(rows.len());
    for row in rows {
        claimed.push(Marked {
            package_id: row.try_get("", "id")?,
            token: row.try_get("", "rejudge_at")?,
        });
    }
    claimed.sort_by_key(|marked| marked.package_id);
    Ok(fan_out(db, config, &claimed).await)
}

async fn clear(db: &DatabaseConnection, finished: &[Marked]) -> Result<(), DbErr> {
    if finished.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = finished.iter().map(|item| item.package_id).collect();
    let tokens: Vec<DateTime<Utc>> = finished.iter().map(|item| item.token).collect();
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE sca_package SET rejudge_at = NULL WHERE id IN (\
           SELECT p.id FROM sca_package p \
           JOIN unnest($1::bigint[], $2::timestamptz[]) AS done(package_id, token) \
             ON p.id = done.package_id AND p.rejudge_at = done.token \
           ORDER BY p.id FOR UPDATE OF p SKIP LOCKED)",
        [ids.into(), tokens.into()],
    ))
    .await?;
    Ok(())
}

async fn fan_out_package(
    db: &DatabaseConnection,
    chunk: i64,
    marked: Marked,
) -> Result<u64, DbErr> {
    let sql = format!(
        "WITH page AS (\
           SELECT cs.component_id, cs.repository FROM sca_inventory_package ip \
           JOIN component_scan cs ON cs.inventory_id = ip.inventory_id \
           JOIN repository r ON r.name = cs.repository \
           WHERE ip.package_id = $1 AND cs.component_id > $2 \
           ORDER BY cs.component_id LIMIT $3), \
         queued AS (\
           INSERT INTO policy_dirty (repository, subject) \
           SELECT repository, '{COMPONENT_PREFIX}' || component_id FROM page \
           ON CONFLICT (repository, subject) DO NOTHING) \
         SELECT count(*) AS n, COALESCE(max(component_id), $2) AS last FROM page"
    );
    let mut after = 0i64;
    let mut subjects = 0u64;
    loop {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql.clone(),
                [marked.package_id.into(), after.into(), chunk.into()],
            ))
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("the fan-out page returned no row".to_owned()))?;
        let found: i64 = row.try_get("", "n")?;
        after = row.try_get("", "last")?;
        subjects += found as u64;
        if found < chunk {
            return Ok(subjects);
        }
    }
}
