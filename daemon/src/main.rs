pub mod almost;
pub mod app;
pub mod detach;
pub mod event;
pub mod event_loop;
pub mod image_pipeline;
pub mod runtime;
pub mod task_pool;
pub mod video_pipeline;
pub mod wallpaper;
pub mod hash;

use ::runtime::config::Config;
use app::VideoApp;
use clap::Parser;
use detach::detach;
use event_loop::EventLoop;
use std::{
    env, fs,
    io::{self, ErrorKind},
};
use tracing::error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start the daemon program in background
    #[arg(long, default_value_t = false)]
    run_in_background: bool,
}

fn main() {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let args = Args::parse();

    if args.run_in_background && let Err(error) = detach() {
        error!(?error, "failed to start daemon in the background");
    }

    let config = 'config: {
        let Some(mut home_dir) = env::home_dir() else {
            error!("can not find home directory");
            break 'config Config::default();
        };

        let config_path = {
            home_dir.push(".config/waywe/config.toml");
            home_dir
        };

        let contents = match fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let config_dir = config_path.parent().unwrap();
                let config = Config::default();

                match fs::create_dir_all(config_dir) {
                    Ok(()) => {}
                    Err(error) => {
                        error!(?error, "failed to create config directory");
                        break 'config config;
                    }
                }

                let config_string = toml::to_string(&config).unwrap();

                if let Err(error) = fs::write(&config_path, &config_string) {
                    error!(?error, "failed to save the default config");
                }

                break 'config config;
            }
            Err(error) => panic!("failed to load config: {error:?}"),
        };

        match toml::from_str(&contents) {
            Ok(config) => config,
            Err(error) => {
                error!(?error, "failed to load config");
                Config::default()
            }
        }
    };

    EventLoop::new(VideoApp::from_config(config)).run();
}
