use super::{WlObject, ffi::wl_argument};
use std::{
    ffi::CStr,
    os::fd::{AsRawFd, BorrowedFd},
    ptr,
};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct OpCode(pub(crate) u16);

impl OpCode {
    pub const INVALID: Self = Self(u16::MAX);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }
}

/// # Safety
///
/// TODO(hack3rmann): write safety docs
/// TODO(ArnoDarkrose): add trait impl for `SmallVec`
pub unsafe trait MessageBuffer {
    fn clear(&mut self);
    fn push(&mut self, argument: wl_argument);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_slice(&self) -> &[wl_argument];
}
static_assertions::assert_obj_safe!(MessageBuffer);

unsafe impl MessageBuffer for Vec<wl_argument> {
    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn push(&mut self, argument: wl_argument) {
        Vec::push(self, argument)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn as_slice(&self) -> &[wl_argument] {
        self
    }
}

pub struct MessageBuilder<'s, Buffer: MessageBuffer> {
    pub(crate) opcode: OpCode,
    pub(crate) buf: &'s mut Buffer,
}

impl<'s, Buffer: MessageBuffer> MessageBuilder<'s, Buffer> {
    /// Creates new [`MessageBuffer`] from given message buffer
    pub fn new(buf: &'s mut Buffer) -> Self {
        buf.clear();
        Self {
            buf,
            opcode: OpCode::INVALID,
        }
    }

    /// Sets id for requests and events.
    pub fn opcode(mut self, value: OpCode) -> Self {
        self.opcode = value;
        self
    }

    /// Writes 32-bit unsigned integer to the message
    pub fn uint(self, value: u32) -> Self {
        self.buf.push(wl_argument { u: value });
        self
    }

    /// Writes 32-bit signed integer to the message
    pub fn int(self, value: i32) -> Self {
        self.buf.push(wl_argument { i: value });
        self
    }

    /// Writes file descriptor to the message
    pub fn file_desc(self, value: BorrowedFd<'s>) -> Self {
        self.buf.push(wl_argument {
            h: value.as_raw_fd(),
        });
        self
    }

    /// Writes [`str`] to the message
    pub fn str<'str: 's>(self, value: &'str CStr) -> Self {
        if value.is_empty() {
            self.buf.push(wl_argument { s: ptr::null() });
        } else {
            self.buf.push(wl_argument { s: value.as_ptr() });
        }

        self
    }

    /// Writes [`WlObject`] to the message
    pub fn object(self, value: &'s mut WlObject) -> Self {
        self.buf.push(wl_argument {
            o: value.raw_proxy_ptr().as_ptr().cast(),
        });
        self
    }

    /// Writes empty object to the message
    pub fn null_object(self) -> Self {
        self.buf.push(wl_argument { o: ptr::null_mut() });
        self
    }
}
