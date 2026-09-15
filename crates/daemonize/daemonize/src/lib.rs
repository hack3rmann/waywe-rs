// Copyright (c) 2016 Fedor Gogolev <knsd@knsd.net>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//!
//! daemonize is a library for writing system daemons. Inspired by the Python library [thesharp/daemonize](https://github.com/thesharp/daemonize).
//!
//! The respository is located at <https://github.com/knsd/daemonize/>.
//!
//! Usage example:
//!
//! ```
//! extern crate daemonize;
//!
//! use std::fs::File;
//!
//! use daemonize::Daemonize;
//!
//! fn main() {
//!     let stdout = File::create("/tmp/daemon.out").unwrap();
//!     let stderr = File::create("/tmp/daemon.err").unwrap();
//!
//!     let daemonize = Daemonize::default()
//!         .pid_file("/tmp/test.pid") // Every method except `new` and `start`
//!         .chown_pid_file(true)      // is optional, see `Daemonize` documentation
//!         .working_directory("/tmp") // for default behaviour.
//!         .user("nobody")
//!         .group("daemon") // Group name
//!         .group(2)        // or group id.
//!         .umask(0o777)    // Set umask, `0o027` by default.
//!         .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
//!         .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.
//!
//!     match daemonize.start() {
//!         Ok(_) => println!("Success, daemonized"),
//!         Err(e) => eprintln!("Error, {}", e),
//!     }
//! }
//! ```

mod error;

use crate::error::{check_err, errno};
use rustix::process::Pid;
use std::{
    env::set_current_dir,
    ffi::{CStr, CString},
    fs::File,
    os::unix::{ffi::OsStringExt, io::AsRawFd},
    path::PathBuf,
    process::{self, exit},
};

pub use crate::error::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum UserImpl {
    Name(String),
    Id(libc::uid_t),
}

/// Expects system user id or name. If name is provided it will be resolved to id later.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct User {
    inner: UserImpl,
}

impl From<&str> for User {
    fn from(t: &str) -> Self {
        Self::from(t.to_owned())
    }
}

impl From<String> for User {
    fn from(value: String) -> Self {
        Self {
            inner: UserImpl::Name(value),
        }
    }
}

impl From<u32> for User {
    fn from(t: u32) -> Self {
        Self {
            inner: UserImpl::Id(t as libc::uid_t),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum GroupImpl {
    Name(String),
    Id(libc::gid_t),
}

/// Expects system group id or name. If name is provided it will be resolved to id later.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Group {
    inner: GroupImpl,
}

impl From<&str> for Group {
    fn from(t: &str) -> Self {
        Self::from(t.to_owned())
    }
}

impl From<String> for Group {
    fn from(value: String) -> Self {
        Self {
            inner: GroupImpl::Name(value),
        }
    }
}

impl From<u32> for Group {
    fn from(t: u32) -> Group {
        Group {
            inner: GroupImpl::Id(t as libc::gid_t),
        }
    }
}

/// File mode creation mask.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Mask {
    inner: libc::mode_t,
}

impl From<u32> for Mask {
    fn from(inner: u32) -> Mask {
        Mask {
            inner: inner as libc::mode_t,
        }
    }
}

#[derive(Debug)]
enum StdioImpl {
    Devnull,
    RedirectToFile(File),
    Keep,
}

/// Describes what to do with a standard I/O stream for a child process.
#[derive(Debug)]
pub struct Stdio {
    inner: StdioImpl,
}

impl Stdio {
    pub fn devnull() -> Self {
        Self {
            inner: StdioImpl::Devnull,
        }
    }

    pub fn keep() -> Self {
        Self {
            inner: StdioImpl::Keep,
        }
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Self {
        Self {
            inner: StdioImpl::RedirectToFile(file),
        }
    }
}

/// Parent process execution outcome.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Parent {
    pub first_child_exit_code: i32,
}

/// Child process execution outcome.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Child;

/// Daemonization process outcome. Can be matched to check is it a parent process or a child
/// process.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Parent(Result<Parent, Error>),
    Child(Result<Child, Error>),
}

