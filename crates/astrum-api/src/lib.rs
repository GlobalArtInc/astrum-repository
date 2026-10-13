pub mod auth;
pub mod error;
pub mod json;
pub mod routes;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use astrum_blob::BlobStores;
use astrum_content::Content;
use astrum_db::Database;
use astrum_db::audit::{self, Event};
use astrum_security::{SecretBox, Security};
use astrum_tasks::Tasks;
use axum::Router;
use axum::http::StatusCode;
use axum::middleware;

pub use auth::Caller;
pub use error::{ApiError, FieldError, Validation};
pub use json::Json;
pub use routes::status::{STATUS, STATUS_WRITABLE};

pub const REST: &str = "/service/rest";

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    db: Database,
    data_dir: PathBuf,
    security: Security,
    secrets: SecretBox,
    blob_stores: Arc<BlobStores>,
    content: Content,
    tasks: Tasks,
}

impl AppState {
    pub fn new(db: Database, data_dir: PathBuf, security: Security, secrets: SecretBox) -> Self {
        let security = security.with_secrets(secrets.clone());
        let blob_stores = Arc::new(BlobStores::new(
            db.conn().clone(),
            &data_dir,
            secrets.clone(),
        ));
        let content = Content::new(db.conn().clone(), blob_stores.clone(), secrets.clone());
        let tasks = Tasks::new(content.clone());
        Self {
            inner: Arc::new(Inner {
                db,
                data_dir,
                security,
                secrets,
                blob_stores,
                content,
                tasks,
            }),
        }
    }

    pub async fn load(
        db: Database,
        data_dir: PathBuf,
        secrets: SecretBox,
    ) -> Result<Self, astrum_security::Error> {
        let security = Security::load(db.conn().clone()).await?;
        Ok(Self::new(db, data_dir, security, secrets))
    }

    pub fn db(&self) -> &Database {
        &self.inner.db
    }

    pub fn data_dir(&self) -> &Path {
        &self.inner.data_dir
    }

    pub fn security(&self) -> &Security {
        &self.inner.security
    }

    pub fn secrets(&self) -> &SecretBox {
        &self.inner.secrets
    }

    pub fn blob_stores(&self) -> &BlobStores {
        &self.inner.blob_stores
    }

    pub fn content(&self) -> &Content {
        &self.inner.content
    }

    pub fn tasks(&self) -> &Tasks {
        &self.inner.tasks
    }

    pub async fn audit(&self, caller: &Caller, event: Event) {
        let initiator = match auth::client_address() {
            Some(address) => format!("{}/{address}", caller.initiator()),
            None => caller.initiator(),
        };
        let event = event.by(initiator);
        if let Err(error) = audit::record(self.db().conn(), event).await {
            tracing::error!(%error, "an audit event was not recorded");
        }
    }
}

pub fn router(state: AppState) -> Router {
    router_with(state, Router::new())
}

pub fn router_with(state: AppState, extra: Router<AppState>) -> Router {
    Router::new()
        .merge(extra)
        .merge(routes::status::routes())
        .merge(routes::security::routes())
        .merge(routes::blobstores::routes())
        .merge(routes::repositories::routes())
        .merge(routes::internal::routes())
        .merge(routes::oidc::routes())
        .merge(routes::content::routes())
        .merge(routes::legacy::routes())
        .merge(routes::components::routes())
        .merge(routes::tasks::routes())
        .merge(routes::cleanup_policies::routes())
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authenticate,
        ))
        .with_state(state)
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
