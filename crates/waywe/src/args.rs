use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set s image/video as a wallpaper
    Show {
        /// Monitor to set wallpaper on
        #[arg(short, long)]
        monitor: Option<String>,
        /// Path to an image or a video
        path: PathBuf,
    },
    /// Start the daemon process
    Start,
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
}
