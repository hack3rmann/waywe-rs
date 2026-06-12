pub mod args;
pub mod command;
pub mod diagnostics;
pub mod package;
pub mod progress;
pub mod status;

use crate::{
    args::{Args, Command},
    command::{
        StartWaitMode, execute_current, execute_pause, execute_preview, execute_show, execute_start,
    },
    diagnostics::DaemonSetupDiagnostics,
    package::execute_package,
};
use clap::Parser as _;
use miette::{Context, Diagnostic, IntoDiagnostic};
use rustix::io::Errno;
use thiserror::Error;
use waywe_ipc::{DaemonCommand, IpcClient};

#[derive(Error, Debug, Diagnostic)]
#[error("no waywe-daemon is running")]
#[diagnostic(
    code(waywe::daemon::not_running),
    help("start the daemon first: `waywe start`")
)]
struct DaemonIsNotRunning;

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    video::init();

    let daemon_command = match Args::parse().command {
        Command::Preview { out, monitor } => {
            execute_preview(&out, monitor.as_deref())
                .into_diagnostic()
                .wrap_err_with(|| format!("while executing preview into '{}'", out.display()))?;
            return Ok(());
        }
        Command::Current { monitor } => {
            execute_current(monitor.as_deref())
                .into_diagnostic()
                .wrap_err("while searching for currenty displayed wallpaper")?;
            return Ok(());
        }
        Command::Start { dont_wait } => {
            let mode = if dont_wait {
                StartWaitMode::DontWait
            } else {
                StartWaitMode::Wait
            };

            if let Err(error) = execute_start(mode) {
                return Err(DaemonSetupDiagnostics::from(error).into());
            }

            return Ok(());
        }
        Command::Show { path, monitor } => execute_show(&path, monitor).into_diagnostic()?,
        Command::Pause { monitor } => execute_pause(monitor).into_diagnostic()?,
        Command::Package { command } => {
            execute_package(command).into_diagnostic()?;
            return Ok(());
        }
    };

    let socket = match IpcClient::<DaemonCommand>::connect() {
        Ok(socket) => socket,
        Err(Errno::CONNREFUSED | Errno::NOENT) => return Err(DaemonIsNotRunning.into()),
        Err(error) => {
            panic!("failed to connect to waywe-daemon: {error}");
        }
    };

    socket.send(daemon_command).into_diagnostic()?;

    Ok(())
}
