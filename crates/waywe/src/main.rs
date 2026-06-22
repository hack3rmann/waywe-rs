pub mod args;
pub mod command;
pub mod diagnostics;
pub mod package;
pub mod progress;
pub mod status;

use crate::{
    args::{Args, Command},
    command::{
        WaitMode, execute_current, execute_preview, execute_show, execute_start, execute_stop,
    },
    diagnostics::DaemonSetupDiagnostics,
    package::execute_package,
};
use clap::Parser as _;
use miette::{Context, Diagnostic, IntoDiagnostic};
use rustix::io::Errno;
use thiserror::Error;
use waywe_ipc::{DaemonCommand, IpcClient, command::PauseMode};

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
                WaitMode::DontWait
            } else {
                WaitMode::Wait
            };

            if let Err(error) = execute_start(mode) {
                return Err(DaemonSetupDiagnostics::from(error).into());
            }

            return Ok(());
        }
        Command::Stop { dont_wait } => {
            let mode = if dont_wait {
                WaitMode::DontWait
            } else {
                WaitMode::Wait
            };

            execute_stop(mode).wrap_err("failed to stop the daemon")?;

            return Ok(());
        }
        Command::Show { path, monitor } => execute_show(&path, monitor).into_diagnostic()?,
        Command::Pause { monitor, on, off } => DaemonCommand::Pause {
            monitor,
            mode: PauseMode::from_on_off(on, off),
        },
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
