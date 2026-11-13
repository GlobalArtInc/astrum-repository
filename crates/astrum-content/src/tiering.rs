use std::time::Duration;

use astrum_blob::{BlobId, Digests, Limiter, Written};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, Statement, TransactionTrait};

use crate::holds;
use crate::store::{self, Asset};
use crate::{Content, ContentError};

const LOCK_STORE: &str = "SELECT 1 AS locked FROM blob_store WHERE name = $1 FOR SHARE";

const SWITCH: &str = "UPDATE asset SET blob_store = $1, blob_id = $2 \
     WHERE id = $3 AND blob_store = $4 AND blob_id = $5 AND sha256 = $6 RETURNING id";

const CURRENT: &str = "SELECT 1 AS current FROM asset \
     WHERE id = $1 AND blob_store = $2 AND blob_id = $3 AND sha256 = $4";

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub id: i64,
    pub blob_store: String,
    pub blob_id: BlobId,
    pub sha256: String,
    pub size: u64,
    pub digests: Digests,
}

impl From<&Asset> for Snapshot {
    fn from(asset: &Asset) -> Self {
        Self {
            id: asset.id,
            blob_store: asset.blob_store.clone(),
            blob_id: asset.blob_id.into(),
            sha256: asset.sha256.clone(),
            size: asset.size.max(0) as u64,
            digests: Digests {
                sha1: asset.sha1.clone(),
                sha256: asset.sha256.clone(),
                sha512: asset.sha512.clone(),
                md5: asset.md5.clone(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub drain: Duration,
    pub hold_ttl: Duration,
    pub limiter: Option<Limiter>,
    pub duplicate_of: Option<BlobId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Moved { new_id: BlobId, bytes: u64 },
    Changed,
    Corrupt,
    Failed(String),
}

enum Switched {
    Done,
    Changed,
}

pub async fn move_asset(
    content: &Content,
    snapshot: Snapshot,
    target: &str,
    options: &Options,
) -> Outcome {
    if let Err(message) = consistent(&snapshot) {
        return Outcome::Failed(message);
    }
    let written = match write(content, &snapshot, target, options).await {
        Ok(written) => written,
        Err(outcome) => return outcome,
    };
    if let Err(error) = holds::hold_for(
        content.db(),
        &written.store,
        written.id,
        options.hold_ttl,
        holds::TIERING,
    )
    .await
    {
        discard(content, &written).await;
        return Outcome::Failed(error.to_string());
    }
    if let Some(outcome) = mismatch(&snapshot, &written, options) {
        discard(content, &written).await;
        return outcome;
    }
    let txn = match content.db().begin().await {
        Ok(txn) => txn,
        Err(error) => {
            discard(content, &written).await;
            return Outcome::Failed(error.to_string());
        }
    };
    match switch(&txn, &snapshot, &written, options).await {
        Ok(Switched::Done) => match txn.commit().await {
            Ok(()) => Outcome::Moved {
                new_id: written.id,
                bytes: written.size,
            },
            Err(error) => {
                tracing::error!(%error, asset = snapshot.id, store = written.store, blob = %written.id, "the commit of a move failed and its outcome is unknown; both blobs stay for the sweep to settle");
                Outcome::Failed(format!(
                    "the commit failed and its outcome is unknown; the blobs stay for the sweep: {error}"
                ))
            }
        },
        Ok(Switched::Changed) => {
            rolled_back(txn).await;
            discard(content, &written).await;
            Outcome::Changed
        }
        Err(error) => {
            rolled_back(txn).await;
            discard(content, &written).await;
            Outcome::Failed(error.to_string())
        }
    }
}

fn consistent(snapshot: &Snapshot) -> Result<(), String> {
    if snapshot.sha256.is_empty() {
        return Err(format!("the asset {} has no SHA-256", snapshot.id));
    }
    if !snapshot
        .sha256
        .eq_ignore_ascii_case(&snapshot.digests.sha256)
    {
        return Err(format!(
            "the snapshot of the asset {} names a SHA-256 that differs from its digests",
            snapshot.id
        ));
    }
    Ok(())
}

async fn write(
    content: &Content,
    snapshot: &Snapshot,
    target: &str,
    options: &Options,
) -> Result<Written, Outcome> {
    let to = content.blob_stores().get(target).await.map_err(failed)?;
    if let Some(earlier) = options.duplicate_of {
        return to
            .duplicate(earlier, &snapshot.digests)
            .await
            .map_err(failed);
    }
    let from = content
        .blob_stores()
        .get(&snapshot.blob_store)
        .await
        .map_err(failed)?;
    match astrum_blob::copy(
        from.as_ref(),
        &snapshot.blob_id,
        &snapshot.digests,
        to.as_ref(),
        options.limiter.as_ref(),
    )
    .await
    {
        Ok(written) => Ok(written),
        Err(error @ astrum_blob::Error::Digest { .. }) => {
            tracing::warn!(%error, asset = snapshot.id, store = snapshot.blob_store, blob = %snapshot.blob_id, "the blob of an asset does not match its digests and was not moved");
            Err(Outcome::Corrupt)
        }
        Err(error) => Err(unsettled(content, snapshot, error).await),
    }
}

fn failed(error: astrum_blob::Error) -> Outcome {
    Outcome::Failed(error.to_string())
}

async fn unsettled(content: &Content, snapshot: &Snapshot, error: astrum_blob::Error) -> Outcome {
    match named(content, snapshot).await {
        Ok(true) => Outcome::Failed(error.to_string()),
        Ok(false) => Outcome::Changed,
        Err(check) => Outcome::Failed(format!(
            "{error}; the asset could not be checked afterwards: {check}"
        )),
    }
}

async fn named(content: &Content, snapshot: &Snapshot) -> Result<bool, ContentError> {
    Ok(content
        .db()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            CURRENT,
            [
                snapshot.id.into(),
                snapshot.blob_store.as_str().into(),
                snapshot.blob_id.uuid().into(),
                snapshot.sha256.as_str().into(),
            ],
        ))
        .await?
        .is_some())
}

fn mismatch(snapshot: &Snapshot, written: &Written, options: &Options) -> Option<Outcome> {
    if written.size == snapshot.size
        && written
            .digests
            .sha256
            .eq_ignore_ascii_case(&snapshot.sha256)
    {
        return None;
    }
    tracing::warn!(
        asset = snapshot.id,
        store = written.store,
        blob = %written.id,
        expected_size = snapshot.size,
        written_size = written.size,
        "a blob written for a move does not match the row of its asset"
    );
    Some(match options.duplicate_of {
        None => Outcome::Corrupt,
        Some(earlier) => Outcome::Failed(format!(
            "the blob {earlier} of the target store does not match the asset {}",
            snapshot.id
        )),
    })
}

async fn switch(
    txn: &DatabaseTransaction,
    snapshot: &Snapshot,
    written: &Written,
    options: &Options,
) -> Result<Switched, ContentError> {
    txn.execute_unprepared("SET LOCAL astrum.action = 'tiered'")
        .await?;
    if holds::consume(txn, &[(written.store.clone(), written.id)]).await? != 1 {
        return Err(ContentError::Conflict(
            "the new blob lost its hold; the move was abandoned".to_owned(),
        ));
    }
    holds::hold_at_least(
        txn,
        &snapshot.blob_store,
        snapshot.blob_id,
        options.drain,
        holds::RETIRED,
    )
    .await?;
    let target_store = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            LOCK_STORE,
            [written.store.as_str().into()],
        ))
        .await?;
    if target_store.is_none() {
        return Ok(Switched::Changed);
    }
    let moved = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            SWITCH,
            [
                written.store.as_str().into(),
                written.id.uuid().into(),
                snapshot.id.into(),
                snapshot.blob_store.as_str().into(),
                snapshot.blob_id.uuid().into(),
                snapshot.sha256.as_str().into(),
            ],
        ))
        .await?;
    Ok(match moved {
        Some(_) => Switched::Done,
        None => Switched::Changed,
    })
}

async fn discard(content: &Content, written: &Written) {
    store::abandon(
        content,
        vec![(written.store.clone(), written.id.uuid())],
        true,
    )
    .await;
}

async fn rolled_back(txn: DatabaseTransaction) {
    if let Err(error) = txn.rollback().await {
        tracing::warn!(%error, "a move transaction could not be rolled back explicitly");
    }
}
