//! Telemetry protocol types and message definitions.
//!
//! This module defines the core data structures used for telemetry message exchange.
//! It includes message types for logging, tracing, and time synchronization, as well
//! as supporting types for execution tracking and attribute management.
//!
//! # Message Types
//!
//! The protocol supports several categories of telemetry messages:
//!
//! - **Log Messages** - Structured logging with severity levels and attributes
//! - **Tracing Messages** - Distributed tracing with spans, events, and links
//! - **Time Sync Messages** - Time synchronization between systems
//!
//! # Thread Tracking
//!
//! Each message is associated with an [`ThreadId`] that uniquely identifies
//! the thread it came from (globally unique across all processes).
//! This allows telemetry data from multiple threads to be correlated and analyzed separately.
//!
//! # Serialization
//!
//! All protocol types implement [`serde::Serialize`] and optionally [`serde::Deserialize`]
//! (when the `alloc` feature is enabled) for easy serialization to various formats.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SpanContext;
pub use crate::id::{ProcessId, SpanId};
#[cfg(feature = "alloc")]
use crate::to_static::ToStatic;
use crate::types::{ListType, StringType, list_from_slice};
use crate::value::KeyValue;

/// A specialised form of [`list_from_slice`] for attributes.
pub fn attribute_list_from_slice<'a>(slice: &'a [KeyValue<'a>]) -> AttributeListType<'a> {
    list_from_slice::<KeyValue<'a>>(slice)
}

/// Type alias for a list of attributes.
pub type AttributeListType<'a> = ListType<'a, KeyValue<'a>>;

#[cfg(feature = "alloc")]
impl ToStatic for AttributeListType<'_> {
    type Static = AttributeListType<'static>;

    fn to_static(&self) -> Self::Static {
        self.iter()
            .map(|item| item.to_static())
            .collect::<Vec<_>>()
            .into()
    }
}

/// A globally-unique id identifying a thread within a specific process.
///
/// The primary purpose of this id is to allow the consumer of telemetry messages to associate
/// spans with the callstack they came from to reconstruct parent-child relationships. On a normal
/// operating system this is the thread, on other systems it should be whatever is the closest
/// equivalent, e.g. for FreeRTOS it would be a task. On a single threaded bare-metal system it
/// would be a constant as there is only the one callstack.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct ThreadId {
    /// The globally-unique id for the process this thread is within.
    pub process: ProcessId,

    /// The process-unique id for this thread within the process.
    raw: u64,
}

impl ThreadId {
    /// Creates a [`ThreadId`] from a raw value.
    ///
    /// Extra care needs to be taken that this is not a constant value or re-used within this
    /// process in any way.
    pub const fn from_raw(process: ProcessId, raw: u64) -> Self {
        Self { process, raw }
    }

    /// Creates a [`ThreadId`] for the current thread, using OS specific means to acquire it.
    #[cfg(feature = "enable")]
    pub(crate) fn current(process: ProcessId) -> Self {
        #[cfg_attr(not(feature = "std"), expect(unreachable_code))]
        Self::from_raw(process, {
            #[cfg(feature = "std")]
            {
                use veecle_osal_std::thread::{Thread, ThreadAbstraction};
                Thread::current_thread_id()
            }

            #[cfg(not(feature = "std"))]
            {
                panic!("not yet supported")
            }
        })
    }
}

impl fmt::Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { process, raw } = self;
        write!(f, "{process}:{raw:032x}")
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
}

impl fmt::Display for ParseThreadIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => f.write_str("missing ':' separator"),
            Self::InvalidProcessId(_) => f.write_str("failed to parse process id"),
            Self::InvalidThreadId(_) => f.write_str("failed to parse thread id"),
        }
    }
}

impl core::error::Error for ParseThreadIdError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::MissingSeparator => None,
            Self::InvalidProcessId(error) => Some(error),
            Self::InvalidThreadId(error) => Some(error),
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
        let thread =
            u64::from_str_radix(thread, 16).map_err(ParseThreadIdError::InvalidThreadId)?;
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
        hex::encode_to_slice(self.raw.to_le_bytes(), &mut bytes[33..]).unwrap();

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
                &"expected : separator at byte 33",
            ));
        }

        let mut process = [0; 16];
        hex::decode_to_slice(&bytes[..32], &mut process).map_err(D::Error::custom)?;
        let process = ProcessId::from_raw(u128::from_le_bytes(process));

        let mut thread = [0; 8];
        hex::decode_to_slice(&bytes[33..], &mut thread).map_err(D::Error::custom)?;

        Ok(Self::from_raw(process, u64::from_le_bytes(thread)))
    }
}

