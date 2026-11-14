use std::io;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use futures_util::stream;

use crate::digest::{self, Hasher};
use crate::limit::Limiter;
use crate::{BlobId, BlobStore, ContentStream, Digests, Error, Written};

type Failure = Arc<Mutex<Option<Error>>>;

struct Checking {
    content: ContentStream,
    hasher: Hasher,
    store: String,
    id: BlobId,
    expected: Digests,
    failure: Failure,
    limiter: Option<Limiter>,
}

pub(crate) async fn through<S: BlobStore + ?Sized>(
    store: &S,
    id: BlobId,
    expected: &Digests,
) -> Result<Written, Error> {
    let (written, _) = pipe(store, id, expected, store, None).await?;
    Ok(written)
}

pub async fn copy(
    from: &dyn BlobStore,
    id: &BlobId,
    digests: &Digests,
    to: &dyn BlobStore,
    limiter: Option<&Limiter>,
) -> Result<Written, Error> {
    let (written, source_size) = pipe(from, *id, digests, to, limiter).await?;
    if written.size != source_size {
        if let Err(error) = to.delete(written.id).await {
            tracing::warn!(store = to.name(), blob = %written.id, %error, "a copy of the wrong size could not be discarded");
        }
        return Err(Error::Digest {
            store: from.name().to_owned(),
            id: *id,
            algorithm: "size",
            expected: source_size.to_string(),
            actual: written.size.to_string(),
        });
    }
    Ok(written)
}

async fn pipe<F: BlobStore + ?Sized, T: BlobStore + ?Sized>(
    from: &F,
    id: BlobId,
    expected: &Digests,
    to: &T,
    limiter: Option<&Limiter>,
) -> Result<(Written, u64), Error> {
    digest::require(from.name(), id, expected)?;
    let Some(content) = from.read(id).await? else {
        return Err(Error::Missing {
            store: from.name().to_owned(),
            id,
        });
    };
    let source_size = content.size;
    let failure = Failure::default();
    let verified = checking(Checking {
        content: content.stream,
        hasher: Hasher::default(),
        store: from.name().to_owned(),
        id,
        expected: expected.clone(),
        failure: failure.clone(),
        limiter: limiter.cloned(),
    });
    match to.put(verified).await {
        Ok(written) => Ok((written, source_size)),
        Err(error) => Err(failure
            .lock()
            .expect("the duplicate failure lock is never poisoned")
            .take()
            .unwrap_or(error)),
    }
}

fn checking(state: Checking) -> ContentStream {
    stream::unfold(Some(state), |state| async move {
        let mut state = state?;
        match state.content.next().await {
            Some(Ok(bytes)) => {
                state.hasher.update(&bytes);
                if let Some(limiter) = &state.limiter {
                    limiter.acquire(bytes.len()).await;
                }
                Some((Ok(bytes), Some(state)))
            }
            Some(Err(error)) => Some((Err(error), None)),
            None => {
                let (_, actual) = state.hasher.finish();
                match digest::check(&state.store, state.id, &state.expected, &actual) {
                    Ok(()) => None,
                    Err(error) => {
                        let message = error.to_string();
                        *state
                            .failure
                            .lock()
                            .expect("the duplicate failure lock is never poisoned") = Some(error);
                        Some((Err(io::Error::other(message)), None))
                    }
                }
            }
        }
    })
    .boxed()
}
