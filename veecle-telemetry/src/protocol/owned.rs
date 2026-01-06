//! Type aliases for owned/sendable usage (IPC, serialization).
//!
//! These aliases use [`Value`] which is fully owned and Send-safe.
//! Use these for telemetry that needs to cross thread boundaries or be serialized.
//!
//!
//! See the [main protocol module][super] docs for more details on the protocol modules provided.

use alloc::string::ToString;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::protocol::{base, transient};

/// Owned storage family using owned data.
///
/// This family uses owned types that are Send-safe and can cross
/// thread boundaries or be serialized.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Owned;

impl base::Sealed for Owned {}
impl base::StorageFamily for Owned {
    type String<'a>
        = alloc::string::String
    where
        Self: 'a;

    type List<'a, T: Clone + core::fmt::Debug + serde::Serialize + 'a>
        = alloc::vec::Vec<T>
    where
        Self: 'a;

    type Value<'a>
        = Value
    where
        Self: 'a;
}

// Re-export non-generic types for convenience.
pub use base::{
    ProcessId, Severity, SpanAddLinkMessage, SpanCloseMessage, SpanContext, SpanEnterMessage,
    SpanExitMessage, SpanId, ThreadId, TimeSyncMessage,
};

/// Key-value pair with owned value (Send-safe, for IPC).
pub type KeyValue = base::KeyValue<'static, Owned>;
/// Instance message with owned values (Send-safe, for IPC).
pub type InstanceMessage = base::InstanceMessage<'static, Owned>;
/// Telemetry message with owned values (Send-safe, for IPC).
pub type TelemetryMessage = base::TelemetryMessage<'static, Owned>;
/// Log message with owned values (Send-safe, for IPC).
pub type LogMessage = base::LogMessage<'static, Owned>;
/// Tracing message with owned values (Send-safe, for IPC).
pub type TracingMessage = base::TracingMessage<'static, Owned>;
/// Span create message with owned values (Send-safe, for IPC).
pub type SpanCreateMessage = base::SpanCreateMessage<'static, Owned>;
/// Span set attribute message with owned values (Send-safe, for IPC).
pub type SpanSetAttributeMessage = base::SpanSetAttributeMessage<'static, Owned>;
/// Span add event message with owned values (Send-safe, for IPC).
pub type SpanAddEventMessage = base::SpanAddEventMessage<'static, Owned>;

/// An owned value that can be sent across thread boundaries.
///
/// Unlike [`transient::Value`], this type is fully owned and does not contain
/// any non-Send types like `format_args!`. This makes it suitable for
/// serialization and sending across threads via channels.
///
/// Cross-serialization compatible with [`transient::Value`] - the transient
/// variants will be converted to owned types during serialization.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::protocol::{owned, transient};
///
/// // Create a `transient::Value` with `format_args!`.
/// let count = 42;
/// let transient = transient::Value::Formatted(format_args!("count: {count}"));
///
/// // Serialize to JSON.
/// let json = serde_json::to_string(&transient)?;
/// assert_eq!(json, r#"{"String":"count: 42"}"#);
///
/// // Deserialize as `owned::Value`.
/// let owned: owned::Value = serde_json::from_str(&json)?;
/// let owned::Value::String(string) = owned else { panic!("unexpected variant") };
/// assert_eq!(string, "count: 42");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg(feature = "alloc")]
pub enum Value {
    /// A string value (owned)
    String(alloc::string::String),

    /// A boolean value
    Bool(bool),

    /// A 64-bit signed integer
    I64(i64),

    /// A 64-bit floating-point number
    F64(f64),
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // For strings, debug print so they will get delimiters, since we are explicitly
            // representing strings rather than directly human-targeted text, and they will be used
            // in situations where knowing where the string ends is important.
            Self::String(value) => write!(f, "{value:?}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::F64(value) => write!(f, "{value}"),
        }
    }
}

// Conversions from transient types to owned types

impl From<transient::InstanceMessage<'_>> for InstanceMessage {
    fn from(value: transient::InstanceMessage<'_>) -> Self {
        InstanceMessage {
            thread_id: value.thread_id,
            message: value.message.into(),
        }
    }
}

