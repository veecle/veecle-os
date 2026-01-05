//! Core protocol type definitions and storage family trait.
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
//! Each message is associated with a [`ThreadId`] that uniquely identifies the thread it came from
//! (globally unique across all processes).
//! This allows telemetry data from multiple threads to be correlated and analyzed separately.

use serde::{Deserialize, Serialize};

pub use crate::SpanContext;
pub use crate::id::{ProcessId, SpanId, ThreadId};

pub(crate) trait Sealed {}

#[expect(private_bounds, reason = "sealed trait")]
/// A trait defining how data is stored in different contexts.
///
/// This trait allows the same protocol messages to be used with different storage strategies, see
/// the [main protocol module][super] docs for more details on the strategies provided.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::protocol::base::StorageFamily;
/// use veecle_telemetry::protocol::transient::Transient;
///
/// // The Transient family uses references
/// let message: <Transient as StorageFamily>::String<'_> = "hello";
/// ```
pub trait StorageFamily: Clone + core::fmt::Debug + Sealed {
    /// The string type for this storage family.
    type String<'a>: AsRef<str> + Clone + core::fmt::Debug + serde::Serialize
    where
        Self: 'a;

    /// The list type for this storage family.
    type List<'a, T: Clone + core::fmt::Debug + serde::Serialize + 'a>: AsRef<[T]>
        + Clone
        + core::fmt::Debug
        + serde::Serialize
    where
        Self: 'a;

    /// The value type for this storage family.
    type Value<'a>: Clone + core::fmt::Debug + serde::Serialize
    where
        Self: 'a;
}

/// A key-value attribute pair used in telemetry data.
///
/// Key-value pairs provide additional context for spans, events, and log messages.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::protocol::transient::KeyValue;
///
/// // Create attributes with different value types
/// let user_id = KeyValue::new("user_id", 123);
/// let username = KeyValue::new("username", "alice");
/// let is_active = KeyValue::new("is_active", true);
/// let score = KeyValue::new("score", 95.5);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "F::String<'a>: serde::de::DeserializeOwned, F::Value<'a>: serde::de::DeserializeOwned"
))]
pub struct KeyValue<'a, F>
where
    F: StorageFamily + 'a,
{
    /// The attribute key (name).
    pub key: F::String<'a>,

    /// The attribute value.
    pub value: F::Value<'a>,
}

impl<'a, F> KeyValue<'a, F>
where
    F: StorageFamily + 'a,
{
    /// Creates a new key-value attribute pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key (name)
    /// * `value` - The attribute value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::protocol::transient::KeyValue;
    ///
    /// let user_id = KeyValue::new("user_id", 123);
    /// let username = KeyValue::new("username", "alice");
    /// ```
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<F::String<'a>>,
        V: Into<F::Value<'a>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl<'a, F> core::fmt::Display for KeyValue<'a, F>
where
    F: StorageFamily + 'a,
    F::Value<'a>: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.key.as_ref(), self.value)
    }
}
/// A telemetry message associated with a specific execution thread.
///
/// This structure wraps a telemetry message with its execution context,
/// allowing messages from different executions to be properly correlated.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "TelemetryMessage<'a, F>: serde::de::DeserializeOwned"))]
pub struct InstanceMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// The thread this message belongs to.
    pub thread_id: ThreadId,

    /// The telemetry message content.
    pub message: TelemetryMessage<'a, F>,
}

/// An enumeration of all possible telemetry message types.
///
/// This enum represents the different categories of telemetry data that can be
/// collected and exported by the system.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "LogMessage<'a, F>: serde::de::DeserializeOwned, TracingMessage<'a, F>: serde::de::DeserializeOwned"
))]
pub enum TelemetryMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// A structured log message with severity and attributes.
    Log(LogMessage<'a, F>),

    /// A time synchronization message for clock coordination.
    TimeSync(TimeSyncMessage),

    /// A distributed tracing message (spans, events, links).
    Tracing(TracingMessage<'a, F>),
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "F::String<'a>: serde::de::DeserializeOwned, F::List<'a, KeyValue<'a, F>>: serde::de::DeserializeOwned"
))]
pub struct LogMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// Timestamp in nanoseconds since Unix epoch (or system start)
    pub time_unix_nano: u64,

    /// The severity level of this log message
    pub severity: Severity,

    /// The message body
    pub body: F::String<'a>,

    /// Key-value attributes providing additional context
    pub attributes: F::List<'a, KeyValue<'a, F>>,
}

