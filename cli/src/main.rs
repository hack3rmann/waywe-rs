use clap::{Parser, Subcommand};
use image::ImageReader;
use runtime::{DaemonCommand, IpcSocket, ipc::Client, profile::SetupProfile};
use rustix::io::Errno;
use std::{
    ffi::CStr,
    path::PathBuf,
    process::{ExitCode, Stdio},
};
use tracing::error;
use video::{FormatContext, MediaType, VideoPixelFormat};

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    video::init();

    let daemon_command = match Args::parse().command {
        Command::Current => {
            let profile = match SetupProfile::read() {
                Ok(profile) => profile,
                Err(error) => {
                    error!(?error, "failed to open profile file");
                    return ExitCode::FAILURE;
                }
            };

            println!("{}", profile.path.display());

            return ExitCode::SUCCESS;
        }
        Command::Start => {
            // NOTE(hack3rmann): waywe-daemon process will daemonize itself
            #[allow(clippy::zombie_processes)]
            let _child = std::process::Command::new("waywe-daemon")
                .arg("--run-in-background")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();

            return ExitCode::SUCCESS;
        }
        Command::Video { path } => {
            let absolute_path = match path.canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    error!(?error, "failed to construct absolute path");
                    return ExitCode::FAILURE;
                }
            };

            if !is_video_path_valid(absolute_path.clone()) {
                error!(?absolute_path, "can not send video to the daemon");
                return ExitCode::FAILURE;
            }

            DaemonCommand::SetVideo {
                path: absolute_path,
            }
        }
        Command::Image { path } => {
            let _reader = match ImageReader::open(&path) {
                Ok(image) => image,
                Err(error) => {
                    error!(?error, "failed to open image");
                    return ExitCode::FAILURE;
                }
            };

            let absolute_path = match path.canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    error!(?error, "failed to construct absolute path");
                    return ExitCode::FAILURE;
                }
            };

            DaemonCommand::SetImage {
                path: absolute_path,
            }
        }
    };

    let socket = match IpcSocket::<Client, DaemonCommand>::connect() {
        Ok(socket) => socket,
        Err(Errno::CONNREFUSED) => {
            error!("no waywe-daemon is running");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            error!(?error, "failed to connect to waywe-daemon");
            return ExitCode::FAILURE;
        }
    };

    socket.send(daemon_command).unwrap();

    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Set a video as a wallpaper
    Video {
        /// Path to the video
        path: PathBuf,
    },
    /// Set an image as a wallpaper
    Image {
        /// Path to the image
        path: PathBuf,
    },
    /// Start the daemon process
    Start,
    /// Get path to the current wallpaper
    Current,
}

fn is_video_path_valid(path: PathBuf) -> bool {
    if !path.exists() {
        error!(?path, "file does not exist");
        return false;
    }

    let path = transmute_extra::pathbuf_into_cstring(path);

    if !is_video_valid(&path) {
        error!(?path, "video is invalid");
        return false;
    }

    true
}

fn is_video_valid(path: &CStr) -> bool {
    let format_context = match FormatContext::from_input(path) {
        Ok(context) => context,
        Err(error) => {
            error!(?path, ?error, "failed to open file");
            return false;
        }
    };

    let best_stream = match format_context.find_best_stream(MediaType::Video) {
        Ok(stream) => stream,
        Err(error) => {
            error!(?path, ?error, "failed to find video stream");
            return false;
        }
    };

    let codec_parameters = best_stream.codec_parameters();

    if !matches!(
        codec_parameters.format(),
        Some(video::AudioVideoFormat::Video(VideoPixelFormat::Yuv420p))
    ) {
        error!(
            format = ?codec_parameters.format(),
            "unsupported video pixel format (planar Y'CbCr 4:2:0 is expected)",
        );
        return false;
    }

    true
}
