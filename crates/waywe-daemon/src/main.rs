pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};
use tap::Pipe;
use tracing::info;
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_ipc::{DaemonSetupResult, config::Config, detach::BINCODE_CONFIG};

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

    if args.run_in_background {
        detach().unwrap();
    }

    let config = Config::read();
    let app = WallpaperApp::from_config(config);

    let mut event_loop = EventLoop::new(app).unwrap_or_else(|err| {
        panic!("failed to construct event loop: {err}");
    });

    if let Some(pipe_path) = &args.init_signal_fifo {
        let mut pipe = File::options()
            .write(true)
            .open(pipe_path)
            .unwrap_or_else(|err| panic!("failed to open signal fifo: {err}"))
            .pipe(BufWriter::new);

        bincode::encode_into_std_write(DaemonSetupResult::Ok(()), &mut pipe, BINCODE_CONFIG)
            .unwrap();
    }

    info!("the daemon is ready to process commands");

    event_loop.run();
}
