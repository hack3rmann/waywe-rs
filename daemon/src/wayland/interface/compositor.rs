//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::wayland::{
        object::ObjectId,
        wire::{Message, MessageBuffer, MessageBuildResult, MessageHeaderDesc},
    };

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface {
        /// The new surface
        pub new_id: ObjectId,
    }

    /// Ask the compositor to create a new surface.
    pub fn create_surface(req: CreateSurface, buf: &mut MessageBuffer) -> MessageBuildResult {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id: ObjectId::WL_COMPOSITOR,
                opcode: 0,
            })
            .new_id(req.new_id)
            .build()
    }

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateRegion {
        /// The new region
        pub new_id: ObjectId,
    }

    /// Ask the compositor to create a new region.
    pub fn craete_region(req: CreateRegion, buf: &mut MessageBuffer) -> MessageBuildResult {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id: ObjectId::WL_COMPOSITOR,
                opcode: 1,
            })
            .new_id(req.new_id)
            .build()
    }
}
