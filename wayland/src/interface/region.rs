//! A region object describes an area.
//!
//! Region objects are used to describe the opaque and input
//! regions of a surface.

pub mod request {
    use crate::{
        interface::Request,
        sys::{
            HasObjectType, ObjectType,
            object_storage::WlObjectStorage,
            wire::{Message, MessageBuffer, OpCode},
        },
    };

    /// Destroy the region. This will invalidate the object ID.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Destroy;

    impl HasObjectType for Destroy {
        const OBJECT_TYPE: ObjectType = ObjectType::Region;
    }

    impl<'s> Request<'s> for Destroy {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<ObjectType> = None;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).build()
        }
    }
}
