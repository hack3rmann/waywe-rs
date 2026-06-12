use file_format::{FileFormat, Kind};
use image::{DynamicImage, ImageError, ImageReader, RgbImage};
use std::{
    ffi::CStr,
    io,
    path::{Path, PathBuf},
    process::{self, ExitStatus, Stdio},
};
use thiserror::Error;
use tracing::error;
use transmute_extra::pathbuf_into_cstring;
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, MediaType, ScalerFlags, ScalerFormat,
    SoftwareScaler, VideoPixelFormat,
};
use waywe_ipc::{
    DaemonCommand, DaemonSetupResult, WallpaperType,
    detach::{BINCODE_CONFIG, SetupPipe},
    profile::{SetupProfile, SetupProfileError},
};

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("failed to open profile file: {0}")]
    ProfileIo(#[from] SetupProfileError),
    #[error("no wallpaper is running")]
    NoWallpaper,
    #[error("unsupported file format '{0:?}'")]
    UnsupportedFileFormat(Kind),
    #[error(transparent)]
    VideoOpen(#[from] BackendError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error("video '{path}' is invalid")]
    InvalidVideo { path: PathBuf },
    #[error("cargo build failed with status {status}")]
    CargoBuild { status: ExitStatus },
    #[error("'{path}' is not a valid wallpaper crate (no Cargo.toml)")]
    InvalidPackageRoot { path: PathBuf },
    #[error("'{manifest}' is not a dylib wallpaper crate")]
    NotADylibCrate { manifest: PathBuf },
    #[error("built dylib not found at '{path}'")]
    DylibNotFound { path: PathBuf },
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),
}

pub fn execute_current(monitor_name: Option<&str>) -> Result<(), ExecuteError> {
    let mut profile = SetupProfile::read()?;

    let Some(info) = (match monitor_name {
        Some(name) => profile.monitors.remove(name),
        None => profile.monitors.into_values().next(),
    }) else {
        return Err(ExecuteError::NoWallpaper);
    };

    println!("{}", info.path.display());

    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StartWaitMode {
    #[default]
    Wait,
    DontWait,
}

impl StartWaitMode {
    pub const fn daemon_arg(self) -> Option<&'static str> {
        match self {
            StartWaitMode::Wait => Some("--wait"),
            StartWaitMode::DontWait => None,
        }
    }
}

pub fn execute_start(mode: StartWaitMode) -> DaemonSetupResult {
    let fifo = match mode {
        StartWaitMode::Wait => Some(SetupPipe::new("/tmp/waywe")),
        StartWaitMode::DontWait => None,
    };

    let fifo_arg = fifo
        .as_ref()
        .map(|fifo| ["--init-signal-fifo", fifo.path.as_path().to_str().unwrap()])
        .into_iter()
        .flatten();

    let mut child = process::Command::new("waywe-daemon")
        .arg("--run-in-background")
        .args(fifo_arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut fifo_file = fifo.as_ref().map(SetupPipe::read);

    let result: DaemonSetupResult = if let Some(fifo) = &mut fifo_file {
        bincode::decode_from_std_read(fifo, BINCODE_CONFIG).unwrap()
    } else {
        Ok(())
    };

    let _status = child.wait().unwrap();

    result
}

pub fn execute_preview(result_path: &Path, monitor_name: Option<&str>) -> Result<(), ExecuteError> {
    let mut profile = SetupProfile::read()?;

    let Some(info) = (match monitor_name {
        Some(name) => profile.monitors.remove(name),
        None => profile.monitors.into_values().next(),
    }) else {
        return Err(ExecuteError::NoWallpaper);
    };

    let image = match info.wallpaper_type {
        WallpaperType::Scene => todo!("scene"),
        WallpaperType::Video => {
            let c_path = pathbuf_into_cstring(info.path.clone());

            let mut format_context = FormatContext::from_input(&c_path)?;
            let best_stream = format_context.find_best_stream(MediaType::Video)?;
            let best_stream_index = best_stream.index();
            let codec_parameters = best_stream.codec_parameters();

            let Some(decoder) = Codec::find_decoder_for_id(codec_parameters.codec_id()) else {
                return Err(ExecuteError::VideoOpen(BackendError::DECODER_NOT_FOUND));
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
                    Err(error) => return Err(error.into()),
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
            )
            .unwrap();

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
            let reader = ImageReader::open(&info.path)?;
            reader.decode()?
        }
    };

    image.save(result_path)?;

    Ok(())
}

pub fn execute_show(
    path: &Path,
    monitor_name: Option<String>,
) -> Result<DaemonCommand, ExecuteError> {
    let file_kind = FileFormat::from_file(path)?.kind();

    Ok(match file_kind {
        Kind::Image => {
            let reader = ImageReader::open(path)?.with_guessed_format()?;
            let _image = reader.decode()?;
            let absolute_path = path.canonicalize()?;

            DaemonCommand::SetImage {
                path: absolute_path,
                monitor: monitor_name,
            }
        }
        Kind::Video => {
            let absolute_path = path.canonicalize()?;

            if !is_video_path_valid(absolute_path.clone()) {
                return Err(ExecuteError::InvalidVideo {
                    path: absolute_path,
                });
            }

            DaemonCommand::SetVideo {
                path: absolute_path,
                monitor: monitor_name,
            }
        }
        Kind::Compressed => {
            let absolute_path = path.canonicalize()?;

            DaemonCommand::SetScene {
                path: absolute_path,
                monitor: monitor_name,
            }
        }
        _ => return Err(ExecuteError::UnsupportedFileFormat(file_kind)),
    })
}

pub fn execute_pause(monitor_name: Option<String>) -> Result<DaemonCommand, ExecuteError> {
    Ok(DaemonCommand::Pause {
        monitor: monitor_name,
    })
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
