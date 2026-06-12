use bincode::error::EncodeError;
use daemonize::Daemonize;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    thread,
};
use tap::Pipe;
use thiserror::Error;
use waywe_ipc::{DaemonSetupError, DaemonSetupResult, detach::BINCODE_CONFIG};

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
        _ = self.file.flush();
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
    Daemonize(#[from] daemonize::Error),
}
