use bytemuck::{Pod, Zeroable};
use std::{
    io::{self, Read, Write},
    mem,
    str::Utf8Error,
};

pub type ObjectId = u32;
pub type NewId = u32;

/// Message header from wire protocol.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct MessageHeader {
    pub object_id: u32,
    pub request_id: u16,
    pub message_len: u16,
}

/// Represents a message from Wire protocol.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Message {
    pub raw: [u32],
}

impl Message {
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
}

/// Reads a message from the stream
pub fn read_message_into(stream: &mut impl Read, buf: &mut Vec<u32>) -> Result<(), io::Error> {
    const HEADER_SIZE: usize = mem::size_of::<MessageHeader>() / mem::size_of::<u32>();

    buf.resize(HEADER_SIZE, 0);
    stream.read_exact(bytemuck::cast_slice_mut(buf))?;

    let header = bytemuck::from_bytes::<MessageHeader>(bytemuck::cast_slice(&buf));
    let len = header.message_len as usize / mem::size_of::<u32>();

    buf.resize(len, 0);
    stream.read_exact(bytemuck::cast_slice_mut(&mut buf[HEADER_SIZE..]))?;

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
        if raw.is_empty() || mem::size_of_val(raw) - (raw[0] as usize) >= 2 * mem::size_of::<u32>() {
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
        let slice_len = (1 + (n_bytes + 3) / 4) as usize;

        let result = self.data.get(..slice_len).and_then(WireStr::new)?;
        self.data = &self.data[slice_len..];

        Some(result)
    }

    /// Reads [`WireStr`] if any's present and converts it to a [`str`].
    pub fn read_str(&mut self) -> Option<&'r str> {
        self.read_wire_str().and_then(|s| s.as_str().ok())
    }
}
