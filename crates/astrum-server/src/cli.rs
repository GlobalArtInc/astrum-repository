use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::VERSION;

#[derive(Debug, Default, Parser)]
#[command(name = "astrum", version = VERSION, about = "Astrum Repository server")]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        help = "Configuration file [env: ASTRUM_CONFIG] [default: <data dir>/etc/astrum.toml]"
    )]
    pub config: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        value_name = "DIR",
        help = "Data directory [env: ASTRUM_DATA_DIR] [default: /astrum-data]"
    )]
    pub data_dir: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        value_name = "ADDRESS",
        help = "Address to listen on [env: ASTRUM_LISTEN] [default: 0.0.0.0:8081]"
    )]
    pub listen: Option<SocketAddr>,
    #[arg(
        long,
        global = true,
        value_name = "DIR",
        help = "Directory of the built user interface [env: ASTRUM_WEB_DIR] [default: /usr/share/astrum/web]"
    )]
    pub web_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    #[command(about = "Bring the database schema up to date and serve requests (the default)")]
    Serve,
    #[command(about = "Bring the database schema up to date and exit")]
    Migrate,
    #[command(about = "Exit with 0 when the server on the configured address is available")]
    Check,
    #[command(about = "Print the version")]
    Version,
}
