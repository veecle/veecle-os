//! Writer for mpsc slots.

use core::fmt::Debug;
use core::pin::Pin;

use super::slot::Slot;
use crate::Sealed;
use crate::cons::Nil;
use crate::datastore::sync::generational;
use crate::datastore::{Datastore, DefinesSlot, Storable, StoreRequest};

/// Writer for an mpsc [`Storable`] type.
///
/// Multiple writers can coexist for the same type, each writing to their own value.
/// A single exclusive [`Reader`] consumes values from all writers.
///
/// The generic type `T` specifies the type of the value being written.
/// The const generic `N` specifies the maximum number of writers.
///
/// # Usage
///
/// Each writer is assigned a unique place to store a value.
/// [`Writer::write`] stores a value in that place and notifies the reader.
/// Like [`single_writer::Writer`], [`Writer::write`] is async and waits to give the [`Reader`] a chance to read the value.
///
/// # Examples
///
/// ```rust
/// // Writing values.
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, mpsc::Writer};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Command(usize);
/// #
/// #[veecle_os_runtime::actor]
/// async fn command_sender<const N: usize>(mut writer: Writer<'_, Command, N>) -> veecle_os_runtime::Never {
///     let mut counter = 0;
///     loop {
///         // After the first write, this call will yield to the executor before allowing the next write.
///         writer.write(Command(counter)).await;
///         counter += 1;
///     }
/// }
/// ```
///
/// [`Reader`]: super::Reader
/// [`single_writer::Writer`]: crate::single_writer::Writer
#[derive(Debug)]
pub struct Writer<'a, T, const N: usize>
where
    T: Storable + 'static,
{
    slot: Pin<&'a Slot<T, N>>,
    waiter: generational::Waiter<'a>,
    index: usize,
}

impl<T, const N: usize> Writer<'_, T, N>
where
    T: Storable + 'static,
{
    /// Writes a new value and notifies the reader.
    #[veecle_telemetry::instrument]
    pub async fn write(&mut self, item: T::DataType) {
        use veecle_telemetry::future::FutureExt;
        let span = veecle_telemetry::span!("write");
        let span_context = span.context();
        async move {
            self.ready().await;

            veecle_telemetry::trace!(
                "Slot written",
                index = format_args!("{:?}", self.index),
                value = format_args!("{item:?}")
            );
            let had_unseen_value = self.slot.write(self.index, item, span_context);

            if had_unseen_value {
                let type_name = core::any::type_name::<T>();

                veecle_telemetry::warn!(
                    "Overwriting unseen value",
                    type_name = type_name,
                    writer_index = self.index as i64
                );
            }

            self.waiter.update_generation();
            self.slot.increment_generation();
        }
        .with_span(span)
        .await;
    }

    /// Waits for the writer to be ready to perform a write operation.
    ///
    /// After awaiting this method, the next call to [`Writer::write()`]
    /// is guaranteed to resolve immediately.
    pub async fn ready(&mut self) {
        let _ = self.waiter.wait().await;
    }
}

impl<'a, T, const N: usize> Writer<'a, T, N>
where
    T: Storable + 'static,
{
    pub(crate) fn new(waiter: generational::Waiter<'a>, slot: Pin<&'a Slot<T, N>>) -> Self {
        let index = slot.take_writer();
        Self {
            slot,
            waiter,
            index,
        }
    }
}

impl<T, const N: usize> DefinesSlot for Writer<'_, T, N>
where
    T: Storable,
{
    type Slot = Nil;
}

impl<T, const N: usize> Sealed for Writer<'_, T, N> where T: Storable + 'static {}

impl<'a, T, const N: usize> StoreRequest<'a> for Writer<'a, T, N>
where
    T: Storable + 'static,
{
    async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self {
        Self::new(datastore.source().waiter(), datastore.slot(requestor))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::datastore::Storable;
    use crate::datastore::mpsc::Writer;
    use crate::datastore::sync::generational;
    use crate::mpsc::slot::Slot;
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
        let slot = pin!(Slot::<Data, 2>::new());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert!(writer.ready().now_or_never().is_none());
        assert!(writer.write(Data {}).now_or_never().is_none());

        source.as_ref().increment_generation();
        assert!(writer.ready().now_or_never().is_some());
        assert!(writer.write(Data {}).now_or_never().is_some());

        assert!(writer.ready().now_or_never().is_none());
        assert!(writer.write(Data {}).now_or_never().is_none());
    }

    #[test]
    fn multiple_writers_get_unique_indices() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Data(usize);
        impl Storable for Data {
            type DataType = Self;
        }

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 3>::new());

        let writer0 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let writer1 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let writer2 = Writer::new(source.as_ref().waiter(), slot.as_ref());

        assert_eq!(writer0.index, 0);
        assert_eq!(writer1.index, 1);
        assert_eq!(writer2.index, 2);
    }

    #[test]
    fn overwrite_existing_value() {
        use futures::FutureExt;
        #[derive(Debug)]
        pub struct Data();
        impl Storable for Data {
            type DataType = Self;
        }

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());
        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());

        source.as_ref().increment_generation();
        assert!(writer.write(Data {}).now_or_never().is_some());

        source.as_ref().increment_generation();
        assert!(writer.write(Data {}).now_or_never().is_some());
    }
}
