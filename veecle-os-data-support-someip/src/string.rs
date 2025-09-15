//! SOME/IP string types.

use core::marker::PhantomData;

use crate::length::LengthField;
use crate::parse::{ByteReader, Parse, ParseError};
use crate::serialize::{ByteWriter, Serialize, SerializeError};

/// A fixed length string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedLengthString<'a, const LENGTH: usize> {
    encoded_string: EncodedString<'a>,
}

impl<'a, const LENGTH: usize> FixedLengthString<'a, LENGTH> {
    /// Creates a new [`FixedLengthString`].
    pub fn new(encoded_string: EncodedString<'a>) -> Self {
        Self { encoded_string }
    }

    /// Returns the encoded inner string.
    pub fn get_encoded(&self) -> &EncodedString<'_> {
        &self.encoded_string
    }
}

impl<'a, const LENGTH: usize> Parse<'a> for FixedLengthString<'a, LENGTH> {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        // TODO: Parse an optional string length. We don't know how this is determined yet.
        // TODO: Assert that the read length is the same as LENGTH.
        let mut string_reader = reader.sub_reader(LENGTH)?;
        let encoded_string = EncodedString::parse_partial(&mut string_reader)?;

        Ok(Self { encoded_string })
    }
}

impl<const LENGTH: usize> Serialize for FixedLengthString<'_, LENGTH> {
    fn required_length(&self) -> usize {
        self.encoded_string.required_length()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        self.encoded_string.serialize_partial(byte_writer)
    }
}

/// A dynamic length string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicLengthString<'a, L> {
    encoded_string: EncodedString<'a>,
    _marker: PhantomData<L>,
}

impl<'a, L> DynamicLengthString<'a, L> {
    /// Creates a new [`DynamicLengthString`].
    pub fn new(encoded_string: EncodedString<'a>) -> Self {
        Self {
            encoded_string,
            _marker: PhantomData,
        }
    }

    /// Returns the encoded inner string.
    pub fn get_encoded(&self) -> &EncodedString<'_> {
        &self.encoded_string
    }
}

impl<'a, L> Parse<'a> for DynamicLengthString<'a, L>
where
    L: LengthField,
{
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let length = L::get_length(reader)?;

        // TODO: <https://veecle.atlassian.net/browse/DEV-242>

        let mut string_reader = reader.sub_reader(length)?;
        let encoded_string = EncodedString::parse_partial(&mut string_reader)?;

        Ok(Self {
            encoded_string,
            _marker: PhantomData,
        })
    }
}

impl<L> Serialize for DynamicLengthString<'_, L>
where
    L: LengthField + Serialize,
{
    fn required_length(&self) -> usize {
        core::mem::size_of::<L>() + self.encoded_string.required_length()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        let reserved_length = byte_writer.reserve_length()?;

        let length =
            L::from_length(byte_writer.write_counted(|byte_writer| {
                self.encoded_string.serialize_partial(byte_writer)
            })?)?;

        byte_writer.write_length(reserved_length, &length)
    }
}

/// Trait for working with UTF-16BE and UTF-16LE strings.
pub trait Utf16Str {
    /// Returns a lossy iterator over the characters of the string.
    fn chars_lossy(&self) -> impl Iterator<Item = char>;
}

/// A UTF-16BE encoded string.
#[derive(Debug, Clone)]
pub struct Utf16BeStr<'a> {
    reader: ByteReader<'a>,
}

impl Serialize for Utf16BeStr<'_> {
    fn required_length(&self) -> usize {
        self.reader.len() + 2
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.reader.remaining_slice())?;
        byte_writer.write_slice(&[0x0, 0x0])
    }
}

impl Utf16Str for Utf16BeStr<'_> {
    fn chars_lossy(&self) -> impl Iterator<Item = char> {
        char::decode_utf16(
            self.reader
                .remaining_slice()
                .chunks_exact(2)
                .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]])),
        )
        .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER))
    }
}

impl PartialEq for Utf16BeStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.chars_lossy().eq(other.chars_lossy())
    }
}

impl Eq for Utf16BeStr<'_> {}

/// A UTF-16LE encoded string.
#[derive(Debug, Clone)]
pub struct Utf16LeStr<'a> {
    reader: ByteReader<'a>,
}

