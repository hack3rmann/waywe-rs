use crate::wayland::{
    object::ObjectId,
    wire::{Message, MessageBuffer, MessageHeaderDesc},
};

pub mod request {
    use super::*;
    use crate::wayland::wire::{Message, MessageBuildResult, MessageHeaderDesc};

    pub fn delete(object_id: ObjectId, buf: &mut MessageBuffer) -> MessageBuildResult {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id,
                opcode: 0,
            })
            .build()
    }
}

pub struct WlSurface {
    pub id: ObjectId,
}

impl WlSurface {
    pub fn damage<'a>(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        buf: &'a mut MessageBuffer,
    ) -> Result<&'a Message, crate::wayland::wire::MessageBuildError> {
        Message::builder(buf)
            .header(MessageHeaderDesc {
                object_id: self.id,
                opcode: 2,
            })
            .int(x)
            .int(y)
            .int(width)
            .int(height)
            .build()
    }
}
