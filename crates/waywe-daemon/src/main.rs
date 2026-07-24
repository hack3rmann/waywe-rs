pub mod config;
pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use crate::detach::{DaemonSetupReporter, UnwrapOrReport};
use clap::Parser;
use detach::detach;
use display_error_chain::ErrorChainExt;
use event_loop::EventLoop;
use miette_diagnostic_chain::DiagnosticChain;
use std::{io, path::PathBuf};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_config::Config;

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
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("wgpu_hal::vulkan::instance=warn".parse().unwrap());

    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(filter)
        .init();

    let args = Args::parse();

    let mut reporter = DaemonSetupReporter::open(args.init_signal_fifo.as_ref()).unwrap();

    if args.run_in_background {
        detach().unwrap_or_report(&mut reporter);
    }

    let config = Config::read_from_config_paths().unwrap_or_else(|error| {
        warn!(
            error = %error.diagnostic_chain(),
            "failed to load config, falling back the the default one"
        );

        Config::default()
    });
    debug!(?config);

    let app = WallpaperApp::from_config(config);

    let mut event_loop = EventLoop::new(app).unwrap_or_else(|err| {
        panic!("failed to construct event loop: {}", err.chain());
    });

    reporter.report(Ok(()));
    info!("the daemon is ready to process commands");

    event_loop.run();

    info!("shutting down the daemon");
}