/// A time synchronization message for coordinating clocks between systems.
///
/// This message contains both local time and absolute time information,
/// allowing downstream systems to correlate local timestamps with real-world time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeSyncMessage {
    /// Local timestamp in system-specific units.
    pub local_timestamp: u64,

    /// Time since Unix epoch in nanoseconds.
    pub since_epoch: u64,
}

/// Messages related to distributed tracing operations.
///
/// This enum encompasses all the different types of tracing messages that can be
/// generated during span lifecycle management and tracing operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "SpanCreateMessage<'a, F>: serde::de::DeserializeOwned, SpanAddEventMessage<'a, F>: serde::de::DeserializeOwned, SpanSetAttributeMessage<'a, F>: serde::de::DeserializeOwned"
))]
pub enum TracingMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// A new span has been created.
    CreateSpan(SpanCreateMessage<'a, F>),

    /// A span has been entered (made current).
    EnterSpan(SpanEnterMessage),

    /// A span has been exited (no longer current).
    ExitSpan(SpanExitMessage),

    /// A span has been closed (completed).
    CloseSpan(SpanCloseMessage),

    /// An event has been added to a span.
    AddEvent(SpanAddEventMessage<'a, F>),

    /// A link has been added to a span.
    AddLink(SpanAddLinkMessage),

    /// An attribute has been set on a span.
    SetAttribute(SpanSetAttributeMessage<'a, F>),
}

/// Message indicating the creation of a new span.
///
/// This message provides all the information needed to create a new span
/// in the trace, including its identity, timing, and initial attributes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "F::String<'a>: serde::de::DeserializeOwned, F::List<'a, KeyValue<'a, F>>: serde::de::DeserializeOwned"
))]
pub struct SpanCreateMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// The unique identifier (within the associated process) for this span.
    pub span_id: SpanId,

    /// The name of the span.
    pub name: F::String<'a>,

    /// Timestamp when the span was started.
    pub start_time_unix_nano: u64,

    /// Initial attributes attached to the span.
    pub attributes: F::List<'a, KeyValue<'a, F>>,
}

/// Message indicating a span has been entered.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanEnterMessage {
    /// The span being entered.
    pub span_id: SpanId,

    /// Timestamp when the span was entered.
    pub time_unix_nano: u64,
}

/// Message indicating a span has been exited.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanExitMessage {
    /// The span being exited.
    pub span_id: SpanId,

    /// Timestamp when the span was exited.
    pub time_unix_nano: u64,
}

/// Message indicating a span has been closed (completed).
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpanCloseMessage {
    /// The span being closed.
    pub span_id: SpanId,

    /// Timestamp when the span was closed.
    pub end_time_unix_nano: u64,
}

/// Message indicating an attribute has been set on a span.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "KeyValue<'a, F>: serde::de::DeserializeOwned"))]
pub struct SpanSetAttributeMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// The span the attribute is being set on, if [`None`] then this applies to the "current span"
    /// as determined by tracking [`SpanEnterMessage`] and [`SpanExitMessage`] pairs.
    pub span_id: Option<SpanId>,

    /// The attribute being set.
    pub attribute: KeyValue<'a, F>,
}

/// Message indicating an event has been added to a span.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "F::String<'a>: serde::de::DeserializeOwned, F::List<'a, KeyValue<'a, F>>: serde::de::DeserializeOwned"
))]
pub struct SpanAddEventMessage<'a, F>
where
    F: StorageFamily + 'a,
{
    /// The span the event is being added to, if [`None`] then this applies to the "current span"
    /// as determined by tracking [`SpanEnterMessage`] and [`SpanExitMessage`] pairs.
    pub span_id: Option<SpanId>,

    /// The name of the event.
    pub name: F::String<'a>,

    /// Timestamp when the event occurred.
    pub time_unix_nano: u64,

    /// Attributes providing additional context for the event.
    pub attributes: F::List<'a, KeyValue<'a, F>>,
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

    /// The span context being linked to.
    pub link: SpanContext,
}
