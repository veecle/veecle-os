//! Implementation of [`Parse`] for various data types.

use super::parse::{Parse, ParseError};
use crate::parse::ByteReader;

impl<'a> Parse<'a> for bool {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let byte = reader.read_byte()?;

        // Only 0 and 1 are valid values for a boolean.
        if byte > 1 {
            return Err(ParseError::MalformedMessage {
                failed_at: core::any::type_name::<Self>(),
            });
        }

        Ok(byte == 1)
    }
}

macro_rules! impl_for_numeric {
    ($ty:ident) => {
        impl<'a> Parse<'a> for $ty {
            fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
                Ok($ty::from_be_bytes(reader.read_array()?))
            }
        }
    };
}

impl_for_numeric!(u8);
impl_for_numeric!(u16);
impl_for_numeric!(u32);
impl_for_numeric!(u64);

impl_for_numeric!(i8);
impl_for_numeric!(i16);
impl_for_numeric!(i32);
impl_for_numeric!(i64);

impl_for_numeric!(f32);
impl_for_numeric!(f64);

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod parse {
    use crate::parse::{ParseError, ParseExt};

    #[test]
    fn bool() {
        test_round_trip!(bool, true, &[1]);
        test_round_trip!(bool, false, &[0]);
    }

    #[test]
    fn malformed_bool() {
        assert_eq!(
            bool::parse(&[2]),
            Err(ParseError::MalformedMessage { failed_at: "bool" })
        );
    }

    #[test]
    fn u8() {
        test_round_trip!(u8, 99, &[99]);
    }

    #[test]
    fn u16() {
        test_round_trip!(u16, 0x40A, &[0x4, 0xA]);
    }

    #[test]
    fn u32() {
        test_round_trip!(u32, 0x40A0834, &[0x4, 0xA, 0x8, 0x34]);
    }

    #[test]
    fn u64() {
        test_round_trip!(
            u64,
            0x40A083410090807,
            &[0x4, 0xA, 0x8, 0x34, 0x10, 0x9, 0x8, 0x7]
        );
    }

    #[test]
    fn i8() {
        test_round_trip!(i8, -1, &[0xFF]);
    }

    #[test]
    fn i16() {
        test_round_trip!(i16, -246, &[0xFF, 0xA]);
    }

    #[test]
    fn i32() {
        test_round_trip!(i32, -16119756, &[0xFF, 0xA, 0x8, 0x34]);
    }

    #[test]
    fn i64() {
        test_round_trip!(
            i64,
            -69233824570472441,
            &[0xFF, 0xA, 0x8, 0x34, 0x10, 0x9, 0x8, 0x7]
        );
    }

    #[test]
    fn f32() {
        test_round_trip!(f32, 9.21298e-40, &[0x0, 0xA, 0x8, 0x34]);
    }

    #[test]
    fn f64() {
        test_round_trip!(
            f64,
            1.395127485645192e-308,
            &[0x0, 0xA, 0x8, 0x34, 0x10, 0x9, 0x8, 0x7]
        );
    }
}
