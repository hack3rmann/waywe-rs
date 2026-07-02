use file_format::{FileFormat, Kind};
use image::{ImageError, ImageReader};
use miette::Diagnostic;
use rustix::{
    io::Errno,
    process::{Pid, Signal, kill_process},
};
use std::{
    ffi::CStr,
    fs::File,
    io::{self, ErrorKind, Read},
    path::{Path, PathBuf},
    process::{self, ExitStatus, Stdio},
    string::FromUtf8Error,
};
use thiserror::Error;
use tracing::error;
use video::{BackendError, FormatContext, MediaType, VideoPixelFormat};
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
pub enum WaitMode {
    #[default]
    Wait,
    DontWait,
}

impl WaitMode {
    pub const fn daemon_arg(self) -> Option<&'static str> {
        match self {
            WaitMode::Wait => Some("--wait"),
            WaitMode::DontWait => None,
        }
    }
}

pub fn execute_start(mode: WaitMode) -> DaemonSetupResult {
    let fifo = match mode {
        WaitMode::Wait => Some(SetupPipe::new("/tmp/waywe")),
        WaitMode::DontWait => None,
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

#[derive(Debug, Error, Diagnostic)]
pub enum ExecuteStopError {
    #[error("failed to open daemon pid file")]
    OpenPidFile(#[source] io::Error),
    #[error("failed to read daemon pid file")]
    ReadPidFile(#[source] io::Error),
    #[error("pid file contains invalid characters")]
    #[diagnostic(
        code(waywe::corrupted_state),
        help("maybe the daemon state is corruped")
    )]
    InvalidPidCharacters(#[from] FromUtf8Error),
    #[error("invalid PID '{0}'")]
    #[diagnostic(
        code(waywe::corrupted_state),
        help("maybe the daemon state is corruped")
    )]
    InvalidPid(String),
    #[error("failed to lock PID file")]
    LockPidFile(#[source] io::Error),
}

pub fn execute_stop(mode: WaitMode) -> Result<(), ExecuteStopError> {
    let mut pid_file = match File::open("/tmp/waywe/daemon.pid") {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(ExecuteStopError::OpenPidFile(error)),
    };

    let mut pid_bytes = Vec::with_capacity(64);
    pid_file
        .read_to_end(&mut pid_bytes)
        .map_err(ExecuteStopError::ReadPidFile)?;

    let pid_str = String::from_utf8(pid_bytes)?;
    let pid_str = pid_str.trim();
    let pid_raw = pid_str
        .parse::<i32>()
        .map_err(|_| ExecuteStopError::InvalidPid(pid_str.to_owned()))?;
    let pid =
        Pid::from_raw(pid_raw).ok_or_else(|| ExecuteStopError::InvalidPid(pid_str.to_owned()))?;

    match kill_process(pid, Signal::TERM) {
        Ok(()) => {}
        // no such process
        Err(Errno::SRCH) => return Ok(()),
        Err(errno) => panic!("{errno}"),
    }

    if mode == WaitMode::DontWait {
        return Ok(());
    }

    pid_file.lock().map_err(ExecuteStopError::LockPidFile)?;
    pid_file.unlock().map_err(ExecuteStopError::LockPidFile)?;

    Ok(())
}

pub fn execute_preview(
    _output: &Path,
    source: &Path,
    width: u32,
    height: u32,
) -> Result<DaemonCommand, ExecuteError> {
    Ok(DaemonCommand::Preview {
        ty: WallpaperType::Scene,
        path: source.to_owned(),
        width,
        height,
    })
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

            DaemonCommand::Show {
                path: absolute_path,
                monitor: monitor_name,
                ty: WallpaperType::Image,
            }
        }
        Kind::Video => {
            let absolute_path = path.canonicalize()?;

            if !is_video_path_valid(absolute_path.clone()) {
                return Err(ExecuteError::InvalidVideo {
                    path: absolute_path,
                });
            }

            DaemonCommand::Show {
                path: absolute_path,
                monitor: monitor_name,
                ty: WallpaperType::Video,
            }
        }
        Kind::Compressed => {
            let absolute_path = path.canonicalize()?;

            DaemonCommand::Show {
                path: absolute_path,
                monitor: monitor_name,
                ty: WallpaperType::Scene,
            }
        }
        _ => return Err(ExecuteError::UnsupportedFileFormat(file_kind)),
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
