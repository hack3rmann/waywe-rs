use daemonize::Daemonize;
use std::{
    fs::{self, File},
    io,
};
use thiserror::Error;

pub fn detach() -> Result<(), DetachError> {
    fs::create_dir_all("/tmp/waywe")?;

    let stdout = File::create("/tmp/waywe/daemon-stdout.log")?;
    let stderr = File::create("/tmp/waywe/daemon-stderr.log")?;

    let daemon = Daemonize::new()
        .pid_file("tmp/waywe/daemon.pid")
        .stdout(stdout)
        .stderr(stderr);

    Ok(daemon.start()?)
}

#[derive(Debug, Error)]
pub enum DetachError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Daemonize(#[from] daemonize::Error),
}
