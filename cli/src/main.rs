use clap::{Parser, Subcommand};
use runtime::{DaemonCommand, IpcSocket, ipc::Client};
use std::{
    ffi::{CStr, CString},
    io,
    path::Path,
    process::ExitCode,
};
use tracing::error;
use video::{FormatContext, MediaType, VideoPixelFormat};

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    video::init();

    let Command::Video { path } = Args::parse().command;

    let absolute_path = match absolutize_path(&path) {
        Ok(path) => path,
        Err(error) => {
            error!(?error, "failed to construct absolute path");
            return ExitCode::FAILURE;
        }
    };

    if !is_path_valid(&absolute_path) {
        error!(?absolute_path, "can not send video to the daemon");
        return ExitCode::FAILURE;
    }

    let socket = IpcSocket::<Client, DaemonCommand>::connect().unwrap();
    socket
        .send(DaemonCommand::SetVideo {
            path: absolute_path,
        })
        .unwrap();

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
    Video {
        /// Path/URL to the video
        path: CString,
    },
}

fn is_path_valid(path: &CStr) -> bool {
    if !path_from(path).exists() {
        error!(?path, "file does not exist");
        return false;
    }

    if !is_video_valid(path) {
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
            "unsupported video format (planar Y'CbCr 4:2:0 is expected)",
        );
        return false;
    }

    true
}

fn absolutize_path(path: &CStr) -> Result<CString, io::Error> {
    let path_str = std::str::from_utf8(path.to_bytes()).unwrap();
    let path_value = Path::new(path_str);

    if !path_value.is_relative() {
        return Ok(path.to_owned());
    }

    let full_path = path_value.canonicalize()?;
    let mut full_path_bytes = full_path.into_os_string().into_encoded_bytes();
    full_path_bytes.push(0);
    Ok(CString::from_vec_with_nul(full_path_bytes).unwrap())
}

fn path_from(path: &CStr) -> &Path {
    let path_str = std::str::from_utf8(path.to_bytes()).unwrap();
    Path::new(path_str)
}
