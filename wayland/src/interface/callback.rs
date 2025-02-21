//! Clients can handle the 'done' event to get notified when
//! the related request is done.
//!
//! Note, because wl_callback objects are created from multiple independent
//! factory interfaces, the wl_callback interface is frozen at version 1.

pub mod event {
    use crate::{
        interface::Event,
        object::ObjectId,
        wire::{Message, MessageHeaderDesc},
    };

    /// Notify the client when the related request is done.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Done {
        /// id of the callback object
        pub object_id: ObjectId,
        /// Request-specific data for the callback
        pub data: u32,
    }

    impl<'s> Event<'s> for Done {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 0,
            }
        }

        fn from_message(message: &'s Message) -> Option<Self> {
            let header = message.header();

            if header.opcode != 0 {
                return None;
            }

            let mut reader = message.reader();
            let data = reader.read_u32()?;

            Some(Self {
                data,
                object_id: ObjectId::try_from(header.object_id).ok()?,
            })
        }
    }
}
