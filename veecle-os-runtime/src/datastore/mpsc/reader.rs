//! Exclusive reader for mpsc slots.

use core::pin::Pin;

use super::slot::Slot;
use crate::Sealed;
use crate::cons::{Cons, Nil};
use crate::datastore::sync::generational;
use crate::datastore::{Datastore, DefinesSlot, Storable, StoreRequest};

/// Exclusive reader for [`Storable`] types from an mpsc slot.
///
/// Reads values written by multiple [`Writer`]s for the same type.
/// Reading a value takes ownership from the slot, marking it as consumed.
///
/// The generic type `T` specifies the type of the value being read.
/// The const generic `N` specifies the maximum number of writers.
///
/// # Usage
///
/// [`Reader::take_all_updated`] reads all available value via closure, waiting if no values are currently available.
///
/// [`Reader::take_all`] reads all available value via closure.
///
/// [`Reader::take_one`] returns one value if available.
///
/// # Examples
///
/// ```rust
/// // Using `take_all` to process all available values.
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, mpsc::Reader};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Command(usize);
/// #
/// #[veecle_os_runtime::actor]
/// async fn command_handler<const N: usize>(mut reader: Reader<'_, Command, N>) -> veecle_os_runtime::Never {
///     loop {
///         reader.wait_for_update().await;
///         reader.take_all(|command| {
///             // Process the command.
///         });
///     }
/// }
/// ```
///
/// [`Writer`]: super::Writer
pub struct Reader<'a, T, const N: usize>
where
    T: Storable + 'static,
{
    slot: Pin<&'a Slot<T, N>>,
    waiter: generational::Waiter<'a>,
}

impl<T, const N: usize> Reader<'_, T, N>
where
    T: Storable + 'static,
{
    /// Returns `true` if an unseen value is available.
    ///
    /// A value becomes "seen" after calling [`take_one`][Self::take_one], [`take_all`][Self::take_all],
    /// or similar read methods.
    ///
    /// May return `true` after a reading method if more unseen values are available.
    #[veecle_telemetry::instrument]
    pub fn is_updated(&self) -> bool {
        self.waiter.is_updated()
    }

    /// Waits for unseen values to become available.
    ///
    /// This future resolving does not imply that `previous_value != new_value`, just that a
    /// [`Writer`][super::Writer] has written a value of `T` since the last read operation.
    ///
    /// Returns `&mut Self` to allow chaining method calls.
    #[veecle_telemetry::instrument]
    pub async fn wait_for_update(&mut self) -> &mut Self {
        loop {
            if self.is_updated() {
                return self;
            }

            self.waiter.update_generation();
            let _ = self.waiter.wait().await;
        }
    }

    /// Takes the next available value, returns `None` if none are available.
    #[veecle_telemetry::instrument]
    pub fn take_one(&mut self) -> Option<T::DataType> {
        for index in 0..self.slot.writer_count() {
            if let Some(value) = self.slot.take(index) {
                veecle_telemetry::trace!(
                    "Slot taken",
                    index = format_args!("{index:?}"),
                    value = format_args!("{value:?}")
                );
                return Some(value);
            }
        }
        // Update the generation if no unseen value is present.
        self.waiter.update_generation();

        None
    }

    /// Reads all available values.
    ///
    /// Takes ownership of each value and passes it to `f`.
    #[veecle_telemetry::instrument]
    pub fn take_all(&mut self, mut f: impl FnMut(T::DataType)) {
        for index in 0..self.slot.writer_count() {
            if let Some(value) = self.slot.take(index) {
                veecle_telemetry::trace!(
                    "Slot taken",
                    index = format_args!("{index:?}"),
                    value = format_args!("{value:?}")
                );
                f(value);
            }
        }

        // Update the generation if no unseen value is present.
        self.waiter.update_generation();
    }

    /// Reads all available values, waiting if none are available.
    ///
    /// Takes ownership of each value and passes it to `f`.
    /// When no values are available, waits for new writes and returns after reading at least one value.
    #[veecle_telemetry::instrument]
    pub async fn take_all_updated(&mut self, mut f: impl FnMut(T::DataType)) {
        loop {
            let mut wait_for_update = true;
            for index in 0..self.slot.writer_count() {
                if let Some(value) = self.slot.take(index) {
                    wait_for_update = false;
                    veecle_telemetry::trace!(
                        "Slot taken",
                        index = format_args!("{index:?}"),
                        value = format_args!("{value:?}")
                    );
                    f(value);
                }
            }

            // Update the generation if no unseen value is present.
            self.waiter.update_generation();

            if wait_for_update {
                let _ = self.waiter.wait().await;
            } else {
                break;
            }
        }
    }
}

