// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.
// Copyright 2025 Veecle GmbH.
//
// This file has been modified from the original TiKV implementation.

//! Unique identifiers for traces and spans.
//!
//! This module provides the core identifier types used throughout the telemetry system
//! to uniquely identify traces and spans in distributed tracing scenarios.
//!
//! # Core Types
//!
//! - [`SpanId`]: An identifier that uniquely identifies a span within a process.
//! - [`SpanContext`]: A combination of process id and span id that uniquely identifies a span globally.

use core::fmt;
use core::str::FromStr;

/// A globally-unique id identifying a process.
///
/// The primary purpose of this id is to provide a globally-unique context within which
/// [`ThreadId`]s and [`SpanContext`]s are guaranteed to be unique. On a normal operating system
/// that is the process, on other systems it should be whatever is the closest equivalent, e.g. for
/// most embedded setups it should be unique for each time the system is restarted.
///
/// [`ThreadId`]: crate::protocol::ThreadId
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct ProcessId(u128);

impl ProcessId {
    /// Uses a random number generator to generate the [`ProcessId`].
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        Self(rng.random())
    }

    /// Creates a [`ProcessId`] from a raw value
    ///
    /// Extra care needs to be taken that this is not a constant value or re-used in any way.
    ///
    /// When possible prefer using [`ProcessId::random`].
    pub const fn from_raw(raw: u128) -> Self {
        Self(raw)
    }

    /// Returns the raw value of this id.
    pub fn to_raw(self) -> u128 {
        self.0
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl FromStr for ProcessId {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u128::from_str_radix(s, 16).map(ProcessId)
    }
}

impl serde::Serialize for ProcessId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut hex_bytes = [0u8; size_of::<u128>() * 2];
        hex::encode_to_slice(self.0.to_le_bytes(), &mut hex_bytes).unwrap();

        serializer.serialize_str(str::from_utf8(&hex_bytes).unwrap())
    }
}

impl<'de> serde::Deserialize<'de> for ProcessId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: [u8; size_of::<u128>()] = hex::serde::deserialize(deserializer)?;

        Ok(ProcessId(u128::from_le_bytes(bytes)))
    }
}

/// A process-unique id for a span.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpanId(pub u64);

#[cfg(feature = "enable")]
impl SpanId {
    #[inline]
    #[doc(hidden)]
    /// Creates a non-zero [`SpanId`].
    pub fn next_id() -> Self {
        use core::sync::atomic;
        static COUNTER: atomic::AtomicU64 = atomic::AtomicU64::new(1);
        SpanId(COUNTER.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl FromStr for SpanId {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(SpanId)
    }
}

impl serde::Serialize for SpanId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut hex_bytes = [0u8; size_of::<u64>() * 2];
        hex::encode_to_slice(self.0.to_le_bytes(), &mut hex_bytes).unwrap();

        serializer.serialize_str(str::from_utf8(&hex_bytes).unwrap())
    }
}

impl<'de> serde::Deserialize<'de> for SpanId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: [u8; size_of::<u64>()] = hex::serde::deserialize(deserializer)?;

        Ok(SpanId(u64::from_le_bytes(bytes)))
    }
}

/// A struct representing the context of a span, including its [`ProcessId`] and [`SpanId`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpanContext {
    /// The id of the process this span belongs to.
    pub process_id: ProcessId,
    /// The unique id of this span.
    pub span_id: SpanId,
}

impl SpanContext {
    /// Creates a new `SpanContext` with the given [`ProcessId`] and [`SpanId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::{ProcessId, SpanId, SpanContext};
    ///
    /// let span_context = SpanContext::new(ProcessId::from_raw(12), SpanId(13));
    /// ```
    pub fn new(process_id: ProcessId, span_id: SpanId) -> Self {
        Self {
            process_id,
            span_id,
        }
    }
}

impl fmt::Display for SpanContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            process_id,
            span_id,
        } = self;
        write!(f, "{process_id}:{span_id}")
    }
}

/// Errors that can occur while parsing [`SpanContext`] from a string.
#[derive(Clone, Debug)]
pub enum ParseSpanContextError {
    /// The string is missing a `:` separator.
    MissingSeparator,

    /// The embedded [`ProcessId`] failed to parse.
    InvalidProcessId(core::num::ParseIntError),

    /// The embedded [`SpanId`] failed to parse.
    InvalidSpanId(core::num::ParseIntError),
}

