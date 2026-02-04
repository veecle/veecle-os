use core::cell::Ref;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::pin::Pin;

use pin_project::pin_project;

use crate::datastore::Storable;
use crate::datastore::slot::{self, Slot};

/// Reader for a [`Storable`] type.
///
/// Allows [`Actor`]s to read a value of a type written by another actor.
/// The generic type `T` from the reader specifies the type of the value that is being read.
///
/// The reader allows reading the current value.
/// If no value for type `T` has been written to yet, [`Reader::read`] will return `None`.
///
/// # Usage
///
/// [`Reader::wait_for_update`] allows waiting until the type is written to.
/// It will return immediately if an unseen value is available.
/// Unseen does not imply the value actually changed, just that an [`Actor`] has written a value.
/// A write of the same value still triggers [`Reader::wait_for_update`] to resolve.
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
/// The [`Reader`] is only aware of the act of writing.
///
/// # Seen values
///
/// The reader tracks whether the current value has been "seen".
/// A value is marked as seen when any read method is called, such as [`Reader::read`] or [`Reader::read_updated`].
/// A new write from a [`Writer`][super::Writer] marks the value as unseen.
///
/// [`Reader::is_updated`] returns `true` if the current value is unseen.
/// [`Reader::wait_for_update`] waits until an unseen value is available.
///
/// # Example
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Reader};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_reader(mut reader: Reader<'_, Foo>) -> veecle_os_runtime::Never {
///     loop {
///         let processed_value = reader.read_updated(|value: &Foo| {
///             // do something with the value.
///         }).await;
///     }
/// }
/// ```
///
/// [`Actor`]: crate::actor::Actor
#[derive(Debug)]
#[pin_project]
pub struct Reader<'a, T>
where
    T: Storable + 'static,
{
    #[pin]
    waiter: slot::Waiter<'a, T>,

    marker: PhantomData<fn(T)>,
}

impl<T> Reader<'_, T>
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
    /// or similar read methods.
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
}

impl<'a, T> Reader<'a, T>
where
    T: Storable + 'static,
{
    /// Creates a new `Reader` from a `slot`.
    pub(crate) fn from_slot(slot: Pin<&'a Slot<T>>) -> Self {
        Reader {
            waiter: slot.waiter(),
            marker: PhantomData,
        }
    }
}

impl<T> super::combined_readers::Sealed for Reader<'_, T> where T: Storable {}

impl<T> super::combined_readers::CombinableReader for Reader<'_, T>
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

    use crate::datastore::{Reader, Slot, Storable, Writer, generational};

    #[test]
    fn wait_for_update() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut reader = Reader::from_slot(slot.as_ref());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert!(reader.wait_for_update().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        reader.wait_for_update().now_or_never().unwrap();
        assert!(reader.is_updated());
    }
}
