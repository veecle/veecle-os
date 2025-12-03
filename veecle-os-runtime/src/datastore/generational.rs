//! [`generational`](self) supports synchronizing the "generation" of data from a producer to multiple consumers.
//! The producer should own a [`Source`] that is in charge of notifying when the generation is incremented.
//! The consumers should have their own [`Waiter`]s referencing this `Source` allowing them to wait for an update to the
//! generation.
//!
//! See the `tests` module for an example.

use core::cell::Cell;
use core::pin::Pin;
use core::task::{Poll, Waker};

use pin_cell::{PinCell, PinMut};
use pin_project::pin_project;
use wakerset::{ExtractedWakers, WakerList, WakerSlot};

/// Tracks the current generation, waking tasks that are `await`ing associated [`Waiter`]s when it increments.
#[derive(Debug, Default)]
#[pin_project]
pub struct Source {
    generation: Cell<usize>,
    #[pin]
    list: PinCell<WakerList>,
}

impl Source {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new waiter for this source.
    ///
    /// # Panics
    ///
    /// If called more times than the `waiter_count` provided on init.
    pub(crate) fn waiter(self: Pin<&Self>) -> Waiter<'_> {
        Waiter::new(self)
    }

    /// Increments the generation of the current [`Source`] and notifies any waiting [`Waiter`]s they can continue.
    pub(crate) fn increment_generation(self: Pin<&Self>) {
        self.generation.set(self.generation.get() + 1);

        let round = PinMut::as_mut(&mut self.project_ref().list.borrow_mut()).begin_extraction();
        let mut wakers = ExtractedWakers::new();
        let mut more = true;
        while more {
            more = PinMut::as_mut(&mut self.project_ref().list.borrow_mut())
                .extract_some_wakers(round, &mut wakers);
            wakers.wake_all();
        }
    }

    fn link(self: Pin<&Self>, slot: Pin<&mut WakerSlot>, waker: Waker) {
        PinMut::as_mut(&mut self.project_ref().list.borrow_mut()).link(slot, waker)
    }

    fn unlink(self: Pin<&Self>, slot: Pin<&mut WakerSlot>) {
        PinMut::as_mut(&mut self.project_ref().list.borrow_mut()).unlink(slot)
    }
}

/// Tracks the last seen generation of a [`Source`], when `await`ed will resolve once the source is at a newer
/// generation.
#[derive(Debug)]
pub(crate) struct Waiter<'a> {
    generation: usize,
    source: Pin<&'a Source>,
}

impl<'a> Waiter<'a> {
    /// Creates a new [`Waiter`].
    pub(crate) fn new(source: Pin<&'a Source>) -> Self {
        Self {
            generation: source.generation.get(),
            source,
        }
    }

    /// Updates the generation from the source [`Source`] to allow waiting for the next generation.
    pub(crate) fn update_generation(&mut self) {
        self.generation = self.source.generation.get();
    }

    pub(crate) async fn wait(&self) -> Result<(), MissedUpdate> {
        // Using a guard here makes sure that the slot is unlinked if this future is dropped before completing.
        struct Guard<'a, 'b> {
            source: Pin<&'a Source>,
            slot: Pin<&'b mut WakerSlot>,
        }

        impl Drop for Guard<'_, '_> {
            fn drop(&mut self) {
                if self.slot.is_linked() {
                    self.source.unlink(self.slot.as_mut());
                }
            }
        }
        use core::pin::pin;

        let mut guard = Guard {
            source: self.source,
            slot: pin!(WakerSlot::new()),
        };

        core::future::poll_fn(|cx| {
            let current = self.source.generation.get();

            // If the generation is the same, we need to register the waker to be woken
            // on next update. Else, it means we already got an update so we can return
            // from the future.
            if current == self.generation {
                self.source.link(guard.slot.as_mut(), cx.waker().clone());
                return Poll::Pending;
            }

            let expected = self.generation + 1;
            if current != expected {
                return Poll::Ready(Err(MissedUpdate { expected, current }));
            }

            Poll::Ready(Ok(()))
        })
        .await
    }
}

/// Indicates that the [`Source`] has had multiple generation updates since the last time [`Waiter::update_generation`]
/// was called, depending on the usecase this may mean some data values were missed.
pub(crate) struct MissedUpdate {
    pub(crate) expected: usize,
    pub(crate) current: usize,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::cell::Cell;
    use std::future::Future;
    use std::pin::pin;

    use crate::datastore::generational;

    #[test]
    fn example() {
        let source = pin!(generational::Source::new());

        let counter = Cell::new(0);
        let sum = Cell::new(0);
        let mut waiter = source.as_ref().waiter();

        let mut future = pin!(async {
            loop {
                let _ = waiter.wait().await;
                waiter.update_generation();
                sum.set(sum.get() + counter.get());
            }
        });

        let mut context = std::task::Context::from_waker(futures::task::noop_waker_ref());

        for i in 1..10 {
            // Before incrementing the generation, nothing should happen.
            assert!(future.as_mut().poll(&mut context).is_pending());
            assert_eq!(sum.get(), (i - 1) * i / 2);

            counter.set(i);
            source.as_ref().increment_generation();

            // After incrementing the generation it should run.
            assert!(future.as_mut().poll(&mut context).is_pending());
            assert_eq!(sum.get(), i * (i + 1) / 2);
        }
    }
}