/// A telemetry message associated with a specific execution thread.
///
/// This structure wraps a telemetry message with its execution context,
/// allowing messages from different executions to be properly correlated.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub struct InstanceMessage<'a> {
    /// The thread this message belongs to
    pub thread: ThreadId,

    /// The telemetry message content
    #[serde(borrow)]
    pub message: TelemetryMessage<'a>,
}

#[cfg(feature = "alloc")]
impl ToStatic for InstanceMessage<'_> {
    type Static = InstanceMessage<'static>;

    fn to_static(&self) -> Self::Static {
        InstanceMessage {
            thread: self.thread,
            message: self.message.to_static(),
        }
    }
}

/// An enumeration of all possible telemetry message types.
///
/// This enum represents the different categories of telemetry data that can be
/// collected and exported by the system.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub enum TelemetryMessage<'a> {
    /// A structured log message with severity and attributes
    Log(#[serde(borrow)] LogMessage<'a>),
    /// A time synchronization message for clock coordination
    TimeSync(TimeSyncMessage),
    /// A distributed tracing message (spans, events, links)
    Tracing(#[serde(borrow)] TracingMessage<'a>),
}

#[cfg(feature = "alloc")]
impl ToStatic for TelemetryMessage<'_> {
    type Static = TelemetryMessage<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            TelemetryMessage::Log(msg) => TelemetryMessage::Log(msg.to_static()),
            TelemetryMessage::TimeSync(msg) => TelemetryMessage::TimeSync(msg.clone()),
            TelemetryMessage::Tracing(msg) => TelemetryMessage::Tracing(msg.to_static()),
        }
    }
}

/// Log message severity levels.
///
/// These levels follow standard logging conventions, ordered from most verbose
/// to most critical.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error,
    /// The "fatal" level.
    ///
    /// Designates critical failures that might crash the program.
    Fatal,
}

/// A structured log message with severity, timestamp, and attributes.
///
/// Log messages can be optionally correlated with traces by including trace and span IDs when available.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub struct LogMessage<'a> {
    /// Timestamp in nanoseconds since Unix epoch (or system start)
    pub time_unix_nano: u64,
    /// The severity level of this log message
    pub severity: Severity,

    /// The message body
    #[serde(borrow)]
    pub body: StringType<'a>,

    /// Key-value attributes providing additional context
    #[serde(borrow)]
    pub attributes: AttributeListType<'a>,
}

#[cfg(feature = "alloc")]
impl ToStatic for LogMessage<'_> {
    type Static = LogMessage<'static>;

    fn to_static(&self) -> Self::Static {
        LogMessage {
            time_unix_nano: self.time_unix_nano,
            severity: self.severity,
            body: self.body.to_static(),
            attributes: self.attributes.to_static(),
        }
    }
}

/// A time synchronization message for coordinating clocks between systems.
///
/// This message contains both local time and absolute time information,
/// allowing downstream systems to correlate local timestamps with real-world time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeSyncMessage {
    /// Local timestamp in system-specific units
    pub local_timestamp: u64,
    /// Time since Unix epoch in nanoseconds
    pub since_epoch: u64,
}

