use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

use astrum_blob::ContentStream;
use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use astrum_repo::RepositoryType;
use bytes::{Buf, Bytes};
use futures_util::StreamExt;
use tokio::runtime::Handle;
use tokio::sync::Semaphore;

use crate::manifest::Layer;
use crate::reference::Digest;
use crate::storage;
use crate::upstream::Upstreams;

use super::cache;
use super::facts::{self, LayerFacts};
use super::limits::Limits;

static SLOT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(1)));

struct BlobReader {
    stream: ContentStream,
    handle: Handle,
    chunk: Bytes,
}

impl Read for BlobReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.chunk.is_empty() {
            match self.handle.block_on(self.stream.next()) {
                Some(Ok(bytes)) => self.chunk = bytes,
                Some(Err(error)) => return Err(error),
                None => return Ok(0),
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
    Ok(match upstreams.blob(ctx, image, digest).await {
        Ok(Some(asset)) => Located::Stored(Box::new(asset)),
        Ok(None) => Located::Missing(format!("the remote has no layer {digest}")),
        Err(error) => Located::Missing(format!("the layer {digest} could not be fetched: {error}")),
    })
}

pub async fn facts_of(
    ctx: &Ctx,
    upstreams: &Upstreams,
    image: &str,
    layer: &Layer,
    limits: Limits,
) -> Result<LayerFacts, ContentError> {
    let key = layer.digest.to_string();
    if let Some(facts) = cache::load(ctx.db(), &key).await? {
        return Ok(facts);
    }
    let asset = match locate(ctx, upstreams, image, &layer.digest).await? {
        Located::Stored(asset) => asset,
        Located::Missing(reason) => return Ok(LayerFacts::unavailable(reason)),
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
        Err(ContentError::NotFound(reason)) => return Ok(LayerFacts::unavailable(reason)),
        Err(error) => return Err(error),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _stop = StopOnDrop(cancel.clone());
    let handle = Handle::current();
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let reader = BlobReader {
            stream,
            handle,
            chunk: Bytes::new(),
        };
        facts::extract(reader, &limits, cancel)
    });
    let outcome = match task.await {
        Ok(outcome) => outcome,
        Err(error) if error.is_panic() => {
            tracing::error!(layer = %key, "the layer reader panicked");
            return Ok(LayerFacts::unavailable(
                "the layer reader stopped unexpectedly",
            ));
        }
        Err(error) => return Err(ContentError::internal(error)),
    };
    match outcome {
        Ok(facts) => {
            cache::store(ctx.db(), &key, &facts).await?;
            Ok(facts)
        }
        Err(failure) => Ok(LayerFacts::unavailable(failure.to_string())),
    }
}
