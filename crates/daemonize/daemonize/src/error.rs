use rustix::io::Errno;

/// This error type for `Daemonize` `start` method.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    Fork(Errno),
    Wait(Errno),
    DetachSession(Errno),
    GroupNotFound,
    GroupContainsNul,
    SetGroup(Errno),
    UserNotFound,
    UserContainsNul,
    SetUser(Errno),
    ChangeDirectory(Errno),
    PathContainsNul,
    OpenPidfile(Errno),
    GetPidfileFlags(Errno),
    SetPidfileFlags(Errno),
    LockPidfile(Errno),
    ChownPidfile(Errno),
    OpenDevnull(Errno),
    RedirectStreams(Errno),
    CloseDevnull(Errno),
    TruncatePidfile(Errno),
    WritePid(Errno),
    WritePidUnspecifiedError,
    Chroot(Errno),
}

impl Error {
    fn description(&self) -> &str {
        match self {
            Error::Fork(_) => "unable to fork",
            Error::Wait(_) => "wait failed",
            Error::DetachSession(_) => "unable to create new session",
            Error::GroupNotFound => "unable to resolve group name to group id",
            Error::GroupContainsNul => "group option contains NUL",
            Error::SetGroup(_) => "unable to set group",
            Error::UserNotFound => "unable to resolve user name to user id",
            Error::UserContainsNul => "user option contains NUL",
            Error::SetUser(_) => "unable to set user",
            Error::ChangeDirectory(_) => "unable to change directory",
            Error::PathContainsNul => "pid_file option contains NUL",
            Error::OpenPidfile(_) => "unable to open pid file",
            Error::GetPidfileFlags(_) => "unable get pid file flags",
            Error::SetPidfileFlags(_) => "unable set pid file flags",
            Error::LockPidfile(_) => "unable to lock pid file",
            Error::ChownPidfile(_) => "unable to chown pid file",
            Error::OpenDevnull(_) => "unable to open /dev/null",
            Error::RedirectStreams(_) => "unable to redirect standard streams to /dev/null",
            Error::CloseDevnull(_) => "unable to close /dev/null",
            Error::TruncatePidfile(_) => "unable to truncate pid file",
            Error::WritePid(_) => "unable to write self pid to pid file",
            Error::WritePidUnspecifiedError => {
                "unable to write self pid to pid file due to unknown reason"
            }
            Error::Chroot(_) => "unable to chroot into directory",
        }
    }

    fn errno(&self) -> Option<Errno> {
        match self {
            Error::Fork(errno) => Some(*errno),
            Error::Wait(errno) => Some(*errno),
            Error::DetachSession(errno) => Some(*errno),
            Error::GroupNotFound => None,
            Error::GroupContainsNul => None,
            Error::SetGroup(errno) => Some(*errno),
            Error::UserNotFound => None,
            Error::UserContainsNul => None,
            Error::SetUser(errno) => Some(*errno),
            Error::ChangeDirectory(errno) => Some(*errno),
            Error::PathContainsNul => None,
            Error::OpenPidfile(errno) => Some(*errno),
            Error::GetPidfileFlags(errno) => Some(*errno),
            Error::SetPidfileFlags(errno) => Some(*errno),
            Error::LockPidfile(errno) => Some(*errno),
            Error::ChownPidfile(errno) => Some(*errno),
            Error::OpenDevnull(errno) => Some(*errno),
            Error::RedirectStreams(errno) => Some(*errno),
            Error::CloseDevnull(errno) => Some(*errno),
            Error::TruncatePidfile(errno) => Some(*errno),
            Error::WritePid(errno) => Some(*errno),
            Error::WritePidUnspecifiedError => None,
            Error::Chroot(errno) => Some(*errno),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())?;
        if let Some(errno) = self.errno() {
            write!(f, ", errno {}", errno)?
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

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
