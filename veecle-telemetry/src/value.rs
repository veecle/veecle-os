//! Key-value attribute types for telemetry data.
//!
//! This module defines the types used to represent attributes in telemetry data.
//! Attributes are key-value pairs that provide additional context for spans,
//! events, and log messages.
//!
//! # Value Types
//!
//! The [`Value`] enum supports common data types:
//! - **String**: Text values (adapted to platform string type)
//! - **Bool**: Boolean values (true/false)
//! - **I64**: 64-bit signed integers
//! - **F64**: 64-bit floating-point numbers
//!
//! # Examples
//!
//! ```rust
//! use veecle_telemetry::types::StringType;
//! use veecle_telemetry::{KeyValue, Value};
//!
//! // Create key-value pairs
//! let user_id = KeyValue::new("user_id", 123);
//! let username = KeyValue::new("username", "alice");
//! let is_admin = KeyValue::new("is_admin", true);
//! let score = KeyValue::new("score", 95.5);
//!
//! // Values can be created from various types
//! let string_value = Value::String("hello".into());
//! let int_value = Value::I64(42);
//! let bool_value = Value::Bool(true);
//! let float_value = Value::F64(3.14);
//! ```

use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::to_static::ToStatic;
use crate::types::StringType;

/// A key-value attribute pair used in telemetry data.
///
/// Key-value pairs provide additional context for spans, events, and log messages.
/// The key is typically a string identifier, and the value can be one of several
/// supported data types.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::types::StringType;
/// use veecle_telemetry::{KeyValue, Value};
///
/// // Create attributes with different value types
/// let user_id = KeyValue::new("user_id", 123);
/// let username = KeyValue::new("username", "alice");
/// let is_active = KeyValue::new("is_active", true);
/// let score = KeyValue::new("score", 95.5);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyValue<'a> {
    /// The attribute key (name)
    #[serde(borrow)]
    pub key: StringType<'a>,
    /// The attribute value
    #[serde(borrow)]
    pub value: Value<'a>,
}

impl<'a> KeyValue<'a> {
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
    /// use veecle_telemetry::KeyValue;
    ///
    /// let user_id = KeyValue::new("user_id", 123);
    /// let username = KeyValue::new("username", "alice");
    /// ```
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<StringType<'a>>,
        V: Into<Value<'a>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl core::fmt::Display for KeyValue<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[cfg(feature = "alloc")]
impl ToStatic for KeyValue<'_> {
    type Static = KeyValue<'static>;

    fn to_static(&self) -> Self::Static {
        KeyValue {
            key: self.key.clone().into_owned().into(),
            value: self.value.to_static(),
        }
    }
}

/// A value that can be stored in a telemetry attribute.
///
/// This enum represents the different types of values that can be associated
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::Value;
///
/// // Create values of different types
/// let text = Value::String("hello world".into());
/// let number = Value::I64(42);
/// let flag = Value::Bool(true);
/// let rating = Value::F64(4.5);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value<'a> {
    /// A string value (adapted to platform string type)
    String(#[serde(borrow)] StringType<'a>),
    /// A boolean value
    Bool(bool),
    /// A 64-bit signed integer
    I64(i64),
    /// A 64-bit floating-point number
    F64(f64),
}

#[cfg(feature = "alloc")]
impl ToStatic for Value<'_> {
    type Static = Value<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            Value::String(s) => Value::String(s.clone().into_owned().into()),
            Value::Bool(b) => Value::Bool(*b),
            Value::I64(i) => Value::I64(*i),
            Value::F64(f) => Value::F64(*f),
        }
    }
}

impl<'a> core::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // For strings, debug print so they will get delimiters, since we are explicitly
            // representing strings rather than directly human-targeted text, and they will be used
            // in situations where knowing where the string ends is important.
            Value::String(value) => write!(f, "{value:?}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::I64(value) => write!(f, "{value}"),
            Value::F64(value) => write!(f, "{value}"),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::borrow::Cow<'a, str>> for Value<'a> {
    fn from(value: alloc::borrow::Cow<'a, str>) -> Self {
        Value::String(value)
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::string::String> for Value<'a> {
    fn from(value: alloc::string::String) -> Self {
        Value::String(value.into())
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a alloc::string::String> for Value<'a> {
    fn from(value: &'a alloc::string::String) -> Self {
        Value::String(value.into())
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        #[cfg(feature = "alloc")]
        {
            Value::String(alloc::borrow::Cow::Borrowed(value))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Value::String(value)
        }
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value<'_> {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl From<f64> for Value<'_> {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}
