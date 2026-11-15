use std::future::Future;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Context;
use astrum_api::AppState;
use astrum_db::Database;
use astrum_security::SecretBox;
use axum::http::{Request, Uri};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{Level, Span};

use crate::config::Config;
use crate::version_line;

const TASK_SHUTDOWN: std::time::Duration = std::time::Duration::from_secs(10);
const REPLICATION_SHUTDOWN: std::time::Duration = std::time::Duration::from_secs(12);
const LOGGING_SYNC: std::time::Duration = std::time::Duration::from_secs(30);
const HIDDEN_SHARE_URI: &str = "/service/share/<hidden>";
const SHARE_SECRET_PREFIX: &[u8] = b"astrum_share_";
const HIDDEN_SHARE_SECRET: &str = "astrum_share_<hidden>";

pub struct Server {
    listener: TcpListener,
    address: SocketAddr,
    state: AppState,
    web_dir: PathBuf,
    tls: Option<std::sync::Arc<rustls::ServerConfig>>,
}

impl Server {
    pub async fn start(config: &Config) -> anyhow::Result<Self> {
        prepare_data_dir(&config.data_dir)?;
        let tls = crate::connectors::tls(&config.connectors)?;
        let db = connect(config).await?;
        db.migrate().await?;
        astrum_security::bootstrap(db.conn(), &config.data_dir).await?;
        astrum_blob::bootstrap(db.conn(), &config.data_dir).await?;
        astrum_tasks::bootstrap(db.conn()).await?;
        let secrets = SecretBox::load_or_create(&config.secrets_key)?;
        let state = AppState::load(db, config.data_dir.clone(), secrets).await?;
        state.set_configuration(crate::config::summary(config));
        state
            .set_trusted_proxies(config.trusted_proxies.clone())
            .map_err(|_| anyhow::anyhow!("the trusted proxies were set twice"))?;
        state.security().ldap().set_cache_ttl(config.ldap_cache_ttl);
        state.security().set_external_tokens(config.external_tokens);
        crate::formats::register(state.content());
        crate::formats::register_tasks(state.tasks());
        state.tasks().register(std::sync::Arc::new(
            astrum_tasks::kinds::advisories::AdvisorySync::new(config.data_dir.join("tmp")),
        ));
        state.tasks().register(std::sync::Arc::new(
            astrum_tasks::kinds::scan::ScanRepository,
        ));
        state.tasks().register(std::sync::Arc::new(
            astrum_api::customer_expiry::CustomerTokenExpiry::new(state.notifier().clone()),
        ));
        crate::formats::register_image_transfers(&state);
        let backfilled = astrum_replication::lifecycle::backfill(state.db().conn())
            .await
            .context("the reconcile tasks of the replications could not be checked")?;
        if backfilled > 0 {
            tracing::info!(
                count = backfilled,
                "created the reconcile tasks of existing replications"
            );
        }
        let listener = TcpListener::bind(config.listen)
            .await
            .with_context(|| format!("cannot listen on {}", config.listen))?;
        let address = listener.local_addr()?;
        state.set_application_port(address.port());
        Ok(Self {
            listener,
            address,
            state,
            web_dir: config.web_dir.clone(),
            tls,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    pub fn state(&self) -> &AppState {
        &self.state
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
        let revisions =
            astrum_api::revisions::spawn(self.state.clone(), astrum_api::revisions::INTERVAL);
        let space = astrum_api::space::spawn(self.state.clone());
        let replication_stop = astrum_replication::dispatch::CancellationToken::new();
        let replication = astrum_replication::dispatch::Dispatcher::with_counters(
            self.state.content().clone(),
            astrum_replication::dispatch::Settings::default(),
            self.state.metrics().replication().clone(),
        )
        .spawn_until(replication_stop.clone());
        let scan = astrum_scan::worker::spawn(
            self.state.content().clone(),
            astrum_scan::worker::Config::load(self.state.db().conn()).await,
            self.state.scan_metrics().clone(),
        );
        let matchers = astrum_scan::matcher::spawn(
            self.state.db().conn().clone(),
            astrum_scan::matcher::Config::load(self.state.db().conn()).await,
            self.state.matcher_metrics().clone(),
        );
        let verdicts = astrum_scan::verdict::spawn(
            self.state.db().conn().clone(),
            astrum_scan::verdict::Config::load(self.state.db().conn()).await,
            self.state.policy_cache().clone(),
            self.state.verdict_metrics().clone(),
        );
        match crate::web::installed(&self.web_dir) {
            true => tracing::info!(dir = %self.web_dir.display(), "serving the user interface"),
            false => tracing::warn!(
                dir = %self.web_dir.display(),
                "the user interface is not installed; the REST API and repositories work without it"
            ),
        }
        let routes = astrum_api::routes_with(self.state.clone(), crate::formats::routes())
            .fallback_service(crate::web::router(self.web_dir.clone()));
        let app = astrum_api::on_domains(self.state.clone(), routes)
            .layer(TraceLayer::new_for_http().make_span_with(request_span));
        let connectors = crate::connectors::Connectors::new(
            self.state.clone(),
            app.clone(),
            self.address.ip(),
            self.tls.clone(),
        );
        let connector_loop = connectors.spawn();
        tracing::info!(address = %self.address, version = %version_line(), "serving requests");
        axum::serve(
            self.listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown)
        .await
        .context("the HTTP server failed")?;
        replication_stop.cancel();
        connector_loop.abort();
        connectors.close().await;
        self.state.tasks().shutdown(TASK_SHUTDOWN).await;
        scheduler.abort();
        logging.abort();
        self.state.notifier().stop();
        let _ = notifier.await;
        heartbeat.abort();
        revisions.abort();
        space.abort();
        if tokio::time::timeout(REPLICATION_SHUTDOWN, replication)
            .await
            .is_err()
        {
            tracing::warn!("the replication dispatcher did not stop in time");
        }
        scan.abort();
        matchers.abort();
        verdicts.abort();
        astrum_api::nodes::leave(&self.state).await;
        if let Err(error) = self.state.content().flush_downloads().await {
            tracing::warn!(%error, "download counts were not saved");
        }
        self.state.db().clone().close().await?;
        tracing::info!("stopped");
        Ok(())
    }
}

fn traced_uri(uri: &Uri) -> String {
    if astrum_api::routes::share::is_endpoint(uri.path()) {
        HIDDEN_SHARE_URI.to_owned()
    } else {
        hide_share_secrets(&uri.to_string())
    }
}

fn share_prefix_end(bytes: &[u8]) -> Option<usize> {
    let mut at = 0;
    for expected in SHARE_SECRET_PREFIX {
        if *expected == b'_' && bytes.get(at..at + 3).is_some_and(is_encoded_underscore) {
            at += 3;
        } else if bytes.get(at).is_some_and(|byte| byte == expected) {
            at += 1;
        } else {
            return None;
        }
    }
    Some(at)
}

fn is_encoded_underscore(bytes: &[u8]) -> bool {
    bytes[0] == b'%' && bytes[1] == b'5' && (bytes[2] == b'F' || bytes[2] == b'f')
}

fn hide_share_secrets(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut hidden = String::with_capacity(text.len());
    let mut copied = 0;
    let mut at = 0;
    while at < bytes.len() {
        let Some(prefix) = share_prefix_end(&bytes[at..]) else {
            at += 1;
            continue;
        };
        let mut end = at + prefix;
        while bytes.get(end).is_some_and(u8::is_ascii_alphanumeric) {
            end += 1;
        }
        hidden.push_str(&text[copied..at]);
        hidden.push_str(HIDDEN_SHARE_SECRET);
        copied = end;
        at = end;
    }
    hidden.push_str(&text[copied..]);
    hidden
}

fn request_span<B>(request: &Request<B>) -> Span {
    tracing::span!(
        target: "tower_http::trace::make_span",
        Level::DEBUG,
        "request",
        method = %request.method(),
        uri = %traced_uri(request.uri()),
        version = ?request.version(),
    )
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

pub async fn reset_ip_allow(config: &Config) -> anyhow::Result<()> {
    let db = connect(config).await?;
    let reset = astrum_security::ip_allow::reset(db.conn()).await;
    if let Err(error) = db.close().await {
        eprintln!(
            "astrum ip-allow reset: warning: the database connection did not close cleanly: {error}"
        );
    }
    reset.context("the IP allow lists could not be cleared")
}

pub async fn reset_two_factor(config: &Config, user_id: &str) -> anyhow::Result<()> {
    let db = connect(config).await?;
    let security = astrum_security::Security::load(db.conn().clone()).await?;
    if !security.model().users.contains_key(user_id) {
        close_with_warning(db, "").await;
        anyhow::bail!("the user {user_id} does not exist");
    }
    if let Err(error) = security.reset(user_id).await {
        close_with_warning(db, "").await;
        return Err(anyhow::Error::new(error)
            .context(format!("the second factor of {user_id} could not be reset")));
    }
    if let Err(error) = astrum_db::audit::record(
        db.conn(),
        astrum_security::two_factor::reset_event(user_id).by("*CLI"),
    )
    .await
    {
        eprintln!(
            "astrum two-factor reset: warning: the second factor of {user_id} was reset, but the audit event was not recorded: {error}"
        );
    }
    close_with_warning(
        db,
        &format!("the second factor of {user_id} was reset, but "),
    )
    .await;
    Ok(())
}

async fn close_with_warning(db: Database, context: &str) {
    if let Err(error) = db.close().await {
        eprintln!(
            "astrum two-factor reset: warning: {context}the database connection did not close cleanly: {error}"
        );
    }
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

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;

    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("the capture lock is never poisoned")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn recorded(uri: &str) -> String {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let sink = buffer.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_ansi(false)
            .with_writer(move || Capture(sink.clone()))
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let request = Request::builder().uri(uri).body(()).unwrap();
            request_span(&request).in_scope(|| tracing::info!("audit"));
        });
        let bytes = buffer
            .lock()
            .expect("the capture lock is never poisoned")
            .clone();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn the_span_of_a_share_request_never_holds_the_secret() {
        let secret = "astrum_share_Zk3Qm9Lw2Xv7Rt5Yb1Nc8Hd4Jf6Gp0SaEu2Io";
        for uri in [
            format!("/service/share/{secret}"),
            format!("/service/share/{secret}/notes/one.txt?x=1"),
            format!("http://files.example.org/service/share/{secret}/"),
        ] {
            let line = recorded(&uri);
            assert!(line.contains("/service/share/<hidden>"), "{line}");
            assert!(line.contains("method=GET"), "{line}");
            assert!(!line.contains(secret), "{line}");
            assert!(!line.contains("one.txt"), "{line}");
        }
    }

    #[test]
    fn a_secret_is_hidden_wherever_it_appears_in_the_uri() {
        let secret = "astrum_share_Zk3Qm9Lw2Xv7Rt5Yb1Nc8Hd4Jf6Gp0SaEu2Io";
        let encoded = secret.replace('_', "%5F");
        let lower = secret.replace('_', "%5f");
        for (uri, expected) in [
            (
                format!("//service/share/{secret}"),
                "//service/share/astrum_share_<hidden>",
            ),
            (
                format!("/SERVICE/share/{secret}/notes/one.txt"),
                "/SERVICE/share/astrum_share_<hidden>/notes/one.txt",
            ),
            (
                format!("/repository/files/a?token={secret}&x=1"),
                "/repository/files/a?token=astrum_share_<hidden>&x=1",
            ),
            (
                format!("/a/b/c/d/e/{secret}/f/g/{secret}"),
                "/a/b/c/d/e/astrum_share_<hidden>/f/g/astrum_share_<hidden>",
            ),
            (format!("/x/{encoded}/y"), "/x/astrum_share_<hidden>/y"),
            (format!("/x?k={lower}"), "/x?k=astrum_share_<hidden>"),
        ] {
            let line = recorded(&uri);
            assert!(line.contains(expected), "{line}");
            assert!(!line.contains("Zk3Qm9"), "{line}");
            assert!(!line.contains("2Io"), "{line}");
        }
    }

    #[test]
    fn the_span_of_an_ordinary_request_keeps_its_uri() {
        let line = recorded("/repository/files/notes/one.txt?x=1");
        assert!(
            line.contains("uri=/repository/files/notes/one.txt?x=1"),
            "{line}"
        );
        let line = recorded("/repository/files/astrum_share/notes_one.txt?x=astrum_");
        assert!(
            line.contains("uri=/repository/files/astrum_share/notes_one.txt?x=astrum_"),
            "{line}"
        );
    }
}
