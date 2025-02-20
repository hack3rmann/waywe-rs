//! The core global object. This is a special singleton object. It
//! is used for internal Wayland protocol features.

use crate::wayland::{
    interface::Event,
    object::ObjectId,
    wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};

pub mod request {
    use crate::wayland::wire::MessageBuildResult;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Sync {
        /// Callback object for the sync request
        pub callback: ObjectId,
    }

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
    pub fn sync(req: Sync, buf: &mut MessageBuffer) -> MessageBuildResult {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 0,
            })
            .uint(req.callback.into())
            .build()
    }

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GetRegistry {
        /// Global registry object
        pub registry: ObjectId,
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
    pub fn get_registry(req: GetRegistry, buf: &mut MessageBuffer) -> MessageBuildResult {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 1,
            })
            .uint(req.registry.into())
            .build()
    }
}

pub mod event {
    use super::*;

    /// The error event is sent out when a fatal (non-recoverable)
    /// error has occurred.  The object_id argument is the object
    /// where the error occurred, most often in response to a request
    /// to that object.  The code identifies the error and is defined
    /// by the object interface.  As such, each interface defines its
    /// own set of error codes.  The message is a brief description
    /// of the error, for (debugging) convenience.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Error<'s> {
        /// Object where the error occurred
        pub object: ObjectId,
        /// Error code
        pub code: u32,
        /// Error description
        pub message: &'s str,
    }

    impl<'s> Event<'s> for Error<'s> {
        fn header_desc() -> Option<MessageHeaderDesc> {
            Some(MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 0,
            })
        }

        fn from_message(message: &'s Message) -> Option<Self> {
            if !message
                .header()
                .corresponds_to(&Self::header_desc().unwrap())
            {
                return None;
            }

            let mut reader = message.reader();

            let object = reader.read_u32()?;
            let code = reader.read_u32()?;
            let message = reader.read_str()?;

            Some(Self {
                object: ObjectId::new(object),
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
        /// Deleted object id
        pub id: ObjectId,
    }

    impl<'s> Event<'s> for DeleteId {
        fn header_desc() -> Option<MessageHeaderDesc> {
            Some(MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 1,
            })
        }

        fn from_message(message: &'s Message) -> Option<Self> {
            if !message
                .header()
                .corresponds_to(&Self::header_desc().unwrap())
            {
                return None;
            }

            let mut reader = message.reader();
            let id = reader.read_u32().unwrap();

            Some(Self {
                id: ObjectId::new(id),
            })
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