impl Outcome {
    pub fn is_parent(&self) -> bool {
        match self {
            Outcome::Parent(_) => true,
            Outcome::Child(_) => false,
        }
    }

    pub fn is_child(&self) -> bool {
        match self {
            Outcome::Parent(_) => false,
            Outcome::Child(_) => true,
        }
    }
}

/// Daemonization options.
///
/// Fork the process in the background, disassociate from its process group and the control terminal.
/// Change umask value to `0o027`, redirect all standard streams to `/dev/null`. Change working
/// directory to `/` or provided value.
///
/// Optionally:
///
///   * maintain and lock the pid-file;
///   * drop user privileges;
///   * drop group privileges;
///   * change root directory;
///   * change the pid-file ownership to provided user (and/or) group;
///
#[derive(Debug)]
pub struct Daemonize {
    directory: PathBuf,
    pid_file: Option<PathBuf>,
    chown_pid_file: bool,
    user: Option<User>,
    group: Option<Group>,
    umask: Mask,
    root: Option<PathBuf>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl Default for Daemonize {
    fn default() -> Self {
        Daemonize {
            directory: PathBuf::from("/"),
            pid_file: None,
            chown_pid_file: false,
            user: None,
            group: None,
            umask: 0o027.into(),
            root: None,
            stdin: Stdio::devnull(),
            stdout: Stdio::devnull(),
            stderr: Stdio::devnull(),
        }
    }
}

impl Daemonize {
    /// Create pid-file at `path`, lock it exclusive and write daemon pid.
    pub fn pid_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.pid_file = Some(path.into());
        self
    }

    /// If `chown` is true, daemonize will change the pid-file ownership, if user or group are provided
    pub fn chown_pid_file(mut self, chown: bool) -> Self {
        self.chown_pid_file = chown;
        self
    }

    /// Change working directory to `path` or `/` by default.
    pub fn working_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.directory = path.into();
        self
    }

    /// Drop privileges to `user`.
    pub fn user<U: Into<User>>(mut self, user: U) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Drop privileges to `group`.
    pub fn group<G: Into<Group>>(mut self, group: G) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Change umask to `mask` or `0o027` by default.
    pub fn umask<M: Into<Mask>>(mut self, mask: M) -> Self {
        self.umask = mask.into();
        self
    }

    /// Change root to `path`
    pub fn chroot(mut self, path: impl Into<PathBuf>) -> Self {
        self.root = Some(path.into());
        self
    }

    /// Configuration for the child process's standard output stream.
    pub fn stdout(mut self, stdio: impl Into<Stdio>) -> Self {
        self.stdout = stdio.into();
        self
    }

    /// Configuration for the child process's standard error stream.
    pub fn stderr(mut self, stdio: impl Into<Stdio>) -> Self {
        self.stderr = stdio.into();
        self
    }

    /// Start daemonization process, terminate parent after first fork, returns privileged action
    /// result to the child.
    pub fn start(self) -> Result<(), Error> {
        match self.execute() {
            Outcome::Parent(Ok(Parent {
                first_child_exit_code,
            })) => process::exit(first_child_exit_code),
            Outcome::Parent(Err(err)) => Err(err),
            Outcome::Child(Ok(_child)) => Ok(()),
            Outcome::Child(Err(err)) => Err(err),
        }
    }

    /// Execute daemonization process, don't terminate parent after first fork.
    pub fn execute(self) -> Outcome {
        match perform_fork() {
            Ok(Some(first_child_pid)) => Outcome::Parent(
                unsafe { waitpid(first_child_pid.as_raw_pid()) }.map(|code| Parent {
                    first_child_exit_code: code,
                }),
            ),
            Err(err) => Outcome::Parent(Err(err)),
            Ok(None) => match self.execute_child() {
                Ok(()) => Outcome::Child(Ok(Child)),
                Err(err) => Outcome::Child(Err(err)),
            },
        }
    }

    fn execute_child(self) -> Result<(), Error> {
        unsafe {
            set_current_dir(&self.directory).map_err(|_| Error::ChangeDirectory(errno()))?;
            set_sid()?;
            libc::umask(self.umask.inner);

            if perform_fork()?.is_some() {
                exit(0)
            };

            let pid_file_fd = self
                .pid_file
                .clone()
                .map(|pid_file| create_pid_file(pid_file))
                .transpose()?;

            redirect_standard_streams(self.stdin, self.stdout, self.stderr)?;

            let uid = self.user.map(|user| get_user(user)).transpose()?;
            let gid = self.group.map(|group| get_group(group)).transpose()?;

            if self.chown_pid_file {
                let args: Option<(PathBuf, libc::uid_t, libc::gid_t)> =
                    match (self.pid_file, uid, gid) {
                        (Some(pid), Some(uid), Some(gid)) => Some((pid, uid, gid)),
                        (Some(pid), None, Some(gid)) => Some((pid, libc::uid_t::MAX - 1, gid)),
                        (Some(pid), Some(uid), None) => Some((pid, uid, libc::gid_t::MAX - 1)),
                        // Or pid file is not provided, or both user and group
                        _ => None,
                    };

                if let Some((pid, uid, gid)) = args {
                    chown_pid_file(pid, uid, gid)?;
                }
            }

            if let Some(pid_file_fd) = pid_file_fd {
                set_cloexec_pid_file(pid_file_fd)?;
            }

            if let Some(root) = &self.root {
                rustix::process::chroot(root).map_err(Error::Chroot)?
            }

            if let Some(gid) = gid {
                set_group(gid)?;
            }

            if let Some(uid) = uid {
                set_user(uid)?;
            }

            if let Some(pid_file_fd) = pid_file_fd {
                write_pid_file(pid_file_fd)?;
            }

            Ok(())
        }
    }
}

