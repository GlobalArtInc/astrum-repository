pub mod auth;
pub mod base_url;
pub mod domains;
pub mod error;
pub mod json;
pub mod limits;
pub mod metrics;
mod notify_mail;
pub mod revisions;
pub mod routes;
pub mod space;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use astrum_blob::BlobStores;
use astrum_content::Content;
use astrum_db::Database;
use astrum_db::audit::{self, Event};
use astrum_logging::Logging;
use astrum_notify::Notifier;
use astrum_security::{SecretBox, Security};
use astrum_tasks::Tasks;
use axum::Router;
use axum::http::StatusCode;
use axum::middleware;

pub use auth::Caller;
pub use error::{ApiError, FieldError, Validation};
pub use json::Json;
pub use routes::internal::ui::nodes;
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
    logging: OnceLock<Logging>,
    configuration: OnceLock<Vec<(String, String)>>,
    started: chrono::DateTime<chrono::Utc>,
    notifier: Notifier,
    base_url: base_url::Cache,
    domains: domains::Cache,
    node_id: uuid::Uuid,
    started_at: chrono::DateTime<chrono::FixedOffset>,
    application_port: OnceLock<u16>,
    repository_changes: tokio::sync::Notify,
    limits: limits::Limiter,
    metrics: metrics::Registry,
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
        let notifier = Notifier::new(db.conn().clone(), secrets.clone());
        notifier.set_mail_sender(Arc::new(notify_mail::NotificationMail(
            astrum_mail::Mailer::new(content.outbound().clone()),
        )));
        Self {
            inner: Arc::new(Inner {
                db,
                data_dir,
                security,
                secrets,
                blob_stores,
                content,
                tasks,
                logging: OnceLock::new(),
                configuration: OnceLock::new(),
                started: chrono::Utc::now(),
                notifier,
                base_url: base_url::Cache::default(),
                domains: domains::Cache::default(),
                node_id: uuid::Uuid::new_v4(),
                started_at: chrono::Utc::now().fixed_offset(),
                application_port: OnceLock::new(),
                repository_changes: tokio::sync::Notify::new(),
                limits: limits::Limiter::default(),
                metrics: metrics::Registry::default(),
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

    pub fn mailer(&self) -> astrum_mail::Mailer {
        astrum_mail::Mailer::new(self.content().outbound().clone())
    }

    pub fn tasks(&self) -> &Tasks {
        &self.inner.tasks
    }

    pub fn logging(&self) -> &Logging {
        self.inner.logging.get_or_init(Logging::detached)
    }

    pub fn set_logging(&self, logging: Logging) {
        let _ = self.inner.logging.set(logging);
    }

    pub fn configuration(&self) -> &[(String, String)] {
        self.inner.configuration.get().map_or(&[], Vec::as_slice)
    }

    pub fn set_configuration(&self, entries: Vec<(String, String)>) {
        let _ = self.inner.configuration.set(entries);
    }

    pub fn started(&self) -> chrono::DateTime<chrono::Utc> {
        self.inner.started
    }

    pub fn notifier(&self) -> &Notifier {
        &self.inner.notifier
    }

    pub fn node_id(&self) -> uuid::Uuid {
        self.inner.node_id
    }

    pub fn started_at(&self) -> chrono::DateTime<chrono::FixedOffset> {
        self.inner.started_at
    }

    pub fn set_application_port(&self, port: u16) {
        let _ = self.inner.application_port.set(port);
    }

    pub fn application_port(&self) -> Option<u16> {
        self.inner.application_port.get().copied()
    }

    pub fn repository_changes(&self) -> &tokio::sync::Notify {
        &self.inner.repository_changes
    }

    pub fn limits(&self) -> &limits::Limiter {
        &self.inner.limits
    }

    pub fn metrics(&self) -> &metrics::Registry {
        &self.inner.metrics
    }

    pub async fn audit(&self, caller: &Caller, event: Event) {
        let initiator = match auth::client_address() {
            Some(address) => format!("{}/{address}", caller.initiator()),
            None => caller.initiator(),
        };
        let mut event = event.by(initiator);
        if let Some(client) = routes::mcp::client() {
            let mut attributes = match std::mem::take(&mut event.attributes) {
                serde_json::Value::Object(attributes) => attributes,
                serde_json::Value::Null => serde_json::Map::new(),
                other => serde_json::Map::from_iter([("details".to_owned(), other)]),
            };
            attributes.insert("via".to_owned(), serde_json::Value::from("mcp"));
            attributes.insert("client".to_owned(), serde_json::Value::from(client));
            event.attributes = serde_json::Value::Object(attributes);
        }
        if let Err(error) = audit::record(self.db().conn(), event).await {
            tracing::error!(%error, "an audit event was not recorded");
        }
    }
}

pub fn router(state: AppState) -> Router {
    router_with(state, Router::new())
}

pub fn router_with(state: AppState, extra: Router<AppState>) -> Router {
    on_domains(state.clone(), routes_with(state, extra))
}

pub fn on_domains(state: AppState, app: Router) -> Router {
    Router::new()
        .fallback_service(app)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics::track,
        ))
        .layer(middleware::from_fn_with_state(state, domains::route))
}

pub fn routes_with(state: AppState, extra: Router<AppState>) -> Router {
    endpoints(extra)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authenticate,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            base_url::scope,
        ))
        .with_state(state)
}

pub(crate) fn loopback(state: &AppState) -> Router {
    endpoints(Router::new()).with_state(state.clone())
}

fn endpoints(extra: Router<AppState>) -> Router<AppState> {
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
        .merge(routes::support::routes())
        .merge(routes::capabilities::routes())
        .merge(routes::routing_rules::routes())
        .merge(routes::outbound::routes())
        .merge(routes::email::routes())
        .merge(routes::mcp::routes())
        .merge(routes::oauth::routes())
        .merge(metrics::routes())
        .fallback(not_found)
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
