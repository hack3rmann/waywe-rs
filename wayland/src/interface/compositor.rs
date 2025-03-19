//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::{ObjectParent, Request},
        object::{HasObjectType, WlObjectType},
        sys::{
            object_storage::WlObjectStorage,
            wire::{MessageBuffer, OpCode, WlMessage},
        },
    };

    /// Ask the compositor to create a new surface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface;

    impl ObjectParent for CreateSurface {
        const CHILD_TYPE: WlObjectType = WlObjectType::Surface;
    }

    impl HasObjectType for CreateSurface {
        const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
    }

    impl<'s> Request<'s> for CreateSurface {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::Surface);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }

    /// Ask the compositor to create a new region.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateRegion;

    impl ObjectParent for CreateRegion {
        const CHILD_TYPE: WlObjectType = WlObjectType::Region;
    }

    impl HasObjectType for CreateRegion {
        const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
    }

    impl<'s> Request<'s> for CreateRegion {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::Region);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }
}
