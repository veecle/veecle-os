use core::cell::Ref;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::pin::Pin;

use crate::datastore::Storable;
use crate::datastore::slot::{self, Slot};

/// Exclusive reader for a [`Storable`] type.
///
/// By being the sole reader for a [`Storable`] type, this reader can move the read value out.
/// The generic type `T` from the reader specifies the type of the value that is being read.
///
/// The reader allows reading the current value.
/// If no value for type `T` has been written yet, [`ExclusiveReader::read`] and
/// [`ExclusiveReader::take`] will return `None`.
///
/// # Usage
///
/// [`ExclusiveReader::wait_for_update`] allows waiting until the type is written to.
/// It will return immediately if an unseen value is available.
/// Unseen does not imply the value actually changed, just that an [`Actor`] has written a value.
/// A write of the same value still triggers [`ExclusiveReader::wait_for_update`] to resolve.
///
/// To illustrate:
/// ```text
/// - Writer writes 5
/// - Reader is woken and reads 5.
///   Reader waits for updates.
/// ...
/// - Writer writes 5 once again.
/// - Reader is woken and reads 5.
/// ...
/// ```
///
/// The reader is woken, even if the new value equals the old one.
/// The [`ExclusiveReader`] is only aware of the act of writing.
///
/// # Seen values
///
/// The reader tracks whether the current value has been "seen".
/// A value is marked as seen when any read method is called, such as [`ExclusiveReader::read`] or [`ExclusiveReader::take`].
/// A new write from a [`Writer`][super::Writer] marks the value as unseen.
///
/// [`ExclusiveReader::is_updated`] returns `true` if the current value is unseen.
/// [`ExclusiveReader::wait_for_update`] waits until an unseen value is available.
///
/// # Example
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, ExclusiveReader};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_reader(mut reader: ExclusiveReader<'_, Foo>) -> veecle_os_runtime::Never {
///     loop {
///         let value = reader.take_updated().await;
///     }
/// }
/// ```
///
/// [`Actor`]: crate::actor::Actor
#[derive(Debug)]
pub struct ExclusiveReader<'a, T>
where
    T: Storable + 'static,
{
    waiter: slot::Waiter<'a, T>,

    marker: PhantomData<fn(T)>,
}

impl<T> ExclusiveReader<'_, T>
where
    T: Storable + 'static,
{
    /// Reads the current value of a type.
    ///
    /// Marks the current value as seen.
    /// This method takes a closure to ensure the reference is not held across await points.
    #[veecle_telemetry::instrument]
    pub fn read<U>(&mut self, f: impl FnOnce(Option<&T::DataType>) -> U) -> U {
        self.waiter.update_generation();
        self.waiter.read(|value| {
            let value = value.as_ref();

            veecle_telemetry::trace!("Slot read", value = format_args!("{value:?}"));
            f(value)
        })
    }

    /// Reads the next unseen value of a type.
    ///
    /// Waits until an unseen value is available, then reads it.
    /// Marks the current value as seen.
    /// This method takes a closure to ensure the reference is not held across await points.
    #[veecle_telemetry::instrument]
    pub async fn read_updated<U>(&mut self, f: impl FnOnce(&T::DataType) -> U) -> U {
        self.wait_for_update().await;
        self.waiter.update_generation();
        self.waiter.read(|value| {
            let value = value.as_ref().unwrap();

            veecle_telemetry::trace!("Slot read", value = format_args!("{value:?}"));
            f(value)
        })
    }

    /// Reads and clones the current value.
    ///
    /// Marks the current value as seen.
    /// This is a wrapper around [`Self::read`] that additionally clones the value.
    /// You can use it instead of `reader.read(|c| c.clone())`.
    pub fn read_cloned(&mut self) -> Option<T::DataType>
    where
        T::DataType: Clone,
    {
        self.read(|t| t.cloned())
    }

    /// Reads and clones the next unseen value.
    ///
    /// Waits until an unseen value is available, then reads it.
    /// Marks the current value as seen.
    /// This is a wrapper around [`Self::read_updated`] that additionally clones the value.
    /// You can use it instead of `reader.read_updated(|c| c.clone())`.
    pub async fn read_updated_cloned(&mut self) -> T::DataType
    where
        T::DataType: Clone,
    {
        self.read_updated(|t| t.clone()).await
    }

    /// Returns `true` if an unseen value is available.
    ///
    /// A value becomes "seen" after calling [`read`][Self::read], [`read_updated`][Self::read_updated],
    /// [`take`][Self::take], [`take_updated`][Self::take_updated], or similar methods.
    pub fn is_updated(&self) -> bool {
        self.waiter.is_updated()
    }

    /// Waits for any write to occur.
    ///
    /// This future resolving does not imply that `previous_value != new_value`, just that a
    /// [`Writer`][super::Writer] has written a value of `T` since the last read operation.
    ///
    ///
    /// This returns `&mut Self` to allow chaining a call to [`read`][Self::read].
    #[veecle_telemetry::instrument]
    pub async fn wait_for_update(&mut self) -> &mut Self {
        self.waiter.wait().await;
        self
    }

    /// Takes the current value of the type, leaving behind `None`.
    ///
    /// Marks the current value as seen.
    pub fn take(&mut self) -> Option<T::DataType> {
        let span = veecle_telemetry::span!("take");
        let _guard = span.enter();

        self.waiter.update_generation();
        let value = self.waiter.take(span.context());

        veecle_telemetry::trace!("Slot value taken", value = format_args!("{value:?}"));

        value
    }

    /// Takes the next unseen value of the type, leaving behind `None`.
    ///
    /// Waits until an unseen value is available, then takes it.
    /// Marks the current value as seen.
    pub async fn take_updated(&mut self) -> T::DataType {
        let span = veecle_telemetry::span!("take");
        let _guard = span.enter();

        self.wait_for_update().await;
        self.waiter.update_generation();

        let value = self.waiter.take(span.context()).unwrap();

        veecle_telemetry::trace!("Slot value taken", value = format_args!("{value:?}"));

        value
    }
}

impl<'a, T> ExclusiveReader<'a, T>
where
    T: Storable + 'static,
{
    /// Creates a new `ExclusiveReader` from a `slot`.
    pub(crate) fn from_slot(slot: Pin<&'a Slot<T>>) -> Self {
        ExclusiveReader {
            waiter: slot.waiter(),
            marker: PhantomData,
        }
    }
}

impl<T> super::combined_readers::Sealed for ExclusiveReader<'_, T> where T: Storable {}

impl<T> super::combined_readers::CombinableReader for ExclusiveReader<'_, T>
where
    T: Storable,
{
    type ToBeRead = Option<T::DataType>;

    fn borrow(&mut self) -> Ref<'_, Self::ToBeRead> {
        self.waiter.update_generation();
        self.waiter.borrow()
    }

    async fn wait_for_update(&mut self) -> &mut Self {
        self.wait_for_update().await
    }

    fn is_updated(&self) -> bool {
        self.is_updated()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;
    use futures::FutureExt;

    use crate::datastore::{ExclusiveReader, Slot, Storable, Writer, generational};

    #[test]
    fn read() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut reader = ExclusiveReader::from_slot(slot.as_ref());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert_eq!(reader.read(|x| x.cloned()), None);
        assert_eq!(reader.read_cloned(), None);

        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        assert_eq!(
            reader.read(|x: Option<&Sensor>| x.cloned()),
            Some(Sensor(1))
        );
        assert_eq!(reader.read_cloned(), Some(Sensor(1)));
    }

    #[test]
    fn take() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut reader = ExclusiveReader::from_slot(slot.as_ref());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert_eq!(reader.take(), None);
        source.as_ref().increment_generation();
        writer.write(Sensor(10)).now_or_never().unwrap();
        assert_eq!(reader.take(), Some(Sensor(10)));
        assert_eq!(reader.take(), None);
    }

    #[test]
    fn wait_for_update() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut reader = ExclusiveReader::from_slot(slot.as_ref());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert!(reader.wait_for_update().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        reader.wait_for_update().now_or_never().unwrap();
        assert!(reader.is_updated());
    }
}
