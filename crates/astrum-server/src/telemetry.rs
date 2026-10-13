use std::io::IsTerminal;
use std::path::Path;

use astrum_logging::{FileWriter, Logging};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry, reload};

use crate::config::{LogConfig, LogFormat};

type Filtered = Layered<reload::Layer<EnvFilter, Registry>, Registry>;
type Boxed = Box<dyn Layer<Filtered> + Send + Sync>;

pub fn init(config: &LogConfig, data_dir: &Path) -> anyhow::Result<Logging> {
    let filter = EnvFilter::try_new(&config.filter)?;
    let (filter, handle) = reload::Layer::new(filter);
    let stderr = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr);
    let stderr: Boxed = match config.format {
        LogFormat::Text => Box::new(stderr),
        LogFormat::Json => Box::new(stderr.json().flatten_event(true)),
    };
    let node = astrum_logging::node_name();
    let (files, file_problem) = match astrum_logging::open_files(data_dir, &node) {
        Ok(files) => (Some(files), None),
        Err(error) => (None, Some(error)),
    };
    let mut layers = vec![stderr];
    if let Some(files) = files.clone() {
        layers.push(Box::new(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(FileWriter(files)),
        ));
    }
    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init()
        .map_err(|error| anyhow::anyhow!(error))?;
    if let Some(error) = file_problem {
        tracing::warn!(%error, dir = %data_dir.join(astrum_logging::LOG_DIR).display(), "logs are not written to files");
    }
    Ok(Logging::new(
        node,
        &config.filter,
        files,
        move |directives| {
            let filter = EnvFilter::try_new(directives).map_err(|error| error.to_string())?;
            handle.reload(filter).map_err(|error| error.to_string())
        },
    ))
}
