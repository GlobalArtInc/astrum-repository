use std::io;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use futures_util::stream;

use crate::digest::{self, Hasher};
use crate::{BlobId, BlobStore, ContentStream, Digests, Error, Written};

type Failure = Arc<Mutex<Option<Error>>>;

struct Checking {
    content: ContentStream,
    hasher: Hasher,
    store: String,
    id: BlobId,
    expected: Digests,
    failure: Failure,
}

pub(crate) async fn through<S: BlobStore + ?Sized>(
    store: &S,
    id: BlobId,
    expected: &Digests,
) -> Result<Written, Error> {
    digest::require(store.name(), id, expected)?;
    let Some(content) = store.read(id).await? else {
        return Err(Error::Missing {
            store: store.name().to_owned(),
            id,
        });
    };
    let failure = Failure::default();
    let verified = checking(Checking {
        content: content.stream,
        hasher: Hasher::default(),
        store: store.name().to_owned(),
        id,
        expected: expected.clone(),
        failure: failure.clone(),
    });
    match store.put(verified).await {
        Ok(written) => Ok(written),
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
