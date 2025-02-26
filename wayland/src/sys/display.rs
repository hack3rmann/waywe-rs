use super::ffi::{wl_display, wl_display_connect_to_fd};
use std::{
    os::fd::{IntoRawFd, OwnedFd},
    ptr::NonNull,
};

pub struct WlDisplay {
    pub ptr: NonNull<wl_display>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Self {
        // FIXME(hack3rmann): deal with errors
        let display =
            NonNull::new(unsafe { wl_display_connect_to_fd(wayland_file_desc.into_raw_fd()) })
                .expect("failed to connect wl_display");

        Self { ptr: display }
    }
}
