use super::object::ObjectId;
use bytemuck::{Pod, Zeroable};
use rustix::net::SendFlags;
use std::{
    io::{self, Read, Write},
    mem,
    os::fd::{AsFd, BorrowedFd, RawFd},
    str::Utf8Error,
};
use thiserror::Error;

/// A buffer for message contents
#[derive(Clone, Debug, PartialEq, Default, Eq, PartialOrd, Ord, Hash)]
pub struct MessageBuffer {
    pub non_fd_data: Vec<u32>,
    pub fd_data: Vec<RawFd>,
}

impl MessageBuffer {
    /// Constructs new empty [`MessageBuffer`]
    pub const fn new() -> Self {
        Self {
            non_fd_data: vec![],
            fd_data: vec![],
        }
    }

    /// Constructs new [`MessageBuffer`] with capacity in bytes
    pub fn with_capacity(n_bytes: usize) -> Self {
        // (.. + 3) / 4 pads string to u32
        let n_words = (n_bytes + 3) >> 2;
        Self {
            non_fd_data: Vec::with_capacity(n_words),
            fd_data: vec![],
        }
    }

    /// Clears buffer leaving capacity untouched
    pub fn clear(&mut self) {
        self.non_fd_data.clear();
        self.fd_data.clear();
    }

    /// Interprets buffer contents as `u32` slice
    pub fn as_slice(&self) -> &[u32] {
        &self.non_fd_data
    }

    /// Interprets buffer contents as `u32` mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.non_fd_data
    }

    /// Tries to interpret buffer contents as a [`Message`]
    ///
    /// # Panic
    ///
    /// Panics if the buffer does not contain [`MessageHeader`]
    pub fn get_message(&self) -> Message<'_> {
        Message::new(&self.non_fd_data, &self.fd_data)
    }
}

/// Message header from wire protocol
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct MessageHeader {
    pub object_id: u32,
    pub opcode: u16,
    pub message_len: u16,
}

impl MessageHeader {
    pub fn corresponds_to(&self, desc: &MessageHeaderDesc) -> bool {
        self.object_id == desc.object_id.into() && self.opcode == desc.opcode
    }
}

const HEADER_SIZE_WORDS: usize = mem::size_of::<MessageHeader>() / mem::size_of::<u32>();

/// Represents a message from Wire protocol
#[derive(Debug, PartialEq, Clone, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Message<'s> {
    main_body: &'s [u32],
    file_descs: &'s [RawFd],
}

impl<'s> Message<'s> {
    pub const fn new(main_body: &'s [u32], file_descs: &'s [RawFd]) -> Self {
        Self {
            main_body,
            file_descs,
        }
    }

    /// Constructs [`MessageReader`] of this message
    pub fn reader(self) -> MessageReader<'s> {
        MessageReader::new(self)
    }

    /// Cast the message to a [`u32`] slice.
    pub fn main_body(&self) -> &'s [u32] {
        self.main_body
    }

    pub fn file_descs(&self) -> &'s [BorrowedFd<'s>] {
        // Safety `BorrowedFd` is repr(transparent) of `RawFd`
        unsafe { mem::transmute(self.file_descs) }
    }

    /// Cast the message to a byte slice.
    pub fn main_body_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.main_body())
    }

    /// Creates a message from raw [`u32`] slice.
    ///
    /// # Panic
    ///
    /// Panics if `mem::size_of_val(src) < mem::size_of::<MessageHeader>()`.
    pub fn from_u32_slice(src: &'s [u32]) -> Self {
        // TODO: validate message + return Result type
        assert!(mem::size_of_val(src) >= mem::size_of::<MessageHeader>());

        Self {
            main_body: src,
            file_descs: &[],
        }
    }

    /// Message header
    pub fn header(&self) -> MessageHeader {
        *bytemuck::from_bytes(bytemuck::cast_slice(
            &self.main_body[..mem::size_of::<MessageHeader>() / mem::size_of::<u32>()],
        ))
    }

    /// Message body (header removed)
    pub fn body(&self) -> &'s [u32] {
        &self.main_body[mem::size_of::<MessageHeader>() / mem::size_of::<u32>()..]
    }

    /// Message length in bytes
    pub fn len(&self) -> usize {
        self.header().message_len as usize
    }

    /// Checks if message is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Constructs a [`MessageBuilder`] on the top of the given [`MessageBuffer`]
    pub fn builder(buf: &mut MessageBuffer) -> MessageBuilder {
        MessageBuilder::new(buf)
    }

    /// Sends the message to the stream
    pub fn send(&self, stream: impl AsFd) -> Result<(), rustix::io::Errno> {
        let mut control_buf = [0u8; rustix::cmsg_space!(ScmRights(1))];
        let mut control = rustix::net::SendAncillaryBuffer::new(&mut control_buf);

        let msg = rustix::net::SendAncillaryMessage::ScmRights(self.file_descs());
        control.push(msg);

        rustix::net::sendmsg(
            stream,
            &[io::IoSlice::new(self.main_body_bytes())],
            &mut control,
            SendFlags::NOSIGNAL,
        )?;

        Ok(())
    }
}

