use crate::wayland::{
    interface::{Event, NewId, Request},
    object::ObjectId,
    wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};

pub mod request {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Sync {
        pub callback: NewId,
    }

    impl Request for Sync {
        fn header_desc() -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<&Message, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc())
                .uint(self.callback.into())
                .build()
        }
    }

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GetRegistry {
        pub registry: NewId,
    }

    impl Request for GetRegistry {
        fn header_desc() -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 1,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<&Message, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc())
                .uint(self.registry.into())
                .build()
        }
    }
}

pub mod event {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Error<'s> {
        pub object: ObjectId,
        pub code: u32,
        pub message: &'s str,
    }

    impl<'s> Event<'s> for Error<'s> {
        fn header_desc() -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 0,
            }
        }

        fn from_message(message: &'s Message) -> Self {
            let mut reader = message.reader();

            let object = reader.read_u32().unwrap();
            let code = reader.read_u32().unwrap();
            let message = reader.read_str().unwrap();

            Self {
                object: ObjectId::new(object),
                code,
                message,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct DeleteId {
        pub id: ObjectId,
    }

    impl<'s> Event<'s> for DeleteId {
        fn header_desc() -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: ObjectId::WL_DISPLAY,
                opcode: 1,
            }
        }

        fn from_message(message: &'s Message) -> Self {
            let mut reader = message.reader();
            let id = reader.read_u32().unwrap();
            Self { id: ObjectId::new(id) }
        }
    }
}

pub mod wl_enum {
    #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub enum Error {
        InvalidObject = 0,
        InvalidMethod = 1,
        NoMemory = 2,
        Implementation = 3,
    }
}
