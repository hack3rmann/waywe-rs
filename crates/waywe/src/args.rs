use clap::{Parser, Subcommand};
use std::{env, path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set an image, video, or package (.ww) as a wallpaper
    Show {
        /// Monitor to set wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
        /// Path to an image, video, or scene package (.ww)
        path: PathBuf,
        /// Don't wait for the daemon to respond
        #[arg(long)]
        dont_wait: bool,
    },
    /// Start the daemon process
    Start {
        /// Don't wait the daemon to start
        #[arg(long)]
        dont_wait: bool,
        /// Path to the `waywe-daemon` binary
        #[arg(long)]
        bin: Option<PathBuf>,
    },
    /// Stop the daemon process
    Stop {
        /// Don't wait the daemon to stop
        #[arg(long)]
        dont_wait: bool,
    },
    /// Get path to the current wallpaper
    Current {
        /// Monitor to set wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
    },
    /// Create a preview for the wallpaper
    Preview {
        /// Path to an image, video, or scene package (.ww)
        path: PathBuf,
        /// Where to store the preview
        #[arg(long, short, default_value = "./preview.png")]
        out: PathBuf,
        /// Width of the resulting image
        #[arg(long, default_value_t = 1920)]
        width: u32,
        /// Height of the resulting image
        #[arg(long, default_value_t = 1080)]
        height: u32,
        /// Advances time (in human-readable notation, like, 420s or 700ms) to this value when rendering
        #[arg(long, short, default_value = "0ms", value_parser = humantime::parse_duration)]
        time: Duration,
    },
    /// Pause/Resume current wallpaper
    Pause {
        /// Monitor to pause the wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
        /// Switches pause on
        #[arg(long, overrides_with = "off")]
        on: bool,
        /// Switches pause off
        #[arg(long, overrides_with = "on")]
        off: bool,
        /// Don't wait for the daemon to respond
        #[arg(long)]
        dont_wait: bool,
    },
    /// Waywe package operations
    Package {
        /// Package command
        #[command(subcommand)]
        command: PackageCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum PackageCommand {
    /// Build Waywe package from source
    Build {
        /// Build in debug mode
        #[arg(long)]
        debug: bool,
        /// Package path
        #[arg(long, short, default_value_t = current_workdir())]
        path: String,
    },
}

fn current_workdir() -> String {
    env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
