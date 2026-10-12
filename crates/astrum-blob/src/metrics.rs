use astrum_db::entity::blob_store_metrics;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, Statement};

use crate::{Error, Totals};

const ADD: &str = "INSERT INTO blob_store_metrics (store, blob_count, total_bytes, updated_at) \
     VALUES ($1, $2, $3, now()) \
     ON CONFLICT (store) DO UPDATE SET \
     blob_count = GREATEST(blob_store_metrics.blob_count + EXCLUDED.blob_count, 0), \
     total_bytes = GREATEST(blob_store_metrics.total_bytes + EXCLUDED.total_bytes, 0), \
     updated_at = now()";

#[derive(Clone, Debug)]
pub struct Metrics {
    db: DatabaseConnection,
    store: String,
}

impl Metrics {
    pub fn new(db: DatabaseConnection, store: impl Into<String>) -> Self {
        Self {
            db,
            store: store.into(),
        }
    }

    pub async fn totals(&self) -> Result<Totals, Error> {
        Ok(read(&self.db, &self.store).await?)
    }

    pub(crate) async fn added(&self, size: u64) {
        self.change(1, size as i64).await;
    }

    pub(crate) async fn removed(&self, size: u64) {
        self.change(-1, -(size as i64)).await;
    }

    async fn change(&self, count: i64, bytes: i64) {
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            ADD,
            [self.store.clone().into(), count.into(), bytes.into()],
        );
        if let Err(error) = self.db.execute(statement).await {
            tracing::warn!(store = %self.store, %error, "the blob store totals were not updated");
        }
    }
}

pub(crate) async fn read(db: &impl ConnectionTrait, store: &str) -> Result<Totals, sea_orm::DbErr> {
    let row = blob_store_metrics::Entity::find_by_id(store.to_owned())
        .one(db)
        .await?;
    Ok(row
        .map(|row| Totals {
            blob_count: row.blob_count.max(0) as u64,
            total_bytes: row.total_bytes.max(0) as u64,
        })
        .unwrap_or_default())
}
