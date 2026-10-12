use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::Context;
use astrum_api::AppState;
use astrum_db::Database;
use astrum_security::SecretBox;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::version_line;

pub struct Server {
    listener: TcpListener,
    address: SocketAddr,
    state: AppState,
}

impl Server {
    pub async fn start(config: &Config) -> anyhow::Result<Self> {
        prepare_data_dir(&config.data_dir)?;
        let db = connect(config).await?;
        db.migrate().await?;
        astrum_security::bootstrap(db.conn(), &config.data_dir).await?;
        let secrets = SecretBox::load_or_create(&config.secrets_key)?;
        let state = AppState::load(db, config.data_dir.clone(), secrets).await?;
        let listener = TcpListener::bind(config.listen)
            .await
            .with_context(|| format!("cannot listen on {}", config.listen))?;
        let address = listener.local_addr()?;
        Ok(Self {
            listener,
            address,
            state,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    pub async fn run(
        self,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> anyhow::Result<()> {
        let app = astrum_api::router(self.state.clone()).layer(TraceLayer::new_for_http());
        tracing::info!(address = %self.address, version = %version_line(), "serving requests");
        axum::serve(self.listener, app)
            .with_graceful_shutdown(shutdown)
            .await
            .context("the HTTP server failed")?;
        self.state.db().clone().close().await?;
        tracing::info!("stopped");
        Ok(())
    }
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
