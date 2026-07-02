pub mod args;
pub mod command;
pub mod diagnostics;
pub mod package;
pub mod progress;
pub mod status;

use crate::{
    args::{Args, Command},
    command::{
        WaitMode, execute_current, execute_pause, execute_preview, execute_show, execute_start,
        execute_stop,
    },
    diagnostics::DaemonSetupDiagnostics,
    package::execute_package,
};
use clap::Parser as _;
use miette::IntoDiagnostic;
use waywe_ipc::command::PauseMode;

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    match Args::parse().command {
        Command::Preview {
            out,
            path,
            width,
            height,
        } => execute_preview(&out, &path, width, height)?,
        Command::Current { monitor } => execute_current(monitor.as_deref())?,
        Command::Start { dont_wait } => {
            if let Err(error) = execute_start(WaitMode::from_dont(dont_wait)) {
                return Err(DaemonSetupDiagnostics::from(error).into());
            }
        }
        Command::Stop { dont_wait } => execute_stop(WaitMode::from_dont(dont_wait))?,
        Command::Show {
            path,
            monitor,
            dont_wait,
        } => execute_show(&path, monitor, WaitMode::from_dont(dont_wait))?,
        Command::Pause {
            monitor,
            on,
            off,
            dont_wait,
        } => execute_pause(
            monitor,
            PauseMode::from_on_off(on, off),
            WaitMode::from_dont(dont_wait),
        )?,
        Command::Package { command } => execute_package(command).into_diagnostic()?,
    }

    Ok(())
}