impl fmt::Display for ParseSpanContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => f.write_str("missing ':' separator"),
            Self::InvalidProcessId(_) => f.write_str("failed to parse process id"),
            Self::InvalidSpanId(_) => f.write_str("failed to parse span id"),
        }
    }
}

impl core::error::Error for ParseSpanContextError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::MissingSeparator => None,
            Self::InvalidProcessId(error) => Some(error),
            Self::InvalidSpanId(error) => Some(error),
        }
    }
}

impl FromStr for SpanContext {
    type Err = ParseSpanContextError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((process_id, span_id)) = s.split_once(":") else {
            return Err(ParseSpanContextError::MissingSeparator);
        };
        let process_id =
            ProcessId::from_str(process_id).map_err(ParseSpanContextError::InvalidProcessId)?;
        let span_id = SpanId::from_str(span_id).map_err(ParseSpanContextError::InvalidSpanId)?;
        Ok(Self {
            process_id,
            span_id,
        })
    }
}

impl serde::Serialize for SpanContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes = [0u8; 49];

        hex::encode_to_slice(self.process_id.to_raw().to_le_bytes(), &mut bytes[..32]).unwrap();
        bytes[32] = b':';
        hex::encode_to_slice(self.span_id.0.to_le_bytes(), &mut bytes[33..]).unwrap();

        serializer.serialize_str(str::from_utf8(&bytes).unwrap())
    }
}

