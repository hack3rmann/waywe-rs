use std::{
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use tracing::error;

pub static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
pub const HANDLED_SIGNALS: [i32; 4] = [libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGHUP];

extern "C" fn signal_handler(_signal_number: i32) {
    SHOULD_EXIT.store(true, Ordering::Relaxed);
}

pub fn setup() {
    let sa_mask = {
        let mut uninit = MaybeUninit::<libc::sigset_t>::zeroed();
        unsafe { libc::sigemptyset(uninit.as_mut_ptr()) };
        unsafe { uninit.assume_init() }
    };

    let action = libc::sigaction {
        sa_sigaction: signal_handler as usize,
        sa_mask,
        sa_flags: 0,
        sa_restorer: None,
    };

    for signal in HANDLED_SIGNALS {
        if unsafe { libc::sigaction(signal, &raw const action, ptr::null_mut()) } != 0 {
            error!(?signal, "failed to setup signal handler");
        }
    }
}
