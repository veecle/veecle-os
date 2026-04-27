//! Traits for emitting telemetry key-value pairs from data types.

use crate::protocol::transient::Value;

/// Allows a type to emit its telemetry representation as key-value pairs into a [`MetricBuffer`].
///
/// Each call to [`flatten`](Flatten::flatten) should write one or more entries into the buffer,
/// where each entry is a static key and a [`Value`].
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
    fn add_metric(&mut self, key: &str, value: Value<'_>);
}

macro_rules! impl_flatten_integer {
    ($($ty:ty),*) => {
        $(
            impl Flatten for $ty {
                fn flatten(&self, buffer: &mut impl MetricBuffer) {
                    buffer.add_metric("value", Value::I64(i64::from(*self)));
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
                    buffer.add_metric("value", Value::F64(f64::from(*self)));
                }
            }
        )*
    };
}

impl_flatten_integer!(u8, u16, u32, i8, i16, i32);
impl_flatten_float!(f32, f64);

impl Flatten for i64 {
    fn flatten(&self, buffer: &mut impl MetricBuffer) {
        buffer.add_metric("value", Value::I64(*self));
    }
}

impl Flatten for bool {
    fn flatten(&self, buffer: &mut impl MetricBuffer) {
        buffer.add_metric("value", Value::Bool(*self));
    }
}
