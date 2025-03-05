use super::{
    ffi::{wl_display, wl_display_connect_to_fd, wl_display_disconnect},
    object::{WlObject, WlObjectHandle, registry::WlRegistry},
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::MessageBuffer,
};
use crate::{
    interface::{Request, WlDisplayGetRegistryRequest},
    sys::ffi::wl_display_roundtrip,
};
use std::{mem::ManuallyDrop, os::fd::{IntoRawFd, OwnedFd}, ptr::NonNull};

/// A handle to libwayland backend
pub struct WlDisplay {
    pub proxy: ManuallyDrop<WlProxy>,
    pub storage: ManuallyDrop<WlObjectStorage>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Self {
        // FIXME(hack3rmann): deal with errors
        let display =
            NonNull::new(unsafe { wl_display_connect_to_fd(wayland_file_desc.into_raw_fd()) })
                .expect("failed to connect wl_display");

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        // Safety: `storage` is dropped before `wl_display` disconnects
        // see `Drop` impl for `WlDisplay`
        Self {
            proxy,
            storage: ManuallyDrop::new(unsafe { WlObjectStorage::new() }),
        }
    }

    pub fn as_raw_display_ptr(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }

    pub fn create_registry(&mut self, buf: &mut impl MessageBuffer) -> WlObjectHandle<WlRegistry> {
        // Safety: parent interface matcher request's one
        let raw_proxy = unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) };

        // Safety: resulting proxy is a valid object provided by libwayland
        let proxy = unsafe { WlProxy::from_raw(NonNull::new(raw_proxy).unwrap()) };

        let proxy_id = proxy.id();

        self.storage
            .insert(WlObject::new(proxy, WlRegistry::default()));

        WlObjectHandle::new(proxy_id)
    }

    pub fn dispatch_all(&self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        assert_ne!(-1, unsafe {
            wl_display_roundtrip(self.as_raw_display_ptr().as_ptr())
        });
    }

    pub fn storage(&self) -> &WlObjectStorage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut WlObjectStorage {
        &mut self.storage
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        // Safety: ensure the storage is dropped before display disconnects
        unsafe { ManuallyDrop::drop(&mut self.storage) };

        // Safety: `self.as_raw_display_ptr()` is a valid display object
        unsafe { wl_display_disconnect(self.as_raw_display_ptr().as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init::connect_wayland_socket, sys::wire::SmallVecMessageBuffer};

    #[test]
    fn get_registry() {
        // Safety: called once on the start of the program
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };

        let mut buf = SmallVecMessageBuffer::<8>::new();

        let mut display = WlDisplay::connect_to_fd(wayland_sock);
        let registry = display.create_registry(&mut buf);

        display.dispatch_all();

        assert!(
            display
                .storage()
                .object(registry)
                .interfaces
                .contains_key(c"wl_compositor")
        );
    }
}
