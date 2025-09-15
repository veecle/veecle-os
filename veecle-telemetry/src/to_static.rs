//! Utilities for converting borrowed data to owned data.
//!
//! This module provides traits and implementations for converting types with lifetime
//! parameters to equivalent types with `'static` lifetime.
//! This is essential for storing telemetry data in contexts that require owned data, such as when sending
//! data across thread boundaries or storing it in global collectors.
//!
//! # Core Trait
//!
//! The [`ToStatic`] trait provides a standardized way to convert borrowed data to
//! owned data while preserving the original structure and semantics.
//!
//! # Usage
//!
//! This is primarily used internally by the telemetry system to ensure that
//! telemetry data can be safely stored and transmitted regardless of the original
//! lifetime constraints of the input data.

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

/// A trait for converting types with lifetime parameters to equivalent types with 'static lifetime.
pub trait ToStatic: Clone {
    /// The same type but with 'static lifetime and owned data.
    type Static: 'static + Clone + Send + Sync;

    /// Converts this type to the equivalent type with 'static lifetime.
    ///
    /// This method creates owned copies of any borrowed data, allowing the
    /// resulting type to be used in contexts that require 'static lifetime.
    fn to_static(&self) -> Self::Static;
}

#[cfg(feature = "alloc")]
impl ToStatic for Cow<'_, str> {
    type Static = Cow<'static, str>;

    fn to_static(&self) -> Self::Static {
        self.clone().into_owned().into()
    }
}
