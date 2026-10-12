pub mod check;
pub mod cli;
pub mod config;
pub mod formats;
mod server;
pub mod telemetry;

pub use server::{Server, migrate, shutdown_signal};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REVISION: Option<&str> = option_env!("ASTRUM_GIT_REVISION");

pub fn version_line() -> String {
    match REVISION.filter(|revision| !revision.is_empty()) {
        Some(revision) => format!("Astrum Repository {VERSION} ({revision})"),
        None => format!("Astrum Repository {VERSION}"),
    }
}
