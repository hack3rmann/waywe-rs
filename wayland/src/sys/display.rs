use super::{
    ffi::{
        wl_argument, wl_display, wl_display_connect_to_fd, wl_display_disconnect, wl_message,
        wl_proxy_add_dispatcher,
    },
    proxy::WlProxy,
    wire::MessageBuffer,
};
use crate::interface::{Request, WlDisplayGetRegistryRequest};
use std::{
    ffi::{CStr, c_int, c_void},
    mem::ManuallyDrop,
    os::fd::{IntoRawFd, OwnedFd},
    ptr::{self, NonNull},
};

unsafe extern "C" fn registry_dispatcher(
    _data: *const c_void,
    _proxy: *mut c_void,
    _opcode: u32,
    message: *const wl_message,
    _arguments: *mut wl_argument,
) -> c_int {
    let name = unsafe { CStr::from_ptr((*message).name) };
    let signature = unsafe { CStr::from_ptr((*message).signature) };
    dbg!(name, signature);
    42
}

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

    pub fn create_registry(&self, buf: &mut impl MessageBuffer) -> WlProxy {
        let raw = NonNull::new(unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) })
            .unwrap();

        unsafe {
            wl_proxy_add_dispatcher(
                raw.as_ptr(),
                registry_dispatcher,
                ptr::null(),
                ptr::null_mut(),
            )
        };

        unsafe { WlProxy::from_raw(raw) }
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        let display = self.proxy.as_raw().cast::<wl_display>().as_ptr();
        unsafe { wl_display_disconnect(display) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        init::connect_wayland_socket,
        sys::{ffi::wl_display_roundtrip, wire::SmallVecMessageBuffer},
    };

    #[test]
    fn get_registry() {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };
        let display = WlDisplay::connect_to_fd(wayland_sock);
        let mut buf = SmallVecMessageBuffer::<8>::new();
        let registry = display.create_registry(&mut buf);

        unsafe { wl_display_roundtrip(display.as_raw_display_ptr().as_ptr()) };

        assert_eq!(registry.interface_name(), "wl_registry");
    }
}
