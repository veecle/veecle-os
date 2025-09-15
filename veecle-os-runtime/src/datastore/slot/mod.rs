mod slot;
mod storable;
mod waiter;

/// Implements [`Storable`][trait@Storable] for a struct or enum.
///
/// # Attributes
///
/// * `data_type = "Type"`: Sets the [`Storable::DataType`][type@Storable::DataType]. Defaults to `Self`.
/// * `crate = ::veecle_os_runtime`: Overrides the path to the `veecle-os-runtime` crate in case the import was renamed.
///
/// ```
/// use core::fmt::Debug;
/// use veecle_os_runtime::Storable;
///
/// // `DataType = Self`
/// #[derive(Debug, Storable)]
/// pub struct Sensor<T>
/// where
///     T: Debug,
/// {
///     test: u8,
///     test0: u8,
///     test1: T,
/// }
///
/// // `DataType = Self`
/// #[derive(Debug, Storable)]
/// pub struct Motor {
///     test: u8,
/// }
///
/// // `DataType = Self`
/// #[derive(Debug, Storable)]
/// pub enum Actuator {
///     Variant1,
///     Variant2(u8),
///     Variant3 { test: u8 },
/// }
///
/// // `DataType = u8`
/// #[derive(Storable)]
/// #[storable(data_type = "u8")]
/// pub struct EventId;
/// ```
#[doc(inline)]
pub use veecle_os_runtime_macros::Storable;

pub(crate) use self::slot::Slot;
pub use self::storable::Storable;
pub(crate) use self::waiter::Waiter;
