use super::{
    ffi::{wl_argument, wl_fixed_t, wl_object, wl_proxy},
    proxy::{AsProxy, WlDynProxyQuery, WlProxyBorrow, WlProxyQuery},
};
use std::{
    ffi::CStr,
    os::fd::{AsRawFd, BorrowedFd, FromRawFd as _, OwnedFd},
    ptr,
};

pub type OpCode = u16;

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

#[derive(Clone, Copy)]
pub struct Message<'s> {
    pub parent: WlProxyBorrow<'s>,
    pub opcode: OpCode,
    pub arguments: &'s [wl_argument],
}

impl<'s> Message<'s> {
    pub fn builder<Buffer: MessageBuffer>(
        buf: &'s mut Buffer,
    ) -> MessageBuilderHeaderless<'s, Buffer> {
        MessageBuilderHeaderless::new(buf)
    }

    pub fn reader(&self) -> MessageReader<'s> {
        MessageReader::new(self.arguments)
    }
}

pub struct MessageBuilderHeaderless<'s, Buffer: MessageBuffer> {
    pub(crate) buf: &'s mut Buffer,
}

impl<'s, Buffer: MessageBuffer> MessageBuilderHeaderless<'s, Buffer> {
    /// Creates new [`MessageBuffer`] from given message buffer
    pub fn new(buf: &'s mut Buffer) -> Self {
        buf.clear();
        Self { buf }
    }

    /// Sets parent object and opcode for the message
    pub fn header(self, parent: &'s impl AsProxy, opcode: OpCode) -> MessageBuilder<'s, Buffer> {
        MessageBuilder::new_header(self.buf, parent, opcode)
    }
}

pub struct MessageBuilder<'s, Buffer: MessageBuffer> {
    pub(crate) buf: &'s mut Buffer,
    pub(crate) parent: WlProxyBorrow<'s>,
    pub(crate) opcode: OpCode,
}

impl<'s, Buffer: MessageBuffer> MessageBuilder<'s, Buffer> {
    pub fn new_header(buf: &'s mut Buffer, parent: &'s impl AsProxy, opcode: OpCode) -> Self {
        Self {
            buf,
            parent: parent.as_proxy(),
            opcode,
        }
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

    pub fn maybe_object(self, value: Option<&'s impl AsProxy>) -> Self {
        self.buf.push(wl_argument {
            o: value
                .map(|proxy| proxy.as_proxy().as_raw().as_ptr())
                .unwrap_or(ptr::null_mut())
                .cast::<wl_object>(),
        });
        self
    }

    /// Writes [`WlObject`] to the message
    pub fn object(self, value: &'s mut impl AsProxy) -> Self {
        self.maybe_object(Some(value))
    }

    /// Writes empty object to the message
    pub fn null_object(self) -> Self {
        self.buf.push(wl_argument { o: ptr::null_mut() });
        self
    }

    pub fn new_id(self) -> Self {
        self.buf.push(wl_argument { n: 0 });
        self
    }

    pub fn build(self) -> Message<'s> {
        Message {
            parent: self.parent,
            opcode: self.opcode,
            arguments: self.buf.as_slice(),
        }
    }
}

pub trait FromWlArgument<'s>: Sized {
    unsafe fn from_argument(value: wl_argument) -> Self;
}

impl FromWlArgument<'_> for i32 {
    unsafe fn from_argument(value: wl_argument) -> Self {
        unsafe { value.i }
    }
}

impl FromWlArgument<'_> for u32 {
    unsafe fn from_argument(value: wl_argument) -> Self {
        unsafe { value.u }
    }
}

impl FromWlArgument<'_> for wl_fixed_t {
    unsafe fn from_argument(value: wl_argument) -> Self {
        unsafe { value.f }
    }
}

impl FromWlArgument<'_> for OwnedFd {
    unsafe fn from_argument(value: wl_argument) -> Self {
        let raw_fd = unsafe { value.h };
        // Safety: file descriptor provided by the libwayland must be owned by us
        unsafe { OwnedFd::from_raw_fd(raw_fd) }
    }
}

impl<T: AsProxy> FromWlArgument<'_> for WlProxyQuery<T> {
    unsafe fn from_argument(value: wl_argument) -> Self {
        let proxy_ptr = unsafe { value.o }.cast::<wl_proxy>();
        // Safety: proxy object provided by the libwayland should be valid or point to null
        unsafe { WlProxyQuery::from_raw(proxy_ptr) }
    }
}

impl FromWlArgument<'_> for WlDynProxyQuery {
    unsafe fn from_argument(value: wl_argument) -> Self {
        let proxy_ptr = unsafe { value.o }.cast::<wl_proxy>();
        // Safety: proxy object provided by the libwayland should be valid or point to null
        unsafe { WlDynProxyQuery::from_raw(proxy_ptr) }
    }
}

impl<'s> FromWlArgument<'s> for &'s CStr {
    unsafe fn from_argument(value: wl_argument) -> Self {
        let ptr = unsafe { value.s };
        // Safety: string provided by the libwayland must be valid
        unsafe { CStr::from_ptr(ptr) }
    }
}

#[derive(Clone, Copy)]
pub struct MessageReader<'s> {
    pub arguments: &'s [wl_argument],
}

impl<'s> MessageReader<'s> {
    pub const fn new(arguments: &'s [wl_argument]) -> Self {
        Self { arguments }
    }

    pub unsafe fn read<A: FromWlArgument<'s>>(&mut self) -> Option<A> {
        let first_arg = self.arguments.first().copied()?;
        self.arguments = &self.arguments[1..];
        Some(unsafe { A::from_argument(first_arg) })
    }
}
