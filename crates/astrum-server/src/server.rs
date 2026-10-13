use std::future::Future;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Context;
use astrum_api::AppState;
use astrum_db::Database;
use astrum_security::SecretBox;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::version_line;

const TASK_SHUTDOWN: std::time::Duration = std::time::Duration::from_secs(10);
const LOGGING_SYNC: std::time::Duration = std::time::Duration::from_secs(30);

pub struct Server {
    listener: TcpListener,
    address: SocketAddr,
    state: AppState,
    web_dir: PathBuf,
}

impl Server {
    pub async fn start(config: &Config) -> anyhow::Result<Self> {
        prepare_data_dir(&config.data_dir)?;
        let db = connect(config).await?;
        db.migrate().await?;
        astrum_security::bootstrap(db.conn(), &config.data_dir).await?;
        astrum_blob::bootstrap(db.conn(), &config.data_dir).await?;
        astrum_tasks::bootstrap(db.conn()).await?;
        let secrets = SecretBox::load_or_create(&config.secrets_key)?;
        let state = AppState::load(db, config.data_dir.clone(), secrets).await?;
        state.set_configuration(crate::config::summary(config));
        state.security().ldap().set_cache_ttl(config.ldap_cache_ttl);
        crate::formats::register(state.content());
        crate::formats::register_tasks(state.tasks());
        let listener = TcpListener::bind(config.listen)
            .await
            .with_context(|| format!("cannot listen on {}", config.listen))?;
        let address = listener.local_addr()?;
        Ok(Self {
            listener,
            address,
            state,
            web_dir: config.web_dir.clone(),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    pub fn with_logging(self, logging: astrum_logging::Logging) -> Self {
        self.state.set_logging(logging);
        self
    }

    pub async fn run(
        self,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> anyhow::Result<()> {
        self.state.content().spawn_background();
        let scheduler = self.state.tasks().spawn();
        let logging = spawn_logging_sync(self.state.clone());
        let notifier = self.state.notifier().spawn();
        let heartbeat = astrum_api::nodes::spawn(self.state.clone());
        match crate::web::installed(&self.web_dir) {
            true => tracing::info!(dir = %self.web_dir.display(), "serving the user interface"),
            false => tracing::warn!(
                dir = %self.web_dir.display(),
                "the user interface is not installed; the REST API and repositories work without it"
            ),
        }
        let app = astrum_api::router_with(self.state.clone(), crate::formats::routes())
            .fallback_service(crate::web::router(self.web_dir.clone()))
            .layer(TraceLayer::new_for_http());
        tracing::info!(address = %self.address, version = %version_line(), "serving requests");
        axum::serve(
            self.listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown)
        .await
        .context("the HTTP server failed")?;
        self.state.tasks().shutdown(TASK_SHUTDOWN).await;
        scheduler.abort();
        logging.abort();
        self.state.notifier().stop();
        let _ = notifier.await;
        heartbeat.abort();
        astrum_api::nodes::leave(&self.state).await;
        if let Err(error) = self.state.content().flush_downloads().await {
            tracing::warn!(%error, "download counts were not saved");
        }
        self.state.db().clone().close().await?;
        tracing::info!("stopped");
        Ok(())
    }
}

fn spawn_logging_sync(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Err(error) = astrum_api::routes::support::sync_logging(&state).await {
                tracing::warn!(%error, "the stored log levels could not be applied");
            }
            tokio::time::sleep(LOGGING_SYNC).await;
        }
    })
}

pub async fn migrate(config: &Config) -> anyhow::Result<Vec<String>> {
    let db = connect(config).await?;
    let applied = db.migrate().await?;
    db.close().await?;
    Ok(applied)
}

pub async fn shutdown_signal() {
    let interrupt = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
    tracing::info!("stopping");
}

async fn connect(config: &Config) -> anyhow::Result<Database> {
    Ok(Database::connect(config.database()?).await?)
}

fn prepare_data_dir(dir: &Path) -> anyhow::Result<()> {
    for path in [dir.to_owned(), dir.join("tmp")] {
        std::fs::create_dir_all(&path)
            .with_context(|| format!("cannot create the data directory {}", path.display()))?;
    }
    Ok(())
}
