pub mod args;
pub mod command;

use crate::{
    args::{Args, Command},
    command::{
        execute_current, execute_image, execute_pause, execute_preview, execute_scene,
        execute_start, execute_video,
    },
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
        Command::Start => {
            execute_start();
            return Ok(());
        }
        Command::Video { path, monitor } => execute_video(&path, monitor)?,
        Command::Image { path, monitor } => execute_image(&path, monitor)?,
        Command::Scene { monitor } => execute_scene(monitor)?,
        Command::Pause { monitor } => execute_pause(monitor)?,
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
