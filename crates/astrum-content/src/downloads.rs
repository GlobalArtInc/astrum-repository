use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

use crate::ContentError;

pub const DOWNLOAD_DAYS_KEPT: i32 = 180;

const SECONDS_PER_DAY: u64 = 86_400;

#[derive(Default)]
pub(crate) struct Downloads {
    pending: Mutex<HashMap<(i64, i32), i64>>,
}

fn today() -> i32 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    i32::try_from(seconds / SECONDS_PER_DAY).unwrap_or(i32::MAX)
}

impl Downloads {
    pub(crate) fn note(&self, asset_id: i64) {
        *self
            .pending
            .lock()
            .expect("the download counter lock is never poisoned")
            .entry((asset_id, today()))
            .or_default() += 1;
    }

    pub(crate) async fn flush(&self, db: &DatabaseConnection) -> Result<(), ContentError> {
        let pending = std::mem::take(
            &mut *self
                .pending
                .lock()
                .expect("the download counter lock is never poisoned"),
        );
        if pending.is_empty() {
            return Ok(());
        }
        let mut ids = Vec::with_capacity(pending.len());
        let mut days = Vec::with_capacity(pending.len());
        let mut counts = Vec::with_capacity(pending.len());
        for ((id, day), count) in pending {
            ids.push(id);
            days.push(day);
            counts.push(count);
        }
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "WITH counted AS ( \
               SELECT * FROM unnest($1::bigint[], $2::integer[], $3::bigint[]) AS counted(id, day, count) \
             ), counted_assets AS ( \
               UPDATE asset SET download_count = asset.download_count + total.count, \
                 last_downloaded_at = now() \
               FROM (SELECT id, sum(count)::bigint AS count FROM counted GROUP BY id) AS total \
               WHERE asset.id = total.id \
               RETURNING asset.id \
             ) \
             INSERT INTO asset_download_day (asset_id, day, downloads) \
             SELECT counted.id, DATE '1970-01-01' + counted.day, counted.count \
             FROM counted JOIN counted_assets ON counted_assets.id = counted.id \
             ON CONFLICT (asset_id, day) DO UPDATE \
               SET downloads = asset_download_day.downloads + EXCLUDED.downloads",
            [ids.into(), days.into(), counts.into()],
        ))
        .await?;
        Ok(())
    }

    pub(crate) async fn prune(db: &DatabaseConnection) -> Result<u64, ContentError> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM asset_download_day WHERE day < DATE '1970-01-01' + $1::integer",
                [(today() - DOWNLOAD_DAYS_KEPT + 1).into()],
            ))
            .await?;
        Ok(result.rows_affected())
    }
}
