//! Type definitions that adapt to different platform capabilities.
//!
//! This module provides type aliases and utilities that adapt to the current
//! platform configuration.
//! The types change behavior based on available features, allowing the same code to work efficiently in both `std` and
//! `no_std` environments.
//!
//! # Platform Adaptation
//!
//! - **With `alloc`**: Uses `Cow<'_, str>` for strings and `Cow<'_, [T]>` for lists
//! - **Without `alloc`**: Uses `&str` for strings and `&[T]` for lists
//!
//! This design allows for efficient zero-copy operation in `no_std` environments
//! while providing flexibility for owned data when allocation is available.
//!
//! # Examples
//!
//! ```rust
//! use veecle_telemetry::types::{ListType, StringType, list_from_slice};
//!
//! // StringType adapts to the platform
//! let message: StringType = "Hello, world!".into();
//!
//! // ListType adapts to the platform
//! let data = [1, 2, 3, 4, 5];
//! let list: ListType<'_, i32> = list_from_slice(&data);
//! ```

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

/// A string type which changes depending on the platform.
///
/// When the `alloc` feature is enabled, this is `Cow<'a, str>` which can hold
/// either borrowed or owned string data.
/// When `alloc` is disabled, this is
/// `&'a str` which only holds borrowed string data.
#[cfg(feature = "alloc")]
pub type StringType<'a> = Cow<'a, str>;

/// A string type which changes depending on the platform.
///
/// When the `alloc` feature is enabled, this is `Cow<'a, str>` which can hold
/// either borrowed or owned string data.
/// When `alloc` is disabled, this is `&'a str` which only holds borrowed string data.
#[cfg(not(feature = "alloc"))]
pub type StringType<'a> = &'a str;

/// A list type which changes depending on the platform.
///
/// When the `alloc` feature is enabled, this is `Cow<'a, [T]>` which can hold
/// either borrowed or owned slice data.
/// When `alloc` is disabled, this is `&'a [T]` which only holds borrowed slice data.
#[cfg(feature = "alloc")]
pub type ListType<'a, T> = Cow<'a, [T]>;

/// A list type which changes depending on the platform.
///
/// When the `alloc` feature is enabled, this is `Cow<'a, [T]>` which can hold
/// either borrowed or owned slice data. When `alloc` is disabled, this is
/// `&'a [T]` which only holds borrowed slice data.
#[cfg(not(feature = "alloc"))]
pub type ListType<'a, T> = &'a [T];

/// Converts a slice to the currently active [`ListType`].
///
/// This function adapts to the current platform configuration:
/// - With `alloc`: Creates a `Cow::Borrowed` from the slice
/// - Without `alloc`: Returns the slice directly
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::types::{ListType, list_from_slice};
///
/// let data = [1, 2, 3, 4, 5];
/// let list: ListType<'_, i32> = list_from_slice(&data);
/// assert_eq!(list.len(), 5);
/// ```
pub fn list_from_slice<T>(slice: &[T]) -> ListType<'_, T>
where
    T: Clone,
{
    #[cfg(feature = "alloc")]
    {
        slice.into()
    }
    #[cfg(not(feature = "alloc"))]
    {
        slice
    }
}
