use crate::{
    args::ConfigCommand,
    command::{ConnectDaemonError, WaitMode, connect_daemon},
};
use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;
use waywe_config::{Config, ReadConfigError};
use waywe_ipc::{
    ClientError, DaemonCommand,
    command::{DaemonError, DaemonResponse},
};

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    DaemonConnect(#[from] ConnectDaemonError),
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    DaemonError(#[from] DaemonError),
    #[error("unexpected daemon response {0:#?}")]
    #[diagnostic(
        code(waywe::unexpected_daemon_response),
        help(
            "this is a bug, please create a GitHub issue: https://github.com/hack3rmann/waywe-rs/issues/new"
        )
    )]
    UnexpectedDaemonResponse(DaemonResponse),
    #[error(transparent)]
    #[diagnostic(code(waywe::config::failed_to_read))]
    Config(#[from] ReadConfigError),
}

pub fn execute_config(command: ConfigCommand) -> Result<(), ConfigError> {
    match command {
        ConfigCommand::Validate { path } => execute_validate(path),
        ConfigCommand::Reload { path, dont_wait } => {
            execute_reload(path, WaitMode::from_dont(dont_wait))
        }
    }
}

fn execute_reload(path: Option<PathBuf>, wait_mode: WaitMode) -> Result<(), ConfigError> {
    let socket = connect_daemon()?;

    socket.send(DaemonCommand::ConfigReload { path })?;

    if wait_mode == WaitMode::DontWait {
        return Ok(());
    }

    let response = socket.recv()??;

    if response != DaemonResponse::ConfigReloaded {
        return Err(ConfigError::UnexpectedDaemonResponse(response));
    }

    Ok(())
}

fn execute_validate(path: Option<PathBuf>) -> Result<(), ConfigError> {
    let config = Config::read_from(path.as_ref())?;
    eprintln!("{config:#?}");

    Ok(())
}
