//! Message constructors and parsers for wayland

use super::proxy::{WlProxy, WlProxyQuery};
use crate::{interface::Event, object::InterfaceMessageArgument};
use std::{
    ffi::CStr,
    mem::{self, MaybeUninit},
    os::fd::{AsRawFd, BorrowedFd, FromRawFd as _, OwnedFd},
    ptr, slice,
};
use wayland_sys::{wl_fixed_t, wl_object, wl_proxy, WlArgument, WlFixed};

#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

/// The code of the performing operation on the interface
pub type OpCode = u16;

/// # Safety
///
/// - the implementor ensures all calls are valid
/// - the implementor ensures the caller of these functions can not destinguish
///   the behavior of them from the [`Vec`] ones
pub unsafe trait WlMessageBuffer {
    /// Clears the buffer
    fn clear(&mut self);

    /// Adds an argument to the buffer
    fn push(&mut self, argument: WlArgument);

    /// Buffer length in elements
    fn len(&self) -> usize;

    /// Buffer is empty (len = 0)
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Slice of collected arguments (up to length)
    fn as_slice(&self) -> &[WlArgument];
}
static_assertions::assert_obj_safe!(WlMessageBuffer);

unsafe impl WlMessageBuffer for WlVecMessageBuffer {
    fn clear(&mut self) {
        Vec::clear(&mut self.0);
    }

    fn push(&mut self, argument: WlArgument) {
        Vec::push(&mut self.0, argument)
    }

    fn len(&self) -> usize {
        Vec::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(&self.0)
    }

    fn as_slice(&self) -> &[WlArgument] {
        &self.0
    }
}

#[cfg(feature = "smallvec")]
unsafe impl<const N: usize> WlMessageBuffer for WlSmallVecMessageBuffer<N> {
    fn clear(&mut self) {
        SmallVec::clear(&mut self.0)
    }

    fn push(&mut self, argument: WlArgument) {
        SmallVec::push(&mut self.0, argument)
    }

    fn len(&self) -> usize {
        SmallVec::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        SmallVec::is_empty(&self.0)
    }

    fn as_slice(&self) -> &[WlArgument] {
        &self.0
    }
}

/// Message buffer based on [`Vec`] implementation
#[derive(Clone, Default)]
pub struct WlVecMessageBuffer(pub(crate) Vec<WlArgument>);

unsafe impl Send for WlVecMessageBuffer {}
unsafe impl Sync for WlVecMessageBuffer {}

impl WlVecMessageBuffer {
    /// Constructs new [`WlVecMessageBuffer`]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Constructs new [`WlVecMessageBuffer`] with allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

/// Message buffer based on [`SmallVec`] implementation
#[cfg(feature = "smallvec")]
#[derive(Clone, Default)]
pub struct WlSmallVecMessageBuffer<const N: usize>(pub(crate) SmallVec<[WlArgument; N]>);

#[cfg(feature = "smallvec")]
unsafe impl<const N: usize> Send for WlSmallVecMessageBuffer<N> {}

#[cfg(feature = "smallvec")]
unsafe impl<const N: usize> Sync for WlSmallVecMessageBuffer<N> {}

#[cfg(feature = "smallvec")]
impl<const N: usize> WlSmallVecMessageBuffer<N> {
    /// Constructs new [`WlSmallVecMessageBuffer`]
    pub const fn new() -> Self {
        Self(SmallVec::new_const())
    }

    /// Constructs new [`WlSmallVecMessageBuffer`] with allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }
}

/// Message buffer constrained to the stack.
#[derive(Clone)]
pub struct WlStackMessageBuffer {
    len: usize,
    buf: [MaybeUninit<WlArgument>; WlStackMessageBuffer::CAPACITY],
}

unsafe impl Send for WlStackMessageBuffer {}
unsafe impl Sync for WlStackMessageBuffer {}

impl WlStackMessageBuffer {
    /// Maximum number of arguments allowed to be passed into request
    pub const CAPACITY: usize = 20;

    /// Constructs new [`WlStackMessageBuffer`]
    pub const fn new() -> Self {
        Self {
            len: 0,
            buf: [MaybeUninit::uninit(); Self::CAPACITY],
        }
    }

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

impl Default for WlStackMessageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl WlMessageBuffer for WlStackMessageBuffer {
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
pub struct WlMessage<'s> {
    /// The opcode for the request/event
    pub opcode: OpCode,
    /// Additional arguments for the request/event
    pub arguments: &'s [WlArgument],
}

unsafe impl Send for WlMessage<'_> {}
unsafe impl Sync for WlMessage<'_> {}

impl<'s> WlMessage<'s> {
    /// Returns a builder for the message
    pub fn builder<Buffer: WlMessageBuffer>(
        buf: &'s mut Buffer,
    ) -> WlMessageBuilderHeaderless<'s, Buffer> {
        WlMessageBuilderHeaderless::new(buf)
    }

