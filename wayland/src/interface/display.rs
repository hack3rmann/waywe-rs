//! The core global object. This is a special singleton object. It
//! is used for internal Wayland protocol features.

use crate::{
    interface::{Event, Request},
    sys::wire::{Message, MessageBuffer},
};

pub mod request {
    use super::*;
    use crate::sys::{InterfaceObjectType, wire::OpCode};

    /// The sync request asks the server to emit the 'done' event
    /// on the returned wl_callback object.  Since requests are
    /// handled in-order and events are delivered in-order, this can
    /// be used as a barrier to ensure all previous requests and the
    /// resulting events have been handled.
    ///
    /// The object returned by this request will be destroyed by the
    /// compositor after the callback is fired and as such the client must not
    /// attempt to use it after that point.
    ///
    /// The callback_data passed in the callback is undefined and should be ignored.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Sync;

    impl<'b> Request<'b> for Sync {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<InterfaceObjectType> = Some(InterfaceObjectType::Callback);

        fn build_message(self, buf: &'b mut impl MessageBuffer) -> Message<'b> {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }

    /// This request creates a registry object that allows the client
    /// to list and bind the global objects available from the
    /// compositor.
    ///
    /// It should be noted that the server side resources consumed in
    /// response to a get_registry request can only be released when the
    /// client disconnects, not when the client side proxy is destroyed.
    /// Therefore, clients should invoke get_registry as infrequently as
    /// possible to avoid wasting memory.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GetRegistry;

    impl<'b> Request<'b> for GetRegistry {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<InterfaceObjectType> = Some(InterfaceObjectType::Registry);

        fn build_message(self, buf: &'b mut impl MessageBuffer) -> Message<'b> {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }
}

pub mod event {
    use super::*;
    use crate::sys::{proxy::WlProxyQuery, wire::OpCode};
    use std::ffi::CStr;

    /// The error event is sent out when a fatal (non-recoverable)
    /// error has occurred.  The object_id argument is the object
    /// where the error occurred, most often in response to a request
    /// to that object.  The code identifies the error and is defined
    /// by the object interface.  As such, each interface defines its
    /// own set of error codes.  The message is a brief description
    /// of the error, for (debugging) convenience.
    pub struct Error<'s> {
        /// Object where the error occurred
        pub object: WlProxyQuery,
        /// Error code
        pub code: u32,
        /// Error description
        pub message: &'s CStr,
    }

    impl<'s> Event<'s> for Error<'s> {
        const CODE: OpCode = 0;

        fn from_message(message: Message<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();

            // Safety: event provided by libwayland matches our interface
            // and opcode therefore it must have the arguments below
            let object = unsafe { reader.read::<WlProxyQuery>()? };
            let code = unsafe { reader.read::<u32>()? };
            let message = unsafe { reader.read::<&CStr>()? };

            Some(Self {
                object,
                code,
                message,
            })
        }
    }

    /// This event is used internally by the object ID management
    /// logic. When a client deletes an object that it had created,
    /// the server will send this event to acknowledge that it has
    /// seen the delete request. When the client receives this event,
    /// it will know that it can safely reuse the object ID.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct DeleteId {
        id: u32,
    }

    impl<'s> Event<'s> for DeleteId {
        const CODE: OpCode = 1;

        fn from_message(message: Message<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();

            let id = unsafe { reader.read::<u32>()? };

            Some(Self { id })
        }
    }
}

pub mod wl_enum {
    /// These errors are global and can be emitted in response to any server request
    #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub enum Error {
        /// Server couldn't find object
        InvalidObject = 0,
        /// Method doesn't exist on the specified interface or malformed request
        InvalidMethod = 1,
        /// Server is out of memory
        NoMemory = 2,
        /// Implementation error in compositor
        Implementation = 3,
    }
}
