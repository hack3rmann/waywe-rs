use crate::{
    interface::{Event, Request, WlDisplayGetRegistryRequest, WlRegistryGlobalEvent},
    object::ObjectId,
    sys::ffi::wl_display_roundtrip,
};

use super::{
    ffi::{wl_display, wl_display_connect_to_fd, wl_display_disconnect},
    object::{Dispatch, WlObject, WlObjectHandle},
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::{Message, MessageBuffer},
};
use std::{collections::HashMap, ffi::CString, mem::ManuallyDrop, os::fd::IntoRawFd, ptr::NonNull};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: ObjectId,
    pub version: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WlRegistry {
    pub interfaces: HashMap<CString, WlRegistryGlobalInfo>,
}

impl Dispatch for WlRegistry {
    fn dispatch(&mut self, message: Message<'_>) {
        let Some(event) = WlRegistryGlobalEvent::from_message(message) else {
            return;
        };

        self.interfaces.insert(
            event.interface.to_owned(),
            WlRegistryGlobalInfo {
                name: event.name,
                version: event.version,
            },
        );
    }
}

/// A handle to libwayland backend
pub struct WlDisplay {
    pub proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: impl IntoRawFd) -> Self {
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

    pub fn create_registry(
        &self,
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
    ) -> WlObjectHandle<WlRegistry> {
        let raw_proxy = unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) };
        let proxy = unsafe { WlProxy::from_raw(NonNull::new(raw_proxy).unwrap()) };
        let proxy_id = proxy.id();

        storage.insert(WlObject::new(proxy, WlRegistry::default()));

        WlObjectHandle::new(proxy_id)
    }

    pub fn dispatch_all(&self, _storage: &mut WlObjectStorage) {
        // NOTE(hack3rmann): by requireing `&mut WlObjectStorage` we safely
        // capture all objects mutably therefore no object is borrowed outside
        // the dispatcher

        assert_ne!(-1, unsafe {
            wl_display_roundtrip(self.as_raw_display_ptr().as_ptr())
        });
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
    use crate::{init::connect_wayland_socket, sys::wire::SmallVecMessageBuffer};

    use super::*;

    #[test]
    fn get_registry() {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };

        let mut buf = SmallVecMessageBuffer::<8>::new();
        let mut storage = WlObjectStorage::default();

        let display = WlDisplay::connect_to_fd(wayland_sock);
        let registry = display.create_registry(&mut buf, &mut storage);

        display.dispatch_all(&mut storage);

        dbg!(storage.object(registry));
    }
}
