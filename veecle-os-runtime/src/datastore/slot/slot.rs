use super::super::generational;
use super::Waiter;
use crate::Storable;
use core::any::TypeId;
use core::cell::{Cell, Ref, RefCell, RefMut};
use core::pin::Pin;

use pin_project::pin_project;
#[cfg(feature = "veecle-telemetry")]
use veecle_telemetry::SpanContext;

#[pin_project]
pub(crate) struct Slot<T>
where
    T: Storable + 'static,
{
    #[pin]
    source: generational::Source,
    writer_taken: Cell<bool>,

    #[cfg(feature = "veecle-telemetry")]
    writer_context: Cell<Option<SpanContext>>,

    item: RefCell<Option<T::DataType>>,
}

impl<T> core::fmt::Debug for Slot<T>
where
    T: Storable + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_struct("Slot");

        debug.field("source", &self.source);
        debug.field("writer_taken", &self.writer_taken);

        #[cfg(feature = "veecle-telemetry")]
        debug.field("writer_context", &self.writer_context.get());

        debug.field("item", &"<opaque>");

        debug.finish()
    }
}

impl<T> Slot<T>
where
    T: Storable + 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            item: RefCell::new(None),
            source: generational::Source::new(),
            writer_taken: Cell::new(false),

            #[cfg(feature = "veecle-telemetry")]
            writer_context: Cell::new(None),
        }
    }

    /// Takes the current value of the slot, leaving behind `None`.
    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub(crate) fn take(&self) -> Option<T::DataType> {
        self.borrow_mut().take()
    }
}

impl<T> Slot<T>
where
    T: Storable + 'static,
{
    /// Returns the type name of the value stored in this slot.
    ///
    /// # Panics
    ///
    /// If called while the [`Self::borrow_mut`] guard is held.
    pub(crate) fn inner_type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    /// Returns a new waiter for this slot.
    pub(crate) fn waiter(self: Pin<&Self>) -> Waiter<'_, T> {
        Waiter::new(self, self.project_ref().source.waiter())
    }

    pub(crate) fn take_writer(&self) {
        let type_name = self.inner_type_name();
        assert!(
            !self.writer_taken.replace(true),
            "attempted to acquire the writer for slot<{type_name}> multiple times",
        );
    }

    pub(crate) fn borrow(&self) -> Ref<'_, Option<T::DataType>> {
        #[cfg(feature = "veecle-telemetry")]
        if let Some(writer_context) = self.writer_context.get() {
            veecle_telemetry::CurrentSpan::add_link(writer_context);
        }

        self.item.borrow()
    }

    pub(super) fn borrow_mut(&self) -> RefMut<'_, Option<T::DataType>> {
        #[cfg(feature = "veecle-telemetry")]
        {
            // TODO(DEV-531): this should be added, but only for an in place update
            // if let Some(writer_context) = self.writer_context.get() {
            //     veecle_telemetry::CurrentSpan::add_link(writer_context);
            // }
            self.writer_context
                .set(veecle_telemetry::SpanContext::current());
        }

        self.item.borrow_mut()
    }

    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub(crate) fn read<U>(&self, f: impl FnOnce(&Option<T::DataType>) -> U) -> U {
        f(&*self.borrow())
    }

    #[cfg_attr(feature = "veecle-telemetry", veecle_telemetry::instrument)]
    pub(crate) fn modify(&self, f: impl FnOnce(&mut Option<T::DataType>)) {
        f(&mut *self.borrow_mut())
    }

    pub(crate) fn increment_generation(self: Pin<&Self>) {
        self.project_ref().source.increment_generation();
    }

    pub(crate) fn assert_is_type<U>(self: Pin<&Self>) -> Pin<&Slot<U>>
    where
        U: Storable,
    {
        if TypeId::of::<T>() == TypeId::of::<U>() {
            // SAFETY:
            // `Pin::map_unchecked`: We're only transforming the type, so it retains its pinned-ness.
            // `cast` + `as_ref`: We verified above that the stored value is of the right type.
            unsafe {
                Pin::map_unchecked(self, |this| {
                    core::ptr::NonNull::from_ref(this)
                        .cast::<Slot<U>>()
                        .as_ref()
                })
            }
        } else {
            panic!("invalid cast")
        }
    }
}
