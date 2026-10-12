use std::collections::HashMap;
use std::sync::Mutex;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

use crate::ContentError;

#[derive(Default)]
pub(crate) struct Downloads {
    pending: Mutex<HashMap<i64, i64>>,
}

impl Downloads {
    pub(crate) fn note(&self, asset_id: i64) {
        *self
            .pending
            .lock()
            .expect("the download counter lock is never poisoned")
            .entry(asset_id)
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
        let (ids, counts): (Vec<i64>, Vec<i64>) = pending.into_iter().unzip();
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE asset SET download_count = asset.download_count + counted.count, \
             last_downloaded_at = now() \
             FROM unnest($1::bigint[], $2::bigint[]) AS counted(id, count) \
             WHERE asset.id = counted.id",
            [ids.into(), counts.into()],
        ))
        .await?;
        Ok(())
    }
}
