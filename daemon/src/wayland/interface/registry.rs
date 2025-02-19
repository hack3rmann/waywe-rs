use crate::wayland::{
    interface::{Event, Request},
    object::ObjectId,
    wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};

pub mod request {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Bind {
        pub name: ObjectId,
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

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Global<'s> {
        pub name: ObjectId,
        pub interface: &'s str,
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

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GlobalRemove {
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
