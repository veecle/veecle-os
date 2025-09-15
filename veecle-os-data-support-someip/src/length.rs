//! SOME/IP length fields.

use crate::parse::{ByteReader, Parse, ParseError};
use crate::serialize::SerializeError;

mod private {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
}

/// A SOME/IP length field. Can be either [`u8`], [`u16`], or [`u32`].
pub trait LengthField: private::Sealed + Sized {
    /// Parses the length field and returns the length as a [`usize`].
    fn get_length(reader: &mut ByteReader) -> Result<usize, ParseError>;

    /// Creates a length field from the length as [`usize`].
    fn from_length(length: usize) -> Result<Self, SerializeError>;
}

impl LengthField for u8 {
    fn get_length(reader: &mut ByteReader) -> Result<usize, ParseError> {
        Self::parse_partial(reader).map(|length| length as usize)
    }

    fn from_length(length: usize) -> Result<Self, SerializeError> {
        length
            .try_into()
            .map_err(|_| SerializeError::LengthOverflow)
    }
}

impl LengthField for u16 {
    fn get_length(reader: &mut ByteReader) -> Result<usize, ParseError> {
        Self::parse_partial(reader).map(|length| length as usize)
    }

    fn from_length(length: usize) -> Result<Self, SerializeError> {
        length
            .try_into()
            .map_err(|_| SerializeError::LengthOverflow)
    }
}

impl LengthField for u32 {
    fn get_length(reader: &mut ByteReader) -> Result<usize, ParseError> {
        Self::parse_partial(reader).map(|length| length as usize)
    }

    fn from_length(length: usize) -> Result<Self, SerializeError> {
        length
            .try_into()
            .map_err(|_| SerializeError::LengthOverflow)
    }
}

/// A type representing no length field.
#[derive(Debug)]
pub struct NoLengthField;

/// An optional SOME/IP length field. Can be either [`u8`], [`u16`], [`u32`], or [`NoLengthField`].
pub trait OptionalLengthField {
    /// Parses the optional length field and returns an option containing the length as a [`usize`].
    ///
    /// Noop for [`NoLengthField`].
    fn try_get_length(reader: &mut ByteReader) -> Result<Option<usize>, ParseError>;
}

impl<T> OptionalLengthField for T
where
    T: LengthField,
{
    fn try_get_length(reader: &mut ByteReader) -> Result<Option<usize>, ParseError> {
        T::get_length(reader).map(Some)
    }
}

impl OptionalLengthField for NoLengthField {
    fn try_get_length(_: &mut ByteReader) -> Result<Option<usize>, ParseError> {
        Ok(None)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod length_field {
    use super::LengthField;
    use crate::parse::ByteReader;
    use crate::serialize::SerializeError;

    #[test]
    fn parse_u8() {
        const TEST_DATA: &[u8] = &[9];

        let mut reader = ByteReader::new(TEST_DATA);
        assert_eq!(u8::get_length(&mut reader), Ok(9));
    }

    #[test]
    fn parse_u16() {
        const TEST_DATA: &[u8] = &[0, 9];

        let mut reader = ByteReader::new(TEST_DATA);
        assert_eq!(u16::get_length(&mut reader), Ok(9));
    }

    #[test]
    fn parse_u32() {
        const TEST_DATA: &[u8] = &[0, 0, 0, 9];

        let mut reader = ByteReader::new(TEST_DATA);
        assert_eq!(u32::get_length(&mut reader), Ok(9));
    }

    #[test]
    fn from_length_u8() {
        assert_eq!(u8::from_length(u8::MAX as usize), Ok(u8::MAX));
        assert_eq!(
            u8::from_length(u8::MAX as usize + 1),
            Err(SerializeError::LengthOverflow)
        );
    }

    #[test]
    fn from_length_u16() {
        assert_eq!(u16::from_length(u16::MAX as usize), Ok(u16::MAX));
        assert_eq!(
            u16::from_length(u16::MAX as usize + 1),
            Err(SerializeError::LengthOverflow)
        );
    }

    #[test]
    fn from_length_u32() {
        assert_eq!(u32::from_length(u32::MAX as usize), Ok(u32::MAX));
        assert_eq!(
            u32::from_length(u32::MAX as usize + 1),
            Err(SerializeError::LengthOverflow)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod optional_field {
    use super::{NoLengthField, OptionalLengthField};
    use crate::parse::ByteReader;

    #[test]
    fn any() {
        const TEST_DATA: &[u8] = &[0, 0, 0, 9];

        let mut reader = ByteReader::new(TEST_DATA);
        assert_eq!(u32::try_get_length(&mut reader), Ok(Some(9)));
    }

    #[test]
    fn none() {
        const TEST_DATA: &[u8] = &[];

        let mut reader = ByteReader::new(TEST_DATA);
        assert_eq!(NoLengthField::try_get_length(&mut reader), Ok(None));
    }
}
