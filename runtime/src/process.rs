use rustix::{io::Errno, process};
use std::{ffi::CStr, process::abort, ptr};

#[deprecated = "not yet stable"]
#[allow(dead_code)]
fn spawn_detached(name: &CStr) -> Result<(), Errno> {
    let mut pid = unsafe { libc::fork() };

    if pid != 0 {
        return Ok(());
    } else if pid == -1 {
        let errno = unsafe { libc::__errno_location().read() };
        return Err(Errno::from_raw_os_error(errno));
    }

    pid = unsafe { libc::fork() };

    if pid != 0 {
        unsafe { libc::_exit(0) };
    } else if pid == -1 {
        abort();
    }

    let args = [ptr::null()];
    let env = [ptr::null()];

    let _session_id = process::setsid()?;

    let _minus_one = unsafe { libc::execve(name.as_ptr(), args.as_ptr(), env.as_ptr()) };
    let error = Errno::from_raw_os_error(unsafe { libc::__errno_location().read() });

    tracing::error!(?error, "execve failed");
    abort();
}
