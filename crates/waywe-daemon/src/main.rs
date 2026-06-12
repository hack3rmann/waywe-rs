pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use crate::detach::DaemonResultPipe;
use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use std::{io, path::PathBuf};
use tracing::info;
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_ipc::config::Config;

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

fn main() {
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
        detach().unwrap();
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
}
