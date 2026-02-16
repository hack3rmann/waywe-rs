pub mod detach;
pub mod event_loop;
pub mod wallpaper;
pub mod wallpaper_app;

use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use std::{
    env, fs,
    io::{self, ErrorKind},
};
use tracing::{info, error};
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

    // Waywe does not create the config file for you,
    // but it looks for one in the following locations on UNIX systems:
    // 1.  $XDG_CONFIG_HOME/waywe/config.toml
    // 2.  $HOME/.config/waywe/config.toml
    // 3.  /etc/waywe/config.toml
    let config = 'config: {
        let trailing = "waywe/config.toml";
        let xdg_path = env::var_os("XDG_CONFIG_HOME").map(|xdg| {
            let mut p = std::path::PathBuf::from(xdg);
            p.push(trailing);
            p
        });

        let home_path = env::home_dir().map(|mut home| {
            home.push(".config");
            home.push(trailing);
            home
        });

        let etc_path = {
            let mut etc = std::path::PathBuf::from("/etc");
            etc.push(trailing);
            Some(etc)
        };

        for path in [xdg_path, home_path, etc_path].into_iter().flatten() {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            info!("loaded config at {}", path.display());
                            break 'config config;
                        }
                        Err(error) => {
                            error!(?error, "invalid config at {}", path.display());
                            continue;
                        }
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    continue;
                }
                Err(error) => {
                    error!(?error, "failed to read config at {}", path.display());
                    continue;
                }
            }
        }

        Config::default()
    };

    let app = WallpaperApp::from_config(config);
    EventLoop::new(app).run();
}