/// Reads a message from the stream
pub fn read_message_into<S: Read + ?Sized>(
    stream: &mut S,
    buf: &mut MessageBuffer,
) -> Result<(), io::Error> {
    buf.non_fd_data.resize(HEADER_SIZE_WORDS, 0);
    stream.read_exact(bytemuck::cast_slice_mut(&mut buf.non_fd_data))?;

    let header = bytemuck::from_bytes::<MessageHeader>(bytemuck::cast_slice(&buf.non_fd_data));
    let len = header.message_len as usize / mem::size_of::<u32>();

    buf.non_fd_data.resize(len, 0);
    stream.read_exact(bytemuck::cast_slice_mut(
        &mut buf.non_fd_data[HEADER_SIZE_WORDS..],
    ))?;

    Ok(())
}

/// Writes a message to the stream
pub fn write_message(stream: &mut impl Write, message: &Message) -> Result<(), io::Error> {
    assert_eq!(
        message.len(),
        mem::size_of::<MessageHeader>() + std::mem::size_of_val(message.body())
    );

    stream.write_all(message.main_body_bytes())
}

/// A string which
///
/// - null-terminated
/// - prefixed with u32 length in bytes (including null-terminator)
/// - encoding-unconstrained
/// - padded to u32 with undefined data
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireStr {
    pub raw: [u32],
}

impl WireStr {
    /// Creates [`WireStr`] out of raw data.
    ///
    /// # Error
    ///
    /// It will return `None` if raw data does not satify all invariants (see [`WireStr`]).
    pub fn new(raw: &[u32]) -> Option<&WireStr> {
        // has it's length as a first entry
        if raw.is_empty() || mem::size_of_val(raw) - (raw[0] as usize) >= 2 * mem::size_of::<u32>()
        {
            return None;
        }

        let null_end = raw.last().unwrap();

        // null-terminated
        if null_end.to_le_bytes().into_iter().all(|byte| byte != 0) {
            return None;
        }

        Some(unsafe { mem::transmute::<&[u32], &Self>(raw) })
    }

    /// Creates a [`str`] from [`WireStr`] if source string is UTF-8 encoded.
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        let len = self.raw[0] as usize - 1;
        let bytes: &[u8] = &bytemuck::cast_slice(&self.raw[1..])[..len];
        std::str::from_utf8(bytes)
    }
}

/// Wire protocol message reader.
#[derive(Debug, PartialEq, Default, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct MessageReader<'r> {
    pub data: &'r [u32],
}

impl<'r> MessageReader<'r> {
    /// Binds the reader to a message.
    pub fn new(message: Message<'r>) -> Self {
        Self {
            data: bytemuck::cast_slice(message.body()),
        }
    }

    /// Reads [`u32`] from message if any's present
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.data.is_empty() {
            return None;
        }

        let value = self.data[0];
        self.data = &self.data[1..];

        Some(value)
    }

    /// Reads [`i32`] from message if any's present
    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|value| value as i32)
    }

    /// Reads [`WireStr`] from message if any's present
    pub fn read_wire_str(&mut self) -> Option<&'r WireStr> {
        if self.data.is_empty() {
            return None;
        }

        let n_bytes = self.data[0];

        // string(.. + 3) >> 2 pads to u32
        let string_len_words = (n_bytes + 3) >> 2;

        // first word is length
        let slice_len = 1 + string_len_words as usize;

        let result = self.data.get(..slice_len).and_then(WireStr::new)?;
        self.data = &self.data[slice_len..];

        Some(result)
    }

    /// Reads [`WireStr`] if any's present and converts it to a [`str`].
    pub fn read_str(&mut self) -> Option<&'r str> {
        self.read_wire_str().and_then(|s| s.as_str().ok())
    }
}

