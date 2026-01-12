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
use core::num::NonZeroU64;
use core::str::FromStr;

/// A globally-unique id identifying a process.
///
/// The primary purpose of this id is to provide a globally-unique context within which
/// [`ThreadId`]s and [`SpanContext`]s are guaranteed to be unique. On a normal operating system
/// that is the process, on other systems it should be whatever is the closest equivalent, e.g. for
/// most embedded setups it should be unique for each time the system is restarted.
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

/// A globally-unique id identifying a thread within a specific process.
///
/// The primary purpose of this id is to allow the consumer of telemetry messages to associate
/// spans with the callstack they came from to reconstruct parent-child relationships. On a normal
/// operating system this is the thread, on other systems it should be whatever is the closest
/// equivalent, e.g. for FreeRTOS it would be a task. On a single threaded bare-metal system it
/// would be a constant as there is only the one callstack.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ThreadId {
    /// The globally-unique id for the process this thread is within.
    pub process: ProcessId,

    /// The process-unique id for this thread within the process.
    raw: NonZeroU64,
}

impl ThreadId {
    /// Creates a [`ThreadId`] from a raw value.
    ///
    /// Extra care needs to be taken that this is not a constant value or re-used within this
    /// process in any way.
    pub const fn from_raw(process: ProcessId, raw: NonZeroU64) -> Self {
        Self { process, raw }
    }

    /// Returns the raw value of this id.
    pub fn raw(&self) -> NonZeroU64 {
        self.raw
    }
}

impl fmt::Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { process, raw } = self;
        write!(f, "{process}:{raw:016x}")
    }
}

/// Errors that can occur while parsing [`ThreadId`] from a string.
#[derive(Clone, Debug)]
pub enum ParseThreadIdError {
    /// The string is missing a `:` separator.
    MissingSeparator,

    /// The embedded [`ProcessId`] failed to parse.
    InvalidProcessId(core::num::ParseIntError),

    /// The embedded [`ThreadId`] failed to parse.
    InvalidThreadId(core::num::ParseIntError),

    /// The embedded [`ThreadId`] had a zero value.
    ZeroThreadId,
}

impl fmt::Display for ParseThreadIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => f.write_str("missing ':' separator"),
            Self::InvalidProcessId(_) => f.write_str("failed to parse process id"),
            Self::InvalidThreadId(_) => f.write_str("failed to parse thread id"),
            Self::ZeroThreadId => f.write_str("zero thread id"),
        }
    }
}

impl core::error::Error for ParseThreadIdError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::MissingSeparator => None,
            Self::InvalidProcessId(error) => Some(error),
            Self::InvalidThreadId(error) => Some(error),
            Self::ZeroThreadId => None,
        }
    }
}

impl FromStr for ThreadId {
    type Err = ParseThreadIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((process, thread)) = s.split_once(":") else {
            return Err(ParseThreadIdError::MissingSeparator);
        };
        let process = ProcessId::from_str(process).map_err(ParseThreadIdError::InvalidProcessId)?;
        let thread = NonZeroU64::new(
            u64::from_str_radix(thread, 16).map_err(ParseThreadIdError::InvalidThreadId)?,
        )
        .ok_or(ParseThreadIdError::ZeroThreadId)?;
        Ok(Self::from_raw(process, thread))
    }
}

impl serde::Serialize for ThreadId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes = [0u8; 49];

        hex::encode_to_slice(self.process.to_raw().to_le_bytes(), &mut bytes[..32]).unwrap();
        bytes[32] = b':';
        hex::encode_to_slice(self.raw().get().to_le_bytes(), &mut bytes[33..]).unwrap();

        serializer.serialize_str(str::from_utf8(&bytes).unwrap())
    }
}

impl<'de> serde::Deserialize<'de> for ThreadId {
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
        let process = ProcessId::from_raw(u128::from_le_bytes(process));

        let mut thread = [0; 8];
        hex::decode_to_slice(&bytes[33..], &mut thread).map_err(D::Error::custom)?;
        let thread = NonZeroU64::new(u64::from_le_bytes(thread))
            .ok_or_else(|| D::Error::custom("zero thread id"))?;

        Ok(Self::from_raw(process, thread))
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
    use std::vec::Vec;

    use test_case::test_case;

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