impl Serialize for Utf16LeStr<'_> {
    fn required_length(&self) -> usize {
        self.reader.len() + 2
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.reader.remaining_slice())?;
        byte_writer.write_slice(&[0x0, 0x0])
    }
}

impl Utf16Str for Utf16LeStr<'_> {
    fn chars_lossy(&self) -> impl Iterator<Item = char> {
        char::decode_utf16(
            self.reader
                .remaining_slice()
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])),
        )
        .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER))
    }
}

impl PartialEq for Utf16LeStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.chars_lossy().eq(other.chars_lossy())
    }
}

impl Eq for Utf16LeStr<'_> {}

/// Extension trait for creating UTF-8 strings from UTF-16 strings.
pub trait Utf16StrExt {
    /// Returns the number of bytes required to store this string as UTF-8.
    fn utf8_length(&self) -> usize;

    /// Create a UTF-8 string from the UTF-16 encoded string in the provided buffer.
    fn create_str<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a str, SerializeError>;
}

impl<T> Utf16StrExt for T
where
    T: Utf16Str,
{
    fn utf8_length(&self) -> usize {
        self.chars_lossy().map(char::len_utf8).sum()
    }

    fn create_str<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a str, SerializeError> {
        let mut offset = 0;

        for character in self.chars_lossy() {
            if character.len_utf8() > buffer[offset..].len() {
                return Err(SerializeError::StorageBufferTooSmall);
            }

            let written = character.encode_utf8(&mut buffer[offset..]);
            offset += written.len();
        }

        Ok(core::str::from_utf8(&buffer[..offset]).unwrap())
    }
}

/// A string encoded as either UTF-8, UTF-16-BE or UTF-16-LE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodedString<'a> {
    /// A UTF-8 encoded string.
    Utf8(&'a str),
    /// A UTF-16-BE encoded string.
    Utf16Be(Utf16BeStr<'a>),
    /// A UTF-16-LE encoded string.
    Utf16Le(Utf16LeStr<'a>),
}

impl<'a> EncodedString<'a> {
    /// Creates a UTF-8 encoded string.
    pub fn create(text: &'a str) -> Self {
        Self::Utf8(text)
    }

    /// Creates a UTF-16-BE encoded string.
    pub fn create_utf16_be(text: &str, buffer: &'a mut [u8]) -> Result<Self, SerializeError> {
        let mut offset = 0;

        for character in text.chars() {
            let mut character_buffer = [0u16; 2];
            let encoded = character.encode_utf16(&mut character_buffer);

            for character in encoded.iter_mut() {
                if buffer[offset..].len() < 2 {
                    return Err(SerializeError::StorageBufferTooSmall);
                }

                buffer[offset..][..2].copy_from_slice(character.to_be_bytes().as_slice());
                offset += 2;
            }
        }

        let reader = ByteReader::new(&buffer[..offset]);
        Ok(Self::Utf16Be(Utf16BeStr { reader }))
    }

    /// Creates a UTF-16-LE encoded string.
    pub fn create_utf16_le(text: &str, buffer: &'a mut [u8]) -> Result<Self, SerializeError> {
        let mut offset = 0;

        for character in text.chars() {
            let mut character_buffer = [0u16; 2];
            let encoded = character.encode_utf16(&mut character_buffer);

            for character in encoded.iter_mut() {
                if buffer[offset..].len() < 2 {
                    return Err(SerializeError::StorageBufferTooSmall);
                }

                buffer[offset..][..2].copy_from_slice(character.to_le_bytes().as_slice());
                offset += 2;
            }
        }

        let reader = ByteReader::new(&buffer[..offset]);
        Ok(Self::Utf16Le(Utf16LeStr { reader }))
    }
}

// Byte order mark for UTF-8 and UTF-16.
//
// See: https://en.wikipedia.org/wiki/Byte_order_mark
const UTF_8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
const UTF_16_BE_BOM: [u8; 2] = [0xFE, 0xFF];
const UTF_16_LE_BOM: [u8; 2] = [0xFF, 0xFE];

impl<'a> Parse<'a> for EncodedString<'a> {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        if reader.consume_matching_bytes(&UTF_8_BOM) {
            let slice = reader.remaining_slice();

            // UTF-8 strings must be null-terminated.
            let Some(zero_position) = slice.iter().position(|byte| *byte == 0) else {
                return Err(ParseError::PayloadTooShort);
            };

            let str_slice = reader.read_slice(zero_position).unwrap();
            let _zero = reader.read_slice(1).unwrap();

            let str =
                core::str::from_utf8(str_slice).map_err(|_| ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                })?;

            return Ok(Self::Utf8(str));
        }

        if reader.consume_matching_bytes(&UTF_16_BE_BOM) {
            let mut slice = reader.remaining_slice();

            // Odd-length UTF-16 strings: ignore last byte.
            if slice.len() % 2 != 0 {
                slice = &slice[..slice.len() - 1];
            }

            // UTF-16 strings must end with 0x0000 (two null bytes).
            let Some(zero_position) = slice
                .chunks_exact(2)
                .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
                .position(|character| character == 0)
            else {
                return Err(ParseError::PayloadTooShort);
            };

            let str_reader = reader.sub_reader(zero_position * 2).unwrap();
            let _zeroes = reader.read_slice(2).unwrap();

            return Ok(Self::Utf16Be(Utf16BeStr { reader: str_reader }));
        }

        if reader.consume_matching_bytes(&UTF_16_LE_BOM) {
            let mut slice = reader.remaining_slice();

            // Odd-length UTF-16 strings: ignore last byte.
            if slice.len() % 2 != 0 {
                slice = &slice[..slice.len() - 1];
            }

            // UTF-16 strings must end with 0x0000 (two null bytes).
            let Some(zero_position) = slice
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .position(|character| character == 0)
            else {
                return Err(ParseError::PayloadTooShort);
            };

            let str_reader = reader.sub_reader(zero_position * 2).unwrap();
            let _zeroes = reader.read_slice(2).unwrap();

            return Ok(Self::Utf16Le(Utf16LeStr { reader: str_reader }));
        }

        Err(ParseError::MalformedMessage {
            failed_at: core::any::type_name::<Self>(),
        })
    }
}

