//! A singleton global registry object.  The server has a number of
//! global objects that are available to all clients.  These objects
//! typically represent an actual object on the server (for example,
//! an input device) or they are singleton objects that provide
//! extension functionality.
//!
//! When a client creates a registry object, the registry object
//! will emit a global event for each global currently in the
//! registry.  Globals come and go as a result of device or
//! monitor hotplugs, reconfiguration or other events, and the
//! registry will send out `global` and `global_remove` events to
//! keep the client up to date with the changes.  To mark the end
//! of the initial burst of events, the client can use the
//! `wl_display.sync`request immediately after calling
//! `wl_display.get_registry`.
//!
//! A client can bind to a global object by using the bind
//! request.  This creates a client-side handle that lets the object
//! emit events to the client and lets the client invoke requests on
//! the object.

use crate::{
    object::WlObjectId,
    sys::wire::{WlMessage, WlMessageBuffer},
};

/// Requests for `wl_registry` interface
pub mod request {
    use super::*;
    use crate::{
        ffi,
        object::{HasObjectType, InterfaceMessageArgument, WlObjectType},
        sys::{
            object::{WlObject, dispatch::State, registry::WlRegistry},
            proxy::WlProxy,
            wire::OpCode,
        },
    };
    use std::{marker::PhantomData, ptr::NonNull};

    /// Binds a new, client-created object to the server using the
    /// specified name as the identifier.
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Bind<T: HasObjectType> {
        _p: PhantomData<T>,
    }

    impl<T: HasObjectType> Bind<T> {
        /// Constructs new bind request
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

        fn build_message(self, buf: &mut impl WlMessageBuffer, name: WlObjectId) -> WlMessage<'_> {
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
            buf: &mut impl WlMessageBuffer,
        ) -> Option<WlProxy> {
            let message = self.build_message(buf, registry.name_of(T::OBJECT_TYPE)?);
            let interface = &raw const *Self::OUTGOING_INTERFACE.backend_interface();

            let raw_proxy = unsafe {
                ffi::wl_proxy_marshal_array_constructor(
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

/// Events for `wl_registry` interface
pub mod event {
    pub use super::super::generated::wayland::registry::event::*;
}
