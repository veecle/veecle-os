//! Trait for parsing of SOME/IP data types.

// Re-export the derive macro.
pub use veecle_os_data_support_someip_macros::Parse;

/// An error while parsing a SOME/IP payload type.
#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    /// The payload slice has too few bytes to parse the SOME/IP payload type.
    #[error("the payload is too short")]
    PayloadTooShort,
    /// The payload slice has more bytes than the SOME/IP payload type expected.
    #[error("the payload is too long. Expected {expected} found {found}")]
    PayloadTooLong {
        /// Expected payload size.
        expected: usize,
        /// Provided payload size.
        found: usize,
    },
    /// The message is malformed.
    #[error("malformed message. Failed to parse {failed_at}")]
    MalformedMessage {
        /// Name of the type that was malformed.
        failed_at: &'static str,
    },
}

/// Reads bytes from an underlying byte-slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteReader<'a> {
    /// Slice of bytes the reader reads from.
    data: &'a [u8],
    /// Reader offset into the slice.
    offset: usize,
}

impl<'a> ByteReader<'a> {
    /// Creates a new reader for a slice of bytes.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Creates a second reader for a sub-slice of this reader. The slice of the second reader starts at the current
    /// `offset` and ends at `offset + length`. Also advances this readers offset by `length`.
    ///
    /// A lot of variable length SOME/IP types are prefixed with their length in bytes, so this allows creating a
    /// sub-reader and reading bytes until it [is empty](Self::is_empty).
    pub fn sub_reader(&mut self, length: usize) -> Result<Self, ParseError> {
        let Some(new_offset) = self.offset.checked_add(length) else {
            return Err(ParseError::PayloadTooShort);
        };

        if new_offset > self.data.len() {
            return Err(ParseError::PayloadTooShort);
        }

        let current_offset = self.offset;
        self.offset = new_offset;

        let data = &self.data[current_offset..self.offset];

        Ok(Self { offset: 0, data })
    }

    /// Returns a new sub-reader with the remaining slice and advances the reader.
    pub fn take_remaining(&mut self) -> Self {
        let data = &self.data[self.offset..];

        self.offset = self.data.len();

        Self { data, offset: 0 }
    }

    /// Reads a single byte and advances the reader.
    pub fn read_byte(&mut self) -> Result<u8, ParseError> {
        if self.offset >= self.data.len() {
            return Err(ParseError::PayloadTooShort);
        }

        let byte = self.data[self.offset];

        self.offset += 1;

        Ok(byte)
    }

    /// Returns a slice of bytes with the given length and advances the reader.
    pub fn read_slice(&mut self, length: usize) -> Result<&'a [u8], ParseError> {
        let Some(new_offset) = self.offset.checked_add(length) else {
            return Err(ParseError::PayloadTooShort);
        };

        if new_offset > self.data.len() {
            return Err(ParseError::PayloadTooShort);
        }

        let result = &self.data[self.offset..self.offset + length];

        self.offset += length;

        Ok(result)
    }

    /// Reads an array of `N` bytes and advances the reader.
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ParseError> {
        let Some(new_offset) = self.offset.checked_add(N) else {
            return Err(ParseError::PayloadTooShort);
        };

        if new_offset > self.data.len() {
            return Err(ParseError::PayloadTooShort);
        }

        let result = self.data[self.offset..self.offset + N]
            .try_into()
            .expect("returned slice should always be N bytes long");

        self.offset += N;

        Ok(result)
    }

    /// Returns the remaining slice without advancing the offset.
    pub fn remaining_slice(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }

    /// Consumes bytes matching the provided input. Returns whether or not there was a match.
    ///
    /// Returns false if are there not enough bytes to compare to.
    pub fn consume_matching_bytes(&mut self, compare_to: &[u8]) -> bool {
        let length = compare_to.len();

        let Some(new_offset) = self.offset.checked_add(length) else {
            return false;
        };

        if new_offset > self.data.len() || &self.data[self.offset..][..length] != compare_to {
            return false;
        }

        self.offset += length;

        true
    }

    /// Returns the length of the remaining slice.
    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    /// Returns true if there are no bytes left to read.
    pub fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }
}

/// A trait for parsing SOME/IP payload types from a slice of bytes.
pub trait Parse<'a>: Sized {
    /// Parses a SOME/IP payload type from a given slice of bytes.
    ///
    /// For parsing using the entire slice, see [`ParseExt`].
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError>;
}

/// An extension trait to expose a nicer API to the user.
pub trait ParseExt<'a>: Sized {
    /// Parses a SOME/IP payload type from a given slice of bytes using [`Parse`] and
    /// validates all the bytes of the slice were used during parsing.
    fn parse(slice: &'a [u8]) -> Result<Self, ParseError>;
}

impl<'a, T> ParseExt<'a> for T
where
    T: Parse<'a>,
{
    fn parse(slice: &'a [u8]) -> Result<Self, ParseError> {
        let mut reader = ByteReader::new(slice);
        let this = Self::parse_partial(&mut reader)?;

        match reader.is_empty() {
            true => Ok(this),
            false => Err(ParseError::PayloadTooLong {
                expected: reader.offset,
                found: slice.len(),
            }),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod parse_ext {
    use pretty_assertions::assert_eq;

    use super::{ParseError, ParseExt};

    #[test]
    fn payload_too_long() {
        assert_eq!(
            u32::parse(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            Err(ParseError::PayloadTooLong {
                expected: 4,
                found: 5
            })
        );
    }
}
