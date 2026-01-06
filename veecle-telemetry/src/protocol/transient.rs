//! Type aliases for transient usage (with `format_args!` support).
//!
//! These aliases use [`Value`] which may contain non-Send types like `format_args!`.
//! Use these for local telemetry operations that don't need to cross thread boundaries.
//!
//! See the [main protocol module][super] docs for more details on the protocol modules provided.

use crate::protocol::base;
use serde::Serialize;

/// Transient storage family using borrowed data.
///
/// This family uses references for zero-copy operation, suitable for
/// local telemetry that doesn't need to cross thread boundaries.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Transient;

impl base::Sealed for Transient {}
impl base::StorageFamily for Transient {
    type String<'a>
        = &'a str
    where
        Self: 'a;

    type List<'a, T: Clone + core::fmt::Debug + serde::Serialize + 'a>
        = &'a [T]
    where
        Self: 'a;

    type Value<'a>
        = Value<'a>
    where
        Self: 'a;
}

// Re-export non-generic types for convenience.
pub use base::{
    ProcessId, Severity, SpanAddLinkMessage, SpanCloseMessage, SpanContext, SpanEnterMessage,
    SpanExitMessage, SpanId, ThreadId, TimeSyncMessage,
};

/// Key-value pair with transient value (supports `format_args!`).
pub type KeyValue<'a> = base::KeyValue<'a, Transient>;
/// Instance message with transient values (supports `format_args!`).
pub type InstanceMessage<'a> = base::InstanceMessage<'a, Transient>;
/// Telemetry message with transient values (supports `format_args!`).
pub type TelemetryMessage<'a> = base::TelemetryMessage<'a, Transient>;
/// Log message with transient values (supports `format_args!`).
pub type LogMessage<'a> = base::LogMessage<'a, Transient>;
/// Tracing message with transient values (supports `format_args!`).
pub type TracingMessage<'a> = base::TracingMessage<'a, Transient>;
/// Span create message with transient values (supports `format_args!`).
pub type SpanCreateMessage<'a> = base::SpanCreateMessage<'a, Transient>;
/// Span set attribute message with transient values (supports `format_args!`).
pub type SpanSetAttributeMessage<'a> = base::SpanSetAttributeMessage<'a, Transient>;
/// Span add event message with transient values (supports `format_args!`).
pub type SpanAddEventMessage<'a> = base::SpanAddEventMessage<'a, Transient>;

/// A transient value that can be stored in a telemetry attribute.
///
/// This enum represents values that may contain non-Send types like `format_args!`,
/// making them suitable for local use but not for sending across threads.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::protocol::transient::Value;
///
/// // Create values of different types
/// let text = Value::String("hello world");
/// let number = Value::I64(42);
/// let flag = Value::Bool(true);
/// let rating = Value::F64(4.5);
/// ```
#[derive(Clone, Debug, Serialize)]
pub enum Value<'a> {
    /// A string value
    String(&'a str),

    /// A `format_args!` call.
    #[serde(rename(serialize = "String"))]
    Formatted(core::fmt::Arguments<'a>),

    /// A boolean value
    Bool(bool),

    /// A 64-bit signed integer
    I64(i64),

    /// A 64-bit floating-point number
    F64(f64),
}

impl<'a> core::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // For strings, debug print so they will get delimiters, since we are explicitly
            // representing strings rather than directly human-targeted text, and they will be used
            // in situations where knowing where the string ends is important.
            Self::String(value) => write!(f, "{value:?}"),
            Self::Formatted(value) => write!(f, "{value:?}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::F64(value) => write!(f, "{value}"),
        }
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Value::String(value)
    }
}

impl<'a> From<&'a &str> for Value<'a> {
    fn from(value: &'a &str) -> Self {
        Value::String(value)
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a alloc::string::String> for Value<'a> {
    fn from(value: &'a alloc::string::String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<core::fmt::Arguments<'a>> for Value<'a> {
    fn from(value: core::fmt::Arguments<'a>) -> Self {
        Value::Formatted(value)
    }
}

impl<'a> From<&core::fmt::Arguments<'a>> for Value<'a> {
    fn from(value: &core::fmt::Arguments<'a>) -> Self {
        Value::Formatted(*value)
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'a> From<&bool> for Value<'a> {
    fn from(value: &bool) -> Self {
        Value::Bool(*value)
    }
}

impl<'a> From<i64> for Value<'a> {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl<'a> From<&i64> for Value<'a> {
    fn from(value: &i64) -> Self {
        Value::I64(*value)
    }
}

impl<'a> From<f64> for Value<'a> {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}

impl<'a> From<&f64> for Value<'a> {
    fn from(value: &f64) -> Self {
        Value::F64(*value)
    }
}