    #[test_case(SpanId(0), r#""0000000000000000""#, "0000000000000000")]
    #[test_case(SpanId(1), r#""0100000000000000""#, "0000000000000001")]
    #[test_case(SpanId(0x123), r#""2301000000000000""#, "0000000000000123")]
    #[test_case(SpanId(u64::MAX), r#""ffffffffffffffff""#, "ffffffffffffffff")]
    #[test_case(
        SpanId(0xFEDCBA9876543210),
        r#""1032547698badcfe""#,
        "fedcba9876543210"
    )]
    #[test_case(
        ProcessId::from_raw(0),
        r#""00000000000000000000000000000000""#,
        "00000000000000000000000000000000"
    )]
    #[test_case(
        ProcessId::from_raw(1),
        r#""01000000000000000000000000000000""#,
        "00000000000000000000000000000001"
    )]
    #[test_case(
        ProcessId::from_raw(0x123),
        r#""23010000000000000000000000000000""#,
        "00000000000000000000000000000123"
    )]
    #[test_case(
        ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
        r#""1032547698badcfef0debc9a78563412""#,
        "123456789abcdef0fedcba9876543210"
    )]
    #[test_case(
        ProcessId::from_raw(u128::MAX),
        r#""ffffffffffffffffffffffffffffffff""#,
        "ffffffffffffffffffffffffffffffff"
    )]
    #[test_case(
        ThreadId::from_raw(ProcessId::from_raw(0), NonZeroU64::new(1).unwrap()),
        r#""00000000000000000000000000000000:0100000000000000""#,
        "00000000000000000000000000000000:0000000000000001"
    )]
    #[test_case(
        ThreadId::from_raw(ProcessId::from_raw(0x123), NonZeroU64::new(0x456).unwrap()),
        r#""23010000000000000000000000000000:5604000000000000""#,
        "00000000000000000000000000000123:0000000000000456"
    )]
    #[test_case(
        ThreadId::from_raw(ProcessId::from_raw(u128::MAX), NonZeroU64::new(u64::MAX).unwrap()),
        r#""ffffffffffffffffffffffffffffffff:ffffffffffffffff""#,
        "ffffffffffffffffffffffffffffffff:ffffffffffffffff"
    )]
    #[test_case(
        ThreadId::from_raw(
            ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
            NonZeroU64::new(0xFEDCBA9876543210).unwrap(),
        ),
        r#""1032547698badcfef0debc9a78563412:1032547698badcfe""#,
        "123456789abcdef0fedcba9876543210:fedcba9876543210"
    )]
    #[test_case(
        SpanContext::new(ProcessId::from_raw(0), SpanId(0)),
        r#""00000000000000000000000000000000:0000000000000000""#,
        "00000000000000000000000000000000:0000000000000000"
    )]
    #[test_case(
        SpanContext::new(ProcessId::from_raw(0x123), SpanId(0x456)),
        r#""23010000000000000000000000000000:5604000000000000""#,
        "00000000000000000000000000000123:0000000000000456"
    )]
    #[test_case(
        SpanContext::new(ProcessId::from_raw(u128::MAX), SpanId(u64::MAX)),
        r#""ffffffffffffffffffffffffffffffff:ffffffffffffffff""#,
        "ffffffffffffffffffffffffffffffff:ffffffffffffffff"
    )]
    #[test_case(
        SpanContext::new(
            ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
            SpanId(0xFEDCBA9876543210)
        ),
        r#""1032547698badcfef0debc9a78563412:1032547698badcfe""#,
        "123456789abcdef0fedcba9876543210:fedcba9876543210"
    )]
    fn serialization<T>(value: T, expected_json: &str, expected_display: &str)
    where
        T: fmt::Display
            + serde::Serialize
            + FromStr<Err: fmt::Debug>
            + serde::de::DeserializeOwned
            + fmt::Debug
            + Eq,
    {
        assert_eq!(serde_json::to_string(&value).unwrap(), expected_json);
        assert_eq!(value, serde_json::from_str::<T>(expected_json).unwrap());

        assert_eq!(format!("{value}"), expected_display);
        assert_eq!(value, T::from_str(expected_display).unwrap());
    }

    #[test_case("")]
    #[test_case("xyz")]
    fn span_id_from_str_error(input: &str) {
        assert!(SpanId::from_str(input).is_err());
    }

    #[test_case("")]
    #[test_case("ffffffffffffffffffffffffffffffff0")]
    #[test_case("xyz")]
    fn process_id_from_str_error(input: &str) {
        assert!(ProcessId::from_str(input).is_err());
    }

    #[test_case("")]
    #[test_case("00000000000000000000000000000000")]
    #[test_case("00000000000000000000000000000000:0000000000000000")]
    #[test_case("00000000000000000000000000000000:xyz")]
    #[test_case("00000000000000000000000000000001:")]
    #[test_case(":0000000000000001")]
    #[test_case("xyz")]
    #[test_case("xyz:0000000000000001")]
    fn thread_id_from_str_error(input: &str) {
        assert!(ThreadId::from_str(input).is_err());
    }

    #[test_case("")]
    #[test_case("00000000000000000000000000000000")]
    #[test_case("00000000000000000000000000000000:xyz")]
    #[test_case("00000000000000000000000000000001:")]
    #[test_case(":0000000000000000")]
    #[test_case("xyz")]
    #[test_case("xyz:0000000000000000")]
    fn span_context_from_str_error(input: &str) {
        assert!(SpanContext::from_str(input).is_err());
    }
}
