use std::io;
use std::sync::Mutex;

use axum::body::Body;
use axum::http::{HeaderMap, Method};
use futures_util::TryStreamExt;

use astrum_blob::ContentStream;

use crate::error::ContentError;

pub struct ContentRequest {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub headers: HeaderMap,
    body: Mutex<Option<Body>>,
}

impl ContentRequest {
    pub fn new(
        method: Method,
        raw_path: &str,
        query: Option<String>,
        headers: HeaderMap,
        body: Body,
    ) -> Result<Self, ContentError> {
        Ok(Self {
            method,
            path: normalize(raw_path)?,
            query,
            headers,
            body: Mutex::new(Some(body)),
        })
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    pub fn is_directory(&self) -> bool {
        self.path.is_empty() || self.path.ends_with('/')
    }

    pub fn take_body(&self) -> Body {
        self.body
            .lock()
            .expect("the request body lock is never poisoned")
            .take()
            .unwrap_or_else(Body::empty)
    }

    pub fn into_stream(self) -> ContentStream {
        Box::pin(
            self.take_body()
                .into_data_stream()
                .map_err(io::Error::other),
        )
    }

    pub async fn bytes(&self, limit: usize) -> Result<bytes::Bytes, ContentError> {
        axum::body::to_bytes(self.take_body(), limit)
            .await
            .map_err(|error| {
                ContentError::BadRequest(format!("the request body cannot be read: {error}"))
            })
    }
}

pub fn normalize(raw: &str) -> Result<String, ContentError> {
    let mut path = String::with_capacity(raw.len() + 1);
    let trailing = raw.ends_with('/') && !raw.trim_matches('/').is_empty();
    for segment in raw.split('/').filter(|segment| !segment.is_empty()) {
        if segment == "." || segment == ".." {
            return Err(ContentError::BadRequest(format!(
                "the path {raw} is not allowed"
            )));
        }
        if segment.chars().any(|c| c.is_control()) {
            return Err(ContentError::BadRequest(
                "the path holds control characters".to_owned(),
            ));
        }
        path.push('/');
        path.push_str(segment);
    }
    if trailing || path.is_empty() {
        path.push('/');
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_normalised() {
        assert_eq!(normalize("a/b/c.txt").unwrap(), "/a/b/c.txt");
        assert_eq!(normalize("/a//b/c.txt").unwrap(), "/a/b/c.txt");
        assert_eq!(normalize("a/b/").unwrap(), "/a/b/");
        assert_eq!(normalize("").unwrap(), "/");
        assert_eq!(normalize("/").unwrap(), "/");
        assert_eq!(normalize("sp ace/a+b.txt").unwrap(), "/sp ace/a+b.txt");
    }

    #[test]
    fn dot_segments_and_control_characters_are_refused() {
        for bad in ["a/../b", "./a", "a/\u{0}b", "a/b\n"] {
            assert!(normalize(bad).is_err(), "{bad:?}");
        }
    }
}
