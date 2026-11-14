use astrum_blob::BlobId;
use astrum_content::holds;
use async_trait::async_trait;
use sea_orm::{DbBackend, FromQueryResult, Statement, TransactionTrait};
use uuid::Uuid;

use super::expired::BlobHoldExpiry;
use crate::kind::{Expiry, TaskError};

const BATCH: i64 = 500;

const TAKE_EXPIRED: &str = "DELETE FROM replication_staged_blob WHERE (repository, sha256) IN ( \
     SELECT repository, sha256 FROM replication_staged_blob WHERE expires_at <= now() \
     ORDER BY expires_at LIMIT $1 FOR UPDATE SKIP LOCKED) \
     RETURNING blob_store, blob_id";

#[derive(FromQueryResult)]
struct Staged {
    blob_store: String,
    blob_id: Uuid,
}

pub struct StagedBlobExpiry;

#[async_trait]
impl Expiry for StagedBlobExpiry {
    fn name(&self) -> &'static str {
        "Replicated blobs staged for 24 hours without a component"
    }

    async fn expire(&self, content: &astrum_content::Content) -> Result<u64, TaskError> {
        let mut removed = 0u64;
        loop {
            let txn = content.db().begin().await?;
            let staged = Staged::find_by_statement(Statement::from_sql_and_values(
                DbBackend::Postgres,
                TAKE_EXPIRED,
                [BATCH.into()],
            ))
            .all(&txn)
            .await?;
            let blobs: Vec<(String, BlobId)> = staged
                .iter()
                .map(|row| (row.blob_store.clone(), row.blob_id.into()))
                .collect();
            holds::lapse(&txn, &blobs).await?;
            txn.commit().await?;
            removed += staged.len() as u64;
            if (staged.len() as i64) < BATCH {
                break;
            }
        }
        if removed > 0 {
            BlobHoldExpiry.expire(content).await?;
        }
        Ok(removed)
    }
}
