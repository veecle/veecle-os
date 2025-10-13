use super::super::generational;
use super::Slot;
use crate::Storable;
use core::pin::Pin;

pub(crate) struct Waiter<'a, T>
where
    T: Storable + 'static,
{
    slot: Pin<&'a Slot<T>>,
    waiter: generational::Waiter<'a>,
}

impl<T> core::fmt::Debug for Waiter<'_, T>
where
    T: Storable + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Waiter")
            .field("slot", &self.slot)
            .field("waiter", &self.waiter)
            .finish()
    }
}

impl<'a, T> Waiter<'a, T>
where
    T: Storable + 'static,
{
    pub(super) fn new(slot: Pin<&'a Slot<T>>, waiter: generational::Waiter<'a>) -> Self {
        Self { slot, waiter }
    }

    /// Updates the last seen generation of this waiter so that we will wait for a newer value.
    pub(crate) fn update_generation(&mut self) {
        self.waiter.update_generation();
    }

    pub(crate) fn borrow(&self) -> core::cell::Ref<'_, Option<T::DataType>> {
        self.slot.borrow()
    }

    pub(crate) fn read<U>(&self, f: impl FnOnce(&Option<T::DataType>) -> U) -> U {
        self.slot.read(f)
    }

    pub(crate) fn inner_type_name(&self) -> &'static str {
        self.slot.inner_type_name()
    }

    pub(crate) async fn wait(&self) {
        if let Err(generational::MissedUpdate { current, expected }) = self.waiter.wait().await {
            // While we are unsure about timing and such, I would at least keep a warning
            // if we miss value. We can decide later on how we handle this case more
            // properly.
            let type_name = self.slot.inner_type_name();

            veecle_telemetry::warn!(
                "Missed update for type",
                type_name = type_name,
                current = current as i64,
                expected = expected as i64
            );
        }
    }
}

impl<'a, T> Waiter<'a, T>
where
    T: Storable + 'static,
{
    /// Takes the current value of the slot, leaving behind `None`.
    pub(crate) fn take(&mut self) -> Option<T::DataType> {
        self.slot.take()
    }
}
