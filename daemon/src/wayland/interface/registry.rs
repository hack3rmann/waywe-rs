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

use crate::wayland::{
    interface::{Event, Request},
    object::ObjectId,
    wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};

pub mod request {
    use super::*;

    /// Binds a new, client-created object to the server using the
    /// specified name as the identifier.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Bind {
        /// Unique numeric name of the object
        pub name: ObjectId,
        /// Bounded object
        pub new_id: ObjectId,
    }

    impl Request for Bind {
        fn header_desc() -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: ObjectId::WL_REGISTRY,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<&Message, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc())
                .uint(self.name.into())
                .uint(self.new_id.into())
                .build()
        }
    }
}

pub mod event {
    use super::*;

    /// Notify the client of global objects.
    ///
    /// The event notifies the client that a global object with
    /// the given name is now available, and it implements the
    /// given version of the given interface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Global<'s> {
        /// Numeric name of the global object
        pub name: ObjectId,
        /// Interface implemented by the object
        pub interface: &'s str,
        /// Interface version
        pub version: u32,
    }

    impl<'s> Event<'s> for Global<'s> {
        fn header_desc() -> Option<MessageHeaderDesc> {
            Some(MessageHeaderDesc {
                object_id: ObjectId::WL_REGISTRY,
                opcode: 0,
            })
        }

        fn from_message(message: &'s Message) -> Option<Self> {
            let mut reader = message.reader();

            let name = reader.read_u32()?;
            let interface = reader.read_str()?;
            let version = reader.read_u32()?;

            Some(Self {
                name: ObjectId::new(name),
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
        pub name: ObjectId,
    }

    impl<'s> Event<'s> for GlobalRemove {
        fn header_desc() -> Option<MessageHeaderDesc> {
            Some(MessageHeaderDesc {
                object_id: ObjectId::WL_REGISTRY,
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
            let name = reader.read_u32().unwrap();

            Some(Self {
                name: ObjectId::new(name),
            })
        }
    }
}
