use core::cell::Ref;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::pin::Pin;

use pin_project::pin_project;

use crate::datastore::Storable;
use crate::datastore::initialized_reader::InitializedReader;
use crate::datastore::slot::{self, Slot};

/// Reader for a [`Storable`] type.
///
/// Allows [`Actor`]s to read a value of a type written by another actor.
/// The generic type `T` from the reader specifies the type of the value that is being read.
///
/// The reader allows reading the current value.
/// If no value for type `T` has been written to yet, [`Reader::read`] will return `None`.
/// See [`Self::wait_init`] for creating a reader that ensures available values for `T`.
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
/// The reader is woken, even if the new value equals the old one. The [`Reader`] is only aware of the act of writing.
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
/// async fn foo_reader(mut reader: Reader<'_, Foo>) -> std::convert::Infallible {
///     loop {
///         let processed_value = reader.wait_for_update().await.read(|value: Option<&Foo>| {
///             // do something with the value.
///         });
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
    /// Can be combined with [`Self::wait_for_update`] to wait for the value to be updated before reading it.
    ///
    /// This method takes a closure to ensure the reference is not held across await points.
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub fn read<U>(&self, f: impl FnOnce(Option<&T::DataType>) -> U) -> U {
        self.waiter.read(|value| {
            let value = value.as_ref();

            // TODO(DEV-532): add debug format
            #[cfg(feature = "veecle-telemetry")]
            veecle_telemetry::trace!("Slot read", type_name = self.waiter.inner_type_name());
            f(value)
        })
    }

    /// Reads and clones the current value.
    ///
    /// This is a wrapper around [`Self::read`] that additionally clones the value.
    /// You can use it instead of `reader.read(|c| c.clone())`.
    pub fn read_cloned(&self) -> Option<T::DataType>
    where
        T::DataType: Clone,
    {
        self.read(|t| t.cloned())
    }

    /// Waits for any write to occur.
    ///
    /// This future resolving does not imply that `previous_value != new_value`, just that a
    /// [`Writer`][super::Writer] has written a value of `T` since the last time this future resolved.
    ///
    /// This returns `&mut Self` to allow chaining a call to methods accessing the value, for example
    /// [`read`][Self::read`].
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub async fn wait_for_update(&mut self) -> &mut Self {
        self.waiter.wait().await;
        self.waiter.update_generation();
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

    /// Converts the `Reader` into an [`InitializedReader`].
    ///
    /// Pends until a value for `T` is available or resolves immediately if a value is already available.
    /// This will not mark the value as seen, [`InitializedReader::wait_for_update`] is unaffected by this method.
    pub async fn wait_init(self) -> InitializedReader<'a, T> {
        if self.read(|t| t.is_none()) {
            self.waiter.wait().await;
        }
        InitializedReader::new(self.waiter)
    }
}

impl<T> super::combined_readers::Sealed for Reader<'_, T> where T: Storable {}

impl<T> super::combined_readers::CombinableReader for Reader<'_, T>
where
    T: Storable,
{
    type ToBeRead = Option<T::DataType>;

    fn borrow(&self) -> Ref<'_, Self::ToBeRead> {
        self.waiter.borrow()
    }

    async fn wait_for_update(&mut self) {
        self.wait_for_update().await;
    }
}

#[cfg(test)]
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

        reader
            .wait_for_update()
            .now_or_never()
            .unwrap()
            .read(|x| assert_eq!(x, Some(&Sensor(1))));
    }
}
