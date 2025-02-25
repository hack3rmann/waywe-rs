//! The wl_shm_pool object encapsulates a piece of memory shared
//! between the compositor and client.  Through the wl_shm_pool
//! object, the client can allocate shared memory wl_buffer objects.
//! All objects created through the same pool share the same
//! underlying mapped memory. Reusing the mapped memory avoids the
//! setup/teardown overhead and is useful when interactively resizing
//! a surface or for many small buffers.

use crate::interface::Request;
use crate::object::ObjectId;
use crate::wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc};

pub mod request {
    use crate::interface::WlShmFormat;

    use super::*;

    /// Create a wl_buffer object from the pool.
    ///
    /// The buffer is created offset bytes into the pool and has
    /// width and height as specified.  The stride argument specifies
    /// the number of bytes from the beginning of one row to the beginning
    /// of the next.  The format is the pixel format of the buffer and
    /// must be one of those advertised through the wl_shm.format event.
    ///
    /// A buffer will keep a reference to the pool it was created from
    /// so it is valid to destroy the pool immediately after creating
    /// a buffer from it.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateBuffer {
        pub object_id: ObjectId,
        pub id: ObjectId,
        pub width: i32,
        pub height: i32,
        pub stride: i32,
        pub format: WlShmFormat,
    }

    impl Request for CreateBuffer {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(Self::header_desc(self))
                .uint(self.id.into())
                .int(self.width)
                .int(self.height)
                .int(self.stride)
                .uint(self.format.into())
                .build()
        }
    }
}
