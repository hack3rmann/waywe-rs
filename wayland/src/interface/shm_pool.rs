//! The wl_shm_pool object encapsulates a piece of memory shared
//! between the compositor and client.  Through the wl_shm_pool
//! object, the client can allocate shared memory wl_buffer objects.
//! All objects created through the same pool share the same
//! underlying mapped memory. Reusing the mapped memory avoids the
//! setup/teardown overhead and is useful when interactively resizing
//! a surface or for many small buffers.

use crate::interface::Request;
use crate::sys::wire::{Message, MessageBuffer, OpCode};

pub mod request {
    use crate::{interface::WlShmFormat, sys::proxy::WlProxy};

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
        /// Buffer byte offset within the pool
        pub offset: i32,
        /// Buffer width, in pixels
        pub width: i32,
        /// Buffer height, in pixels
        pub height: i32,
        /// Number of bytes from the beginning of one row to the beginning of the next row
        pub stride: i32,
        /// Buffer pixel format
        pub format: WlShmFormat,
    }

    impl<'b> Request<'b> for CreateBuffer {
        const CODE: OpCode = 0;

        fn build_message(
            self,
            parent: &'b WlProxy,
            buf: &'b mut impl MessageBuffer,
        ) -> Message<'b> {
            Message::builder(buf)
                .header(parent, Self::CODE)
                .new_id()
                .int(self.offset)
                .int(self.width)
                .int(self.height)
                .int(self.stride)
                .uint(self.format.into())
                .build()
        }
    }

    ///Destroy the shared memory pool.
    ///The mmapped memory will be released when all
    ///buffers that have been created from this pool
    ///are gone.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Destroy;

    impl<'b> Request<'b> for Destroy {
        const CODE: OpCode = 1;

        fn build_message(
            self,
            parent: &'b WlProxy,
            buf: &'b mut impl MessageBuffer,
        ) -> Message<'b> {
            Message::builder(buf).header(parent, Self::CODE).build()
        }
    }

    ///This request will cause the server to remap the backing memory
    ///for the pool from the file descriptor passed when the pool was
    ///created, but using the new size.  This request can only be
    ///used to make the pool bigger.
    ///This request only changes the amount of bytes that are mmapped
    ///by the server and does not touch the file corresponding to the
    ///file descriptor passed at creation time. It is the client's
    ///responsibility to ensure that the file is at least as big as
    ///the new pool size.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Resize {
        /// new size of the pool, in bytes
        pub size: i32,
    }

    impl<'b> Request<'b> for Resize {
        const CODE: OpCode = 2;

        fn build_message(
            self,
            parent: &'b WlProxy,
            buf: &'b mut impl MessageBuffer,
        ) -> Message<'b> {
            Message::builder(buf)
                .header(parent, Self::CODE)
                .int(self.size)
                .build()
        }
    }
}
