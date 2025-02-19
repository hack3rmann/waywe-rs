//! Clients can handle the 'done' event to get notified when
//! the related request is done.
//! 
//! Note, because wl_callback objects are created from multiple independent
//! factory interfaces, the wl_callback interface is frozen at version 1.

pub mod event {
    use crate::wayland::{
        interface::Event,
        wire::{Message, MessageHeaderDesc},
    };

	  /// Notify the client when the related request is done.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Done {
        /// Request-specific data for the callback
        pub data: u32,
    }

    impl<'s> Event<'s> for Done {
        fn header_desc() -> Option<MessageHeaderDesc> {
            None
        }

        fn from_message(message: &'s Message) -> Option<Self> {
            let mut reader = message.reader();
            let data = reader.read_u32()?;
            Some(Self { data })
        }
    }
}