/// Conceptually the same as [`MessageHeader`] but without `message_len` field
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct MessageHeaderDesc {
    pub object_id: ObjectId,
    pub opcode: u16,
}

impl From<MessageHeader> for MessageHeaderDesc {
    fn from(value: MessageHeader) -> Self {
        Self {
            object_id: ObjectId::new(value.object_id),
            opcode: value.opcode,
        }
    }
}

/// Helper struct to build wire messages
#[derive(Debug, PartialEq)]
pub struct MessageBuilder<'b> {
    pub buf: &'b mut MessageBuffer,
}

impl<'b> MessageBuilder<'b> {
    /// Makes a new builder from buffer
    pub fn new(buf: &'b mut MessageBuffer) -> Self {
        buf.clear();
        Self { buf }
    }

    /// Builds the message.
    ///
    /// # Errors
    ///
    /// - no header has been written
    pub fn build(self) -> Result<Message<'b>, MessageBuildError> {
        let len = self.buf.non_fd_data.len() * mem::size_of::<u32>();
        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            self.buf
                .non_fd_data
                .get_mut(..HEADER_SIZE_WORDS)
                .ok_or(MessageBuildError::NoHeader)?,
        ));

        header.message_len = len as u16;

        Ok(self.buf.get_message())
    }

    /// Shorthand for `.build()?.send(stream)?`
    pub fn build_send(self, stream: impl AsFd) -> Result<(), MessageBuildError> {
        self.build()?.send(stream)?;
        Ok(())
    }

    fn correct_header(&mut self) {
        if self.buf.non_fd_data.len() < HEADER_SIZE_WORDS {
            self.buf.non_fd_data.resize(HEADER_SIZE_WORDS, 0);
        }
    }

    /// Writes entire header. Equivalent to `.object_id(id).opcode(op)`
    pub fn header(mut self, desc: MessageHeaderDesc) -> Self {
        self.correct_header();

        self.buf.non_fd_data[..HEADER_SIZE_WORDS].copy_from_slice(bytemuck::cast_slice(
            bytemuck::bytes_of(&MessageHeader {
                object_id: desc.object_id.into(),
                opcode: desc.opcode,
                message_len: 0,
            }),
        ));

        self
    }

    /// Sets object id to send requests to or to receive events from.
    pub fn object_id(mut self, value: ObjectId) -> Self {
        self.correct_header();

        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            &mut self.buf.non_fd_data[..HEADER_SIZE_WORDS],
        ));

        header.object_id = value.into();
        self
    }

    /// Sets id for requests and events.
    pub fn opcode(mut self, value: u16) -> Self {
        self.correct_header();

        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            &mut self.buf.non_fd_data[..HEADER_SIZE_WORDS],
        ));

        header.opcode = value;
        self
    }

    /// Writes 32-bit unsigned integer to the message
    pub fn uint(mut self, value: u32) -> Self {
        self.correct_header();
        self.buf.non_fd_data.push(value);
        self
    }

    pub fn file_desc(mut self, value: RawFd) -> Self {
        self.correct_header();
        self.buf.fd_data.push(value);
        self
    }

    /// Writes 32-bit signed integer to the message
    pub fn int(self, value: i32) -> Self {
        self.uint(value as u32)
    }

    /// Writes [`str`] to the message
    pub fn str(mut self, value: &str) -> Self {
        self.correct_header();

        // string with zero-byte suffix padded to u32
        //   = ((len + 1) + 3) / 4 = (len + 4) / 4 = len / 4 + 1
        //   len + 1: add zero to string
        //   (.. + 3) / 4: pad to u32
        let str_len_words = (value.len() >> 2) + 1;
        let cur_buf_len = self.buf.non_fd_data.len();

        self.buf
            .non_fd_data
            .resize(cur_buf_len + 1 + str_len_words, 0);
        self.buf.non_fd_data[cur_buf_len] = value.len() as u32 + 1;

        let dst_slice: &mut [u8] =
            bytemuck::cast_slice_mut(&mut self.buf.non_fd_data[cur_buf_len + 1..]);
        dst_slice[..value.len()].clone_from_slice(value.as_bytes());

        self
    }
}

#[derive(Error, Debug)]
pub enum MessageBuildError {
    #[error("header should be written before message build")]
    NoHeader,

    #[error(transparent)]
    IoError(#[from] rustix::io::Errno),
}
