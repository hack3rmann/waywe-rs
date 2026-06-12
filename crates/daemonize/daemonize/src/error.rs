use rustix::io::Errno;
use thiserror::Error;

/// This error type for `Daemonize` `start` method.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum Error {
    #[error("unable to fork: {0}")]
    Fork(Errno),
    #[error("wait failed: {0}")]
    Wait(Errno),
    #[error("unable to create new session: {0}")]
    DetachSession(Errno),
    #[error("unable to resolve group name to group id")]
    GroupNotFound,
    #[error("group option contains NUL")]
    GroupContainsNul,
    #[error("unable to set group: {0}")]
    SetGroup(Errno),
    #[error("unable to resolve user name to user id")]
    UserNotFound,
    #[error("user option contains NUL")]
    UserContainsNul,
    #[error("unable to set user: {0}")]
    SetUser(Errno),
    #[error("unable to change directory: {0}")]
    ChangeDirectory(Errno),
    #[error("pid_file option contains NUL")]
    PathContainsNul,
    #[error("unable to open pid file: {0}")]
    OpenPidfile(Errno),
    #[error("unable get pid file flags: {0}")]
    GetPidfileFlags(Errno),
    #[error("unable set pid file flags: {0}")]
    SetPidfileFlags(Errno),
    #[error("unable to lock pid file: {0}")]
    LockPidfile(Errno),
    #[error("unable to chown pid file: {0}")]
    ChownPidfile(Errno),
    #[error("unable to open /dev/null: {0}")]
    OpenDevnull(Errno),
    #[error("unable to redirect standard streams to /dev/null: {0}")]
    RedirectStreams(Errno),
    #[error("unable to close /dev/null: {0}")]
    CloseDevnull(Errno),
    #[error("unable to truncate pid file: {0}")]
    TruncatePidfile(Errno),
    #[error("unable to write self pid to pid file: {0}")]
    WritePid(Errno),
    #[error("unable to write self pid to pid file due to unknown reason")]
    WritePidUnspecifiedError,
    #[error("unable to chroot into directory")]
    Chroot(Errno),
}

impl Error {
    pub fn errno(self) -> Option<Errno> {
        match self {
            Error::Fork(errno)
            | Error::Wait(errno)
            | Error::DetachSession(errno)
            | Error::SetGroup(errno)
            | Error::SetUser(errno)
            | Error::ChangeDirectory(errno)
            | Error::OpenPidfile(errno)
            | Error::GetPidfileFlags(errno)
            | Error::SetPidfileFlags(errno)
            | Error::LockPidfile(errno)
            | Error::ChownPidfile(errno)
            | Error::OpenDevnull(errno)
            | Error::RedirectStreams(errno)
            | Error::CloseDevnull(errno)
            | Error::TruncatePidfile(errno)
            | Error::WritePid(errno)
            | Error::Chroot(errno) => Some(errno),
            Error::GroupNotFound
            | Error::WritePidUnspecifiedError
            | Error::PathContainsNul
            | Error::UserNotFound
            | Error::UserContainsNul
            | Error::GroupContainsNul => None,
        }
    }
}

pub trait Num {
    fn is_err(&self) -> bool;
}

impl Num for i8 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i16 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i32 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for i64 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

impl Num for isize {
    fn is_err(&self) -> bool {
        *self == -1
    }
}

pub fn check_err<N: Num, F: FnOnce(Errno) -> Error>(ret: N, f: F) -> Result<N, Error> {
    if ret.is_err() {
        Err(f(errno()))
    } else {
        Ok(ret)
    }
}

pub fn errno() -> Errno {
    let value = std::io::Error::last_os_error()
        .raw_os_error()
        .expect("errno");

    Errno::from_raw_os_error(value)
}
