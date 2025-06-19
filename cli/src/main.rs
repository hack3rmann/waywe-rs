use clap::{Parser, Subcommand};
use image::{DynamicImage, ImageReader, RgbImage};
use runtime::{
    DaemonCommand, IpcSocket, WallpaperType,
    ipc::Client,
    profile::{SetupProfile, SetupProfileError},
};
use rustix::io::Errno;
use std::{
    ffi::CStr,
    io::ErrorKind,
    path::PathBuf,
    process::{ExitCode, Stdio},
};
use tracing::error;
use transmute_extra::pathbuf_into_cstring;
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, MediaType, ScalerFlags, ScalerFormat, SoftwareScaler, VideoPixelFormat
};

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    video::init();

    let daemon_command = match Args::parse().command {
        Command::Preview { out } => {
            let profile = match SetupProfile::read() {
                Ok(profile) => profile,
                Err(error) => {
                    error!(?error, "failed to open profile file");
                    return ExitCode::FAILURE;
                }
            };

            let image = match profile.wallpaper_type {
                WallpaperType::Video => {
                    let c_path = pathbuf_into_cstring(profile.path.clone().into_owned());

                    let mut format_context = match FormatContext::from_input(&c_path) {
                        Ok(context) => context,
                        Err(error) => {
                            error!(?error, path = ?profile.path, "failed to open video");
                            return ExitCode::FAILURE;
                        }
                    };

                    let best_stream = match format_context.find_best_stream(MediaType::Video) {
                        Ok(stream) => stream,
                        Err(error) => {
                            error!(?error, path = ?profile.path, "failed to find video stream");
                            return ExitCode::FAILURE;
                        }
                    };

                    let best_stream_index = best_stream.index();
                    let codec_parameters = best_stream.codec_parameters();

                    let Some(decoder) = Codec::find_decoder_for_id(codec_parameters.codec_id())
                    else {
                        error!(path = ?profile.path, "failed to find decoder");
                        return ExitCode::FAILURE;
                    };

                    let mut codec_context =
                        CodecContext::from_parameters(codec_parameters, Some(decoder)).unwrap();
                    codec_context.open(decoder).unwrap();

                    let frame = loop {
                        let packet = format_context.read_any_packet().unwrap();

                        if packet.stream_index() != best_stream_index {
                            continue;
                        }

                        codec_context.send_packet(&packet).unwrap();

                        let mut frame = Frame::new();

                        match codec_context.receive_frame(&mut frame) {
                            Ok(()) => break frame,
                            Err(BackendError::EAGAIN) => continue,
                            Err(error) => {
                                error!(?error, "failed to receive a frame");
                                return ExitCode::FAILURE;
                            }
                        }
                    };

                    let mut scaler = SoftwareScaler::new(
                        ScalerFormat {
                            size: frame.size(),
                            format: frame.format().unwrap(),
                        },
                        ScalerFormat {
                            // TODO(hack3rmann): scale appropriately
                            size: frame.size(),
                            format: VideoPixelFormat::Rgb24,
                        },
                        ScalerFlags::BILINEAR,
                    ).unwrap();

                    let mut scaled_frame = Frame::new();
                    scaler.run(&frame, &mut scaled_frame).unwrap();

                    let image = RgbImage::from_vec(
                        scaled_frame.width(),
                        scaled_frame.height(),
                        scaled_frame.data(0).to_owned(),
                    )
                    .expect("buffer size expected to be `width * height * pixel_size`");

                    DynamicImage::ImageRgb8(image)
                }
                WallpaperType::Image => {
                    let reader = match ImageReader::open(&profile.path) {
                        Ok(reader) => reader,
                        Err(error) => {
                            error!(?error, path = ?profile.path, "failed to open image file");
                            return ExitCode::FAILURE;
                        }
                    };

                    match reader.decode() {
                        Ok(image) => image,
                        Err(error) => {
                            error!(?error, path = ?profile.path, "failed to decode image");
                            return ExitCode::FAILURE;
                        }
                    }
                }
            };

            if let Err(error) = image.save(&out) {
                error!(?error, ?out, "failed to save preview image");
                return ExitCode::FAILURE;
            }

            return ExitCode::SUCCESS;
        }
        Command::Current => {
            let profile = match SetupProfile::read() {
                Ok(profile) => profile,
                Err(SetupProfileError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                    return ExitCode::FAILURE;
                }
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
    /// Create a preview for the wallpaper
    Preview {
        /// Where to store the preview
        out: PathBuf,
    },
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
