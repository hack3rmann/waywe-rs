pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use crate::detach::{DaemonResultPipe, DetachError};
use clap::Parser;
use detach::{DaemonizeError, detach};
use event_loop::EventLoop;
use std::{io, path::PathBuf, process::ExitCode};
use tracing::info;
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_ipc::{DaemonSetupError, config::Config};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start the daemon program in background
    #[arg(long)]
    run_in_background: bool,
    /// Write result into a FIFO after the daemon initializes itself
    #[arg(long)]
    init_signal_fifo: Option<PathBuf>,
}

fn main() -> ExitCode {
    let filter = EnvFilter::builder()
        .parse("info,wgpu_hal::vulkan::instance=warn")
        .unwrap();

    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(filter)
        .init();

    let args = Args::parse();

    let mut init_pipe = args
        .init_signal_fifo
        .as_ref()
        .map(DaemonResultPipe::open)
        .map(Result::unwrap);

    if args.run_in_background {
        match detach() {
            Ok(()) => {}
            Err(DetachError::Daemonize(DaemonizeError::LockPidfile(errno))) => {
                if let Some(mut pipe) = init_pipe.take() {
                    pipe.write(Err(DaemonSetupError::DaemonPidLock {
                        info: errno.to_string(),
                    }))
                    .unwrap();
                }

                panic!();
            }
            Err(error) => panic!("{error}"),
        }
    }

    let config = Config::read();
    let app = WallpaperApp::from_config(config);

    let mut event_loop = EventLoop::new(app).unwrap_or_else(|err| {
        panic!("failed to construct event loop: {err}");
    });

    if let Some(mut pipe) = init_pipe.take() {
        pipe.write(Ok(())).unwrap();
    }

    info!("the daemon is ready to process commands");

    event_loop.run();

    ExitCode::SUCCESS
}
