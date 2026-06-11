pub mod args;
pub mod command;
pub mod package;
pub mod progress;
pub mod status;

use crate::{
    args::{Args, Command},
    command::{
        StartWaitMode, execute_current, execute_pause, execute_preview, execute_show, execute_start,
    },
    package::execute_package,
};
use anyhow::{Context as _, bail};
use clap::Parser as _;
use rustix::io::Errno;
use waywe_ipc::{DaemonCommand, IpcSocket, ipc::Client};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    video::init();

    let daemon_command = match Args::parse().command {
        Command::Preview { out, monitor } => {
            execute_preview(&out, monitor.as_deref())?;
            return Ok(());
        }
        Command::Current { monitor } => {
            execute_current(monitor.as_deref())?;
            return Ok(());
        }
        Command::Start { dont_wait } => {
            let mode = if dont_wait {
                StartWaitMode::DontWait
            } else {
                StartWaitMode::Wait
            };

            execute_start(mode);
            return Ok(());
        }
        Command::Show { path, monitor } => execute_show(&path, monitor)?,
        Command::Pause { monitor } => execute_pause(monitor)?,
        Command::Package { command } => {
            execute_package(command)?;
            return Ok(());
        }
    };

    let socket = match IpcSocket::<Client, DaemonCommand>::connect() {
        Ok(socket) => socket,
        Err(Errno::CONNREFUSED) => {
            bail!("no waywe-daemon is running");
        }
        Err(error) => {
            bail!("failed to connect to waywe-daemon: {error}");
        }
    };

    socket
        .send(daemon_command)
        .context("failed to set a command to the daemon")?;

    Ok(())
}
