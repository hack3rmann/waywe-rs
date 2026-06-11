use bincode::{Decode, Encode, error::DecodeError};
use daemonize::{Daemonize, Outcome};
use std::{
    fs::{self, File},
    io::{self, Write},
    process, thread,
};
use thiserror::Error;

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Encode, Decode, Error)]
pub enum DaemonSetupError {
    #[error("waywe-daemon panicked")]
    Panicked,
    #[error("failed to daemonize waywe-daemon process")]
    DaemonizeFailed,
}

pub type DaemonSetupResult = Result<(), DaemonSetupError>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Copy)]
pub enum DetachMode {
    #[default]
    Wait,
    DontWait,
}

/// Call `.signal(true)` after the daemon has finished initializing.
pub struct ReadyChannel(File);

impl ReadyChannel {
    pub const fn new(pipe: File) -> Self {
        Self(pipe)
    }

    pub fn signal(mut self, result: DaemonSetupResult) {
        self.write(result);
        // Drop closes the write end so the parent does not hang.
    }

    pub fn write(&mut self, result: DaemonSetupResult) {
        let bytes = bincode::encode_to_vec(result, BINCODE_CONFIG).unwrap();
        _ = self.0.write_all(&bytes);
    }
}

impl Drop for ReadyChannel {
    fn drop(&mut self) {
        if !thread::panicking() {
            return;
        }

        self.write(Err(DaemonSetupError::Panicked));
    }
}

fn daemon_start_wait(daemon: Daemonize<()>) -> Result<Option<ReadyChannel>, DetachError> {
    let (mut parent_read, child_write) = pipe()?;

    match daemon.execute() {
        Outcome::Parent(result) => {
            drop(child_write);

            match result {
                Ok(parent) if parent.first_child_exit_code == 0 => {}
                Ok(parent) => process::exit(parent.first_child_exit_code),
                Err(error) => return Err(error.into()),
            }

            let result: DaemonSetupResult =
                match bincode::decode_from_std_read(&mut parent_read, BINCODE_CONFIG) {
                    Ok(res) => res,
                    Err(DecodeError::Io {
                        inner,
                        additional: _,
                    }) => return Err(DetachError::Io(inner)),
                    Err(_) => unreachable!(),
                };

            result?;
            process::exit(0);
        }
        Outcome::Child(result) => {
            drop(parent_read);

            let channel = ReadyChannel::new(child_write);

            match result {
                Ok(_) => Ok(Some(channel)),
                Err(error) => {
                    channel.signal(Err(DaemonSetupError::DaemonizeFailed));
                    Err(error.into())
                }
            }
        }
    }
}

pub fn detach(mode: DetachMode) -> Result<Option<ReadyChannel>, DetachError> {
    fs::create_dir_all("/tmp/waywe")?;

    let stdout = File::create("/tmp/waywe/daemon-stdout.log")?;
    let stderr = File::create("/tmp/waywe/daemon-stderr.log")?;

    let daemon = Daemonize::new()
        .pid_file("/tmp/waywe/daemon.pid")
        .stdout(stdout)
        .stderr(stderr);

    match mode {
        DetachMode::DontWait => {
            daemon.start()?;
            Ok(None)
        }
        DetachMode::Wait => daemon_start_wait(daemon),
    }
}

fn pipe() -> io::Result<(File, File)> {
    let (read, write) = rustix::pipe::pipe()?;
    Ok((File::from(read), File::from(write)))
}

#[derive(Debug, Error)]
pub enum DetachError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Daemonize(#[from] daemonize::Error),
    #[error("daemon failed during initialization: {0}")]
    DaemonSetup(#[from] DaemonSetupError),
}
