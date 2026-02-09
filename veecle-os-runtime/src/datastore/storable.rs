//! Storable trait for types that can be stored in a slot.

use core::fmt::Debug;

/// Marks a type as an identifier for the inner `DataType`, which can be transferred via a slot.
///
/// # Usage
///
/// The trait allows separating the data type from the identifier type.
/// This allows registering multiple data type of the same type with unique identifiers, avoiding collisions.
///
/// This trait can be derived via the [`Storable`][derive@crate::datastore::Storable] derive macro,
/// see its documentation for more details.
///
/// ```
/// use veecle_os_runtime::Storable;
///
/// // Identifier type.
/// #[derive(Debug, Default)]
/// struct Sensor;
///
/// impl Storable for Sensor {
///     // Data type.
///     type DataType = u8;
/// }
/// ```
///
/// If the data type is unique and should be used as the identifier, the data type can be set to `Self`.
/// ```
/// use veecle_os_runtime::Storable;
///
/// // Identifier type.
/// #[derive(Debug, Default)]
/// struct Sensor{
///     // ...
/// };
///
/// impl Storable for Sensor {
///     // Data type.
///     type DataType = Self;
/// }
/// ```
pub trait Storable {
    /// The data type being read/written from/to a slot.
    type DataType: Debug;
}
