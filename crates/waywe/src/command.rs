use file_format::{FileFormat, Kind};
use miette::Diagnostic;
use rustix::{
    io::Errno,
    process::{Pid, Signal, kill_process},
};
use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    path::{Path, PathBuf},
    process::{self, ExitStatus, Stdio},
    string::FromUtf8Error,
};
use thiserror::Error;
use waywe_ipc::{
    ClientError, DaemonCommand, DaemonSetupResult, IpcClient, WallpaperType,
    command::{DaemonError, DaemonResponse, DaemonResult, PauseMode},
    detach::{BINCODE_CONFIG, SetupPipe},
    profile::{SetupProfile, SetupProfileError},
};

#[derive(Debug, Error, Diagnostic)]
pub enum ExecuteError {
    #[error("failed to open profile file: {0}")]
    ProfileIo(#[from] SetupProfileError),
    #[error("no wallpaper is running")]
    NoWallpaper,
    #[error("unsupported file format '{0:?}'")]
    UnsupportedFileFormat(Kind),
    #[error(transparent)]
    Io(#[from] io::Error),
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
    #[error(transparent)]
    ConnectDaemon(#[from] ConnectDaemonError),
    #[error(transparent)]
    Ipc(#[from] ClientError),
    #[error("unexpected daemon response {0:#?}")]
    #[diagnostic(
        code(waywe::unexpected_daemon_response),
        help(
            "this is a bug, please create a GitHub issue: https://github.com/hack3rmann/waywe-rs/issues/new"
        )
    )]
    UnexpectedDaemonResponse(DaemonResponse),
    #[error("daemon returned an error")]
    #[diagnostic(code(waywe::daemon::response_error))]
    DaemonError(#[from] DaemonError),
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
    pub const fn from_dont(dont_wait: bool) -> Self {
        if dont_wait {
            Self::DontWait
        } else {
            Self::Wait
        }
    }

    pub const fn daemon_arg(self) -> Option<&'static str> {
        match self {
            WaitMode::Wait => Some("--wait"),
            WaitMode::DontWait => None,
        }
    }
}

pub fn execute_pause(
    monitor: Option<String>,
    mode: PauseMode,
    wait_mode: WaitMode,
) -> Result<(), ExecuteError> {
    let socket = connect_daemon()?;

    socket.send(DaemonCommand::Pause { monitor, mode })?;

    if wait_mode == WaitMode::DontWait {
        return Ok(());
    }

    let response = socket.recv()??;

    if response != DaemonResponse::PauseDone {
        return Err(ExecuteError::UnexpectedDaemonResponse(response));
    }

    Ok(())
}

pub fn execute_start(mode: WaitMode) -> DaemonSetupResult {
    let fifo = match mode {
        WaitMode::Wait => Some(SetupPipe::new_in("/tmp/waywe")),
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
) -> Result<(), ExecuteError> {
    let command = DaemonCommand::Preview {
        ty: WallpaperType::Scene,
        path: source.to_owned(),
        width,
        height,
    };

    let socket = connect_daemon()?;

    socket.send(command)?;
    let response = socket.recv()??;

    let DaemonResponse::Preview {
        width,
        height,
        rgba,
    } = response
    else {
        return Err(ExecuteError::UnexpectedDaemonResponse(response));
    };

    tracing::debug!(?width, ?height, ?rgba, "preview");

    Ok(())
}

pub type DaemonSocket = IpcClient<DaemonCommand, DaemonResult>;

#[derive(Error, Debug, Diagnostic)]
pub enum ConnectDaemonError {
    #[error("no waywe-daemon is running")]
    #[diagnostic(
        code(waywe::daemon::not_running),
        help("start the daemon first: `waywe start`")
    )]
    NotRunning(#[source] Errno),
    #[error("unexpected OS error")]
    #[diagnostic(code(waywe::daemon::connect_failed))]
    OtherOs(#[from] Errno),
}

pub fn connect_daemon() -> Result<DaemonSocket, ConnectDaemonError> {
    DaemonSocket::connect().map_err(|errno| match errno {
        Errno::CONNREFUSED | Errno::NOENT => ConnectDaemonError::NotRunning(errno),
        _ => ConnectDaemonError::OtherOs(errno),
    })
}

pub fn execute_show(
    path: &Path,
    monitor_name: Option<String>,
    wait_mode: WaitMode,
) -> Result<(), ExecuteError> {
    let file_kind = FileFormat::from_file(path)?.kind();
    let absolute_path = path.canonicalize()?;

    let command = match file_kind {
        Kind::Image => DaemonCommand::Show {
            path: absolute_path,
            monitor: monitor_name,
            ty: WallpaperType::Image,
        },
        Kind::Video => DaemonCommand::Show {
            path: absolute_path,
            monitor: monitor_name,
            ty: WallpaperType::Video,
        },
        Kind::Compressed => DaemonCommand::Show {
            path: absolute_path,
            monitor: monitor_name,
            ty: WallpaperType::Scene,
        },
        _ => return Err(ExecuteError::UnsupportedFileFormat(file_kind)),
    };

    let socket = connect_daemon()?;

    socket.send(command)?;

    if wait_mode == WaitMode::DontWait {
        return Ok(());
    }

    let response = socket.recv()??;

    if response != DaemonResponse::WallpaperSet {
        return Err(ExecuteError::UnexpectedDaemonResponse(response));
    }

    Ok(())
}
