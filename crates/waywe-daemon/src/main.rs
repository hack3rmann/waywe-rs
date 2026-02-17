pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use std::io;
use tracing::error;
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_ipc::config::Config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start the daemon program in background
    #[arg(long, default_value_t = false)]
    run_in_background: bool,
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

    if args.run_in_background
        && let Err(error) = detach()
    {
        error!(?error, "failed to start daemon in the background");
    }

    let config = Config::read();
    let app = WallpaperApp::from_config(config);

    EventLoop::new(app).run();
}
