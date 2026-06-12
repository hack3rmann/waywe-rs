use miette::Diagnostic;
use thiserror::Error;
use waywe_ipc::DaemonSetupError;

#[derive(Debug, Error, Diagnostic)]
pub enum DaemonSetupDiagnostics {
    #[error("waywe-daemon panicked:\n{message}")]
    #[diagnostic(
        code(waywe::daemon::panicked),
        help(
            "this is a bug, please create a GitHub issue: https://github.com/hack3rmann/waywe-rs/issues/new"
        )
    )]
    Panicked { message: String },
    #[error("waywe-daemon is running already")]
    #[diagnostic(code(waywe::daemon::already_running), help("stop the daemon first"))]
    DaemonAlreadyRunning {
        #[source]
        error: DaemonSetupError,
    },
}

impl From<DaemonSetupError> for DaemonSetupDiagnostics {
    fn from(error: DaemonSetupError) -> Self {
        match error {
            DaemonSetupError::Panicked { message } => Self::Panicked { message },
            error @ DaemonSetupError::DaemonPidLock { .. } => Self::DaemonAlreadyRunning { error },
        }
    }
}
