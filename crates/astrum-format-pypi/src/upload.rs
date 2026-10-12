use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use astrum_blob::ContentStream;
use astrum_content::store;
use astrum_content::{Asset, ContentError, ContentRequest, Ctx};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use md5::Md5;
use sha2::{Digest, Sha256};

use crate::hosted::{self, Package};
use crate::metadata::CoreMetadata;

const MAX_FIELD: u64 = 8 * 1024 * 1024;
const CONTENT: &str = "content";

#[derive(Clone, Debug, Default)]
struct Expected {
    sha256: Option<String>,
    md5: Option<String>,
}

impl Expected {
    fn from(fields: &[(String, String)]) -> Self {
        let get = |name: &str| {
            fields
                .iter()
                .find(|(field, value)| field == name && !value.trim().is_empty())
                .map(|(_, value)| value.trim().to_ascii_lowercase())
        };
        Expected {
            sha256: get("sha256_digest"),
            md5: get("md5_digest"),
        }
    }

    fn check(&self, sha256: &str, md5: &str) -> Result<(), String> {
        for (wanted, found, algorithm) in
            [(&self.sha256, sha256, "sha256"), (&self.md5, md5, "md5")]
        {
            if let Some(wanted) = wanted
                && wanted != found
            {
                return Err(format!(
                    "the {algorithm} digest of the file is {found}, but the form says {wanted}"
                ));
            }
        }
        Ok(())
    }
}

struct Verified {
    inner: ContentStream,
    sha256: Sha256,
    md5: Md5,
    expected: Expected,
    failure: Arc<Mutex<Option<String>>>,
    finished: bool,
}

impl Stream for Verified {
    type Item = io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                self.sha256.update(&bytes);
                self.md5.update(&bytes);
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(None) => {
                self.finished = true;
                let sha256 = hex::encode(self.sha256.finalize_reset());
                let md5 = hex::encode(self.md5.finalize_reset());
                match self.expected.check(&sha256, &md5) {
                    Ok(()) => Poll::Ready(None),
                    Err(message) => {
                        *self
                            .failure
                            .lock()
                            .expect("the digest failure lock is never poisoned") =
                            Some(message.clone());
                        Poll::Ready(Some(Err(io::Error::other(message))))
                    }
                }
            }
            other => other,
        }
    }
}

fn bad(error: multer::Error) -> ContentError {
    ContentError::BadRequest(format!("the upload form cannot be read: {error}"))
}

fn field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value.as_str())
}

pub async fn legacy(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    let boundary = request
        .header("content-type")
        .and_then(|value| multer::parse_boundary(value).ok())
        .ok_or_else(|| {
            ContentError::BadRequest("the upload must be a multipart/form-data request".to_owned())
        })?;
    let limits = multer::Constraints::new().size_limit(
        multer::SizeLimit::new()
            .per_field(MAX_FIELD)
            .for_field(CONTENT, u64::MAX),
    );
    let mut multipart = multer::Multipart::with_constraints(
        request.take_body().into_data_stream(),
        boundary,
        limits,
    );
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut stored: Option<(Asset, Expected)> = None;
    while let Some(part) = multipart.next_field().await.map_err(bad)? {
        let name = part.name().unwrap_or_default().to_owned();
        if name == CONTENT {
            if stored.is_some() {
                return Err(ContentError::BadRequest(
                    "the upload holds more than one content part".to_owned(),
                ));
            }
            match field(&fields, ":action") {
                Some("file_upload") => {}
                other => {
                    return Err(ContentError::BadRequest(format!(
                        "the :action {} is not supported; only file_upload is",
                        other.unwrap_or("(missing)")
                    )));
                }
            }
            let package = Package {
                metadata: CoreMetadata::from_form(
                    fields
                        .iter()
                        .map(|(name, value)| (name.as_str(), value.as_str())),
                ),
                filename: hosted::file_name(part.file_name())?,
            };
            let expected = Expected::from(&fields);
            let failure = Arc::new(Mutex::new(None));
            let body: ContentStream = Box::pin(Verified {
                inner: Box::pin(part.map_err(io::Error::other)),
                sha256: Sha256::new(),
                md5: Md5::new(),
                expected: expected.clone(),
                failure: failure.clone(),
                finished: false,
            });
            let asset = match hosted::store_package(ctx, &package, body).await {
                Ok(asset) => asset,
                Err(error) => {
                    let mismatch = failure
                        .lock()
                        .expect("the digest failure lock is never poisoned")
                        .take();
                    return Err(match mismatch {
                        Some(message) => ContentError::BadRequest(message),
                        None => error,
                    });
                }
            };
            stored = Some((asset, expected));
        } else if part.file_name().is_some() {
            let mut part = part;
            while part.chunk().await.map_err(bad)?.is_some() {}
        } else {
            let text = part.text().await.map_err(bad)?;
            fields.push((name, text));
        }
    }
    let Some((asset, before)) = stored else {
        return Err(ContentError::BadRequest(
            "the upload has no content part".to_owned(),
        ));
    };
    let after = Expected::from(&fields);
    let late = Expected {
        sha256: after.sha256.filter(|_| before.sha256.is_none()),
        md5: after.md5.filter(|_| before.md5.is_none()),
    };
    if let Err(message) = late.check(&asset.sha256, &asset.md5) {
        store::delete_asset(&ctx.content, asset).await?;
        return Err(ContentError::BadRequest(message));
    }
    Ok(StatusCode::OK.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digests_are_compared_when_given() {
        let expected = Expected::from(&[
            ("sha256_digest".to_owned(), "AB".to_owned()),
            ("md5_digest".to_owned(), " ".to_owned()),
        ]);
        assert_eq!(expected.sha256.as_deref(), Some("ab"));
        assert_eq!(expected.md5, None);
        assert!(expected.check("ab", "anything").is_ok());
        assert!(expected.check("cd", "anything").is_err());
        assert!(Expected::default().check("x", "y").is_ok());
    }

    #[tokio::test]
    async fn the_stream_fails_at_its_end_on_a_mismatch() {
        let failure = Arc::new(Mutex::new(None));
        let chunks: Vec<io::Result<Bytes>> = vec![Ok(Bytes::from_static(b"hello"))];
        let verified = Verified {
            inner: Box::pin(futures_util::stream::iter(chunks)),
            sha256: Sha256::new(),
            md5: Md5::new(),
            expected: Expected {
                sha256: Some("00".to_owned()),
                md5: None,
            },
            failure: failure.clone(),
            finished: false,
        };
        let collected: Vec<io::Result<Bytes>> = futures_util::StreamExt::collect(verified).await;
        assert_eq!(collected.len(), 2);
        assert!(collected[1].is_err());
        assert!(
            failure
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .contains("2cf24dba")
        );

        let chunks: Vec<io::Result<Bytes>> = vec![Ok(Bytes::from_static(b"hello"))];
        let fine = Verified {
            inner: Box::pin(futures_util::stream::iter(chunks)),
            sha256: Sha256::new(),
            md5: Md5::new(),
            expected: Expected {
                sha256: None,
                md5: Some("5d41402abc4b2a76b9719d911017c592".to_owned()),
            },
            failure: Arc::new(Mutex::new(None)),
            finished: false,
        };
        let collected: Vec<io::Result<Bytes>> = futures_util::StreamExt::collect(fine).await;
        assert!(collected.iter().all(Result::is_ok));
    }
}
