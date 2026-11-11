use std::process::ExitCode;

use astrum_server::cli::{Cli, Command, IpAllowCommand};
use astrum_server::config::{self, Config};
use astrum_server::{
    Server, check, migrate, reset_ip_allow, shutdown_signal, telemetry, version_line,
};
use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Serve);
    if command == Command::Version {
        println!("{}", version_line());
        return ExitCode::SUCCESS;
    }
    let config = match config::load(&cli, &|name| std::env::var(name).ok()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("astrum: {error}");
            return ExitCode::FAILURE;
        }
    };
    match command {
        Command::Check => match check::check(config.listen).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("astrum check: {error:#}");
                ExitCode::FAILURE
            }
        },
        Command::IpAllow {
            action: IpAllowCommand::Reset,
        } => match reset_ip_allow(&config).await {
            Ok(()) => {
                println!(
                    "the IP allow lists were cleared: every address is allowed on every surface; running servers follow within a few seconds"
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("astrum ip-allow reset: {error:#}");
                ExitCode::FAILURE
            }
        },
        Command::Serve | Command::Migrate => {
            let logging = match telemetry::init(&config.log, &config.data_dir) {
                Ok(logging) => logging,
                Err(error) => {
                    eprintln!("astrum: cannot set up logging: {error:#}");
                    return ExitCode::FAILURE;
                }
            };
            match run(command, &config, logging).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    tracing::error!(error = format!("{error:#}"), "astrum stopped with an error");
                    ExitCode::FAILURE
                }
            }
        }
        Command::Version => unreachable!(),
    }
}

async fn run(
    command: Command,
    config: &Config,
    logging: astrum_logging::Logging,
) -> anyhow::Result<()> {
    tracing::info!(
        version = %version_line(),
        data_dir = %config.data_dir.display(),
        file = %config.file.display(),
        file_found = config.file_found,
        "starting"
    );
    match command {
        Command::Migrate => migrate(config).await.map(|_| ()),
        _ => {
            Server::start(config)
                .await?
                .with_logging(logging)
                .run(shutdown_signal())
                .await
        }
    }
}
