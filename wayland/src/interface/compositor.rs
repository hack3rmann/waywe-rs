//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::Request,
        object::ObjectId,
        wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
    };

    /// Ask the compositor to create a new surface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface {
        pub object_id: ObjectId,
        /// The new surface
        pub new_id: ObjectId,
    }

    impl Request for CreateSurface {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc(self))
                .uint(self.new_id.into())
                .build()
        }
    }

    /// Ask the compositor to create a new region.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateRegion {
        pub object_id: ObjectId,
        /// The new region
        pub new_id: ObjectId,
    }

    impl Request for CreateRegion {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 1,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc(self))
                .uint(self.new_id.into())
                .build()
        }
    }
}