fn perform_fork() -> Result<Option<Pid>, Error> {
    match syscalls::fork() {
        Ok(pid) => Ok(Pid::from_raw(pid)),
        Err(errno) => Err(Error::Fork(errno)),
    }
}

unsafe fn waitpid(pid: libc::pid_t) -> Result<libc::c_int, Error> {
    let mut child_ret = 0;
    check_err(
        unsafe { libc::waitpid(pid, &mut child_ret, 0) },
        Error::Wait,
    )?;
    Ok(child_ret)
}

unsafe fn set_sid() -> Result<(), Error> {
    check_err(unsafe { libc::setsid() }, Error::DetachSession)?;
    Ok(())
}

unsafe fn redirect_standard_streams(
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
) -> Result<(), Error> {
    let devnull_fd = check_err(
        unsafe { libc::open(b"/dev/null\0" as *const [u8; 10] as _, libc::O_RDWR) },
        Error::OpenDevnull,
    )?;

    let process_stdio = |fd, stdio: Stdio| {
        match stdio.inner {
            StdioImpl::Devnull => {
                check_err(
                    unsafe { libc::dup2(devnull_fd, fd) },
                    Error::RedirectStreams,
                )?;
            }
            StdioImpl::RedirectToFile(file) => {
                let raw_fd = file.as_raw_fd();
                check_err(unsafe { libc::dup2(raw_fd, fd) }, Error::RedirectStreams)?;
            }
            StdioImpl::Keep => (),
        };
        Ok(())
    };

    process_stdio(libc::STDIN_FILENO, stdin)?;
    process_stdio(libc::STDOUT_FILENO, stdout)?;
    process_stdio(libc::STDERR_FILENO, stderr)?;

    check_err(unsafe { libc::close(devnull_fd) }, Error::CloseDevnull)?;

    Ok(())
}

unsafe fn get_group(group: Group) -> Result<libc::gid_t, Error> {
    match group.inner {
        GroupImpl::Id(id) => Ok(id),
        GroupImpl::Name(name) => {
            let s = CString::new(name).map_err(|_| Error::GroupContainsNul)?;
            match unsafe { get_gid_by_name(&s) } {
                Some(id) => unsafe { get_group(id.into()) },
                None => Err(Error::GroupNotFound),
            }
        }
    }
}

unsafe fn set_group(group: libc::gid_t) -> Result<(), Error> {
    check_err(unsafe { libc::setgid(group) }, Error::SetGroup)?;
    Ok(())
}

