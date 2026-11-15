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
    #[arg(
        long,
        global = true,
        value_name = "ADDRESSES",
        value_delimiter = ',',
        help = "Addresses or CIDR ranges of the reverse proxies whose X-Forwarded-For is believed, comma-separated [env: ASTRUM_TRUSTED_PROXIES] [default: none]"
    )]
    pub trusted_proxies: Vec<String>,
    #[arg(
        long,
        global = true,
        help = "Let OpenID Connect and LDAP accounts hold personal access tokens [env: ASTRUM_EXTERNAL_TOKENS] [default: off]"
    )]
    pub external_tokens: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    #[command(about = "Bring the database schema up to date and serve requests (the default)")]
    Serve,
    #[command(about = "Bring the database schema up to date and exit")]
    Migrate,
    #[command(about = "Exit with 0 when the server on the configured address is available")]
    Check,
    #[command(about = "Print the version")]
    Version,
    #[command(
        name = "ip-allow",
        about = "Manage the IP allow lists from the command line"
    )]
    IpAllow {
        #[command(subcommand)]
        action: IpAllowCommand,
    },
    #[command(
        name = "two-factor",
        about = "Manage second factors from the command line"
    )]
    TwoFactor {
        #[command(subcommand)]
        action: TwoFactorCommand,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
pub enum IpAllowCommand {
    #[command(
        about = "Clear the management, content and agents lists directly in the database, so every address is allowed again; works while the server is stopped"
    )]
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum TwoFactorCommand {
    #[command(
        about = "Remove the authenticator, passkeys and recovery codes of a user, and close the user's sessions"
    )]
    Reset {
        #[arg(
            value_name = "USER_ID",
            help = "Id of the user whose second factor is reset"
        )]
        user_id: String,
    },
}