    /// Returns a reader for this message
    pub fn reader(&self) -> WlMessageReader<'s> {
        WlMessageReader::new(self.arguments)
    }

    /// Tries to parse this message as an event `E`
    pub fn as_event<E: Event<'s>>(self) -> Option<E> {
        E::from_message(self)
    }
}

/// Builder of the message header
pub struct WlMessageBuilderHeaderless<'s, Buffer: WlMessageBuffer> {
    pub(crate) buf: &'s mut Buffer,
}

impl<'s, Buffer: WlMessageBuffer> WlMessageBuilderHeaderless<'s, Buffer> {
    /// Creates new [`WlMessageBuffer`] from given message buffer
    pub fn new(buf: &'s mut Buffer) -> Self {
        buf.clear();
        Self { buf }
    }

    /// Sets parent object and opcode for the message
    pub fn opcode(self, opcode: OpCode) -> WlMessageBuilder<'s, Buffer> {
        WlMessageBuilder::new_header(self.buf, opcode)
    }
}

/// Builder of the message body
pub struct WlMessageBuilder<'s, Buffer: WlMessageBuffer> {
    pub(crate) buf: &'s mut Buffer,
    pub(crate) opcode: OpCode,
}

impl<'s, Buffer: WlMessageBuffer> WlMessageBuilder<'s, Buffer> {
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

    /// Writes a file descriptor to the message
    pub fn fd(self, value: BorrowedFd<'s>) -> Self {
        self.buf.push(WlArgument::raw_fd(value.as_raw_fd()));
        self
    }

    /// Writes a `wl_fixed` number to the message
    pub fn fixed(self, value: WlFixed) -> Self {
        self.buf.push(WlArgument::fixed(value));
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

    /// Writes object to the message or leaves this field empty
    pub fn maybe_object(self, value: Option<&'s WlProxy>) -> Self {
        self.buf.push(WlArgument::object(
            value
                .map(|proxy| proxy.as_raw().as_ptr())
                .unwrap_or(ptr::null_mut())
                .cast::<wl_object>(),
        ));
        self
    }

    /// Writes [`WlProxy`] to the message
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
    pub fn build(self) -> WlMessage<'s> {
        WlMessage {
            opcode: self.opcode,
            arguments: self.buf.as_slice(),
        }
    }
}

/// Provides a coversion function from [`WlArgument`](wayland_sys::wl_argument)
pub trait FromArgument<'s>: Sized {
    /// # Safety
    ///
    /// The value extracted from `WlArgument` shoud be the same
    /// as the value written to this union
    unsafe fn from_argument(value: WlArgument) -> Self;
}

impl FromArgument<'_> for i32 {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.i }
    }
}

impl FromArgument<'_> for u32 {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.u }
    }
}

impl FromArgument<'_> for wl_fixed_t {
    unsafe fn from_argument(value: WlArgument) -> Self {
        unsafe { value.f }
    }
}

impl FromArgument<'_> for OwnedFd {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let raw_fd = unsafe { value.h };
        // Safety: file descriptor provided by the libwayland must be owned by us
        unsafe { OwnedFd::from_raw_fd(raw_fd) }
    }
}

impl FromArgument<'_> for WlProxyQuery {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let proxy_ptr = unsafe { value.o }.cast::<wl_proxy>();
        // Safety: proxy object provided by the libwayland should be valid or point to null
        unsafe { WlProxyQuery::from_raw(proxy_ptr) }
    }
}

impl<'s> FromArgument<'s> for &'s CStr {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let ptr = unsafe { value.s };
        // Safety: string provided by the libwayland must be valid
        unsafe { CStr::from_ptr(ptr) }
    }
}

impl<'s, T> FromArgument<'s> for &'s [T] {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let raw = unsafe { value.a.read() };
        unsafe { slice::from_raw_parts(raw.data.cast(), raw.size / mem::size_of::<T>()) }
    }
}

impl<'s> FromArgument<'s> for BorrowedFd<'s> {
    unsafe fn from_argument(value: WlArgument) -> Self {
        let raw = unsafe { value.h };
        unsafe { BorrowedFd::borrow_raw(raw) }
    }
}

/// Represents a message reader capable of converting [`WlArgument`]s to values
#[derive(Clone, Copy)]
pub struct WlMessageReader<'s> {
    /// The rest of message's arguments
    pub arguments: &'s [WlArgument],
}

impl<'s> WlMessageReader<'s> {
    /// Constructs new [`WlMessageReader`]
    pub const fn new(arguments: &'s [WlArgument]) -> Self {
        Self { arguments }
    }

    /// Reads a values from the next arguments of the message
    ///
    /// # Safety
    ///
    /// An argument being read by this call thould have the same type
    /// as the argument written to the message before
    pub unsafe fn read<A: FromArgument<'s>>(&mut self) -> Option<A> {
        let first_arg = self.arguments.first().copied()?;
        self.arguments = &self.arguments[1..];
        Some(unsafe { A::from_argument(first_arg) })
    }
}
