mod access;
mod downloads;
mod error;
pub mod format;
pub mod group;
mod http;
pub mod ids;
pub mod import;
pub mod mime;
pub mod proxy;
mod request;
pub mod routing;
mod serve;
pub mod store;
pub mod zip;

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
    outbound: astrum_outbound::Outbound,
    formats: RwLock<HashMap<&'static str, Arc<dyn Format>>>,
    proxy: proxy::ProxyState,
    routing: routing::RoutingCache,
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
                outbound: astrum_outbound::Outbound::new(db.clone(), secrets.clone()),
                db,
                blob_stores,
                secrets,
                formats: RwLock::new(HashMap::new()),
                proxy: proxy::ProxyState::default(),
                routing: routing::RoutingCache::default(),
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

    pub fn outbound(&self) -> &astrum_outbound::Outbound {
        &self.inner.outbound
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
            request_path: None,
            trusted: false,
        }
    }

    pub async fn routing_allows(
        &self,
        repository: &Repository,
        path: &str,
    ) -> Result<bool, ContentError> {
        self.inner.routing.allows(self.db(), repository, path).await
    }

    pub fn forget_routing_rules(&self) {
        self.inner.routing.forget();
    }

    pub fn note_download(&self, asset_id: i64) {
        self.inner.downloads.note(asset_id);
    }

    pub async fn audit_download(&self, asset: &Asset, initiator: &str) {
        let last = asset
            .last_downloaded_at
            .map(|at| at.with_timezone(&chrono::Utc));
        if !self
            .inner
            .downloads
            .first_since_audit(asset.id, last, chrono::Utc::now())
        {
            return;
        }
        let event =
            astrum_db::audit::Event::new("repository.asset", "updated-downloaded", &asset.path)
                .by(initiator)
                .with(serde_json::json!({
                    "kind": asset.kind,
                    "path": asset.path,
                    "repository.name": asset.repository,
                }));
        if let Err(error) = astrum_db::audit::record(self.db(), event).await {
            tracing::warn!(%error, "a download was not recorded in the audit log");
        }
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

    pub fn forget_ssrf_settings(&self) {
        self.inner.proxy.forget_ssrf();
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
    request_path: Option<Arc<str>>,
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

    pub fn with_request_path(mut self, path: &str) -> Self {
        self.request_path = Some(Arc::from(astrum_repo::routing::request_path(path)));
        self
    }

    pub fn request_path(&self) -> Option<&str> {
        self.request_path.as_deref()
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
