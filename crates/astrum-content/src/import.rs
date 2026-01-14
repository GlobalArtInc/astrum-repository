use std::sync::Arc;

use astrum_blob::{BlobId, BlobStore};
use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;
use futures_util::TryStreamExt;
use serde_json::Value;

use crate::ContentError;
use crate::store::{ComponentKey, PutAsset};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    Import,
    Derived,
    Skip(String),
}

#[derive(Clone, Debug, Default)]
pub struct ImportedComponent {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    pub attributes: Value,
}

impl ImportedComponent {
    pub fn key(&self) -> ComponentKey {
        ComponentKey {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            kind: self.kind.clone(),
            attributes: self.attributes.clone(),
        }
    }
}

pub struct Imported {
    pub repository: RepositoryType,
    pub config: Arc<Repository>,
    pub path: String,
    pub kind: String,
    pub content_type: String,
    pub attributes: Value,
    pub component: Option<ImportedComponent>,
    pub store: Arc<dyn BlobStore>,
    pub blob: BlobId,
    pub size: u64,
    pub sha256: String,
}

impl Imported {
    pub fn cached(&self) -> bool {
        self.repository == RepositoryType::Proxy
    }

    pub fn put(&self, path: impl Into<String>) -> PutAsset {
        match self.cached() {
            true => PutAsset::cached(path),
            false => PutAsset {
                path: path.into(),
                attributes: Value::Object(Default::default()),
                ..Default::default()
            },
        }
    }

    pub async fn bytes(&self, limit: u64) -> Result<Vec<u8>, ContentError> {
        if self.size > limit {
            return Err(ContentError::BadRequest(format!(
                "{} holds {} bytes, more than the {limit} its format reads",
                self.path, self.size
            )));
        }
        let content = self.store.read(self.blob).await?.ok_or_else(|| {
            ContentError::not_found(format!("the blob of {} is missing", self.path))
        })?;
        collect(content.stream, self.size as usize).await
    }

    pub async fn head(&self, length: u64) -> Result<Vec<u8>, ContentError> {
        if self.size == 0 {
            return Ok(Vec::new());
        }
        let content = self
            .store
            .read_range(self.blob, 0, length)
            .await?
            .ok_or_else(|| {
                ContentError::not_found(format!("the blob of {} is missing", self.path))
            })?;
        collect(content.stream, content.length as usize).await
    }

    pub async fn range(&self, offset: u64, length: u64) -> Result<Vec<u8>, ContentError> {
        let content = self
            .store
            .read_range(self.blob, offset, length)
            .await?
            .ok_or_else(|| {
                ContentError::not_found(format!("the blob of {} is missing", self.path))
            })?;
        collect(content.stream, content.length as usize).await
    }
}

async fn collect(
    mut stream: astrum_blob::ContentStream,
    expected: usize,
) -> Result<Vec<u8>, ContentError> {
    let mut collected = Vec::with_capacity(expected);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        collected.extend_from_slice(&chunk);
    }
    Ok(collected)
}

#[derive(Debug)]
pub enum Body {
    Source,
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct Described {
    pub put: PutAsset,
    pub body: Body,
    pub extra: Vec<(PutAsset, Vec<u8>)>,
    pub keep_component: bool,
}

impl Described {
    pub fn source(put: PutAsset) -> Self {
        Self {
            put,
            body: Body::Source,
            extra: Vec::new(),
            keep_component: false,
        }
    }

    pub fn bytes(put: PutAsset, bytes: Vec<u8>) -> Self {
        Self {
            put,
            body: Body::Bytes(bytes),
            extra: Vec::new(),
            keep_component: false,
        }
    }

    pub fn keeping_component(mut self) -> Self {
        self.keep_component = true;
        self
    }

    pub fn with(mut self, put: PutAsset, bytes: Vec<u8>) -> Self {
        self.extra.push((put, bytes));
        self
    }
}
