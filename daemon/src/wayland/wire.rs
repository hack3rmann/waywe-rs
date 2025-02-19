use super::object::ObjectId;
use bytemuck::{Pod, Zeroable};
use std::{
    io::{self, Read, Write},
    mem,
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Default, Eq, PartialOrd, Ord, Hash)]
pub struct MessageBuffer(pub(crate) Vec<u32>);

impl MessageBuffer {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(n_bytes: usize) -> Self {
        // (.. + 3) / 4 pads string to u32
        let n_words = (n_bytes + 3) >> 2;
        Self(Vec::with_capacity(n_words))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.0
    }

    pub fn get_message(&self) -> &Message {
        Message::from_u32_slice(self.as_slice())
    }
}

/// Message header from wire protocol.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct MessageHeader {
    pub object_id: u32,
    pub opcode: u16,
    pub message_len: u16,
}

const HEADER_SIZE_WORDS: usize = mem::size_of::<MessageHeader>() / mem::size_of::<u32>();

/// Represents a message from Wire protoc#ol.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Message {
    pub raw: [u32],
}

impl Message {
    pub fn reader(&self) -> MessageReader<'_> {
        MessageReader::new(self)
    }

    /// Cast the message to a [`u32`] slice.
    pub fn as_u32_slice(&self) -> &[u32] {
        unsafe { mem::transmute(self) }
    }

    /// Cast the message to a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self.as_u32_slice())
    }

    /// Creates a message from raw [`u32`] slice.
    ///
    /// # Panic
    ///
    /// Panics if `mem::size_of_val(src) < mem::size_of::<MessageHeader>()`.
    pub fn from_u32_slice(src: &[u32]) -> &Self {
        // TODO: validate message + return Result type
        assert!(mem::size_of_val(src) >= mem::size_of::<MessageHeader>());
        unsafe { mem::transmute(src) }
    }

    /// Message header
    pub fn header(&self) -> MessageHeader {
        *bytemuck::from_bytes(bytemuck::cast_slice(
            &self.raw[..mem::size_of::<MessageHeader>() / mem::size_of::<u32>()],
        ))
    }

    /// Message body (header removed)
    pub fn body(&self) -> &[u32] {
        &self.raw[mem::size_of::<MessageHeader>() / mem::size_of::<u32>()..]
    }

    /// Message length in bytes
    pub fn len(&self) -> usize {
        self.header().message_len as usize
    }

    pub fn send<S: Write + ?Sized>(&self, stream: &mut S) -> Result<(), io::Error> {
        stream.write_all(self.as_bytes())
    }

    pub fn builder(buf: &mut MessageBuffer) -> MessageBuilder {
        MessageBuilder::new(buf)
    }
}

/// Reads a message from the stream
pub fn read_message_into<S: Read + ?Sized>(
    stream: &mut S,
    buf: &mut MessageBuffer,
) -> Result<(), io::Error> {
    buf.0.resize(HEADER_SIZE_WORDS, 0);
    stream.read_exact(bytemuck::cast_slice_mut(&mut buf.0))?;

    let header = bytemuck::from_bytes::<MessageHeader>(bytemuck::cast_slice(&buf.0));
    let len = header.message_len as usize / mem::size_of::<u32>();

    buf.0.resize(len, 0);
    stream.read_exact(bytemuck::cast_slice_mut(&mut buf.0[HEADER_SIZE_WORDS..]))?;

    Ok(())
}

/// Writes a message to the stream
pub fn write_message(stream: &mut impl Write, message: &Message) -> Result<(), io::Error> {
    assert_eq!(
        message.len(),
        mem::size_of::<MessageHeader>() + mem::size_of::<u32>() * message.body().len()
    );

    stream.write_all(message.as_bytes())
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

        Some(unsafe { mem::transmute(raw) })
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
    pub fn new(message: &'r Message) -> Self {
        Self {
            data: bytemuck::cast_slice(message.body()),
        }
    }

    /// Reads [`u32`] from message if any's present
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.data.len() == 0 {
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
        if self.data.len() == 0 {
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

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct MessageHeaderDesc {
    pub object_id: ObjectId,
    pub opcode: u16,
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
    pub fn build(self) -> Result<&'b Message, MessageBuildError> {
        let len = self.buf.0.len() * mem::size_of::<u32>();
        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            self.buf
                .0
                .get_mut(..HEADER_SIZE_WORDS)
                .ok_or(MessageBuildError::NoHeader)?,
        ));

        header.message_len = len as u16;

        dbg!(bytemuck::cast_slice::<u32, u8>(self.buf.0.as_slice()));

        Ok(self.buf.get_message())
    }

    /// Shorthand for `.build()?.send(stream)?`
    pub fn build_send(self, stream: &mut impl Write) -> Result<(), MessageBuildError> {
        self.build()?.send(stream)?;
        Ok(())
    }

    fn correct_header(&mut self) {
        if self.buf.0.len() < HEADER_SIZE_WORDS {
            self.buf.0.resize(HEADER_SIZE_WORDS, 0);
        }
    }

    /// Writes entire header. Equivalent to `.object_id(id).opcode(op)`
    pub fn header(mut self, desc: MessageHeaderDesc) -> Self {
        self.correct_header();

        self.buf.0[..HEADER_SIZE_WORDS].copy_from_slice(bytemuck::cast_slice(bytemuck::bytes_of(
            &MessageHeader {
                object_id: desc.object_id.into(),
                opcode: desc.opcode,
                message_len: 0,
            },
        )));

        self
    }

    /// Sets object id to send requests to or to receive events from.
    pub fn object_id(mut self, value: ObjectId) -> Self {
        self.correct_header();

        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            &mut self.buf.0[..HEADER_SIZE_WORDS],
        ));

        header.object_id = value.into();
        self
    }

    /// Sets id for requests and events.
    pub fn opcode(mut self, value: u16) -> Self {
        self.correct_header();

        let header = bytemuck::from_bytes_mut::<MessageHeader>(bytemuck::cast_slice_mut(
            &mut self.buf.0[..HEADER_SIZE_WORDS],
        ));

        header.opcode = value;
        self
    }

    /// Writes 32-bit unsigned integer to the message
    pub fn uint(mut self, value: u32) -> Self {
        self.correct_header();
        self.buf.0.push(value);
        self
    }

    /// Writes 32-bit signed integer to the message
    pub fn int(self, value: i32) -> Self {
        self.uint(value as u32)
    }

    pub fn str(mut self, value: &str) -> Self {
        self.correct_header();

        // string with zero-byte suffix padded to u32
        //   = ((len + 1) + 3) / 4 = (len + 4) / 4 = len / 4 + 1
        //   len + 1: add zero to string
        //   (.. + 3) / 4: pad to u32
        let str_len_words = (value.len() >> 2) + 1;
        let cur_buf_len = self.buf.0.len();

        self.buf.0.resize(cur_buf_len + 1 + str_len_words, 0);
        self.buf.0[cur_buf_len] = value.len() as u32 + 1;

        let dst_slice: &mut [u8] = bytemuck::cast_slice_mut(&mut self.buf.0[cur_buf_len + 1..]);
        dst_slice[..value.len()].clone_from_slice(value.as_bytes());

        self
    }
}

#[derive(Error, Debug)]
pub enum MessageBuildError {
    #[error("header should be written before message build")]
    NoHeader,

    #[error(transparent)]
    IoError(#[from] io::Error),
}
