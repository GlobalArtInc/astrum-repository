use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use astrum_blob::ContentStream;
use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use astrum_repo::RepositoryType;
use bytes::{Buf, Bytes};
use futures_util::StreamExt;
use tokio::runtime::Handle;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::manifest::Layer;
use crate::reference::Digest;
use crate::storage;
use crate::upstream::Upstreams;

use super::cache;
use super::facts::{self, Failure, LayerFacts};
use super::limits::Limits;
use super::stream::Guard;

static SLOT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(1)));

struct BlobReader {
    stream: ContentStream,
    handle: Handle,
    chunk: Bytes,
    deadline: tokio::time::Instant,
}

impl BlobReader {
    fn new(stream: ContentStream, deadline: Instant) -> Self {
        BlobReader {
            stream,
            handle: Handle::current(),
            chunk: Bytes::new(),
            deadline: tokio::time::Instant::from_std(deadline),
        }
    }
}

impl Read for BlobReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.chunk.is_empty() {
            let next = tokio::time::timeout_at(self.deadline, self.stream.next());
            match self.handle.block_on(next) {
                Ok(Some(Ok(bytes))) => self.chunk = bytes,
                Ok(Some(Err(error))) => return Err(error),
                Ok(None) => return Ok(0),
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "the blob store did not answer before the deadline",
                    ));
                }
            }
        }
        let taken = buf.len().min(self.chunk.len());
        buf[..taken].copy_from_slice(&self.chunk[..taken]);
        self.chunk.advance(taken);
        Ok(taken)
    }
}

struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

enum Located {
    Stored(Box<Asset>),
    Missing(String),
}

async fn locate(
    ctx: &Ctx,
    upstreams: &Upstreams,
    image: &str,
    digest: &Digest,
) -> Result<Located, ContentError> {
    if let Some(asset) = storage::find(ctx, &storage::blob_path(digest)).await? {
        return Ok(Located::Stored(Box::new(asset)));
    }
    if ctx.kind() != RepositoryType::Proxy {
        return Ok(Located::Missing(format!(
            "the layer {digest} is not stored in this repository"
        )));
    }
    match upstreams.blob(ctx, image, digest).await {
        Ok(Some(asset)) => Ok(Located::Stored(Box::new(asset))),
        Ok(None) => Ok(Located::Missing(format!(
            "the remote has no layer {digest}"
        ))),
        Err(error @ ContentError::Internal(_)) => Err(error),
        Err(error) => Ok(Located::Missing(format!(
            "the layer {digest} could not be fetched: {error}"
        ))),
    }
}

fn extract_blocking(
    permit: OwnedSemaphorePermit,
    reader: BlobReader,
    limits: &Limits,
    guard: Guard,
) -> Result<LayerFacts, Failure> {
    let _permit = permit;
    facts::extract(reader, limits, guard)
}

fn unavailable(key: &str, reason: impl Into<String>) -> LayerFacts {
    let reason = reason.into();
    tracing::warn!(layer = %key, %reason, "a layer could not be read and the inventory is incomplete");
    LayerFacts::unavailable(reason)
}

pub async fn facts_of(
    ctx: &Ctx,
    upstreams: &Upstreams,
    image: &str,
    layer: &Layer,
    limits: Limits,
    deadline: Instant,
) -> Result<LayerFacts, ContentError> {
    let key = layer.digest.to_string();
    if let Some(facts) = cache::load(ctx.db(), &key).await? {
        return Ok(facts);
    }
    let asset = match locate(ctx, upstreams, image, &layer.digest).await? {
        Located::Stored(asset) => asset,
        Located::Missing(reason) => return Ok(unavailable(&key, reason)),
    };
    let permit = SLOT
        .clone()
        .acquire_owned()
        .await
        .map_err(ContentError::internal)?;
    if let Some(facts) = cache::load(ctx.db(), &key).await? {
        return Ok(facts);
    }
    let stream = match storage::stream_of(ctx, &asset).await {
        Ok(stream) => stream,
        Err(ContentError::NotFound(reason)) => return Ok(unavailable(&key, reason)),
        Err(error) => return Err(error),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _stop = StopOnDrop(cancel.clone());
    let guard = Guard::new(cancel, deadline);
    let reader = BlobReader::new(stream, deadline);
    let task =
        tokio::task::spawn_blocking(move || extract_blocking(permit, reader, &limits, guard));
    let outcome = match task.await {
        Ok(outcome) => outcome,
        Err(error) if error.is_panic() => {
            return Ok(unavailable(&key, "the layer reader stopped unexpectedly"));
        }
        Err(error) => return Err(ContentError::internal(error)),
    };
    match outcome {
        Ok(facts) => {
            if facts.incomplete {
                tracing::warn!(layer = %key, reasons = ?facts.reasons, "the facts of a layer are incomplete");
            }
            cache::store(ctx.db(), &key, &facts, limits.cache_limit).await?;
            Ok(facts)
        }
        Err(Failure::Source(message)) => Err(ContentError::internal(io::Error::other(format!(
            "the layer {key} could not be read: {message}"
        )))),
        Err(failure @ (Failure::Cancelled | Failure::Deadline)) => {
            Ok(unavailable(&key, failure.to_string()))
        }
    }
}

#[cfg(test)]
mod tests;
