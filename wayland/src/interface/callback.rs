//! Clients can handle the 'done' event to get notified when
//! the related request is done.
//!
//! Note, because wl_callback objects are created from multiple independent
//! factory interfaces, the wl_callback interface is frozen at version 1.

pub mod event {
    use crate::{
        interface::Event,
        sys::wire::{WlMessage, OpCode},
    };

    /// Notify the client when the related request is done.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Done {
        /// Request-specific data for the callback
        pub data: u32,
    }

    impl<'s> Event<'s> for Done {
        const CODE: OpCode = 0;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();
            let data = unsafe { reader.read::<u32>()? };

            Some(Self { data })
        }
    }
}
