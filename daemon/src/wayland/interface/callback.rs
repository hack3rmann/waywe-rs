pub mod event {
    use crate::wayland::{
        interface::Event,
        wire::{Message, MessageHeaderDesc},
    };

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Done {
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
