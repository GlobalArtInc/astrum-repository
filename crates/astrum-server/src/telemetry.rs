use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

use crate::config::{LogConfig, LogFormat};

pub fn init(config: &LogConfig) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(&config.filter)?;
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr);
    let installed = match config.format {
        LogFormat::Text => builder.try_init(),
        LogFormat::Json => builder.json().flatten_event(true).try_init(),
    };
    installed.map_err(|error| anyhow::anyhow!(error))
}