impl From<transient::TelemetryMessage<'_>> for TelemetryMessage {
    fn from(value: transient::TelemetryMessage<'_>) -> Self {
        match value {
            transient::TelemetryMessage::Log(msg) => TelemetryMessage::Log(msg.into()),
            transient::TelemetryMessage::Tracing(msg) => TelemetryMessage::Tracing(msg.into()),
            transient::TelemetryMessage::TimeSync(msg) => TelemetryMessage::TimeSync(msg),
        }
    }
}

impl From<transient::LogMessage<'_>> for LogMessage {
    fn from(value: transient::LogMessage<'_>) -> Self {
        LogMessage {
            time_unix_nano: value.time_unix_nano,
            severity: value.severity,
            body: value.body.to_string(),
            attributes: Vec::from_iter(value.attributes.as_ref().iter().map(|kv| kv.into())),
        }
    }
}

impl From<transient::TracingMessage<'_>> for TracingMessage {
    fn from(value: transient::TracingMessage<'_>) -> Self {
        match value {
            transient::TracingMessage::CreateSpan(msg) => TracingMessage::CreateSpan(msg.into()),
            transient::TracingMessage::EnterSpan(msg) => TracingMessage::EnterSpan(msg),
            transient::TracingMessage::ExitSpan(msg) => TracingMessage::ExitSpan(msg),
            transient::TracingMessage::CloseSpan(msg) => TracingMessage::CloseSpan(msg),
            transient::TracingMessage::AddEvent(msg) => TracingMessage::AddEvent(msg.into()),
            transient::TracingMessage::AddLink(msg) => TracingMessage::AddLink(msg),
            transient::TracingMessage::SetAttribute(msg) => {
                TracingMessage::SetAttribute(msg.into())
            }
        }
    }
}

impl From<transient::SpanCreateMessage<'_>> for SpanCreateMessage {
    fn from(value: transient::SpanCreateMessage<'_>) -> Self {
        SpanCreateMessage {
            span_id: value.span_id,
            name: value.name.to_string(),
            start_time_unix_nano: value.start_time_unix_nano,
            attributes: Vec::from_iter(value.attributes.as_ref().iter().map(|kv| kv.into())),
        }
    }
}

impl From<transient::SpanSetAttributeMessage<'_>> for SpanSetAttributeMessage {
    fn from(value: transient::SpanSetAttributeMessage<'_>) -> Self {
        SpanSetAttributeMessage {
            span_id: value.span_id,
            attribute: value.attribute.into(),
        }
    }
}

impl From<transient::SpanAddEventMessage<'_>> for SpanAddEventMessage {
    fn from(value: transient::SpanAddEventMessage<'_>) -> Self {
        SpanAddEventMessage {
            span_id: value.span_id,
            name: value.name.to_string(),
            time_unix_nano: value.time_unix_nano,
            attributes: Vec::from_iter(value.attributes.as_ref().iter().map(|kv| kv.into())),
        }
    }
}

impl From<transient::KeyValue<'_>> for KeyValue {
    fn from(value: transient::KeyValue<'_>) -> Self {
        KeyValue {
            key: value.key.to_string(),
            value: value.value.into(),
        }
    }
}

impl From<&transient::KeyValue<'_>> for KeyValue {
    fn from(value: &transient::KeyValue<'_>) -> Self {
        KeyValue {
            key: value.key.to_string(),
            value: (&value.value).into(),
        }
    }
}

impl From<transient::Value<'_>> for Value {
    fn from(value: transient::Value<'_>) -> Self {
        match value {
            transient::Value::String(s) => Value::String(s.to_string()),
            transient::Value::Formatted(s) => Value::String(s.to_string()),
            transient::Value::Bool(b) => Value::Bool(b),
            transient::Value::I64(i) => Value::I64(i),
            transient::Value::F64(f) => Value::F64(f),
        }
    }
}

impl From<&transient::Value<'_>> for Value {
    fn from(value: &transient::Value<'_>) -> Self {
        match value {
            transient::Value::String(s) => Value::String(s.to_string()),
            transient::Value::Formatted(s) => Value::String(s.to_string()),
            transient::Value::Bool(b) => Value::Bool(*b),
            transient::Value::I64(i) => Value::I64(*i),
            transient::Value::F64(f) => Value::F64(*f),
        }
    }
}
