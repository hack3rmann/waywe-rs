//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::{ObjectParent, Request},
        sys::{
            HasObjectType, ObjectType,
            object::{region::WlRegion, surface::WlSurface},
            object_storage::WlObjectStorage,
            wire::{Message, MessageBuffer, OpCode},
        },
    };

    /// Ask the compositor to create a new surface.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSurface;

    impl ObjectParent for CreateSurface {
        type Child = WlSurface;
    }

    impl HasObjectType for CreateSurface {
        const OBJECT_TYPE: ObjectType = ObjectType::Compositor;
    }

    impl<'s> Request<'s> for CreateSurface {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<ObjectType> = Some(ObjectType::Surface);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }

    /// Ask the compositor to create a new region.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateRegion;

    impl ObjectParent for CreateRegion {
        type Child = WlRegion;
    }

    impl HasObjectType for CreateRegion {
        const OBJECT_TYPE: ObjectType = ObjectType::Compositor;
    }

    impl<'s> Request<'s> for CreateRegion {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<ObjectType> = Some(ObjectType::Region);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }
}
