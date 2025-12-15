//! Key-value attribute types for telemetry data.
//!
//! This module defines the types used to represent attributes in telemetry data.
//! Attributes are key-value pairs that provide additional context for spans,
//! events, and log messages.
//!
//! # Value Types
//!
//! The [`TransientValue`] enum supports common data types:
//! - **String**: Text values (adapted to platform string type)
//! - **Formatted**: Lazy format arguments (for `no_std` environments)
//! - **Bool**: Boolean values (true/false)
//! - **I64**: 64-bit signed integers
//! - **F64**: 64-bit floating-point numbers
//!
//! # Examples
//!
//! ```rust
//! use veecle_telemetry::types::StringType;
//! use veecle_telemetry::{KeyValue, TransientValue};
//!
//! // Create key-value pairs
//! let user_id = KeyValue::new("user_id", 123);
//! let username = KeyValue::new("username", "alice");
//! let is_admin = KeyValue::new("is_admin", true);
//! let score = KeyValue::new("score", 95.5);
//!
//! // Values can be created from various types
//! let string_value = TransientValue::String("hello".into());
//! let int_value = TransientValue::I64(42);
//! let bool_value = TransientValue::Bool(true);
//! let float_value = TransientValue::F64(3.14);
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
/// use veecle_telemetry::{KeyValue, TransientValue};
///
/// // Create attributes with different value types
/// let user_id = KeyValue::new("user_id", 123);
/// let username = KeyValue::new("username", "alice");
/// let is_active = KeyValue::new("is_active", true);
/// let score = KeyValue::new("score", 95.5);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyValue<'a, V> {
    /// The attribute key (name)
    #[serde(borrow)]
    pub key: StringType<'a>,

    /// The attribute value
    pub value: V,
}

impl<'a> KeyValue<'a, TransientValue<'a>> {
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
        V: Into<TransientValue<'a>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl<V> core::fmt::Display for KeyValue<'_, V>
where
    V: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[cfg(feature = "alloc")]
impl<V> ToStatic for KeyValue<'_, V>
where
    V: ToStatic,
{
    type Static = KeyValue<'static, V::Static>;

    fn to_static(&self) -> Self::Static {
        KeyValue {
            key: self.key.clone().into_owned().into(),
            value: self.value.to_static(),
        }
    }
}

/// A transient value that can be stored in a telemetry attribute.
///
/// This enum represents values that may contain non-Send types like `format_args!`,
/// making them suitable for local use but not for sending across threads.
/// Use [`OwnedValue`] for values that need to cross thread boundaries.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::TransientValue;
///
/// // Create values of different types
/// let text = TransientValue::String("hello world".into());
/// let number = TransientValue::I64(42);
/// let flag = TransientValue::Bool(true);
/// let rating = TransientValue::F64(4.5);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransientValue<'a> {
    /// A string value (adapted to platform string type)
    String(#[serde(borrow)] StringType<'a>),

    /// A `format_args!` call.
    #[serde(rename(serialize = "String"))]
    #[serde(skip_deserializing)]
    Formatted(core::fmt::Arguments<'a>),

    /// A boolean value
    Bool(bool),

    /// A 64-bit signed integer
    I64(i64),

    /// A 64-bit floating-point number
    F64(f64),
}

#[cfg(feature = "alloc")]
impl ToStatic for TransientValue<'_> {
    type Static = OwnedValue;

    fn to_static(&self) -> Self::Static {
        use alloc::string::ToString;

        match self {
            Self::String(s) => OwnedValue::String(s.clone().into_owned()),
            Self::Formatted(s) => OwnedValue::String(s.to_string()),
            Self::Bool(b) => OwnedValue::Bool(*b),
            Self::I64(i) => OwnedValue::I64(*i),
            Self::F64(f) => OwnedValue::F64(*f),
        }
    }
}

impl<'a> core::fmt::Display for TransientValue<'a> {
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

#[cfg(feature = "alloc")]
impl<'a> From<alloc::borrow::Cow<'a, str>> for TransientValue<'a> {
    fn from(value: alloc::borrow::Cow<'a, str>) -> Self {
        TransientValue::String(value)
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::string::String> for TransientValue<'a> {
    fn from(value: alloc::string::String) -> Self {
        TransientValue::String(value.into())
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a alloc::string::String> for TransientValue<'a> {
    fn from(value: &'a alloc::string::String) -> Self {
        TransientValue::String(value.into())
    }
}

impl<'a> From<&'a str> for TransientValue<'a> {
    fn from(value: &'a str) -> Self {
        #[cfg(feature = "alloc")]
        {
            TransientValue::String(alloc::borrow::Cow::Borrowed(value))
        }
        #[cfg(not(feature = "alloc"))]
        {
            TransientValue::String(value)
        }
    }
}

impl<'a> From<core::fmt::Arguments<'a>> for TransientValue<'a> {
    fn from(value: core::fmt::Arguments<'a>) -> Self {
        TransientValue::Formatted(value)
    }
}

impl From<bool> for TransientValue<'_> {
    fn from(value: bool) -> Self {
        TransientValue::Bool(value)
    }
}

impl From<i64> for TransientValue<'_> {
    fn from(value: i64) -> Self {
        TransientValue::I64(value)
    }
}

impl From<f64> for TransientValue<'_> {
    fn from(value: f64) -> Self {
        TransientValue::F64(value)
    }
}

/// An owned value that can be sent across thread boundaries.
///
/// Unlike [`TransientValue`], this type is fully owned and does not contain
/// any non-Send types like `format_args!`. This makes it suitable for
/// serialization and sending across threads via channels.
///
/// Cross-serialization compatible with [`TransientValue`] - the transient
/// variants will be converted to owned types during serialization.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::{OwnedValue, TransientValue};
///
/// // Create a TransientValue with format_args!
/// let count = 42;
/// let transient = TransientValue::Formatted(format_args!("count: {count}"));
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&transient)?;
/// assert_eq!(json, r#"{"String":"count: 42"}"#);
///
/// // Deserialize as OwnedValue
/// let owned: OwnedValue = serde_json::from_str(&json)?;
/// let OwnedValue::String(string) = owned else { panic!("unexpected variant") };
/// assert_eq!(string, "count: 42");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg(feature = "alloc")]
pub enum OwnedValue {
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
impl ToStatic for OwnedValue {
    type Static = OwnedValue;

    fn to_static(&self) -> Self::Static {
        self.clone()
    }
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for OwnedValue {
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
