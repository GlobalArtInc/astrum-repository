use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;

use astrum_blob::{BlobId, ContentStream, Digests, Written};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use sea_orm::{
    ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement, TransactionTrait, Value,
};
use uuid::Uuid;

use crate::replicated::{Reason, Refusal};
use crate::transfer::{BlobSource, BundleAsset, Sink};
use crate::{Content, ContentError, Ctx, holds};

pub const STAGED_HOURS: i32 = 24;
pub const MAX_STAGED: i64 = 10_000;
pub const MAX_BLOB_BYTES: u64 = 64 << 30;
const HOLD_MARGIN_HOURS: u64 = 24;

const PRESENT: &str = "SELECT sha256 FROM asset WHERE repository = $1 AND sha256 = ANY($2::text[]) \
     UNION \
     SELECT sha256 FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = ANY($2::text[]) AND expires_at > now()";

const COUNT_STAGED: &str = "SELECT count(*) AS n FROM replication_staged_blob WHERE repository = $1 AND expires_at > now()";

const DROP_EXPIRED: &str = "DELETE FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = $2 AND expires_at <= now() \
     RETURNING blob_store, blob_id";

const ASSET_EXISTS: &str =
    "SELECT 1 AS present FROM asset WHERE repository = $1 AND sha256 = $2 LIMIT 1";

const ASSET_BLOB: &str = "SELECT blob_store, blob_id, sha1, sha256, sha512, md5 FROM asset \
     WHERE repository = $1 AND sha256 = $2 ORDER BY id LIMIT 1";

const STAGED_BLOB: &str = "SELECT blob_store, blob_id, sha1, sha256, sha512, md5 \
     FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = $2 AND expires_at > now()";

const RELEASE_STAGED: &str = "DELETE FROM replication_staged_blob s \
     WHERE s.repository = $1 AND s.sha256 = ANY($2::text[]) \
     AND EXISTS (SELECT 1 FROM asset a WHERE a.repository = s.repository AND a.sha256 = s.sha256) \
     RETURNING s.blob_store, s.blob_id";

const STAGE: &str = "INSERT INTO replication_staged_blob \
     (repository, sha256, blob_store, blob_id, size, sha1, sha512, md5, uploaded_by, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now() + make_interval(hours => $10)) \
     ON CONFLICT (repository, sha256) DO NOTHING";

fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn internal(what: &str) -> ContentError {
    ContentError::internal(anyhow::anyhow!("{what}"))
}

#[derive(FromQueryResult)]
struct Digest {
    sha256: String,
}

#[derive(FromQueryResult)]
struct Stale {
    blob_store: String,
    blob_id: Uuid,
}

