pub mod config;
mod digest;
pub mod file;
pub mod group;
mod id;
mod metrics;
pub mod quota;
mod registry;
pub mod s3;
mod stream;

use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;

pub use config::{FileSettings, FillPolicy, GroupSettings, Settings, StoreConfig, StoreKind};
pub use digest::Digests;
pub use file::{Adoption, FileBlobStore};
pub use group::GroupBlobStore;
pub use id::BlobId;
pub use metrics::Metrics;
pub use quota::{QuotaKind, QuotaStatus, SoftQuota};
pub use registry::{BLOBS_DIR, BlobStores, DEFAULT_STORE, bootstrap};
pub use s3::{
    S3BlobStore, S3Bucket, S3Connection, S3Encryption, S3EncryptionKind, S3FailoverBucket, S3Keys,
    S3Security, S3Settings,
};

pub type ContentStream = BoxStream<'static, io::Result<Bytes>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("there is no blob store named \"{0}\"")]
    UnknownStore(String),
    #[error("a read from {offset} is past the end of a blob of {size} bytes")]
    Range { offset: u64, size: u64 },
    #[error("cannot use {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the content could not be read: {0}")]
    Content(#[source] io::Error),
    #[error("S3 refused {operation}: {message}")]
    S3 {
        operation: &'static str,
        message: String,
    },
    #[error("the blob store \"{store}\" is not configured correctly: {message}")]
    Config { store: String, message: String },
    #[error("the blob store \"{0}\" is in use and cannot be deleted")]
    InUse(String),
    #[error("a blob store named \"{0}\" exists already")]
    Duplicate(String),
    #[error(transparent)]
    Database(#[from] astrum_db::Error),
    #[error("a stored secret cannot be opened: {0}")]
    Secret(#[from] astrum_security::secrets::SecretError),
}

impl From<sea_orm::DbErr> for Error {
    fn from(error: sea_orm::DbErr) -> Self {
        Error::Database(astrum_db::Error::Query(error))
    }
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Error {
        let path = path.into();
        move |source| Error::Io { path, source }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    pub id: BlobId,
    pub store: String,
    pub size: u64,
    pub digests: Digests,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub blob_count: u64,
    pub total_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredBlob {
    pub id: BlobId,
    pub size: u64,
    pub modified: SystemTime,
}

pub struct Content {
    pub size: u64,
    pub offset: u64,
    pub length: u64,
    pub stream: ContentStream,
}

impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Content")
            .field("size", &self.size)
            .field("offset", &self.offset)
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}

#[async_trait]
pub trait BlobStore: Send + Sync {
    fn name(&self) -> &str;

    fn kind(&self) -> StoreKind;

    async fn put(&self, content: ContentStream) -> Result<Written, Error>;

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error>;

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error>;

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error>;

    async fn exists(&self, id: BlobId) -> Result<bool, Error> {
        Ok(self.size(id).await?.is_some())
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error>;

    async fn list(&self, after: Option<BlobId>, limit: usize) -> Result<Vec<StoredBlob>, Error>;

    async fn totals(&self) -> Result<Totals, Error>;

    async fn available_space(&self) -> Result<Option<u64>, Error>;

    async fn check(&self) -> Result<(), Error>;
}

pub fn content_of(bytes: impl Into<Bytes>) -> ContentStream {
    futures_util::stream::once(std::future::ready(Ok(bytes.into()))).boxed()
}

pub(crate) fn clamp_range(size: u64, offset: u64, length: u64) -> Result<u64, Error> {
    if offset > size {
        return Err(Error::Range { offset, size });
    }
    Ok(length.min(size - offset))
}
