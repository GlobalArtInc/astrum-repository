use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_blob::ContentStream;
use bytes::Bytes;
use futures_util::{Stream, StreamExt, stream};

use crate::error::Error;
use crate::limit::Limiter;

pub const DOWNLOAD_IDLE: Duration = Duration::from_secs(120);

#[derive(Clone, Debug, Default)]
pub struct Failures {
    slot: Arc<Mutex<Option<Error>>>,
}

impl Failures {
    fn set(&self, error: Error) {
        *self
            .slot
            .lock()
            .expect("the failure slot lock is never poisoned") = Some(error);
    }

    pub fn take(&self) -> Option<Error> {
        self.slot
            .lock()
            .expect("the failure slot lock is never poisoned")
            .take()
    }
}

pub struct Download {
    pub length: Option<u64>,
    stream: ContentStream,
    failures: Failures,
}

impl Download {
    pub(crate) fn new(length: Option<u64>, stream: ContentStream, failures: Failures) -> Self {
        Self {
            length,
            stream,
            failures,
        }
    }

    pub fn into_parts(self) -> (ContentStream, Failures) {
        (self.stream, self.failures)
    }
}

struct Reading<S> {
    chunks: Pin<Box<S>>,
    limiter: Option<Limiter>,
    failures: Failures,
    secret: String,
    done: bool,
}

pub(crate) fn body_of<S>(
    chunks: S,
    limiter: Option<Limiter>,
    failures: Failures,
    secret: String,
) -> ContentStream
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
{
    let reading = Reading {
        chunks: Box::pin(chunks),
        limiter,
        failures,
        secret,
        done: false,
    };
    stream::unfold(reading, |mut reading| async move {
        if reading.done {
            return None;
        }
        let read = tokio::time::timeout(DOWNLOAD_IDLE, reading.chunks.next()).await;
        let failed = match read {
            Ok(Some(Ok(bytes))) => {
                if let Some(limiter) = &reading.limiter {
                    limiter.acquire(bytes.len()).await;
                }
                return Some((Ok(bytes), reading));
            }
            Ok(None) => return None,
            Ok(Some(Err(error))) => Error::transport(&error).redacted(&reading.secret),
            Err(_) => Error::Retryable {
                reason: format!(
                    "the download made no progress for {} seconds",
                    DOWNLOAD_IDLE.as_secs()
                ),
                retry_after: None,
                status: None,
            },
        };
        let text = failed.to_string();
        reading.failures.set(failed);
        reading.done = true;
        Some((Err(io::Error::other(text)), reading))
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use tokio::time::Instant;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn a_download_that_goes_quiet_fails_as_retryable_after_the_idle_time() {
        let chunks =
            stream::iter([Ok::<Bytes, reqwest::Error>(Bytes::from("ab"))]).chain(stream::pending());
        let failures = Failures::default();
        let mut body = body_of(chunks, None, failures.clone(), String::new());
        assert_eq!(body.next().await.unwrap().unwrap(), Bytes::from("ab"));
        assert!(failures.take().is_none());
        let started = Instant::now();
        assert!(body.next().await.unwrap().is_err());
        assert_eq!(started.elapsed(), DOWNLOAD_IDLE);
        assert!(matches!(
            failures.take(),
            Some(Error::Retryable { status: None, .. })
        ));
        assert!(body.next().await.is_none());
    }
}
