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
/// use veecle_os_runtime::{Flatten, MetricBuffer};
///
/// // Identifier type.
/// #[derive(Debug, Default)]
/// struct Sensor;
///
/// impl Flatten for Sensor {
///     fn flatten(&self, _buffer: &mut impl MetricBuffer) {}
/// }
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
/// use veecle_os_runtime::{Flatten, MetricBuffer};
///
/// // Identifier type.
/// #[derive(Debug, Default)]
/// struct Sensor{
///     // ...
/// };
///
/// impl Flatten for Sensor {
///     fn flatten(&self, _buffer: &mut impl MetricBuffer) {}
/// }
///
/// impl Storable for Sensor {
///     // Data type.
///     type DataType = Self;
/// }
/// ```
pub trait Storable {
    /// The data type being read/written from/to a slot.
    type DataType: Debug + Flatten;
}

/// Allows a type to emit its telemetry representation as key-value pairs into a [`MetricBuffer`].
///
/// Each call to [`flatten`](Flatten::flatten) should write one or more entries into the buffer,
/// where each entry is a static key and a [`Value`](veecle_telemetry::protocol::transient::Value).
///
/// For primitive types (e.g. `u8`, `f32`, `bool`), a default implementation is provided that
/// emits a single `"value"` key. Composite types should implement this trait to emit one entry
/// per field. Types that do not produce telemetry can provide an empty implementation.
pub trait Flatten {
    /// Writes telemetry key-value pairs into the given `buffer`.
    fn flatten(&self, buffer: &mut impl MetricBuffer);
}

/// Receiver for key-value telemetry entries produced by [`Flatten::flatten`].
///
/// Implementors define how and where metric entries are stored.
pub trait MetricBuffer {
    /// Adds a metric entry to this buffer.
    fn add_metric(
        &mut self,
        key: &'static str,
        value: veecle_telemetry::protocol::transient::Value<'static>,
    );
}

macro_rules! impl_flatten_integer {
    ($($ty:ty),*) => {
        $(
            impl Flatten for $ty {
                fn flatten(&self, buffer: &mut impl MetricBuffer) {
                    buffer.add_metric("value", veecle_telemetry::protocol::transient::Value::I64(*self as i64));
                }
            }
        )*
    };
}

macro_rules! impl_flatten_float {
    ($($ty:ty),*) => {
        $(
            impl Flatten for $ty {
                fn flatten(&self, buffer: &mut impl MetricBuffer) {
                    buffer.add_metric("value", veecle_telemetry::protocol::transient::Value::F64(*self as f64));
                }
            }
        )*
    };
}

impl_flatten_integer!(u8, u16, u32, i8, i16, i32, i64);
impl_flatten_float!(f32, f64);

impl Flatten for bool {
    fn flatten(&self, buffer: &mut impl MetricBuffer) {
        buffer.add_metric(
            "value",
            veecle_telemetry::protocol::transient::Value::Bool(*self),
        );
    }
}

impl<T: Flatten, const N: usize> Flatten for [T; N] {
    fn flatten(&self, buffer: &mut impl MetricBuffer) {
        for item in self {
            item.flatten(buffer);
        }
    }
}
