//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::{ObjectParent, Request},
        object::{HasObjectType, WlObjectType},
        sys::{
            object::dispatch::State, object_storage::WlObjectStorage, wire::{MessageBuffer, OpCode, WlMessage}
        },
    };

    /// Ask the compositor to create a new surface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface;

    impl ObjectParent for CreateSurface {
        const CHILD_TYPE: WlObjectType = WlObjectType::WlSurface;
    }

    impl HasObjectType for CreateSurface {
        const OBJECT_TYPE: WlObjectType = WlObjectType::WlCompositor;
    }

    impl<'s> Request<'s> for CreateSurface {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::WlSurface);

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
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
        const CHILD_TYPE: WlObjectType = WlObjectType::WlRegion;
    }

    impl HasObjectType for CreateRegion {
        const OBJECT_TYPE: WlObjectType = WlObjectType::WlCompositor;
    }

    impl<'s> Request<'s> for CreateRegion {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::WlRegion);

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }
}
