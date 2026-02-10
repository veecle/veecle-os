//! Slot implementation for mpsc slots.

use crate::datastore::sync::generational;
use crate::datastore::{SlotTrait, Storable};
use core::any::TypeId;
use core::cell::{Cell, RefCell};
use core::pin::Pin;

use pin_project::pin_project;
use veecle_telemetry::SpanContext;

/// Runtime storage for multiple storable values in an mpsc slot.
///
/// Each writer gets its own value indexed by position.
#[pin_project]
pub struct Slot<T, const N: usize>
where
    T: Storable + 'static,
{
    #[pin]
    source: generational::Source,
    writer_count: Cell<usize>,
    items: [RefCell<Option<T::DataType>>; N],
    writer_contexts: [Cell<Option<SpanContext>>; N],
}

impl<T, const N: usize> Slot<T, N>
where
    T: Storable + 'static,
{
    /// Creates a new mpsc `Slot`.
    pub(crate) fn new() -> Self {
        Self {
            source: generational::Source::new(),
            writer_count: Cell::new(0),
            items: core::array::from_fn(|_| RefCell::new(None)),
            writer_contexts: core::array::from_fn(|_| Cell::new(None)),
        }
    }

    /// Assigns a writer index and returns it.
    ///
    /// # Panics
    ///
    /// If called more than `N` times.
    pub(crate) fn take_writer(&self) -> usize {
        let index = self.writer_count();
        let type_name = core::any::type_name::<T>();
        assert!(
            index < N,
            "too many writers for mpsc slot<{type_name}>: capacity is {N}",
        );
        self.writer_count.set(index + 1);
        index
    }

    /// Returns the number of writers that have been assigned.
    pub(crate) fn writer_count(&self) -> usize {
        self.writer_count.get()
    }

    /// Writes a value to the slot at the given index.
    ///
    /// Stores the provided `span_context` to connect this write to the next read operation.
    /// Returns `true` if a previous unseen value was overwritten.
    #[veecle_telemetry::instrument]
    pub(crate) fn write(
        &self,
        index: usize,
        value: T::DataType,
        span_context: Option<SpanContext>,
    ) -> bool {
        self.writer_contexts[index].set(span_context);
        self.items[index].borrow_mut().replace(value).is_some()
    }

    /// Takes the value from a writer's slot, leaving behind `None`.
    ///
    /// Links the current span to the writer's span context.
    #[veecle_telemetry::instrument]
    pub(crate) fn take(&self, index: usize) -> Option<T::DataType> {
        if let Some(writer_context) = self.writer_contexts[index].take() {
            veecle_telemetry::CurrentSpan::add_link(writer_context);
        }
        self.items[index].borrow_mut().take()
    }

    /// Returns a new waiter for this slot's source.
    pub(crate) fn waiter(self: Pin<&Self>) -> generational::Waiter<'_> {
        self.project_ref().source.waiter()
    }

    /// Increments the slot generation and wakes the reader.
    pub(crate) fn increment_generation(self: Pin<&Self>) {
        self.project_ref().source.increment_generation();
    }
}

