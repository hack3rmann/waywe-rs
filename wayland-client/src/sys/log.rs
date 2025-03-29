use std::{
    cell::UnsafeCell,
    ffi::{CStr, c_char, c_int},
    slice, str,
};
use va_list::VaList;
use wayland_sys::wl_log_set_handler_client;

pub(crate) const MAX_LOG_MESSAGE_LEN: usize = 256;

thread_local! {
    pub(crate) static BUFFER: UnsafeCell<[u8; MAX_LOG_MESSAGE_LEN]>
        = const { UnsafeCell::new([0; MAX_LOG_MESSAGE_LEN]) };
}

/// # Safety
///
/// - `format` should be a valid format c-string corresponding to `args` values
/// - `args` are valid
pub(crate) unsafe extern "C" fn wl_log_raw(format: *const c_char, args: VaList) {
    let buffer_ptr = BUFFER.with(UnsafeCell::get).cast::<c_char>();

    // # Safety
    //
    // - `buffer_ptr` points to a valid buffer of `MAX_LOG_MESSAGE_LEN` bytes
    // - `format` is a valid format c-string corresponding to `args` values
    // - `args` are valid
    let result = unsafe { vsnprintf(buffer_ptr, MAX_LOG_MESSAGE_LEN, format, args) };

    let bytes = if result > 0 {
        if result as usize + 1 == MAX_LOG_MESSAGE_LEN {
            tracing::error!(MAX_LOG_MESSAGE_LEN, "error message is too large");
        }

        // Safety: if `vsnprintf` returns number greater than 0, then it has
        // wrote this number of bytes into the buffer, therefore `result`
        // is the slice's length.
        unsafe { slice::from_raw_parts(buffer_ptr.cast::<u8>(), result as usize) }
    } else {
        // Safety: `format` is a valid c-string
        unsafe { CStr::from_ptr(format) }.to_bytes()
    };

    // Safety: wayland log messages are valid UTF-8
    let raw_message = unsafe { str::from_utf8_unchecked(bytes) };

    let message = trim_last_linebreak(raw_message);

    tracing::error!("{message}");
}

fn trim_last_linebreak(source: &str) -> &str {
    if source.ends_with(['\n', '\r']) {
        // Safety: both '\n' and '\r' are ASCII characters
        // the string without any of them at the end will have `len = souce.len() - 1`
        unsafe { source.get_unchecked(..source.len() - 1) }
    } else if source.ends_with("\r\n") {
        // Safety: both '\n' and '\r' are ASCII characters
        // the string without both of then at the end will have `len = souce.len() - 2`
        unsafe { source.get_unchecked(..source.len() - 2) }
    } else {
        source
    }
}

/// Setup wayland client logger
pub(crate) fn setup() {
    unsafe { wl_log_set_handler_client(wl_log_raw) };
}

// NOTE(hack3rmann): the crate `libc` does not provide this function se we do.
#[link(name = "c")]
unsafe extern "C" {
    pub(crate) fn vsnprintf(
        buffer: *mut c_char,
        max_len: usize,
        format: *const c_char,
        args: VaList,
    ) -> c_int;
}
