use super::{
    ffi::{wl_display, wl_display_connect_to_fd, wl_display_disconnect},
    proxy::WlProxy,
};
use std::{
    mem::ManuallyDrop,
    os::fd::{IntoRawFd, OwnedFd},
    ptr::NonNull,
};

/// A handle to libwayland backend
pub struct WlDisplay {
    pub proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Self {
        // FIXME(hack3rmann): deal with errors
        let display =
            NonNull::new(unsafe { wl_display_connect_to_fd(wayland_file_desc.into_raw_fd()) })
                .expect("failed to connect wl_display");

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        Self { proxy }
    }

    pub fn as_raw_display_ptr(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        let display = self.proxy.as_raw().cast::<wl_display>().as_ptr();
        unsafe { wl_display_disconnect(display) };
    }
}
