//! A compositor.  This object is a singleton global.  The
//! compositor is in charge of combining the contents of multiple
//! surfaces into one displayable output.

pub mod request {
    use crate::{
        interface::{ObjectParent, Request},
        sys::{
            object::{region::WlRegion, surface::WlSurface}, wire::{Message, MessageBuffer, OpCode}, HasObjectType, ObjectType
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

    impl<'b> Request<'b> for CreateSurface {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<ObjectType> = Some(ObjectType::Surface);

        fn build_message(self, buf: &'b mut impl MessageBuffer) -> Message<'b> {
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

    impl<'b> Request<'b> for CreateRegion {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<ObjectType> = Some(ObjectType::Region);

        fn build_message(self, buf: &'b mut impl MessageBuffer) -> Message<'b> {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }
}
