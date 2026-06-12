use bincode::error::EncodeError;
use daemonize::Daemonize;
use std::error::Error;
use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::Path,
    thread,
};
use tap::Pipe;
use thiserror::Error;
use waywe_ipc::{DaemonSetupError, DaemonSetupResult, detach::BINCODE_CONFIG};

pub use daemonize::Error as DaemonizeError;

pub struct DaemonResultPipe {
    file: BufWriter<File>,
}

impl DaemonResultPipe {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        Ok(Self {
            file: File::options().write(true).open(path)?.pipe(BufWriter::new),
        })
    }

    pub fn write(&mut self, result: DaemonSetupResult) -> Result<(), EncodeError> {
        bincode::encode_into_std_write(result, &mut self.file, BINCODE_CONFIG)?;
        Ok(())
    }
}

impl Drop for DaemonResultPipe {
    fn drop(&mut self) {
        if !thread::panicking() {
            return;
        }

        let message = fs::read_to_string("/tmp/waywe/daemon-stderr.log").unwrap_or_default();

        _ = self.write(Err(DaemonSetupError::Panicked { message }));
    }
}

pub struct DaemonSetupReporter {
    pipe: Option<DaemonResultPipe>,
}

impl DaemonSetupReporter {
    pub fn open(path: Option<impl AsRef<Path>>) -> Result<Self, io::Error> {
        let Some(path) = path else {
            return Ok(Self { pipe: None });
        };

        Ok(Self {
            pipe: Some(DaemonResultPipe::open(path)?),
        })
    }

    pub fn report(&mut self, result: DaemonSetupResult) {
        let Some(mut pipe) = self.pipe.take() else {
            return;
        };
        _ = pipe.write(result);
    }
}

pub fn detach() -> Result<(), DetachError> {
    fs::create_dir_all("/tmp/waywe")?;

    let stdout = File::create("/tmp/waywe/daemon-stdout.log")?;
    let stderr = File::create("/tmp/waywe/daemon-stderr.log")?;

    Daemonize::default()
        .pid_file("/tmp/waywe/daemon.pid")
        .stdout(stdout)
        .stderr(stderr)
        .start()?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum DetachError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Daemonize(#[from] DaemonizeError),
}

impl Report for DetachError {
    fn to_setup_error(&self) -> Option<DaemonSetupError> {
        Some(match self {
            DetachError::Daemonize(DaemonizeError::LockPidfile(errno)) => {
                DaemonSetupError::DaemonPidLock {
                    info: errno.to_string(),
                }
            }
            _ => return None,
        })
    }
}

pub trait Report {
    fn to_setup_error(&self) -> Option<DaemonSetupError>;
}

pub trait UnwrapOrReport {
    type Output;

    fn unwrap_or_report(self, reporter: &mut DaemonSetupReporter) -> Self::Output;
}

impl<T, E: Report + Error> UnwrapOrReport for Result<T, E> {
    type Output = T;

    #[track_caller]
    fn unwrap_or_report(self, reporter: &mut DaemonSetupReporter) -> Self::Output {
        match self {
            Ok(output) => output,
            Err(error) => match error.to_setup_error() {
                Some(setup) => {
                    reporter.report(Err(setup));
                    panic!("called `.unwrap_or_report` on reported Err: {error}")
                }
                None => panic!("called `.unwrap_or_report` on unreportable Err: {error}"),
            },
        }
    }
}
