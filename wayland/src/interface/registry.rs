//! The singleton global registry object.  The server has a number of
//! global objects that are available to all clients.  These objects
//! typically represent an actual object in the server (for example,
//! an input device) or they are singleton objects that provide
//! extension functionality.
//!
//! When a client creates a registry object, the registry object
//! will emit a global event for each global currently in the
//! registry.  Globals come and go as a result of device or
//! monitor hotplugs, reconfiguration or other events, and the
//! registry will send out global and global_remove events to
//! keep the client up to date with the changes.  To mark the end
//! of the initial burst of events, the client can use the
//! wl_display.sync request immediately after calling
//! wl_display.get_registry.
//!
//! A client can bind to a global object by using the bind
//! request.  This creates a client-side handle that lets the object
//! emit events to the client and lets the client invoke requests on
//! the object.

use crate::{
    interface::Event,
    object::WlObjectId,
    sys::wire::{WlMessage, MessageBuffer},
};

pub mod request {
    use wayland_sys::wl_proxy_marshal_array_constructor;

    use super::*;
    use crate::{object::{HasObjectType, InterfaceMessageArgument, WlObjectType}, sys::{
        object::{dispatch::State, registry::WlRegistry, WlObject},
        proxy::WlProxy,
        wire::OpCode,
    }};
    use std::{marker::PhantomData, ptr::NonNull};

    /// Binds a new, client-created object to the server using the
    /// specified name as the identifier.
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Bind<T: HasObjectType> {
        _p: PhantomData<T>,
    }

    impl<T: HasObjectType> Bind<T> {
        pub const fn new() -> Self {
            Self { _p: PhantomData }
        }
    }

    impl<T: HasObjectType> Default for Bind<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T: HasObjectType> Bind<T> {
        const OPCODE: OpCode = 0;
        const OUTGOING_INTERFACE: WlObjectType = T::OBJECT_TYPE;

        fn build_message(self, buf: &mut impl MessageBuffer, name: WlObjectId) -> WlMessage<'_> {
            WlMessage::builder(buf)
                .opcode(Self::OPCODE)
                .interface(InterfaceMessageArgument {
                    object_type: T::OBJECT_TYPE,
                    name,
                })
                .build()
        }

        /// # Safety
        ///
        /// - `parent` proxy should match the parent interface
        /// - resulting `WlProxy` object should be owned by `ObjectStorage` after call
        pub unsafe fn send<S: State>(
            self,
            registry: &WlObject<WlRegistry<S>>,
            buf: &mut impl MessageBuffer,
        ) -> Option<WlProxy> {
            let message = self.build_message(buf, registry.name_of(T::OBJECT_TYPE)?);
            let interface = &raw const *Self::OUTGOING_INTERFACE.backend_interface();

            let raw_proxy = unsafe {
                wl_proxy_marshal_array_constructor(
                    registry.proxy().as_raw().as_ptr(),
                    message.opcode.into(),
                    message.arguments.as_ptr().cast_mut(),
                    interface,
                )
            };

            Some(unsafe { WlProxy::from_raw(NonNull::new(raw_proxy)?) })
        }
    }
}

pub mod event {
    use super::*;
    use crate::sys::wire::OpCode;
    use std::ffi::CStr;

    /// Notify the client of global objects.
    ///
    /// The event notifies the client that a global object with
    /// the given name is now available, and it implements the
    /// given version of the given interface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Global<'s> {
        /// Numeric name of the global object
        pub name: WlObjectId,
        /// Interface implemented by the object
        pub interface: &'s CStr,
        /// Interface version
        pub version: u32,
    }

    impl<'s> Event<'s> for Global<'s> {
        const CODE: OpCode = 0;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();

            let name = WlObjectId::try_from(unsafe { reader.read::<u32>()? }).ok()?;
            let interface = unsafe { reader.read::<&CStr>()? };
            let version = unsafe { reader.read::<u32>()? };

            Some(Self {
                name,
                interface,
                version,
            })
        }
    }

    /// Notify the client of removed global objects.
    ///
    /// This event notifies the client that the global identified
    /// by name is no longer available.  If the client bound to
    /// the global using the bind request, the client should now
    /// destroy that object.
    ///
    /// The object remains valid and requests to the object will be
    /// ignored until the client destroys it, to avoid races between
    /// the global going away and a client sending a request to it.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GlobalRemove {
        /// Numeric name of the global object
        pub name: WlObjectId,
    }

    impl<'s> Event<'s> for GlobalRemove {
        const CODE: OpCode = 1;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();
            let name = WlObjectId::try_from(unsafe { reader.read::<u32>()? }).ok()?;

            Some(Self { name })
        }
    }
}