impl<T, const N: usize> SlotTrait for Slot<T, N>
where
    T: Storable + 'static,
{
    fn new() -> Self {
        Slot::new()
    }

    fn data_type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn data_type_name() -> &'static str {
        core::any::type_name::<T>()
    }

    fn validate_access_pattern(
        (writers, writers_list): (usize, impl Iterator<Item = &'static str>),
        (exclusive_readers, exclusive_readers_list): (usize, impl Iterator<Item = &'static str>),
        (non_exclusive_readers, non_exclusive_readers_list): (
            usize,
            impl Iterator<Item = &'static str>,
        ),
    ) {
        use crate::datastore::format_types;

        let type_name = Self::data_type_name();

        if writers < 1 {
            panic!(
                "missing writer for mpsc `{type_name}`, read by: {}",
                format_types(exclusive_readers_list),
            );
        }
        if writers > N {
            panic!(
                "too many writers ({writers}) for mpsc `{type_name}` with capacity {N}: {}",
                format_types(writers_list),
            );
        }
        if exclusive_readers != 1 {
            panic!(
                "mpsc `{type_name}` requires exactly 1 exclusive reader, found {exclusive_readers}: {}",
                format_types(exclusive_readers_list),
            );
        }
        if non_exclusive_readers != 0 {
            panic!(
                "mpsc `{type_name}` does not support non-exclusive readers: {}",
                format_types(non_exclusive_readers_list),
            );
        }
    }
}

impl<T, const N: usize> core::fmt::Debug for Slot<T, N>
where
    T: Storable + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Slot")
            .field("source", &self.source)
            .field("writer_count", &self.writer_count)
            .field("items", &"<opaque>")
            .finish()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;

    use crate::datastore::{SlotTrait, Storable};

    use super::Slot;

    #[derive(Debug)]
    struct Data(usize);
    impl Storable for Data {
        type DataType = Self;
    }

    #[test]
    fn new_initializes_correctly() {
        let slot = pin!(Slot::<Data, 3>::new());
        assert_eq!(slot.writer_count(), 0);
        for index in 0..3 {
            assert!(slot.take(index).is_none());
        }
    }

    #[test]
    fn take_writer_assigns_sequential_indices() {
        let slot = pin!(Slot::<Data, 3>::new());
        assert_eq!(slot.take_writer(), 0);
        assert_eq!(slot.take_writer(), 1);
        assert_eq!(slot.take_writer(), 2);
        assert_eq!(slot.writer_count(), 3);
    }

    #[test]
    #[should_panic(expected = "too many writers for mpsc slot")]
    fn take_writer_panics_at_capacity() {
        let slot = pin!(Slot::<Data, 2>::new());
        slot.take_writer();
        slot.take_writer();
        slot.take_writer();
    }

    #[test]
    fn write_and_take() {
        let slot = pin!(Slot::<Data, 2>::new());
        assert!(!slot.write(0, Data(42), None));
        assert_eq!(slot.take(0).unwrap().0, 42);
        assert!(slot.take(1).is_none());
    }

    #[test]
    fn write_returns_true_when_overwriting() {
        let slot = pin!(Slot::<Data, 2>::new());
        assert!(!slot.write(0, Data(10), None));
        assert!(slot.write(0, Data(20), None));
        assert_eq!(slot.take(0).unwrap().0, 20);
    }

    #[test]
    fn take_removes_value() {
        let slot = pin!(Slot::<Data, 2>::new());
        slot.write(0, Data(10), None);
        let taken = slot.take(0);
        assert_eq!(taken.unwrap().0, 10);
        assert!(slot.take(0).is_none());
    }

    #[test]
    fn increment_generation_wakes_waiter() {
        use futures::FutureExt;

        let slot = pin!(Slot::<Data, 2>::new());
        let waiter = slot.as_ref().waiter();
        assert!(!waiter.is_updated());

        slot.as_ref().increment_generation();
        assert!(waiter.is_updated());
        assert!(waiter.wait().now_or_never().is_some());
    }

    #[test]
    fn validate_accepts_valid_pattern() {
        Slot::<Data, 2>::validate_access_pattern(
            (1, ["writer"].into_iter()),
            (1, ["reader"].into_iter()),
            (0, [].into_iter()),
        );
    }

    #[test]
    #[should_panic(expected = "missing writer for mpsc")]
    fn validate_rejects_no_writer() {
        Slot::<Data, 2>::validate_access_pattern(
            (0, [].into_iter()),
            (1, ["reader"].into_iter()),
            (0, [].into_iter()),
        );
    }

    #[test]
    #[should_panic(expected = "too many writers")]
    fn validate_rejects_too_many_writers() {
        Slot::<Data, 2>::validate_access_pattern(
            (3, ["w1", "w2", "w3"].into_iter()),
            (1, ["reader"].into_iter()),
            (0, [].into_iter()),
        );
    }

    #[test]
    #[should_panic(expected = "requires exactly 1 exclusive reader, found 0")]
    fn validate_rejects_no_reader() {
        Slot::<Data, 2>::validate_access_pattern(
            (1, ["writer"].into_iter()),
            (0, [].into_iter()),
            (0, [].into_iter()),
        );
    }

    #[test]
    #[should_panic(expected = "requires exactly 1 exclusive reader, found 2")]
    fn validate_rejects_multiple_readers() {
        Slot::<Data, 2>::validate_access_pattern(
            (1, ["writer"].into_iter()),
            (2, ["r1", "r2"].into_iter()),
            (0, [].into_iter()),
        );
    }

    #[test]
    #[should_panic(expected = "does not support non-exclusive readers")]
    fn validate_rejects_non_exclusive_reader() {
        Slot::<Data, 2>::validate_access_pattern(
            (1, ["writer"].into_iter()),
            (1, ["reader"].into_iter()),
            (1, ["non_exclusive"].into_iter()),
        );
    }
}
