pub mod almost;
pub mod app;
pub mod detach;
pub mod event_loop;
pub mod image_pipeline;
pub mod runtime;
pub mod video_pipeline;
pub mod wallpaper;
pub mod event;
pub mod task_pool;

use app::VideoApp;
use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use tracing::error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start the daemon program in background
    #[arg(long, default_value_t = false)]
    run_in_background: bool,
}

fn main() {
    let args = Args::parse();

    if args.run_in_background {
        if let Err(error) = detach() {
            error!(?error, "failed to start daemon in the background");
        }
    }

    EventLoop::<VideoApp>::default().run();
}
