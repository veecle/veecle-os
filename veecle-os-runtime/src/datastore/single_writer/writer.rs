//! Writer for single-writer slots.

use super::slot::Slot;
use crate::Sealed;
use crate::cons::{Cons, Nil};
use crate::datastore::Datastore;
use crate::datastore::modify::Modify;
use crate::datastore::sync::generational;
use crate::datastore::{DefinesSlot, Storable, StoreRequest};
use core::fmt::Debug;
use core::pin::Pin;

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
/// # use veecle_os_runtime::{Storable, single_writer::Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(mut writer: Writer<'_, Foo>) -> veecle_os_runtime::Never {
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
/// # use veecle_os_runtime::{Storable, single_writer::Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo(usize);
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(
///     mut writer: Writer<'_, Foo>,
/// ) -> veecle_os_runtime::Never {
///     loop {
///         // This call will yield to any readers needing to read the last value if the value was accessed mutably.
///         // The closure will run after yielding and right before continuing to the rest of the function.
///         writer.modify(|mut previous_value| {
///             if let Some(value) = previous_value.as_mut(){
///                 *value = Foo(value.0 + 1);
///             }
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
/// # use veecle_os_runtime::{Storable, single_writer::{Reader, Writer}};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_writer(mut writer: Writer<'_, Foo>) -> veecle_os_runtime::Never {
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
/// [`Reader`]: super::Reader
#[derive(Debug)]
pub struct Writer<'a, T>
where
    T: Storable + 'static,
{
    slot: Pin<&'a Slot<T>>,
    waiter: generational::Waiter<'a>,
}

impl<T> Writer<'_, T>
where
    T: Storable + 'static,
{
    /// Writes a new value and notifies readers.
    #[veecle_telemetry::instrument]
    pub async fn write(&mut self, item: T::DataType) {
        self.modify(|mut slot| {
            let _ = *slot.insert(item);
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

    /// Updates the value in-place and notifies readers if the value was modified.
    ///
    /// Readers are notified that the value was modified if it's mutably accessed via `DerefMut`
    pub async fn modify(&mut self, f: impl FnOnce(Modify<T::DataType>)) {
        use veecle_telemetry::future::FutureExt;
        let span = veecle_telemetry::span!("modify");
        let span_context = span.context();
        (async move {
            self.ready().await;
            let mut modified = false;

            self.slot.modify(
                |value| {
                    let modify_wrapper = Modify::new(value, &mut modified);
                    f(modify_wrapper);
                    if modified {
                        veecle_telemetry::trace!(
                            "Slot modified",
                            value = format_args!("{value:?}")
                        );
                    }
                },
                span_context,
            );

            // Only block writes and notify readers if the value was modified.
            if modified {
                self.waiter.update_generation();
                self.slot.increment_generation();
            }
        })
        .with_span(span)
        .await;
    }
}

impl<'a, T> Writer<'a, T>
where
    T: Storable + 'static,
{
    pub(crate) fn new(waiter: generational::Waiter<'a>, slot: Pin<&'a Slot<T>>) -> Self {
        slot.take_writer();
        Self { slot, waiter }
    }
}

impl<'a, T> DefinesSlot for Writer<'a, T>
where
    T: Storable,
{
    type Slot = Cons<Slot<T>, Nil>;
}

impl<T> Sealed for Writer<'_, T> where T: Storable + 'static {}

impl<'a, T> StoreRequest<'a> for Writer<'a, T>
where
    T: Storable + 'static,
{
    async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self {
        Writer::new(datastore.source().waiter(), datastore.slot(requestor))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::datastore::Storable;
    use crate::datastore::single_writer::{Slot, Writer};
    use crate::datastore::sync::generational;
    use core::pin::pin;
    use std::ops::DerefMut;

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
    fn modify_only_blocks_next_write_when_returning_true() {
        use futures::FutureExt;

        #[derive(Debug)]
        pub struct Data;
        impl Storable for Data {
            type DataType = Self;
        }

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data>::new());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        source.as_ref().increment_generation();
        assert!(writer.ready().now_or_never().is_some());

        assert!(
            writer
                .modify(|value| {
                    let _ = *value;
                })
                .now_or_never()
                .is_some()
        );
        assert!(writer.write(Data).now_or_never().is_some());

        // After a real write the writer should be blocked again.
        assert!(writer.ready().now_or_never().is_none());

        source.as_ref().increment_generation();
        assert!(writer.ready().now_or_never().is_some());

        assert!(
            writer
                .modify(|mut value| {
                    let _ = value.deref_mut();
                })
                .now_or_never()
                .is_some()
        );
        assert!(writer.ready().now_or_never().is_none());
    }
}
