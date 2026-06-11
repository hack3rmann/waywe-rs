pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use std::io;

use crate::{
    detach::{DetachError, DetachMode, ReadyChannel},
    event_loop::CreateEventLoopError,
};
use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use tracing::info;
use tracing_subscriber::EnvFilter;
use wallpaper_app::WallpaperApp;
use waywe_ipc::{
    DaemonSetupError,
    config::Config,
    detach::{BINCODE_CONFIG, EXIT_CODE_DAEMON_SETUP},
    ipc::server::CreateServerError,
};
use waywe_runtime::CreateRuntimeError;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start the daemon program in background
    #[arg(long)]
    run_in_background: bool,
    /// Wait for the daemon to initialize its state
    #[arg(long)]
    wait: bool,
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

    let detach_mode = if args.wait {
        DetachMode::Wait
    } else {
        DetachMode::DontWait
    };

    let mut ready_channel = handle_detach(&args, detach_mode);

    let config = Config::read();
    let app = WallpaperApp::from_config(config);

    let mut event_loop = EventLoop::new(app).unwrap_or_else(|err| {
        if let CreateEventLoopError::CreateRuntime(CreateRuntimeError::Ipc(
            CreateServerError::AcquireFileLock,
        )) = err
            && let Some(channel) = &mut ready_channel
        {
            channel.write(Err(DaemonSetupError::DaemonFileLock));
        }
        panic!("failed to construct event loop: {err}");
    });

    if let Some(channel) = ready_channel {
        channel.signal(Ok(()));
        info!("the daemon is ready to process commands");
    }

    event_loop.run();
}

fn handle_detach(args: &Args, mode: DetachMode) -> Option<ReadyChannel> {
    if !args.run_in_background {
        return None;
    }

    detach(mode).unwrap_or_else(|err| handle_detach_error(err))
}

fn handle_detach_error(error: DetachError) -> ! {
    match error {
        DetachError::DaemonSetup(encodable) => {
            bincode::encode_into_std_write(encodable, &mut std::io::stdout(), BINCODE_CONFIG)
                .unwrap();
            std::process::exit(EXIT_CODE_DAEMON_SETUP);
        }
        error => panic!("failed to start daemon in the background: {error}"),
    }
}