impl Serialize for EncodedString<'_> {
    fn required_length(&self) -> usize {
        match self {
            EncodedString::Utf8(string) => UTF_8_BOM.len() + string.len() + 1,
            EncodedString::Utf16Be(string) => UTF_16_BE_BOM.len() + string.required_length(),
            EncodedString::Utf16Le(string) => UTF_16_LE_BOM.len() + string.required_length(),
        }
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        match self {
            EncodedString::Utf8(string) => {
                byte_writer.write_slice(&UTF_8_BOM)?;
                byte_writer.write_slice(string.as_bytes())?;
                byte_writer.write_byte(0x0)
            }
            EncodedString::Utf16Be(string) => {
                byte_writer.write_slice(&UTF_16_BE_BOM)?;
                string.serialize_partial(byte_writer)
            }
            EncodedString::Utf16Le(string) => {
                byte_writer.write_slice(&UTF_16_LE_BOM)?;
                string.serialize_partial(byte_writer)
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod fixed_length_string {

    use crate::parse::ParseExt;
    use crate::string::{EncodedString, FixedLengthString};

    #[test]
    fn conversion() {
        const EXPECTED_BYTES: &[u8] = &[
            0xEF, 0xBB, 0xBF, // BOM
            b'T', b'E', b'S', b'T', // Message
            0x0,  // Zero for termination
        ];

        let string = FixedLengthString::<'_, 8>::new(EncodedString::create("TEST"));

        test_round_trip!(FixedLengthString::<'_, 8>, string, EXPECTED_BYTES);
    }

    #[test]
    fn parse_invalid_encoced_string() {
        const TEST_BYTES: &[u8] = &[
            0xEF, 0xFF, 0xFF, // Invalid BOM
            b'T', b'E', b'S', b'T', // Message
            0x0,  // Zero for termination
        ];

        assert!(matches!(
            FixedLengthString::<'_, 8>::parse(TEST_BYTES),
            Err(crate::parse::ParseError::MalformedMessage { .. })
        ));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod dynamic_length_string {

    use crate::parse::ParseExt;
    use crate::serialize::{SerializeError, SerializeExt};
    use crate::string::{DynamicLengthString, EncodedString};

    #[test]
    fn conversion() {
        const EXPECTED_BYTES: &[u8] = &[
            0x0, 0x0, 0x0, 0x8, // Length (u32)
            0xEF, 0xBB, 0xBF, // BOM
            b'T', b'E', b'S', b'T', // Message
            0x0,  // Zero for termination
        ];

        let string = DynamicLengthString::<'_, u32>::new(EncodedString::create("TEST"));

        test_round_trip!(DynamicLengthString::<'_, u32>, string, EXPECTED_BYTES);
    }

    #[test]
    fn parse_invalid_encoced_string() {
        const TEST_BYTES: &[u8] = &[
            0x0, 0x0, 0x0, 0x8, // Length (u32)
            0xEF, 0xFF, 0xFF, // Invalid BOM
            b'T', b'E', b'S', b'T', // Message
            0x0,  // Zero for termination
        ];

        assert!(matches!(
            DynamicLengthString::<'_, u32>::parse(TEST_BYTES),
            Err(crate::parse::ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn serialize_length_overflow() {
        let long_str = core::str::from_utf8(&[b'a'; 256]).unwrap();

        let string = DynamicLengthString::<'_, u8>::new(EncodedString::create(long_str));

        assert_eq!(
            string.serialize(&mut [0; 512]),
            Err(SerializeError::LengthOverflow)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod encoded_string {

    use super::Utf16StrExt;
    use crate::parse::{ByteReader, Parse, ParseError, ParseExt};
    use crate::serialize::SerializeError;
    use crate::string::{EncodedString, Utf16Str};

    #[test]
    fn create_utf8() {
        let string = EncodedString::create("TEßT");
        assert!(matches!(string, EncodedString::Utf8("TEßT")));
    }

    #[test]
    fn create_utf16_be() {
        let mut buffer = [0u8; 8];
        let string = EncodedString::create_utf16_be("TEßT", &mut buffer).unwrap();

        let EncodedString::Utf16Be(string) = string else {
            panic!("incorrect encoded string type");
        };

        let mut buffer = [0u8; 64];
        let string = string.create_str(&mut buffer).unwrap();

        assert_eq!(string, "TEßT");
    }

    #[test]
    fn create_utf16_be_insufficient_buffer() {
        let mut buffer = [0u8; 7];

        assert_eq!(
            EncodedString::create_utf16_be("TEßT", &mut buffer),
            Err(SerializeError::StorageBufferTooSmall)
        );
    }

    #[test]
    fn create_utf16_le() {
        let mut buffer = [0u8; 8];
        let string = EncodedString::create_utf16_le("TEßT", &mut buffer).unwrap();

        let EncodedString::Utf16Le(string) = string else {
            panic!("incorrect encoded string type");
        };

        let mut buffer = [0u8; 64];
        let string = string.create_str(&mut buffer).unwrap();

        assert_eq!(string, "TEßT");
    }

    #[test]
    fn create_utf16_le_insufficient_buffer() {
        let mut buffer = [0u8; 7];

        assert_eq!(
            EncodedString::create_utf16_le("TEßT", &mut buffer),
            Err(SerializeError::StorageBufferTooSmall)
        );
    }

    #[test]
    fn utf8_valid() {
        const EXPECTED_BYTES: &[u8] = &[
            0xEF, 0xBB, 0xBF, // BOM
            b'T', b'E', b'S', b'T', // Message
            0x0,  // Zero for termination
        ];

        let encoded = EncodedString::Utf8("TEST");

        test_round_trip!(EncodedString, encoded, EXPECTED_BYTES);
    }

    #[test]
    fn parse_utf8_invalid() {
        const BYTES: &[u8] = &[
            0xEF, 0xBB, 0xBF, // BOM
            0xE2, 0x28, 0xA1, // Invalid UTF-8 sequence
            0x0,  // Zero for termination
        ];

        assert!(matches!(
            EncodedString::parse(BYTES),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn utf16_be_valid() {
        const EXPECTED_BYTES: &[u8] = &[
            0xFE, 0xFF, // BOM
            0x0, b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', // Message
            0x0, 0x0, // Zero for termination
        ];

        let mut buffer = [0u8; 64];
        let encoded = EncodedString::create_utf16_be("TEST", &mut buffer).unwrap();

        test_round_trip!(EncodedString, encoded, EXPECTED_BYTES);
    }

    #[test]
    fn parse_utf16_be_uneven_length() {
        const BYTES: &[u8] = &[
            0xFE, 0xFF, // BOM
            0x0, b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', // Message
            0x0, 0x0,  // Zero for termination
            0xFF, // Incorrect bytes
        ];

        const EXPECTED_CHARACTERS: &[char] = &['T', 'E', 'S', 'T'];

        let mut reader = ByteReader::new(BYTES);

        let Ok(EncodedString::Utf16Be(string)) = EncodedString::parse_partial(&mut reader) else {
            panic!("failed to parse UTF-16BE string");
        };

        assert!(
            string.chars_lossy().eq(EXPECTED_CHARACTERS.iter().copied()),
            "expected string doesn't match"
        );
    }

    #[test]
    fn parse_utf16_le_valid() {
        const EXPECTED_BYTES: &[u8] = &[
            0xFF, 0xFE, // BOM
            b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', 0x0, // Message
            0x0, 0x0, // Zero for termination
        ];

        let mut buffer = [0u8; 64];
        let encoded = EncodedString::create_utf16_le("TEST", &mut buffer).unwrap();

        test_round_trip!(EncodedString, encoded, EXPECTED_BYTES);
    }

    #[test]
    fn parse_utf16_le_uneven_length() {
        const BYTES: &[u8] = &[
            0xFF, 0xFE, // BOM
            b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', 0x0, // Message
            0x0, 0x0,  // Zero for termination
            0xFF, // Incorrect bytes
        ];

        const EXPECTED_CHARACTERS: &[char] = &['T', 'E', 'S', 'T'];

        let mut reader = ByteReader::new(BYTES);

        let Ok(EncodedString::Utf16Le(string)) = EncodedString::parse_partial(&mut reader) else {
            panic!("failed to parse UTF-16LE string");
        };

        assert!(
            string.chars_lossy().eq(EXPECTED_CHARACTERS.iter().copied()),
            "expected string doesn't match"
        );
    }

    #[test]
    fn invali_bom() {
        const BYTES: &[u8] = &[
            0xEF, 0xBB, 0xBB, // BOM
        ];

        let mut reader = ByteReader::new(BYTES);

        assert!(matches!(
            EncodedString::parse_partial(&mut reader),
            Err(ParseError::MalformedMessage { .. })
        ));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod utf16_ext {
    use crate::parse::{ByteReader, Parse};
    use crate::serialize::SerializeError;
    use crate::string::{EncodedString, Utf16StrExt};

    #[test]
    fn utf16_required_length() {
        const BYTES: &[u8] = &[
            0xFE, 0xFF, // BOM
            0x0, b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', // Message
            0x0, 0x0, // Zero for termination
        ];

        let mut reader = ByteReader::new(BYTES);

        let Ok(EncodedString::Utf16Be(string)) = EncodedString::parse_partial(&mut reader) else {
            panic!("failed to decode UTF-16BE string");
        };

        assert_eq!(string.utf8_length(), 4);
    }

    #[test]
    fn utf16_create_str() {
        const BYTES: &[u8] = &[
            0xFE, 0xFF, // BOM
            0x0, b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', // Message
            0x0, 0x0, // Zero for termination
        ];

        let mut reader = ByteReader::new(BYTES);

        let Ok(EncodedString::Utf16Be(string)) = EncodedString::parse_partial(&mut reader) else {
            panic!("failed to decode UTF-16BE string");
        };

        let mut buffer = [0u8; 4];
        let utf8_string = string.create_str(&mut buffer).unwrap();

        assert_eq!(utf8_string, "TEST");
    }

    #[test]
    fn utf16_create_str_insufficient_buffer() {
        const BYTES: &[u8] = &[
            0xFE, 0xFF, // BOM
            0x0, b'T', 0x0, b'E', 0x0, b'S', 0x0, b'T', // Message
            0x0, 0x0, // Zero for termination
        ];

        let mut reader = ByteReader::new(BYTES);

        let Ok(EncodedString::Utf16Be(string)) = EncodedString::parse_partial(&mut reader) else {
            panic!("failed to decode UTF-16BE string");
        };

        let mut buffer = [0u8; 3];
        assert_eq!(
            string.create_str(&mut buffer),
            Err(SerializeError::StorageBufferTooSmall)
        );
    }
}