/// Messages related to distributed tracing operations.
///
/// This enum encompasses all the different types of tracing messages that can be
/// generated during span lifecycle management and tracing operations.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub enum TracingMessage<'a> {
    /// A new span has been created
    CreateSpan(#[serde(borrow)] SpanCreateMessage<'a>),
    /// A span has been entered (made current)
    EnterSpan(SpanEnterMessage),
    /// A span has been exited (no longer current)
    ExitSpan(SpanExitMessage),
    /// A span has been closed (completed)
    CloseSpan(SpanCloseMessage),
    /// An event has been added to a span
    AddEvent(#[serde(borrow)] SpanAddEventMessage<'a>),
    /// A link has been added to a span
    AddLink(SpanAddLinkMessage),
    /// An attribute has been set on a span
    SetAttribute(#[serde(borrow)] SpanSetAttributeMessage<'a>),
}

#[cfg(feature = "alloc")]
impl ToStatic for TracingMessage<'_> {
    type Static = TracingMessage<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            TracingMessage::CreateSpan(msg) => TracingMessage::CreateSpan(msg.to_static()),
            TracingMessage::EnterSpan(msg) => TracingMessage::EnterSpan(*msg),
            TracingMessage::ExitSpan(msg) => TracingMessage::ExitSpan(*msg),
            TracingMessage::CloseSpan(msg) => TracingMessage::CloseSpan(*msg),
            TracingMessage::AddEvent(msg) => TracingMessage::AddEvent(msg.to_static()),
            TracingMessage::AddLink(msg) => TracingMessage::AddLink(*msg),
            TracingMessage::SetAttribute(msg) => TracingMessage::SetAttribute(msg.to_static()),
        }
    }
}

/// Message indicating the creation of a new span.
///
/// This message provides all the information needed to create a new span
/// in the trace, including its identity, timing, and initial attributes.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub struct SpanCreateMessage<'a> {
    /// The unique identifier (within the associated process) for this span.
    pub span_id: SpanId,

    /// The name of the span
    #[serde(borrow)]
    pub name: StringType<'a>,

    /// Timestamp when the span was started
    pub start_time_unix_nano: u64,

    /// Initial attributes attached to the span
    #[serde(borrow)]
    pub attributes: AttributeListType<'a>,
}

#[cfg(feature = "alloc")]
impl ToStatic for SpanCreateMessage<'_> {
    type Static = SpanCreateMessage<'static>;

    fn to_static(&self) -> Self::Static {
        SpanCreateMessage {
            span_id: self.span_id,
            name: self.name.to_static(),
            start_time_unix_nano: self.start_time_unix_nano,
            attributes: self.attributes.to_static(),
        }
    }
}

/// Message indicating a span has been entered.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanEnterMessage {
    /// The span being entered
    pub span_id: SpanId,

    /// Timestamp when the span was entered
    pub time_unix_nano: u64,
}

/// Message indicating a span has been exited.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanExitMessage {
    /// The span being exited
    pub span_id: SpanId,

    /// Timestamp when the span was exited
    pub time_unix_nano: u64,
}

/// Message indicating a span has been closed (completed).
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanCloseMessage {
    /// The span being closed
    pub span_id: SpanId,

    /// Timestamp when the span was closed
    pub end_time_unix_nano: u64,
}

/// Message indicating an attribute has been set on a span.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpanSetAttributeMessage<'a> {
    /// The span the attribute is being set on, if [`None`] then this applies to the "current span"
    /// as determined by tracking [`SpanEnterMessage`] and [`SpanExitMessage`] pairs.
    pub span_id: Option<SpanId>,

    /// The attribute being set
    #[serde(borrow)]
    pub attribute: KeyValue<'a>,
}

#[cfg(feature = "alloc")]
impl ToStatic for SpanSetAttributeMessage<'_> {
    type Static = SpanSetAttributeMessage<'static>;

    fn to_static(&self) -> Self::Static {
        SpanSetAttributeMessage {
            span_id: self.span_id,
            attribute: self.attribute.to_static(),
        }
    }
}

/// Message indicating an event has been added to a span.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "alloc", derive(Deserialize))]
pub struct SpanAddEventMessage<'a> {
    /// The span the event is being added to, if [`None`] then this applies to the "current span"
    /// as determined by tracking [`SpanEnterMessage`] and [`SpanExitMessage`] pairs.
    pub span_id: Option<SpanId>,

    /// The name of the event
    #[serde(borrow)]
    pub name: StringType<'a>,

    /// Timestamp when the event occurred
    pub time_unix_nano: u64,

    /// Attributes providing additional context for the event
    #[serde(borrow)]
    pub attributes: AttributeListType<'a>,
}

#[cfg(feature = "alloc")]
impl ToStatic for SpanAddEventMessage<'_> {
    type Static = SpanAddEventMessage<'static>;

    fn to_static(&self) -> Self::Static {
        SpanAddEventMessage {
            span_id: self.span_id,
            name: self.name.to_static(),
            time_unix_nano: self.time_unix_nano,
            attributes: self.attributes.to_static(),
        }
    }
}

