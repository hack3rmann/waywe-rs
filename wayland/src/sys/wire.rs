use super::{
    InterfaceMessageArgument,
    proxy::{WlProxy, WlProxyQuery},
};
use smallvec::SmallVec;
use std::{
    ffi::CStr,
    mem::MaybeUninit,
    os::fd::{AsRawFd, BorrowedFd, FromRawFd as _, OwnedFd},
    ptr,
};
use wayland_sys::{WlArgument, wl_fixed_t, wl_object, wl_proxy};

/// The code of the performing operation on the interface
pub type OpCode = u16;

/// # Safety
///
/// - the implementor ensures all calls are valid
/// - the implementor ensures the caller of these functions can not destinguish
///   the behavior of them from the [`Vec`] ones
pub unsafe trait MessageBuffer {
    fn clear(&mut self);
    fn push(&mut self, argument: WlArgument);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_slice(&self) -> &[WlArgument];
}
static_assertions::assert_obj_safe!(MessageBuffer);

unsafe impl MessageBuffer for VecMessageBuffer {
    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn push(&mut self, argument: WlArgument) {
        Vec::push(self, argument)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn as_slice(&self) -> &[WlArgument] {
        self
    }
}

unsafe impl<const N: usize> MessageBuffer for SmallVecMessageBuffer<N> {
    fn clear(&mut self) {
        SmallVec::clear(self)
    }

    fn push(&mut self, argument: WlArgument) {
        SmallVec::push(self, argument)
    }

    fn len(&self) -> usize {
        SmallVec::len(self)
    }

    fn is_empty(&self) -> bool {
        SmallVec::is_empty(self)
    }

    fn as_slice(&self) -> &[WlArgument] {
        self
    }
}

pub type VecMessageBuffer = Vec<WlArgument>;
pub type SmallVecMessageBuffer<const N: usize> = SmallVec<[WlArgument; N]>;

/// Message buffer constrained to the stack.
#[derive(Clone)]
pub struct StackMessageBuffer {
    len: usize,
    buf: [MaybeUninit<WlArgument>; StackMessageBuffer::CAPACITY],
}

impl StackMessageBuffer {
    pub const CAPACITY: usize = 20;

    /// Clears the buffer
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    /// Adds `argument` to the end of the buffer
    pub const fn push(&mut self, argument: WlArgument) {
        assert!(
            self.len < Self::CAPACITY,
            "failed to push to already filled `StackMessageBuffer`",
        );

        self.buf[self.len].write(argument);
        self.len += 1;
    }

    /// Buffer length (in elements)
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Checks buffer length is zero
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Buffer as a slice of arguments
    pub const fn as_slice(&self) -> &[WlArgument] {
        let ptr = (&raw const self.buf).cast::<WlArgument>();
        unsafe { std::slice::from_raw_parts(ptr, self.len) }
    }
}

unsafe impl MessageBuffer for StackMessageBuffer {
    fn clear(&mut self) {
        Self::clear(self);
    }

    fn push(&mut self, argument: WlArgument) {
        Self::push(self, argument);
    }

    fn len(&self) -> usize {
        Self::len(self)
    }

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn as_slice(&self) -> &[WlArgument] {
        Self::as_slice(self)
    }
}

/// Represents the message on the libwayland backend
#[derive(Clone, Copy)]
pub struct Message<'s> {
    /// The opcode for the request/event
    pub opcode: OpCode,
    /// Additional arguments for the request/event
    pub arguments: &'s [WlArgument],
}

impl<'s> Message<'s> {
    /// Returns the builder for the message
    pub fn builder<Buffer: MessageBuffer>(
        buf: &'s mut Buffer,
    ) -> MessageBuilderHeaderless<'s, Buffer> {
        MessageBuilderHeaderless::new(buf)
    }

    /// Returns the reader for this message
    pub fn reader(&self) -> MessageReader<'s> {
        MessageReader::new(self.arguments)
    }
}

/// Builder of the message header
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
    pub fn opcode(self, opcode: OpCode) -> MessageBuilder<'s, Buffer> {
        MessageBuilder::new_header(self.buf, opcode)
    }
}

/// Builder of the message body
pub struct MessageBuilder<'s, Buffer: MessageBuffer> {
    pub(crate) buf: &'s mut Buffer,
    pub(crate) opcode: OpCode,
}