#[derive(FromQueryResult)]
struct Stored {
    blob_store: String,
    blob_id: Uuid,
    sha1: String,
    sha256: String,
    sha512: String,
    md5: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Staged {
    Created,
    Present,
}

pub async fn absent(
    db: &impl ConnectionTrait,
    repository: &str,
    digests: Vec<String>,
) -> Result<Vec<String>, DbErr> {
    let present: std::collections::HashSet<String> = Digest::find_by_statement(statement(
        PRESENT,
        vec![repository.into(), digests.clone().into()],
    ))
    .all(db)
    .await?
    .into_iter()
    .map(|row| row.sha256)
    .collect();
    let mut seen = std::collections::HashSet::new();
    Ok(digests
        .into_iter()
        .filter(|digest| !present.contains(digest) && seen.insert(digest.clone()))
        .collect())
}

pub async fn staged_room(db: &impl ConnectionTrait, repository: &str) -> Result<(), Refusal> {
    let row = db
        .query_one(statement(COUNT_STAGED, vec![repository.into()]))
        .await?
        .ok_or_else(|| internal("the count of staged blobs was not returned"))?;
    let staged: i64 = row
        .try_get("", "n")
        .map_err(|error| ContentError::internal(anyhow::Error::from(error)))?;
    match staged >= MAX_STAGED {
        true => Err(Refusal::Status {
            reason: Reason::TooMany,
            code: "too_many_staged",
            message: format!(
                "{repository} already holds {MAX_STAGED} staged blobs; commit the components that need them or wait for them to expire"
            ),
        }),
        false => Ok(()),
    }
}

fn hold_ttl() -> Duration {
    Duration::from_secs((STAGED_HOURS as u64 + HOLD_MARGIN_HOURS) * 3600)
}

pub async fn stage(
    content: &Content,
    repository: &str,
    written: &Written,
    uploaded_by: &str,
) -> Result<Staged, Refusal> {
    let size = i64::try_from(written.size).map_err(|_| internal("a blob is larger than i64"))?;
    let txn = content.db().begin().await?;
    let stale = Stale::find_by_statement(statement(
        DROP_EXPIRED,
        vec![repository.into(), written.digests.sha256.clone().into()],
    ))
    .all(&txn)
    .await?;
    let stale: Vec<(String, BlobId)> = stale
        .into_iter()
        .map(|row| (row.blob_store, row.blob_id.into()))
        .collect();
    holds::lapse(&txn, &stale).await?;
    let named = txn
        .query_one(statement(
            ASSET_EXISTS,
            vec![repository.into(), written.digests.sha256.clone().into()],
        ))
        .await?
        .is_some();
    let inserted = match named {
        true => 0,
        false => txn
            .execute(statement(
                STAGE,
                vec![
                    repository.into(),
                    written.digests.sha256.clone().into(),
                    written.store.clone().into(),
                    written.id.uuid().into(),
                    size.into(),
                    written.digests.sha1.clone().into(),
                    written.digests.sha512.clone().into(),
                    written.digests.md5.clone().into(),
                    uploaded_by.into(),
                    STAGED_HOURS.into(),
                ],
            ))
            .await?
            .rows_affected(),
    };
    if inserted > 0 {
        holds::hold(&txn, &written.store, written.id, hold_ttl())
            .await
            .map_err(ContentError::from)?;
    }
    txn.commit().await?;
    Ok(match inserted {
        0 => Staged::Present,
        _ => Staged::Created,
    })
}

pub async fn discard(content: &Content, written: &Written) {
    let deleted = match content.blob_stores().get(&written.store).await {
        Ok(store) => store.delete(written.id).await.map(|_| ()),
        Err(error) => Err(error),
    };
    if let Err(error) = deleted {
        tracing::warn!(%error, store = written.store, blob = %written.id, "a replicated blob that was not kept stays in its store until compaction collects it");
    }
}

pub async fn release(ctx: &Ctx, digests: &[String]) {
    let released = async {
        let txn = ctx.db().begin().await?;
        let freed: Vec<(String, BlobId)> = Stale::find_by_statement(statement(
            RELEASE_STAGED,
            vec![ctx.name().into(), digests.to_vec().into()],
        ))
        .all(&txn)
        .await?
        .into_iter()
        .map(|row| (row.blob_store, row.blob_id.into()))
        .collect();
        holds::lapse(&txn, &freed).await?;
        txn.commit().await?;
        Ok::<(), ContentError>(())
    }
    .await;
    if let Err(error) = released {
        tracing::warn!(%error, repository = ctx.name(), "staged blobs of a committed component stay until they expire");
    }
}

pub struct Counted {
    inner: ContentStream,
    seen: u64,
    limit: u64,
    exceeded: Arc<AtomicBool>,
}

impl Counted {
    pub fn new(inner: ContentStream, limit: u64) -> (Self, Arc<AtomicBool>) {
        let exceeded = Arc::new(AtomicBool::new(false));
        let counted = Self {
            inner,
            seen: 0,
            limit,
            exceeded: exceeded.clone(),
        };
        (counted, exceeded)
    }
}

impl Stream for Counted {
    type Item = io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let polled = self.inner.as_mut().poll_next(cx);
        if let Poll::Ready(Some(Ok(bytes))) = &polled {
            self.seen = self.seen.saturating_add(bytes.len() as u64);
            if self.seen > self.limit {
                self.exceeded.store(true, Ordering::Relaxed);
                return Poll::Ready(Some(Err(io::Error::other(
                    "the blob is larger than a replicated blob may be",
                ))));
            }
        }
        polled
    }
}

pub struct StagedSource {
    content: Content,
    repository: String,
    lacking: Mutex<Vec<String>>,
}

impl StagedSource {
    pub fn new(content: Content, repository: &str) -> Self {
        Self {
            content,
            repository: repository.to_owned(),
            lacking: Mutex::new(Vec::new()),
        }
    }

    fn lack(&self, sha256: &str) -> ContentError {
        self.lacking
            .lock()
            .expect("the lacking lock is never poisoned")
            .push(sha256.to_owned());
        ContentError::Conflict(format!("the blob {sha256} is not on this server"))
    }

    pub fn lacked(&self) -> Vec<String> {
        self.lacking
            .lock()
            .expect("the lacking lock is never poisoned")
            .clone()
    }

    async fn find(&self, sql: &str, sha256: &str) -> Result<Option<Stored>, ContentError> {
        Ok(Stored::find_by_statement(statement(
            sql,
            vec![self.repository.clone().into(), sha256.into()],
        ))
        .one(self.content.db())
        .await?)
    }
}

#[async_trait]
impl BlobSource for StagedSource {
    async fn write(&self, sink: &Sink, asset: &BundleAsset) -> Result<Written, ContentError> {
        let sha256 = asset.sha256.to_ascii_lowercase();
        let found = match self.find(ASSET_BLOB, &sha256).await? {
            Some(found) => found,
            None => self
                .find(STAGED_BLOB, &sha256)
                .await?
                .ok_or_else(|| self.lack(&sha256))?,
        };
        let digests = Digests {
            sha1: found.sha1,
            sha256: found.sha256,
            sha512: found.sha512,
            md5: found.md5,
        };
        let id = BlobId::from(found.blob_id);
        if found.blob_store == sink.name {
            return match sink.store.duplicate(id, &digests).await {
                Ok(written) => Ok(written),
                Err(astrum_blob::Error::Missing { .. }) => Err(self.lack(&sha256)),
                Err(error) => Err(error.into()),
            };
        }
        let origin = self.content.blob_stores().get(&found.blob_store).await?;
        let content = origin.read(id).await?.ok_or_else(|| self.lack(&sha256))?;
        Ok(sink.store.put(content.stream).await?)
    }
}
