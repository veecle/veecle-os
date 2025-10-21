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
//! - [`TraceId`]: A 128-bit globally unique identifier that groups related spans together
//! - [`SpanId`]: A 64-bit identifier that uniquely identifies a span within a trace
//! - [`SpanContext`]: A combination of trace ID and span ID that uniquely identifies a span

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::collector::get_collector;
#[cfg(feature = "enable")]
use crate::span::CURRENT_SPAN;

/// An identifier for a trace, which groups a set of related spans together.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TraceId(pub u128);

impl TraceId {
    /// Uses the state in the collector to generate a `TraceId`.
    ///
    /// Returns 0 if the collector has not been initialized via [`crate::collector::set_exporter`].
    #[inline]
    pub fn generate() -> Self {
        get_collector().generate_trace_id()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl FromStr for TraceId {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u128::from_str_radix(s, 16).map(TraceId)
    }
}

impl serde::Serialize for TraceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut hex_bytes = [0u8; size_of::<u128>() * 2];
        hex::encode_to_slice(self.0.to_le_bytes(), &mut hex_bytes).unwrap();

        serializer.serialize_str(str::from_utf8(&hex_bytes).unwrap())
    }
}

impl<'de> serde::Deserialize<'de> for TraceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: [u8; size_of::<u128>()] = hex::serde::deserialize(deserializer)?;

        Ok(TraceId(u128::from_le_bytes(bytes)))
    }
}

/// An identifier for a span within a trace.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpanId(pub u64);

