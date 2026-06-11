use bincode::{Decode, Encode};
use thiserror::Error;

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Clone, Encode, Decode, Error)]
pub enum DaemonSetupError {
    #[error("waywe-daemon panicked")]
    Panicked { message: String },
    #[error("failed to daemonize waywe-daemon process")]
    DaemonizeFailed,
    #[error("failed to acquire daemon file lock")]
    DaemonFileLock,
    #[error("failed to acquire daemon pid file lock")]
    DaemonPidLock,
}

pub type DaemonSetupResult = Result<(), DaemonSetupError>;

pub const EXIT_CODE_PANIC: i32 = 101;
pub const EXIT_CODE_DAEMON_SETUP: i32 = 169;
