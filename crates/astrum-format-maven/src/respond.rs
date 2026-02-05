use std::time::{Duration, SystemTime};

use astrum_content::{Asset, ContentError, Ctx};
use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

use crate::path::Hash;

pub const XML: &str = "application/xml";
pub const JSON: &str = "application/json";
pub const TEXT: &str = "text/plain";
pub const HTML: &str = "text/html;charset=utf-8";

#[derive(Clone, Debug)]
pub struct Document {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub last_modified: Option<DateTime<Utc>>,
}

impl Document {
    pub fn xml(text: String, last_modified: Option<DateTime<Utc>>) -> Self {
        Self {
            bytes: text.into_bytes(),
            content_type: XML,
            last_modified,
        }
    }
}

pub enum Found {
    Stored(Box<Asset>),
    Generated(Document),
}

impl Found {
    pub fn digest(&self, hash: Hash) -> String {
        match self {
            Found::Stored(asset) => match hash {
                Hash::Sha1 => asset.sha1.clone(),
                Hash::Md5 => asset.md5.clone(),
                Hash::Sha256 => asset.sha256.clone(),
                Hash::Sha512 => asset.sha512.clone(),
            },
            Found::Generated(document) => digest(hash, &document.bytes),
        }
    }

    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        match self {
            Found::Stored(asset) => Some(asset.blob_updated_at.with_timezone(&Utc)),
            Found::Generated(document) => document.last_modified,
        }
    }

    pub async fn bytes(&self, ctx: &Ctx, limit: usize) -> Result<Vec<u8>, ContentError> {
        match self {
            Found::Stored(asset) => read(ctx, asset, limit).await,
            Found::Generated(document) => Ok(document.bytes.clone()),
        }
    }
}

pub fn digest(hash: Hash, bytes: &[u8]) -> String {
    match hash {
        Hash::Sha1 => hex::encode(Sha1::digest(bytes)),
        Hash::Md5 => hex::encode(Md5::digest(bytes)),
        Hash::Sha256 => hex::encode(Sha256::digest(bytes)),
        Hash::Sha512 => hex::encode(Sha512::digest(bytes)),
    }
}

pub async fn read(ctx: &Ctx, asset: &Asset, limit: usize) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as usize > limit {
        return Err(ContentError::BadRequest(format!(
            "{} is too large to read",
            asset.path
        )));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let content = store
        .read(asset.blob_id.into())
        .await?
        .ok_or_else(|| ContentError::not_found(asset.path.clone()))?;
    let mut stream = content.stream;
    let mut collected = Vec::with_capacity(asset.size.max(0) as usize);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if collected.len() + chunk.len() > limit {
            return Err(ContentError::BadRequest(format!(
                "{} is too large to read",
                asset.path
            )));
        }
        collected.extend_from_slice(&chunk);
    }
    Ok(collected)
}

fn whole_second(time: DateTime<Utc>) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(time.timestamp().max(0) as u64)
}

fn not_modified(headers: &HeaderMap, etag: &str, last_modified: Option<SystemTime>) -> bool {
    if let Some(tags) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
    {
        return tags
            .split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag);
    }
    let since = headers
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| httpdate::parse_http_date(value).ok());
    matches!((since, last_modified), (Some(since), Some(modified)) if modified <= since)
}

pub fn document(
    method: &Method,
    headers: &HeaderMap,
    document: Document,
) -> Result<Response, ContentError> {
    let etag = format!("\"{}\"", digest(Hash::Sha1, &document.bytes));
    let last_modified = document.last_modified.map(whole_second);
    let mut builder = Response::builder().header(header::ETAG, &etag);
    if let Some(modified) = last_modified {
        builder = builder.header(header::LAST_MODIFIED, httpdate::fmt_http_date(modified));
    }
    if not_modified(headers, &etag, last_modified) {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    builder = builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, document.content_type)
        .header(header::CONTENT_LENGTH, document.bytes.len());
    let body = match *method {
        Method::HEAD => Body::empty(),
        _ => Body::from(document.bytes),
    };
    builder.body(body).map_err(ContentError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[test]
    fn digests_are_lowercase_hex() {
        assert_eq!(
            digest(Hash::Sha1, b""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(digest(Hash::Md5, b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(digest(Hash::Sha256, b"").len(), 64);
        assert_eq!(digest(Hash::Sha512, b"").len(), 128);
    }

    #[tokio::test]
    async fn documents_answer_conditions_and_heads() {
        let time = DateTime::<Utc>::from_timestamp(1_760_000_000, 500).unwrap();
        let made = || Document {
            bytes: b"<x/>".to_vec(),
            content_type: XML,
            last_modified: Some(time),
        };
        let response = document(&Method::GET, &HeaderMap::new(), made()).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let etag = response.headers()[header::ETAG].clone();
        let modified = response.headers()[header::LAST_MODIFIED].clone();
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "4");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"<x/>");
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, etag);
        assert_eq!(
            document(&Method::GET, &headers, made()).unwrap().status(),
            StatusCode::NOT_MODIFIED
        );
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_MODIFIED_SINCE, modified);
        assert_eq!(
            document(&Method::GET, &headers, made()).unwrap().status(),
            StatusCode::NOT_MODIFIED
        );
        let head = document(&Method::HEAD, &HeaderMap::new(), made()).unwrap();
        assert_eq!(head.headers()[header::CONTENT_LENGTH], "4");
        let body = head.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }
}