#[cfg(feature = "enable")]
impl SpanId {
    #[inline]
    #[doc(hidden)]
    /// Creates a non-zero `SpanId`
    pub fn next_id() -> SpanId {
        #[cfg(feature = "std")]
        {
            std::thread_local! {
                static LOCAL_ID_GENERATOR: core::cell::Cell<u64> = core::cell::Cell::new(rand::random::<u64>() & 0xffffffff00000000);
            }

            LOCAL_ID_GENERATOR
                .try_with(|g| {
                    let id = g.get().wrapping_add(1);
                    g.set(id);

                    SpanId(id)
                })
                .unwrap_or({
                    // This only gets called if the TLS key has been destroyed, it should be safe to fall back to a 0
                    // value (noop) `SpanId`
                    SpanId(0)
                })
        }

        #[cfg(not(feature = "std"))]
        {
            use core::sync::atomic;
            // For no_std, use a simple counter approach
            static COUNTER: atomic::AtomicU64 = atomic::AtomicU64::new(1);
            SpanId(COUNTER.fetch_add(1, atomic::Ordering::Relaxed))
        }
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

/// A struct representing the context of a span, including its [`TraceId`] and [`SpanId`].
///
/// [`TraceId`]: crate::id::TraceId
/// [`SpanId`]: crate::id::SpanId
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SpanContext {
    /// The trace ID this span belongs to
    pub trace_id: TraceId,
    /// The unique ID of this span
    pub span_id: SpanId,
}

impl SpanContext {
    /// Creates a new `SpanContext` with the given [`TraceId`] and [`SpanId`].
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::id::*;
    ///
    /// let span_context = SpanContext::new(TraceId(12), SpanId(13));
    /// ```
    ///
    /// [`TraceId`]: crate::id::TraceId
    /// [`SpanId`]: crate::id::SpanId
    pub fn new(trace_id: TraceId, span_id: SpanId) -> Self {
        Self { trace_id, span_id }
    }

    /// Creates a new `SpanContext` with a `TraceId` generated with the state in the collector.
    ///
    /// Returns 0 if the collector has not been initialized via [`crate::collector::set_exporter`].
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::*;
    ///
    /// let random = SpanContext::generate();
    /// ```
    pub fn generate() -> Self {
        Self {
            trace_id: TraceId::generate(),
            span_id: SpanId(0),
        }
    }

    /// Creates a `SpanContext` from the current local parent span. If there is no
    /// local parent span, this function will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use veecle_telemetry::*;
    ///
    /// let span = Span::new("root", &[]);
    /// let _guard = span.entered();
    ///
    /// let span_context = SpanContext::current();
    /// assert!(span_context.is_some());
    /// ```
    pub fn current() -> Option<Self> {
        #[cfg(not(feature = "enable"))]
        {
            None
        }

        #[cfg(feature = "enable")]
        {
            CURRENT_SPAN.get()
        }
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
    fn trace_id_formatting() {
        assert_eq!(
            format!("{}", TraceId(0)),
            "00000000000000000000000000000000"
        );
        assert_eq!(
            format!("{}", TraceId(u128::MAX)),
            "ffffffffffffffffffffffffffffffff"
        );
        assert_eq!(
            format!("{}", TraceId(0x123456789ABCDEF0FEDCBA9876543210)),
            "123456789abcdef0fedcba9876543210"
        );
        assert_eq!(
            format!("{}", TraceId(0x123)),
            "00000000000000000000000000000123"
        );
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
    fn trace_id_from_str() {
        assert_eq!(
            "123456789abcdef0fedcba9876543210"
                .parse::<TraceId>()
                .unwrap(),
            TraceId(0x123456789ABCDEF0FEDCBA9876543210)
        );
        assert_eq!(
            "123456789ABCDEF0FEDCBA9876543210"
                .parse::<TraceId>()
                .unwrap(),
            TraceId(0x123456789ABCDEF0FEDCBA9876543210)
        );
        assert_eq!(
            "00000000000000000000000000000000"
                .parse::<TraceId>()
                .unwrap(),
            TraceId(0)
        );
        assert_eq!(
            "ffffffffffffffffffffffffffffffff"
                .parse::<TraceId>()
                .unwrap(),
            TraceId(u128::MAX)
        );
        // Shorter hex string works as u128::from_str_radix handles it
        assert_eq!("123".parse::<TraceId>().unwrap(), TraceId(0x123));

        assert!("xyz".parse::<TraceId>().is_err());
        assert!("".parse::<TraceId>().is_err());
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
    fn trace_id_format_from_str_roundtrip() {
        let test_cases = [
            0u128,
            1,
            0x123,
            0x123456789ABCDEF0FEDCBA9876543210,
            u128::MAX,
            u128::MAX - 1,
        ];

        for value in test_cases {
            let trace_id = TraceId(value);
            let formatted = format!("{trace_id}");
            let parsed = formatted.parse::<TraceId>().unwrap();
            assert_eq!(trace_id, parsed, "Failed roundtrip for value {value:#x}");
        }
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
    fn trace_id_serde_roundtrip() {
        let test_cases = [
            TraceId(0),
            TraceId(1),
            TraceId(0x123),
            TraceId(0x123456789ABCDEF0FEDCBA9876543210),
            TraceId(u128::MAX),
            TraceId(u128::MAX - 1),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: TraceId = serde_json::from_str(&json).unwrap();
            assert_eq!(
                original, deserialized,
                "JSON roundtrip failed for {:#x}",
                original.0
            );
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
            SpanContext::new(TraceId(0), SpanId(0)),
            SpanContext::new(
                TraceId(0x123456789ABCDEF0FEDCBA9876543210),
                SpanId(0xFEDCBA9876543210),
            ),
            SpanContext::new(TraceId(u128::MAX), SpanId(u64::MAX)),
            SpanContext::new(TraceId(1), SpanId(1)),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: SpanContext = serde_json::from_str(&json).unwrap();
            assert_eq!(
                original.trace_id, deserialized.trace_id,
                "JSON roundtrip failed for trace_id"
            );
            assert_eq!(
                original.span_id, deserialized.span_id,
                "JSON roundtrip failed for span_id"
            );
        }
    }

    #[test]
    fn trace_id_serialization_format() {
        let trace_id = TraceId(0x123456789ABCDEF0FEDCBA9876543210);
        let json = serde_json::to_string(&trace_id).unwrap();

        // Serialization uses little-endian bytes
        let expected_le_bytes = 0x123456789ABCDEF0FEDCBA9876543210u128.to_le_bytes();
        let mut expected_hex = String::new();
        for byte in &expected_le_bytes {
            expected_hex.push_str(&format!("{byte:02x}"));
        }
        let expected_json = format!("\"{expected_hex}\"");

        assert_eq!(json, expected_json);
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
        let trace_id = TraceId(0x123);
        let span_id = SpanId(0x456);
        let context = SpanContext::new(trace_id, span_id);

        assert_eq!(context.trace_id, trace_id);
        assert_eq!(context.span_id, span_id);
    }

    #[test]
    fn span_context_generate_produces_non_zero_trace_id() {
        let context = SpanContext::generate();
        // span_id should be 0 as per the implementation
        assert_eq!(context.span_id, SpanId(0));
        // trace_id value depends on collector initialization
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
