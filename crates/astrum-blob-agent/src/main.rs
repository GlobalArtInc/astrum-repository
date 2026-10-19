use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use astrum_blob_agent::{Agent, VERSION, serve, tls};
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "astrum-blob-agent",
    version = VERSION,
    about = "Keeps the blobs of a remote blob store of Astrum Repository on this machine"
)]
struct Cli {
    #[arg(
        long,
        env = "ASTRUM_AGENT_DIR",
        value_name = "DIR",
        default_value = "/astrum-agent",
        help = "Directory that holds the blobs"
    )]
    dir: PathBuf,
    #[arg(
        long,
        env = "ASTRUM_AGENT_LISTEN",
        value_name = "ADDRESS",
        default_value = "0.0.0.0:8090",
        help = "Address to listen on"
    )]
    listen: SocketAddr,
    #[arg(
        long,
        env = "ASTRUM_AGENT_TOKEN_FILE",
        value_name = "FILE",
        help = "File that holds the token the server presents; ASTRUM_AGENT_TOKEN takes the token itself"
    )]
    token_file: Option<PathBuf>,
    #[arg(
        long,
        env = "ASTRUM_AGENT_TLS_CERTIFICATE",
        value_name = "FILE",
        requires = "tls_key",
        help = "PEM certificate chain; with a key the agent speaks HTTPS"
    )]
    tls_certificate: Option<PathBuf>,
    #[arg(
        long,
        env = "ASTRUM_AGENT_TLS_KEY",
        value_name = "FILE",
        requires = "tls_certificate",
        help = "PEM private key of the certificate"
    )]
    tls_key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("ASTRUM_AGENT_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(
                error = format!("{error:#}"),
                "the agent stopped with an error"
            );
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let token = match (std::env::var("ASTRUM_AGENT_TOKEN").ok(), &cli.token_file) {
        (Some(token), _) => token,
        (None, Some(file)) => std::fs::read_to_string(file)
            .with_context(|| format!("cannot read the token from {}", file.display()))?,
        (None, None) => anyhow::bail!("set ASTRUM_AGENT_TOKEN or --token-file"),
    };
    let agent = Agent::open(&cli.dir, token.trim()).await?;
    let tls = match (&cli.tls_certificate, &cli.tls_key) {
        (Some(certificate), Some(key)) => Some(tls(certificate, key)?),
        _ => None,
    };
    let listener = tokio::net::TcpListener::bind(cli.listen)
        .await
        .with_context(|| format!("cannot listen on {}", cli.listen))?;
    tracing::info!(
        version = VERSION,
        dir = %agent.root().display(),
        listen = %cli.listen,
        tls = tls.is_some(),
        "serving blobs"
    );
    serve(listener, agent.router(), tls, shutdown()).await?;
    tracing::info!("stopped");
    Ok(())
}

async fn shutdown() {
    let interrupt = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("a SIGTERM handler can be installed");
        tokio::select! {
            _ = interrupt => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = interrupt.await;
    }
}