unsafe fn get_user(user: User) -> Result<libc::uid_t, Error> {
    match user.inner {
        UserImpl::Id(id) => Ok(id),
        UserImpl::Name(name) => {
            let s = CString::new(name).map_err(|_| Error::UserContainsNul)?;
            match unsafe { get_uid_by_name(&s) } {
                Some(id) => unsafe { get_user(id.into()) },
                None => Err(Error::UserNotFound),
            }
        }
    }
}

unsafe fn set_user(user: libc::uid_t) -> Result<(), Error> {
    check_err(unsafe { libc::setuid(user) }, Error::SetUser)?;
    Ok(())
}

unsafe fn create_pid_file(path: PathBuf) -> Result<libc::c_int, Error> {
    let path_c = pathbuf_into_cstring(path)?;

    let fd = check_err(
        unsafe { libc::open(path_c.as_ptr(), libc::O_WRONLY | libc::O_CREAT, 0o666) },
        Error::OpenPidfile,
    )?;

    check_err(
        unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) },
        Error::LockPidfile,
    )?;
    Ok(fd)
}

unsafe fn chown_pid_file(path: PathBuf, uid: libc::uid_t, gid: libc::gid_t) -> Result<(), Error> {
    let path_c = pathbuf_into_cstring(path)?;
    check_err(
        unsafe { libc::chown(path_c.as_ptr(), uid, gid) },
        Error::ChownPidfile,
    )?;
    Ok(())
}

unsafe fn write_pid_file(fd: libc::c_int) -> Result<(), Error> {
    let pid = unsafe { libc::getpid() };
    let pid_buf = format!("{}\n", pid).into_bytes();
    let pid_length = pid_buf.len();
    let pid_c = CString::new(pid_buf).unwrap();
    check_err(unsafe { libc::ftruncate(fd, 0) }, Error::TruncatePidfile)?;

    let written = check_err(
        unsafe { libc::write(fd, pid_c.as_ptr() as *const libc::c_void, pid_length) },
        Error::WritePid,
    )?;

    if written < pid_length as isize {
        return Err(Error::WritePidUnspecifiedError);
    }

    Ok(())
}

unsafe fn set_cloexec_pid_file(fd: libc::c_int) -> Result<(), Error> {
    if cfg!(not(target_os = "redox")) {
        let flags = check_err(
            unsafe { libc::fcntl(fd, libc::F_GETFD) },
            Error::GetPidfileFlags,
        )?;

        check_err(
            unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) },
            Error::SetPidfileFlags,
        )?;
    } else {
        check_err(
            unsafe { libc::ioctl(fd, libc::FIOCLEX) },
            Error::SetPidfileFlags,
        )?;
    }
    Ok(())
}

unsafe fn get_gid_by_name(name: &CStr) -> Option<libc::gid_t> {
    let ptr = unsafe { libc::getgrnam(name.as_ptr() as *const libc::c_char) };
    if ptr.is_null() {
        None
    } else {
        let s = unsafe { ptr.as_ref().unwrap_unchecked() };
        Some(s.gr_gid)
    }
}

unsafe fn get_uid_by_name(name: &CStr) -> Option<libc::uid_t> {
    let ptr = unsafe { libc::getpwnam(name.as_ptr() as *const libc::c_char) };
    if ptr.is_null() {
        None
    } else {
        let s = unsafe { ptr.as_ref().unwrap_unchecked() };
        Some(s.pw_uid)
    }
}

fn pathbuf_into_cstring(path: PathBuf) -> Result<CString, Error> {
    CString::new(path.into_os_string().into_vec()).map_err(|_| Error::PathContainsNul)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod syscalls {
    use rustix::io::Errno;
    use std::arch::asm;

    pub fn fork() -> Result<libc::pid_t, Errno> {
        let ret: i64;

        unsafe {
            asm!(
                "syscall",
                in("rax") libc::SYS_fork as u64,
                lateout("rax") ret,
                lateout("rcx") _,
                lateout("r11") _,
                options(nostack),
            );
        }

        if (-4095..0).contains(&ret) {
            Err(Errno::from_raw_os_error(ret as i32))
        } else {
            Ok(ret as libc::pid_t)
        }
    }
}
