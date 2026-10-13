mod access;
mod downloads;
mod error;
pub mod format;
pub mod group;
mod http;
pub mod ids;
pub mod mime;
pub mod proxy;
mod request;
mod serve;
pub mod store;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use astrum_blob::BlobStores;
use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;
use astrum_security::{SecretBox, SecurityModel, Subject};
use sea_orm::DatabaseConnection;

pub use access::{Grant, View};
pub use downloads::DOWNLOAD_DAYS_KEPT;
pub use error::{CHALLENGE, ContentError};
pub use format::{BrowseLayout, Format, SearchKey, UploadField, UploadForm, UploadSpec, fetch};
pub use proxy::Freshness;
pub use request::{ContentRequest, normalize};
pub use serve::{Serve, serve};
pub use store::{Asset, Component, ComponentKey, PutAsset};

pub const DOWNLOAD_FLUSH: Duration = Duration::from_secs(10);

const DOWNLOAD_DAYS_PRUNE: u32 = 360;

#[derive(Clone)]
pub struct Content {
    inner: Arc<Inner>,
}

struct Inner {
    db: DatabaseConnection,
    blob_stores: Arc<BlobStores>,
    secrets: SecretBox,
    formats: RwLock<HashMap<&'static str, Arc<dyn Format>>>,
    proxy: proxy::ProxyState,
    downloads: downloads::Downloads,
}

impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Content")
    }
}

impl Content {
    pub fn new(db: DatabaseConnection, blob_stores: Arc<BlobStores>, secrets: SecretBox) -> Self {
        Self {
            inner: Arc::new(Inner {
                db,
                blob_stores,
                secrets,
                formats: RwLock::new(HashMap::new()),
                proxy: proxy::ProxyState::default(),
                downloads: downloads::Downloads::default(),
            }),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn blob_stores(&self) -> &BlobStores {
        &self.inner.blob_stores
    }

    pub fn secrets(&self) -> &SecretBox {
        &self.inner.secrets
    }

    pub fn register(&self, format: Arc<dyn Format>) {
        self.inner
            .formats
            .write()
            .expect("the format registry lock is never poisoned")
            .insert(format.id(), format);
    }

    pub fn format(&self, id: &str) -> Option<Arc<dyn Format>> {
        self.inner
            .formats
            .read()
            .expect("the format registry lock is never poisoned")
            .get(id)
            .cloned()
    }

    pub fn formats(&self) -> Vec<Arc<dyn Format>> {
        let mut formats: Vec<_> = self
            .inner
            .formats
            .read()
            .expect("the format registry lock is never poisoned")
            .values()
            .cloned()
            .collect();
        formats.sort_by_key(|format| format.id());
        formats
    }

    pub async fn repository(&self, name: &str) -> Result<Option<Arc<Repository>>, ContentError> {
        let found = astrum_repo::store::find(self.db(), name)
            .await
            .map_err(ContentError::internal)?;
        Ok(found
            .filter(|repository| repository.name == name)
            .map(Arc::new))
    }

    pub fn context(
        &self,
        repository: Arc<Repository>,
        security: Arc<SecurityModel>,
        subject: Option<Subject>,
        base_url: impl Into<String>,
        client_ip: impl Into<String>,
    ) -> Ctx {
        Ctx {
            content: self.clone(),
            repository,
            security,
            subject,
            base_url: base_url.into(),
            client_ip: client_ip.into(),
            trusted: false,
        }
    }

    pub fn note_download(&self, asset_id: i64) {
        self.inner.downloads.note(asset_id);
    }

    pub async fn flush_downloads(&self) -> Result<(), ContentError> {
        self.inner.downloads.flush(self.db()).await
    }

    pub async fn prune_download_days(&self) -> Result<u64, ContentError> {
        downloads::Downloads::prune(self.db()).await
    }

    pub fn spawn_background(&self) {
        let content = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(DOWNLOAD_FLUSH);
            let mut ticks: u32 = 0;
            loop {
                ticker.tick().await;
                if let Err(error) = content.flush_downloads().await {
                    tracing::warn!(%error, "download counts were not saved");
                }
                if ticks.is_multiple_of(DOWNLOAD_DAYS_PRUNE)
                    && let Err(error) = content.prune_download_days().await
                {
                    tracing::warn!(%error, "old download days were not deleted");
                }
                ticks = ticks.wrapping_add(1);
            }
        });
    }

    pub fn forget_repository(&self, name: &str) {
        self.inner.proxy.forget(name);
    }

    pub async fn purge_blobs(&self, blobs: Vec<(String, uuid::Uuid)>) {
        store::delete_blobs(self, blobs).await;
    }

    pub(crate) fn proxy_state(&self) -> &proxy::ProxyState {
        &self.inner.proxy
    }
}

#[derive(Clone)]
pub struct Ctx {
    pub content: Content,
    pub repository: Arc<Repository>,
    pub security: Arc<SecurityModel>,
    pub subject: Option<Subject>,
    pub base_url: String,
    pub client_ip: String,
    trusted: bool,
}

impl Ctx {
    pub fn name(&self) -> &str {
        &self.repository.name
    }

    pub fn format_id(&self) -> &'static str {
        self.repository.format.id
    }

    pub fn kind(&self) -> RepositoryType {
        self.repository.kind
    }

    pub fn db(&self) -> &DatabaseConnection {
        self.content.db()
    }

    pub fn repository_url(&self) -> String {
        format!("{}/repository/{}", self.base_url, self.repository.name)
    }

    pub fn is_trusted(&self) -> bool {
        self.trusted
    }

    pub fn member(&self, repository: Arc<Repository>) -> Ctx {
        Ctx {
            repository,
            trusted: true,
            ..self.clone()
        }
    }

    pub fn uploader(&self) -> String {
        self.subject
            .as_ref()
            .map(|subject| subject.user_id.clone())
            .unwrap_or_else(|| "anonymous".to_owned())
    }
}
