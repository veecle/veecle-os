//! Trait for serializing SOME/IP data types.

use core::marker::PhantomData;

// Re-export the derive macro.
pub use veecle_os_data_support_someip_macros::Serialize;

use crate::length::LengthField;

/// An error while serializing a SOME/IP payload type.
#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum SerializeError {
    /// The payload slice has too few bytes to parse the SOME/IP payload type.
    #[error("the writer buffer is too small")]
    BufferTooSmall,
    /// The capacity of the storage buffer is exceeded.
    #[error("the capacity of the storage buffer too small")]
    StorageBufferTooSmall,
    /// The length of a dynamic structure is bigger than the length field can encode.
    #[error("a length value is too big for the length field")]
    LengthOverflow,
    /// The minimum capacity of a dynamic type is subceeded.
    #[error("the dynamic data type expected more data")]
    DynamicTypeUnderflow,
    /// The maximum capacity of a dynamic type is exceeded.
    #[error("the capacity of the dynamic data type is too small")]
    DynamicTypeOverflow,
}

/// Writes bytes to an underlying byte-slice.
#[derive(Debug)]
pub struct ByteWriter<'a> {
    /// Slice of bytes the writer writes to.
    buffer: &'a mut [u8],
    /// Writer offset into the slice.
    offset: usize,
}

impl<'a> ByteWriter<'a> {
    /// Creates a new byte writer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, offset: 0 }
    }

    /// Writes a single byte.
    pub fn write_byte(&mut self, byte: u8) -> Result<(), SerializeError> {
        if self.offset >= self.buffer.len() {
            return Err(SerializeError::BufferTooSmall);
        }

        self.buffer[self.offset] = byte;
        self.offset += 1;

        Ok(())
    }

    /// Writes a slice of bytes.
    pub fn write_slice(&mut self, slice: &[u8]) -> Result<(), SerializeError> {
        let remaining = &mut self.buffer[self.offset..];

        if remaining.len() < slice.len() {
            return Err(SerializeError::BufferTooSmall);
        }

        remaining[..slice.len()].copy_from_slice(slice);
        self.offset += slice.len();

        Ok(())
    }

    /// Counts the number of bytes written inside the provided closure.
    pub fn write_counted(
        &mut self,
        mut f: impl FnMut(&mut ByteWriter) -> Result<(), SerializeError>,
    ) -> Result<usize, SerializeError> {
        let offset_before = self.offset;

        f(self)?;

        let offset_after = self.offset;

        Ok(offset_after.saturating_sub(offset_before))
    }

    /// Reserves a length field in the writer. Returns a handle that can be used to write the data later.
    pub fn reserve_length<T>(&mut self) -> Result<ReservedLength<T>, SerializeError>
    where
        T: LengthField,
    {
        let remaining = &mut self.buffer[self.offset..];

        if remaining.len() < core::mem::size_of::<T>() {
            return Err(SerializeError::BufferTooSmall);
        }

        let offset = self.offset;
        self.offset += core::mem::size_of::<T>();

        Ok(ReservedLength {
            offset,
            _marker: PhantomData,
        })
    }

    /// Writes the value of a reserved space.
    pub fn write_length<T>(
        &mut self,
        reserved: ReservedLength<T>,
        value: &T,
    ) -> Result<(), SerializeError>
    where
        T: Serialize,
    {
        let mut reserved_writer =
            ByteWriter::new(&mut self.buffer[reserved.offset..][..core::mem::size_of::<T>()]);

        value.serialize_partial(&mut reserved_writer)
    }
}

/// Represents the reserved space for a length field in the writer.
#[derive(Debug)]
pub struct ReservedLength<T> {
    offset: usize,
    _marker: PhantomData<T>,
}

/// A trait for serializing SOME/IP payload types to a slice of bytes.
pub trait Serialize {
    /// Returns the number of bytes required to store the serialized version of self.
    fn required_length(&self) -> usize;

    /// Serializes to a byte writer.
    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError>;
}

/// An extension trait to expose a nicer API to the user.
pub trait SerializeExt: Sized {
    /// Serializes a SOME/IP payload type to a given slice of bytes using [`Serialize`] and returns the number of
    /// bytes written to the buffer.
    fn serialize<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], SerializeError>;
}

impl<T> SerializeExt for T
where
    T: Serialize,
{
    fn serialize<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], SerializeError> {
        let mut writer = ByteWriter::new(buffer);
        let written = writer.write_counted(|writer| self.serialize_partial(writer))?;
        Ok(&buffer[..written])
    }
}
