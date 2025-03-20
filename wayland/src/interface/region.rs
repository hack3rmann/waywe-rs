//! A region object describes an area.
//!
//! Region objects are used to describe the opaque and input
//! regions of a surface.

pub mod request {
    use crate::{
        interface::Request, object::{HasObjectType, WlObjectType}, sys::{
            object::dispatch::State, object_storage::WlObjectStorage, wire::{MessageBuffer, OpCode, WlMessage}
        }
    };

    /// Destroy the region. This will invalidate the object ID.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Destroy;

    impl HasObjectType for Destroy {
        const OBJECT_TYPE: WlObjectType = WlObjectType::Region;
    }

    impl<'s> Request<'s> for Destroy {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<WlObjectType> = None;

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).build()
        }
    }
}