/// Message indicating a link has been added to a span.
///
/// Links connect spans across different traces, representing relationships
/// that are not parent-child hierarchies.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanAddLinkMessage {
    /// The span the link is being added to, if [`None`] then this applies to the "current span"
    /// as determined by tracking [`SpanEnterMessage`] and [`SpanExitMessage`] pairs.
    pub span_id: Option<SpanId>,

    /// The span context being linked to
    pub link: SpanContext,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use alloc::format;
    #[cfg(feature = "alloc")]
    use alloc::string::String;

    use super::*;

    #[test]
    fn thread_id_format_from_str_roundtrip() {
        let test_cases = [
            ThreadId::from_raw(ProcessId::from_raw(0), 0),
            ThreadId::from_raw(
                ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
                0xFEDCBA9876543210,
            ),
            ThreadId::from_raw(ProcessId::from_raw(u128::MAX), u64::MAX),
            ThreadId::from_raw(ProcessId::from_raw(1), 1),
        ];

        for thread_id in test_cases {
            let formatted = format!("{thread_id}");
            let parsed = formatted.parse::<ThreadId>().unwrap();
            assert_eq!(
                thread_id,
                parsed,
                "Failed roundtrip for {:#x}:{:#x}",
                thread_id.process.to_raw(),
                thread_id.raw,
            );
        }
    }

    #[test]
    fn thread_id_serde_roundtrip() {
        let test_cases = [
            ThreadId::from_raw(ProcessId::from_raw(0), 0),
            ThreadId::from_raw(
                ProcessId::from_raw(0x123456789ABCDEF0FEDCBA9876543210),
                0xFEDCBA9876543210,
            ),
            ThreadId::from_raw(ProcessId::from_raw(u128::MAX), u64::MAX),
            ThreadId::from_raw(ProcessId::from_raw(1), 1),
        ];

        for original in test_cases {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: ThreadId = serde_json::from_str(&json).unwrap();
            assert_eq!(original, deserialized);
        }
    }

    #[test]
    fn string_type_conversions() {
        let static_str: StringType<'static> = "static".into();

        let _event = SpanAddEventMessage {
            span_id: Some(SpanId(0)),
            name: static_str,
            time_unix_nano: 0,
            attributes: attribute_list_from_slice(&[]),
        };

        let borrowed_str: StringType = "borrowed".into();

        let _event = SpanAddEventMessage {
            span_id: Some(SpanId(0)),
            name: borrowed_str,
            time_unix_nano: 0,
            attributes: attribute_list_from_slice(&[]),
        };
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn string_type_with_owned_strings() {
        let string = String::from("owned");
        let owned: StringType<'static> = StringType::from(string);

        let _event = SpanAddEventMessage {
            span_id: Some(SpanId(0)),
            name: owned,
            time_unix_nano: 0,
            attributes: attribute_list_from_slice(&[]),
        };
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn to_static_conversion() {
        use alloc::string::String;

        use crate::value::Value;

        // Create some data with non-static lifetime
        let borrowed_name_str = "test_span";
        let borrowed_name: StringType = borrowed_name_str.into();

        let owned_key = String::from("test_key");
        let owned_value = String::from("test_value");
        let attribute = KeyValue {
            key: owned_key.as_str().into(),
            value: Value::String(owned_value.as_str().into()),
        };

        let attributes = [attribute];
        let span_event = SpanAddEventMessage {
            span_id: Some(SpanId(0)),
            name: borrowed_name,
            time_unix_nano: 0,
            attributes: attribute_list_from_slice(&attributes),
        };

        let tracing_message = TracingMessage::AddEvent(span_event);
        let telemetry_message = TelemetryMessage::Tracing(tracing_message);
        let instance_message = InstanceMessage {
            thread: ThreadId::from_raw(ProcessId::from_raw(999), 111),
            message: telemetry_message,
        };

        let static_message: InstanceMessage<'static> = instance_message.to_static();

        // Verify the conversion worked - the static message should have the same data
        if let TelemetryMessage::Tracing(TracingMessage::AddEvent(span_event)) =
            &static_message.message
        {
            assert_eq!(span_event.name.as_ref(), "test_span");
        } else {
            panic!("Expected CreateSpan message");
        }
    }
}
