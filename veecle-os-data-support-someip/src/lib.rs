//! Support for working with SOME/IP within a runtime instance.

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(test)]
macro_rules! test_round_trip {
    ($type:ty, $value:expr, $expected:expr) => {
        let value = $value;

        // Serialize valid.
        let mut buffer = [0u8; 2048];
        let buffer_length = crate::serialize::Serialize::required_length(&value);
        let serialized_data = crate::serialize::SerializeExt::serialize(&value, &mut buffer);

        assert!(matches!(serialized_data, Ok(..)));

        let serialized_data = serialized_data.unwrap();

        // Required length.

        assert_eq!(serialized_data.len(), buffer_length);

        assert_eq!(serialized_data, $expected);

        // Parse valid.

        let parsed = <$type as crate::parse::ParseExt>::parse(serialized_data);

        assert!(matches!(parsed, Ok(..)));
        assert_eq!(parsed.unwrap(), value);

        // Serialize too short
        for cut_off in 0..serialized_data.len().saturating_sub(1) {
            let mut buffer = [0u8; 2048];

            assert!(matches!(
                crate::serialize::SerializeExt::serialize(&value, &mut buffer[..cut_off]),
                Err(crate::serialize::SerializeError::BufferTooSmall)
            ));
        }

        // Parse too short
        for cut_off in 0..serialized_data.len().saturating_sub(1) {
            assert!(matches!(
                <$type as crate::parse::ParseExt>::parse(&serialized_data[..cut_off]),
                Err(crate::parse::ParseError::PayloadTooShort)
                    | Err(crate::parse::ParseError::MalformedMessage { .. })
            ));
        }

        // Parse empty
        assert!(matches!(
            <$type as crate::parse::ParseExt>::parse(&[]),
            Err(crate::parse::ParseError::PayloadTooShort)
                | Err(crate::parse::ParseError::MalformedMessage { .. })
        ));
    };
}

pub mod array;
pub mod header;
pub mod length;
pub mod parse;
pub mod parse_impl;
pub mod serialize;
pub mod serialize_impl;
pub mod service_discovery;
pub mod string;

// Make `Parse` derive macro work inside this crate.
// This is required because the macro expects the `veecle_os_data_support_someip` crate to be imported.
extern crate self as veecle_os_data_support_someip;
