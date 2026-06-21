use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

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
    },
    /// Start the daemon process
    Start {
        /// Don't wait the daemon to start
        #[arg(long)]
        dont_wait: bool,
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
        /// Monitor to set wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
        /// Where to store the preview
        out: PathBuf,
    },
    /// Pause/Resume current wallpaper
    Pause {
        /// Monitor to pause the wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
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