impl<'de> serde::Deserialize<'de> for SpanContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let string = <&str>::deserialize(deserializer)?;

        if string.len() != 49 {
            return Err(D::Error::invalid_length(
                string.len(),
                &"expected 49 byte string",
            ));
        }

        let bytes = string.as_bytes();

        if bytes[32] != b':' {
            return Err(D::Error::invalid_value(
                serde::de::Unexpected::Str(string),
                &"expected : separator at byte 32",
            ));
        }

        let mut process = [0; 16];
        hex::decode_to_slice(&bytes[..32], &mut process).map_err(D::Error::custom)?;

        let mut span = [0; 8];
        hex::decode_to_slice(&bytes[33..], &mut span).map_err(D::Error::custom)?;

        Ok(Self {
            process_id: ProcessId::from_raw(u128::from_le_bytes(process)),
            span_id: SpanId(u64::from_le_bytes(span)),
        })
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use std::collections::HashSet;
    use std::format;
    use std::string::String;
    use std::vec::Vec;

    use super::*;

    #[test]
    #[cfg(not(miri))] // VERY slow with Miri.
    #[allow(clippy::needless_collect)]
    fn unique_id() {
        let handles = std::iter::repeat_with(|| {
            std::thread::spawn(|| {
                std::iter::repeat_with(SpanId::next_id)
                    .take(1000)
                    .collect::<Vec<_>>()
            })
        })
        .take(32)
        .collect::<Vec<_>>();

        let k = handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect::<HashSet<_>>();

        assert_eq!(k.len(), 32 * 1000);
    }

    #[test]
    fn span_id_formatting() {
        assert_eq!(format!("{}", SpanId(0)), "0000000000000000");
        assert_eq!(format!("{}", SpanId(u64::MAX)), "ffffffffffffffff");
        assert_eq!(
            format!("{}", SpanId(0xFEDCBA9876543210)),
            "fedcba9876543210"
        );
        assert_eq!(format!("{}", SpanId(0x123)), "0000000000000123");
    }

    #[test]
    fn span_id_from_str() {
        assert_eq!(
            "fedcba9876543210".parse::<SpanId>().unwrap(),
            SpanId(0xFEDCBA9876543210)
        );
        assert_eq!(
            "FEDCBA9876543210".parse::<SpanId>().unwrap(),
            SpanId(0xFEDCBA9876543210)
        );
        assert_eq!("0000000000000000".parse::<SpanId>().unwrap(), SpanId(0));
        assert_eq!(
            "ffffffffffffffff".parse::<SpanId>().unwrap(),
            SpanId(u64::MAX)
        );
        assert_eq!("123".parse::<SpanId>().unwrap(), SpanId(0x123));

        assert!("xyz".parse::<SpanId>().is_err());
        assert!("".parse::<SpanId>().is_err());
    }

    #[test]
    fn span_id_format_from_str_roundtrip() {
        let test_cases = [0u64, 1, 0x123, 0xFEDCBA9876543210, u64::MAX, u64::MAX - 1];

        for value in test_cases {
            let span_id = SpanId(value);
            let formatted = format!("{span_id}");
            let parsed = formatted.parse::<SpanId>().unwrap();
            assert_eq!(span_id, parsed, "Failed roundtrip for value {value:#x}");
        }
    }

    #[test]
    fn span_id_serde_roundtrip() {
        let test_cases = [
            SpanId(0),
            SpanId(1),
            SpanId(0x123),
            SpanId(0xFEDCBA9876543210),
            SpanId(u64::MAX),
            SpanId(u64::MAX - 1),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: SpanId = serde_json::from_str(&json).unwrap();
            assert_eq!(
                original, deserialized,
                "JSON roundtrip failed for {:#x}",
                original.0
            );
        }
    }

    #[test]
    fn span_context_serde_roundtrip() {
        let test_cases = [
            SpanContext::new(ProcessId::from_raw(0), SpanId(0)),
            SpanContext::new(
                ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
                SpanId(0xFEDCBA9876543210),
            ),
            SpanContext::new(ProcessId::from_raw(u128::MAX), SpanId(u64::MAX)),
            SpanContext::new(ProcessId::from_raw(1), SpanId(1)),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: SpanContext = serde_json::from_str(&json).unwrap();
            assert_eq!(
                original.process_id, deserialized.process_id,
                "JSON roundtrip failed for process_id"
            );
            assert_eq!(
                original.span_id, deserialized.span_id,
                "JSON roundtrip failed for span_id"
            );
        }
    }

    #[test]
    fn span_id_serialization_format() {
        let span_id = SpanId(0xFEDCBA9876543210);
        let json = serde_json::to_string(&span_id).unwrap();

        let expected_le_bytes = 0xFEDCBA9876543210u64.to_le_bytes();
        let mut expected_hex = String::new();
        for byte in &expected_le_bytes {
            expected_hex.push_str(&format!("{byte:02x}"));
        }
        let expected_json = format!("\"{expected_hex}\"");

        assert_eq!(json, expected_json);
    }

    #[test]
    fn span_context_new_and_fields() {
        let process_id = ProcessId::from_raw(0x123);
        let span_id = SpanId(0x456);
        let context = SpanContext::new(process_id, span_id);

        assert_eq!(context.process_id, process_id);
        assert_eq!(context.span_id, span_id);
    }

    #[test]
    fn process_id_format_from_str_roundtrip() {
        let test_cases = [
            0u128,
            1,
            0x123,
            0xFEDCBA9876543210,
            0x123456789ABCDEF0FEDCBA9876543210,
            u128::MAX,
            u128::MAX - 1,
        ];

        for value in test_cases {
            let process_id = ProcessId::from_raw(value);
            let formatted = format!("{process_id}");
            let parsed = formatted.parse::<ProcessId>().unwrap();
            assert_eq!(process_id, parsed, "Failed roundtrip for value {value:#x}");
        }
    }

    #[test]
    fn process_id_serde_roundtrip() {
        let test_cases = [
            ProcessId::from_raw(0),
            ProcessId::from_raw(1),
            ProcessId::from_raw(0x123),
            ProcessId::from_raw(0xFEDCBA9876543210),
            ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
            ProcessId::from_raw(u128::MAX),
            ProcessId::from_raw(u128::MAX - 1),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: ProcessId = serde_json::from_str(&json).unwrap();
            assert_eq!(
                original,
                deserialized,
                "JSON roundtrip failed for {:#x}",
                original.to_raw()
            );
        }
    }

    #[test]
    fn span_context_format_from_str_roundtrip() {
        let test_cases = [
            SpanContext::new(ProcessId::from_raw(0), SpanId(0)),
            SpanContext::new(
                ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
                SpanId(0xFEDCBA9876543210),
            ),
            SpanContext::new(ProcessId::from_raw(u128::MAX), SpanId(u64::MAX)),
            SpanContext::new(ProcessId::from_raw(1), SpanId(1)),
        ];

        for context in test_cases {
            let formatted = format!("{context}");
            let parsed = formatted.parse::<SpanContext>().unwrap();
            assert_eq!(
                context,
                parsed,
                "Failed roundtrip for {:#x}:{:#x}",
                context.process_id.to_raw(),
                context.span_id.0
            );
        }
    }

    #[test]
    fn span_id_next_id_produces_non_zero_values() {
        let ids: Vec<SpanId> = (0..100).map(|_| SpanId::next_id()).collect();

        for id in &ids {
            assert_ne!(id.0, 0, "SpanId::next_id() should not produce zero values");
        }

        let mut unique_ids = HashSet::new();
        for id in &ids {
            assert!(
                unique_ids.insert(id.0),
                "SpanId::next_id() should produce unique values"
            );
        }
    }
}
