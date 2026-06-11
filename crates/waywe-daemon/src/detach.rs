use daemonize::{Daemonize, Outcome};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    process,
};
use thiserror::Error;

const READY: u8 = 0;
const FAILED: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Copy)]
pub enum DetachMode {
    #[default]
    Wait,
    DontWait,
}

/// Call `.signal(true)` after the daemon has finished initializing.
pub struct ReadyChannel(File);

impl ReadyChannel {
    pub fn signal(mut self, ok: bool) {
        let byte = if ok { READY } else { FAILED };
        _ = self.0.write_all(&[byte]);
        // Drop closes the write end so the parent does not hang.
    }
}

fn daemon_start_wait(daemon: Daemonize<()>) -> Result<Option<ReadyChannel>, DetachError> {
    let (mut parent_read, mut child_write) = pipe()?;

    match daemon.execute() {
        Outcome::Parent(result) => {
            drop(child_write);

            match result {
                Ok(parent) if parent.first_child_exit_code == 0 => {}
                Ok(parent) => process::exit(parent.first_child_exit_code),
                Err(error) => return Err(error.into()),
            }

            let mut buf = [0u8; 1];
            parent_read.read_exact(&mut buf)?;

            if buf[0] == READY {
                process::exit(0);
            }

            Err(DetachError::DaemonNotReady)
        }
        Outcome::Child(result) => {
            drop(parent_read);

            match result {
                Ok(_) => Ok(Some(ReadyChannel(child_write))),
                Err(error) => {
                    _ = child_write.write_all(&[FAILED]);
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
    #[error("daemon failed during initialization")]
    DaemonNotReady,
}