impl<'a, T, const N: usize> Reader<'a, T, N>
where
    T: Storable + 'static,
{
    pub(crate) fn from_slot(slot: Pin<&'a Slot<T, N>>) -> Self {
        Reader {
            waiter: slot.waiter(),
            slot,
        }
    }
}

impl<T, const N: usize> core::fmt::Debug for Reader<'_, T, N>
where
    T: Storable + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Reader").field("slot", &self.slot).finish()
    }
}

impl<T, const N: usize> DefinesSlot for Reader<'_, T, N>
where
    T: Storable,
{
    type Slot = Cons<Slot<T, N>, Nil>;
}

impl<T, const N: usize> Sealed for Reader<'_, T, N> where T: Storable + 'static {}

impl<'a, T, const N: usize> StoreRequest<'a> for Reader<'a, T, N>
where
    T: Storable + 'static,
{
    async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self {
        Self::from_slot(datastore.slot(requestor))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;
    use futures::FutureExt;
    use std::cell::Cell;

    use crate::datastore::Storable;
    use crate::datastore::mpsc::{Reader, Writer};
    use crate::datastore::sync::generational;
    use crate::mpsc::slot::Slot;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Data(usize);
    impl Storable for Data {
        type DataType = Self;
    }

    #[test]
    fn is_updated_false_initially() {
        let slot = pin!(Slot::<Data, 2>::new());
        let reader = Reader::from_slot(slot.as_ref());
        assert!(!reader.is_updated());
    }

    #[test]
    fn is_updated_true_after_write() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let reader = Reader::from_slot(slot.as_ref());

        assert!(!reader.is_updated());

        source.as_ref().increment_generation();
        writer.write(Data(1)).now_or_never().unwrap();

        assert!(reader.is_updated());
    }

    #[test]
    fn wait_for_update_pends_then_resolves() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        assert!(reader.wait_for_update().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Data(1)).now_or_never().unwrap();

        reader.wait_for_update().now_or_never().unwrap();
    }

    #[test]
    fn take_all_reads_all_values() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 3>::new());

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        source.as_ref().increment_generation();
        writer0.write(Data(10)).now_or_never().unwrap();

        source.as_ref().increment_generation();
        writer1.write(Data(20)).now_or_never().unwrap();

        reader.wait_for_update().now_or_never().unwrap();

        let mut values = std::vec::Vec::new();
        reader.take_all(|v| values.push(v));
        assert_eq!(values, std::vec![Data(10), Data(20)]);
    }

    #[test]
    fn read_returns_none_when_exhausted() {
        let slot = pin!(Slot::<Data, 2>::new());
        let mut reader = Reader::from_slot(slot.as_ref());
        assert!(reader.take_one().is_none());
    }

    #[test]
    fn after_reading_all_values_is_updated_returns_false() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        source.as_ref().increment_generation();
        writer.write(Data(1)).now_or_never().unwrap();

        assert!(reader.is_updated());

        reader.wait_for_update().now_or_never().unwrap();
        reader.take_all(|_| {});

        assert!(!reader.is_updated());
    }

    #[test]
    fn take_all_updated_takes_values() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 3>::new());

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        source.as_ref().increment_generation();
        writer0.write(Data(10)).now_or_never().unwrap();

        source.as_ref().increment_generation();
        writer1.write(Data(20)).now_or_never().unwrap();

        let values = std::rc::Rc::new(core::cell::RefCell::new(std::vec::Vec::new()));
        let captured = values.clone();

        let mut future = pin!(reader.take_all_updated(move |v| {
            captured.borrow_mut().push(v);
        }));
        assert!(future.as_mut().now_or_never().is_some());
        assert_eq!(*values.borrow(), std::vec![Data(10), Data(20)]);
    }

    #[test]
    fn take_all_updated_waits_when_no_values_available() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let _writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        let mut future = pin!(reader.take_all_updated(|_| {}));

        assert!(future.as_mut().now_or_never().is_none());
    }

    #[test]
    fn take_all_updated_reads_new_values_after_waiting() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        let future_hit = Cell::new(false);

        let mut future = pin!(reader.take_all_updated(|_| {
            future_hit.set(true);
        }));

        assert!(future.as_mut().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Data(99)).now_or_never().unwrap();

        assert!(future.as_mut().now_or_never().is_some());
        assert!(future_hit.get());
    }

    #[test]
    fn is_updated_true_after_reading_one_of_two_writes() {
        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Data, 2>::new());

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let mut reader = Reader::from_slot(slot.as_ref());

        source.as_ref().increment_generation();
        writer0.write(Data(10)).now_or_never().unwrap();
        writer1.write(Data(20)).now_or_never().unwrap();

        let value = reader.take_one();
        assert_eq!(value, Some(Data(10)));

        assert!(reader.is_updated());
    }
}
