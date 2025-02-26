//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::Request,
        sys::{
            proxy::{WlCompositor, WlRegion},
            wire::{Message, MessageBuffer, OpCode},
        },
    };

    /// Ask the compositor to create a new surface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface;

    impl<'b> Request<'b> for CreateSurface {
        type ParentProxy = WlCompositor;

        const CODE: OpCode = 0;

        fn build_message(
            self,
            parent: &'b Self::ParentProxy,
            buf: &'b mut impl MessageBuffer,
        ) -> Message<'b> {
            Message::builder(buf)
                .header(parent, Self::CODE)
                .new_id()
                .build()
        }
    }

    /// Ask the compositor to create a new region.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateRegion;

    impl<'b> Request<'b> for CreateRegion {
        type ParentProxy = WlRegion;

        const CODE: OpCode = 1;

        fn build_message(
            self,
            parent: &'b Self::ParentProxy,
            buf: &'b mut impl MessageBuffer,
        ) -> Message<'b> {
            Message::builder(buf)
                .header(parent, Self::CODE)
                .new_id()
                .build()
        }
    }
}
