pub mod auth;
pub mod error;
pub mod json;
pub mod routes;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use astrum_db::Database;
use astrum_db::audit::{self, Event};
use astrum_security::{SecretBox, Security};
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
}

impl AppState {
    pub fn new(db: Database, data_dir: PathBuf, security: Security, secrets: SecretBox) -> Self {
        Self {
            inner: Arc::new(Inner {
                db,
                data_dir,
                security,
                secrets,
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

    pub async fn audit(&self, caller: &Caller, event: Event) {
        let event = event.by(caller.initiator());
        if let Err(error) = audit::record(self.db().conn(), event).await {
            tracing::error!(%error, "an audit event was not recorded");
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(routes::status::routes())
        .merge(routes::security::routes())
        .merge(routes::blobstores::routes())
        .merge(routes::repositories::routes())
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