impl<'s, Buffer: MessageBuffer> MessageBuilder<'s, Buffer> {
    /// Creates the builder
    pub fn new_header(buf: &'s mut Buffer, opcode: OpCode) -> Self {
        Self { buf, opcode }
    }

    /// Writes 32-bit unsigned integer to the message
    pub fn uint(self, value: u32) -> Self {
        self.buf.push(WlArgument::uint(value));
        self
    }

    /// Writes 32-bit signed integer to the message
    pub fn int(self, value: i32) -> Self {
        self.buf.push(WlArgument::int(value));
        self
    }

    /// Writes file descriptor to the message
    pub fn file_desc(self, value: BorrowedFd<'s>) -> Self {
        self.buf.push(WlArgument::raw_fd(value.as_raw_fd()));
        self
    }

    /// Writes [`str`] to the message
    pub fn str<'str: 's>(self, value: &'str CStr) -> Self {
        if value.is_empty() {
            self.buf.push(WlArgument::c_str(ptr::null()));
        } else {
            self.buf.push(WlArgument::c_str(value.as_ptr()));
        }

        self
    }

    pub fn maybe_object(self, value: Option<&'s WlProxy>) -> Self {
        self.buf.push(WlArgument::object(
            value
                .map(|proxy| proxy.as_raw().as_ptr())
                .unwrap_or(ptr::null_mut())
                .cast::<wl_object>(),
        ));
        self
    }

    /// Writes [`WlObject`] to the message
    pub fn object(self, value: &'s WlProxy) -> Self {
        self.maybe_object(Some(value))
    }

    /// Writes empty object to the message
    pub fn null_object(self) -> Self {
        self.buf.push(WlArgument::object(ptr::null_mut()));
        self
    }

    /// Passes `new_id` argument to the message
    pub fn new_id(self) -> Self {
        self.buf.push(WlArgument::new_id());
        self
    }

    /// Passes interface information to the message
    pub fn interface(self, value: InterfaceMessageArgument) -> Self {
        self.uint(value.name().into())
            .str(value.interface())
            .uint(value.min_supported_version().get())
            .new_id()
    }

    /// Builds the message
    pub fn build(self) -> Message<'s> {
        Message {
            opcode: self.opcode,
            arguments: self.buf.as_slice(),
        }
    }
}

/// Provides a coversion function from [`WlArgument`]
pub trait FromWlArgument<'s>: Sized {
    /// # Safety
    ///
    /// The value extracted from `WlArgument` shoud be the same
    /// as the value written to this union
    unsafe fn from_argument(value: WlArgument) -> Self;
}

impl FromWlArgument<'_> for i32 {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.i }
    }
}

impl FromWlArgument<'_> for u32 {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.u }
    }
}

impl FromWlArgument<'_> for wl_fixed_t {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.f }
    }
}

impl FromWlArgument<'_> for OwnedFd {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let raw_fd = unsafe { value.h };
        // Safety: file descriptor provided by the libwayland must be owned by us
        unsafe { OwnedFd::from_raw_fd(raw_fd) }
    }
}

impl FromWlArgument<'_> for WlProxyQuery {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let proxy_ptr = unsafe { value.o }.cast::<wl_proxy>();
        // Safety: proxy object provided by the libwayland should be valid or point to null
        unsafe { WlProxyQuery::from_raw(proxy_ptr) }
    }
}

impl<'s> FromWlArgument<'s> for &'s CStr {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let ptr = unsafe { value.s };
        // Safety: string provided by the libwayland must be valid
        unsafe { CStr::from_ptr(ptr) }
    }
}

/// Represents a message reader capable of converting [`WlArgument`]s to values
#[derive(Clone, Copy)]
pub struct MessageReader<'s> {
    /// The rest of message's arguments
    pub arguments: &'s [WlArgument],
}

impl<'s> MessageReader<'s> {
    /// Constructs new [`MessageReader`]
    pub const fn new(arguments: &'s [WlArgument]) -> Self {
        Self { arguments }
    }

    /// Reads a values from the next arguments of the message
    ///
    /// # Safety
    ///
    /// The argument read from the message at this point should have
    /// the same type as the argument that was written to the message
    /// at this point before
    pub unsafe fn read<A: FromWlArgument<'s>>(&mut self) -> Option<A> {
        let first_arg = self.arguments.first().copied()?;
        self.arguments = &self.arguments[1..];
        Some(unsafe { A::from_argument(first_arg) })
    }
}
