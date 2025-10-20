use core::fmt::Debug;
use core::marker::PhantomData;
use core::pin::Pin;

use super::slot::Slot;
use super::{Storable, generational};

/// Writer for a [`Storable`] type.
///
/// Allows [`Actor`]s to write a particular type read by another actor.
/// The generic type `T` from the writer specifies the type of the value that is being written.
///
/// # Usage
///
/// All [`Reader`]s are guaranteed to be able to observe every write.
/// For this reason, [`Writer::write`] is an async method.
/// It will resolve once all [`Actor`]s awaiting a [`Reader`] for the same type had the chance to read the value.
/// Typically, this only occurs when trying to write two values back to back.
/// If all [`Reader`]s already had the chance to read the value, [`Writer::write`] will resolve immediately.
/// The same is true for [`Writer::modify`].
///
/// # Examples
///
/// ```rust
/// // Writing a value.
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(mut writer: Writer<'_, Foo>) -> std::convert::Infallible {
///     loop {
///         // This call will yield to any readers needing to read the last value.
///         writer.write(Foo::default()).await;
///     }
/// }
/// ```
///
/// ```rust
/// // Modifying a value.
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(
///     mut writer: Writer<'_, Foo>,
/// ) -> std::convert::Infallible {
///     loop {
///         // This call will yield to any readers needing to read the last value.
///         // The closure will run after yielding and right before continuing to the rest of the function.
///         writer.modify(|previous_value: &mut Option<Foo>| {
///             // mutate the previous value
///         }).await;
///     }
/// }
/// ```
///
/// [`Writer::ready`] allows separating the "waiting" from the "writing",
/// After [`Writer::ready`] returns, the next write or modification will happen immediately.
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Reader, Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(mut writer: Writer<'_, Foo>) -> std::convert::Infallible {
///     loop {
///         // This call may yield to any readers needing to read the last value.
///         writer.ready().await;
///
///         // This call will return immediately.
///         writer.write(Foo::default()).await;
///         // This call will yield to any readers needing to read the last value.
///         writer.write(Foo::default()).await;
///     }
/// }
/// ```
///
/// [`Actor`]: crate::Actor
/// [`Reader`]: crate::Reader
#[derive(Debug)]
pub struct Writer<'a, T>
where
    T: Storable + 'static,
{
    slot: Pin<&'a Slot<T>>,
    waiter: generational::Waiter<'a>,
    marker: PhantomData<fn(T)>,
}

impl<T> Writer<'_, T>
where
    T: Storable + 'static,
{
    /// Writes a new value and notifies readers.
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub async fn write(&mut self, item: T::DataType) {
        self.modify(|slot| {
            let _ = slot.insert(item);
        })
        .await;
    }

    /// Waits for the writer to be ready to perform a write operation.
    ///
    /// After awaiting this method, the next call to [`Writer::write()`]
    /// or [`Writer::modify()`] is guaranteed to resolve immediately.
    pub async fn ready(&mut self) {
        let _ = self.waiter.wait().await;
    }

    /// Updates the value in-place and notifies readers.
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub async fn modify(&mut self, f: impl FnOnce(&mut Option<T::DataType>)) {
        self.ready().await;
        self.waiter.update_generation();

        #[cfg(feature = "veecle-telemetry")]
        let type_name = self.slot.inner_type_name();

        self.slot.modify(|value| {
            f(value);

            // TODO(DEV-532): add debug format
            #[cfg(feature = "veecle-telemetry")]
            veecle_telemetry::trace!("Slot modified", type_name);
        });
        self.slot.increment_generation();
    }

    /// Reads the current value of a type.
    ///
    /// This method takes a closure to ensure the reference is not held across await points.
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub fn read<U>(&self, f: impl FnOnce(Option<&T::DataType>) -> U) -> U {
        #[cfg(feature = "veecle-telemetry")]
        let type_name = self.slot.inner_type_name();
        self.slot.read(|value| {
            let value = value.as_ref();
            // TODO(DEV-532): add debug format
            #[cfg(feature = "veecle-telemetry")]
            veecle_telemetry::trace!("Slot read", type_name);
            f(value)
        })
    }
}

impl<'a, T> Writer<'a, T>
where
    T: Storable + 'static,
{
    pub(crate) fn new(waiter: generational::Waiter<'a>, slot: Pin<&'a Slot<T>>) -> Self {
        slot.take_writer();
        Self {
            slot,
            waiter,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::datastore::{Slot, Storable, Writer, generational};
    use core::pin::pin;

    #[test]
    fn ready_waits_for_increment() {
        use futures::FutureExt;
        #[derive(Debug)]
        pub struct Data();
        impl Storable for Data {
            type DataType = Self;
        }

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data>::new());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        // Part 1. Initially, the writer is not ready. Calls to
        // ready() will not resolve immediately in a single Future::poll() call,
        // indicating that the writer needs more time. Additionally we check that
        // calls to write() are also not resolving immediately, demonstrating that
        // ready() actually was correct.
        assert!(writer.ready().now_or_never().is_none());
        assert!(writer.write(Data {}).now_or_never().is_none());

        // Part 2. Increment the generation, which signals that the writer
        // should be ready again. After the increment, ready() and write()
        // are expected to resolve in a single Future::poll() call.
        source.as_ref().increment_generation();
        assert!(writer.ready().now_or_never().is_some());
        assert!(writer.write(Data {}).now_or_never().is_some());

        // Part 3. Trying to write again before the generation increments should be blocked.
        assert!(writer.ready().now_or_never().is_none());
        assert!(writer.write(Data {}).now_or_never().is_none());
    }

    #[test]
    fn read_reads_latest_written_value() {
        use futures::FutureExt;
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Data(usize);
        impl Storable for Data {
            type DataType = Self;
        }

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data>::new());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        writer.read(|current_data| assert!(current_data.is_none()));

        source.as_ref().increment_generation();

        let want = Data(1);
        writer.write(want).now_or_never().unwrap();
        writer.read(|got| assert_eq!(got, Some(&want)));

        source.as_ref().increment_generation();

        let want = Data(2);
        writer.write(want).now_or_never().unwrap();
        writer.read(|got| assert_eq!(got, Some(&want)));
    }
}
