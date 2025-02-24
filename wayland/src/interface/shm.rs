//! A singleton global object that provides support for shared memory.
//!
//! Clients can create wl_shm_pool objects using the create_pool
//! request.
//!
//! On binding the wl_shm object one or more format events
//! are emitted to inform clients about the valid pixel formats
//! that can be used for buffers.

use crate::interface::Request;
use crate::object::ObjectId;
use crate::wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc};
use std::os::fd::RawFd;

pub mod request {
    use super::*;

    /// Create a new wl_shm_pool object.
    ///
    /// The pool can be used to create shared memory based buffer
    /// objects.  The server will mmap size bytes of the passed file
    /// descriptor, to use as backing memory for the pool.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct CreatePool {
        pub object_id: ObjectId,
        /// Pool to create
        pub id: ObjectId,
        /// File descriptor for the pool
        pub fd: RawFd,
        /// Pool size, in bytes
        pub size: i32,
    }

    impl Request for CreatePool {
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
                .file_desc(self.fd)
                .int(self.size)
                .build()
        }
    }
}
