//! A multi-task executor that uses statically allocated state for tracking task status.

use core::convert::Infallible;
use core::fmt::Debug;
use core::future::Future;
use core::ops::{Add, Div};
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use futures::task::AtomicWaker;
use generic_array::{ArrayLength, GenericArray};
use typenum::operator_aliases::{Quot, Sum};
use typenum::{Const, ToUInt, U};

use crate::datastore::generational;

type UsizeBits = U<{ usize::BITS as usize }>;
type UsizeBitsMinusOne = typenum::operator_aliases::Sub1<UsizeBits>;

/// Helper to force associated type normalization.
trait AddUsizeBitsMinusOne {
    /// See the sole implementation docs for the value.
    type Output;
}

/// Helper to force associated type normalization.
trait DivCeilUsizeBits {
    /// See the sole implementation docs for the value.
    type Output;
}

/// Private API to simplify bounds that users don't need to care about.
///
/// The name is like this because it appears in the public API docs as a bound, but doesn't get linked to.
trait Internal {
    /// See the sole implementation docs for the value.
    ///
    /// The bounds here are what are required by [`WakerShared::new`].
    type LengthInWords: ArrayLength;
}

impl<const LEN: usize> AddUsizeBitsMinusOne for Const<LEN>
where
    Const<LEN>: ToUInt<Output: Add<UsizeBitsMinusOne>>,
{
    /// Calculates `LEN + usize::BITS - 1`.
    type Output = Sum<U<LEN>, UsizeBitsMinusOne>;
}

impl<const LEN: usize> DivCeilUsizeBits for Const<LEN>
where
    Const<LEN>: AddUsizeBitsMinusOne<Output: Div<UsizeBits>>,
{
    /// Calculates `(LEN + usize::BITS - 1) / usize::BITS`, which is the same as `LEN.div_ceil(usize::BITS)`.
    type Output = Quot<<Const<LEN> as AddUsizeBitsMinusOne>::Output, UsizeBits>;
}

impl<const LEN: usize> Internal for Const<LEN>
where
    Const<LEN>: DivCeilUsizeBits<Output: ArrayLength>,
{
    /// The length in "words" (`usize`s) required to store at least `LEN` bits.
    ///
    /// This is essentially `LEN.div_ceil(usize::BITS)`, but implemented as `(LEN + usize::BITS - 1) / usize::BITS`
    /// because that's what operations `typenum` provides.
    ///
    /// The extra intermediate traits are required to force normalization of the associated types.
    type LengthInWords = <Const<LEN> as DivCeilUsizeBits>::Output;
}

/// Helper to simplify getting the `LengthInWords` associated type.
type LengthInWords<const LEN: usize> = <Const<LEN> as Internal>::LengthInWords;

/// Data shared between multiple [`BitWaker`]s associated with a single [`Executor`].
#[derive(Debug)]
struct WakerShared<const LEN: usize>
where
    Const<LEN>: Internal,
{
    /// The outer [`Waker`] that this [`Executor`] is currently associated with.
    waker: AtomicWaker,

    /// Each bit stores the flag for a sub-future of an [`Executor`], to determine which need to run on each poll.
    active: GenericArray<AtomicUsize, LengthInWords<LEN>>,
}

/// Gets the index of the word that the `index` flag is stored in, and the mask to identify the flag within the
/// word.
fn get_active_index_and_mask(index: usize) -> (usize, usize) {
    let word_index = index / usize::BITS as usize;
    let bit_index = index % usize::BITS as usize;
    (word_index, 1 << bit_index)
}

impl<const LEN: usize> WakerShared<LEN>
where
    Const<LEN>: Internal,
{
    /// Creates a new `WakerShared` that can store the state of at least `LEN` sub-futures.
    const fn new() -> Self {
        let active = {
            // Using `const_default` would be nicer, but that can't be used with atomics on 32-bit.
            let mut active = GenericArray::uninit();

            // Set all bits to 1 so that every future is considered woken and will be polled in the first loop.
            let mut index = 0;
            let slice = active.as_mut_slice();
            while index < slice.len() {
                slice[index].write(AtomicUsize::new(usize::MAX));
                index += 1;
            }

            // SAFETY: We initialized every element of the array above.
            unsafe { GenericArray::assume_init(active) }
        };

        Self {
            waker: AtomicWaker::new(),
            active,
        }
    }

    /// Clears the flag for the `index` sub-future, and returns the previous value.
    fn reset(&self, index: usize) -> bool {
        let (active_word, mask) = self.get_active_ref_and_mask(index);
        let previous_value = active_word.fetch_and(!mask, Ordering::Relaxed);
        // Was the bit set in the previous value?
        (previous_value & mask) != 0
    }

    /// Clears all flags and returns the indexes of any that were set.
    fn reset_all(&self) -> impl Iterator<Item = usize> + use<'_, LEN> {
        (0..LEN).filter(|&index| self.reset(index))
    }

    /// Sets the flag for the `index` sub-future, and returns the previous value, waking any currently registered outer
    /// waker in the process.
    fn set(&self, index: usize) -> bool {
        let (active_word, mask) = self.get_active_ref_and_mask(index);
        let previous_value = active_word.fetch_or(mask, Ordering::Relaxed);

        self.waker.wake();

        // Was the bit set in the previous value?
        (previous_value & mask) != 0
    }

    /// Registers the [`Waker`] of the current context as to-be-woken when any sub-future wakes.
    async fn register_current(&self) {
        core::future::poll_fn(|ctx| {
            self.waker.register(ctx.waker());
            Poll::Ready(())
        })
        .await;
    }

    /// Gets the word that the `index` flag is stored in, and the mask to identify the flag within the word.
    fn get_active_ref_and_mask(&self, index: usize) -> (&AtomicUsize, usize) {
        let (index, mask) = get_active_index_and_mask(index);
        (&self.active[index], mask)
    }
}

/// A [`Waker`] that can be used to wake a sub-future within an outer task while tracking which sub-future it is.
#[derive(Debug)]
struct BitWaker<const LEN: usize>
where
    Const<LEN>: Internal,
{
    /// Index for this sub-future within `self.shared`.
    index: usize,

    /// Shared state for all sub-futures running in the same [`Executor`].
    shared: Option<&'static WakerShared<LEN>>,
}

impl<const LEN: usize> BitWaker<LEN>
where
    Const<LEN>: Internal,
{
    // For all following comments:
    // We use this vtable with a data pointer converted from an `&'static Self` in `as_waker`.
    const VTABLE: &RawWakerVTable = &RawWakerVTable::new(
        // `&'static Self: Copy` so we can trivially return a new `RawWaker`.
        |ptr| RawWaker::new(ptr, Self::VTABLE),
        // SAFETY: We can convert back to an `&'static Self` then call its methods.
        |ptr| unsafe { &*ptr.cast::<Self>() }.wake_by_ref(),
        // SAFETY: We can convert back to an `&'static Self` then call its methods.
        |ptr| unsafe { &*ptr.cast::<Self>() }.wake_by_ref(),
        // `&'static Self` has a no-op `drop_in_place` so we don't need to do anything.
        |_| {},
    );

    /// Creates a new [`BitWaker`] that will panic if used, for const-initialization purposes.
    const fn invalid() -> Self {
        Self {
            index: usize::MAX,
            shared: None,
        }
    }

    /// Creates a new [`BitWaker`] for the future at the given `index` in the [`WakerShared`].
    const fn new(index: usize, shared: &'static WakerShared<LEN>) -> Self {
        assert!(index < LEN, "Future index out of bounds.");
        Self {
            index,
            shared: Some(shared),
        }
    }

    /// Set the bit for this waker's `index` in its [`WakerShared`].
    fn wake_by_ref(&self) {
        self.shared.unwrap().set(self.index);
    }

    /// Create a [`Waker`] instance that will wake this [`BitWaker`].
    fn as_waker(&'static self) -> Waker {
        let pointer = (&raw const *self).cast();
        // SAFETY: The vtable functions expect to be called with a data pointer converted from an `&'static Self`.
        unsafe { Waker::new(pointer, Self::VTABLE) }
    }
}

/// Permanent shared state required for the [`Executor`].
#[derive(Debug)]
#[expect(private_bounds)]
pub struct ExecutorShared<const LEN: usize>
where
    Const<LEN>: Internal,
{
    shared: WakerShared<LEN>,
    bit_wakers: [BitWaker<LEN>; LEN],
}

#[expect(private_bounds)]
impl<const LEN: usize> ExecutorShared<LEN>
where
    Const<LEN>: Internal,
{
    /// Create a new instance of the shared state. This is a self-referential data structure so must be initialized in a
    /// `static` like:
    ///
    /// ```rust
    /// use veecle_os_runtime::__exports::ExecutorShared;
    ///
    /// static SHARED: ExecutorShared<5> = ExecutorShared::new(&SHARED);
    /// ```
    pub const fn new(&'static self) -> Self {
        let mut bit_wakers = [const { BitWaker::invalid() }; LEN];
        let mut index = 0;
        while index < LEN {
            bit_wakers[index] = BitWaker::new(index, &self.shared);
            index += 1;
        }
        Self {
            shared: WakerShared::new(),
            bit_wakers,
        }
    }
}

/// Async sub-executor.
///
/// This sub-executor does not handle the main loop of waiting till a waker is woken and then polling the futures, it
/// expects to be run as a task within an executor that does that (e.g. `futures::executor::block_on`,
/// `wasm_bindgen_futures::spawn_local`, `veecle_freertos_integration::task::block_on_future`). Within that outer executor loop this
/// sub-executor tracks which of its futures were the cause of a wake and polls only them.
///
/// Being designed like this allows this sub-executor to be agnostic to how a platform actually runs futures—including
/// esoteric platforms like `wasm32-web` which cannot have a thread-park based executor—while still giving the required
/// guarantees about when and how the sub-futures are polled.
///
/// # Polling strategy
///
/// The executor polls all woken futures in a fixed order.
/// The order corresponds to the order of the elements provided to [`Executor::new()`].
/// Initially, every future will be polled once.
///
/// ## Background
///
/// Any write to a slot should be visible for every reader.
/// To ensure this, every future waiting for a reader must be polled before the writer is polled again.
/// With in-order polling, this is straight-forward to guarantee, as every future that is woken by a write will be
/// polled before the writer is polled again.
// Developer notes:
//
// - [`generational::Source::increment_generation`] is used between each poll iteration in order to track that each
// writer may only be written once per-iteration, if the writing actor has multiple data values to yield.
//
// - An alternative that was considered is a FIFO queue, but this approach requires additional complexity to fulfill the
// read-write requirement.
// Additionally, implementing in-order polling without locks is simpler compared to FIFO-approaches.
// See <https://github.com/veecle/veecle-os/issues/167> for more information.
#[expect(private_bounds)]
pub struct Executor<'a, const LEN: usize>
where
    Const<LEN>: Internal,
{
    /// A generational source provided by the datastore.
    source: Pin<&'a generational::Source>,
    shared: &'static ExecutorShared<LEN>,
    futures: [Pin<&'a mut (dyn Future<Output = Infallible> + 'a)>; LEN],
}

impl<const LEN: usize> core::fmt::Debug for Executor<'_, LEN>
where
    Const<LEN>: Internal,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Executor")
            .field("source", &self.source)
            .field("shared", &self.shared)
            .field("futures", &"<opaque>")
            .finish()
    }
}

#[expect(private_bounds)]
impl<'a, const LEN: usize> Executor<'a, LEN>
where
    Const<LEN>: Internal,
{
    /// Creates a new [`Executor`] from the provided futures.
    pub fn new(
        shared: &'static ExecutorShared<LEN>,
        source: Pin<&'a generational::Source>,
        futures: [Pin<&'a mut (dyn Future<Output = Infallible> + 'a)>; LEN],
    ) -> Self {
        Self {
            source,
            shared,
            futures,
        }
    }

    /// Polls all woken futures once, returns `true` if at least one future was woken.
    pub(crate) fn run_once(&mut self) -> bool {
        let mut polled = false;

        for index in self.shared.shared.reset_all() {
            let future = &mut self.futures[index];
            let waker = self.shared.bit_wakers[index].as_waker();
            let mut context = Context::from_waker(&waker);
            match future.as_mut().poll(&mut context) {
                Poll::Pending => {}
            }
            polled = true;
        }

        self.source.increment_generation();

        polled
    }

    /// Runs all futures in an endless loop.
    pub async fn run(mut self) -> ! {
        loop {
            self.shared.shared.register_current().await;

            // Only run through the list of futures once, relying on the outer executor to re-poll if any self-woke or
            // woke a prior sub-future.
            self.run_once();

            // The sub-futures are responsible for waking if needed, yield here to the executor then continue to poll
            // the sub-futures straight away.
            let mut yielded = false;
            core::future::poll_fn(|_| {
                if yielded {
                    Poll::Ready(())
                } else {
                    yielded = true;
                    Poll::Pending
                }
            })
            .await;
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;
    use core::task::Poll;
    use std::vec::Vec;

    use super::{BitWaker, Executor, ExecutorShared, WakerShared, get_active_index_and_mask};
    use crate::datastore::generational;

    const TWO_WORDS: usize = usize::BITS as usize * 2;

    #[test]
    fn calculate_indices() {
        // First bit in first element.
        assert_eq!(get_active_index_and_mask(0), (0, 1 << 0));

        // Second bit in first element.
        assert_eq!(get_active_index_and_mask(1), (0, 1 << 1));

        // Last bit in first element.
        assert_eq!(
            get_active_index_and_mask(usize::BITS as usize - 1),
            (0, 1 << (usize::BITS as usize - 1))
        );

        // First bit in second element.
        assert_eq!(get_active_index_and_mask(usize::BITS as usize), (1, 1 << 0));

        // Second bit in second element.
        assert_eq!(
            get_active_index_and_mask(usize::BITS as usize + 1),
            (1, 1 << 1)
        );
    }

    #[test]
    fn waker_shared_initializes_as_all_awake() {
        assert_eq!(
            Vec::from_iter(WakerShared::<0>::new().reset_all()),
            // Annotation required because `impl PartialEq<serde_json::Value> for usize` _might_ be seen by `rustc` and
            // make this ambiguous.
            Vec::<usize>::new()
        );
        assert_eq!(
            Vec::from_iter(WakerShared::<1>::new().reset_all()),
            Vec::from_iter(0..1)
        );
        assert_eq!(
            Vec::from_iter(WakerShared::<{ usize::BITS as usize - 1 }>::new().reset_all()),
            Vec::from_iter(0..usize::BITS as usize - 1)
        );
        assert_eq!(
            Vec::from_iter(WakerShared::<{ usize::BITS as usize }>::new().reset_all()),
            Vec::from_iter(0..usize::BITS as usize)
        );
        assert_eq!(
            Vec::from_iter(WakerShared::<{ usize::BITS as usize + 1 }>::new().reset_all()),
            Vec::from_iter(0..usize::BITS as usize + 1)
        );
    }

    #[test]
    fn bitwaker_valid_indexes() {
        static SHARED: WakerShared<TWO_WORDS> = WakerShared::new();
        let mut i = 0;
        while i < TWO_WORDS {
            BitWaker::new(i, &SHARED).wake_by_ref();
            i += 1;
        }
        assert!(std::panic::catch_unwind(|| BitWaker::new(i, &SHARED)).is_err());
    }

    #[test]
    fn extra_code_coverage() {
        static SHARED: ExecutorShared<1> = ExecutorShared::new(&SHARED);

        // Not the expected API usage, but should work to get code-coverage of some methods that are normally only
        // called in `const`-context.
        let _ = ExecutorShared::new(&SHARED);

        let source = pin!(generational::Source::new());
        let futures = [pin!(async move { core::future::pending().await }) as _];
        let executor = Executor::new(&SHARED, source.as_ref(), futures);

        let _ = std::format!("{executor:?}");

        let _ = BitWaker::<1>::invalid();
    }

    #[cfg(not(miri))] // Miri leak-checker doesn't like the leftover thread
    #[test]
    fn executor() {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn({
            move || {
                let source = pin!(generational::Source::new());

                static SHARED: ExecutorShared<1> = ExecutorShared::new(&SHARED);
                let futures = [pin!(async move {
                    let mut yielded = false;
                    core::future::poll_fn(|cx| {
                        if yielded {
                            Poll::Ready(())
                        } else {
                            yielded = true;
                            cx.waker().wake_by_ref();

                            #[expect(clippy::waker_clone_wake, reason = "for code-coverage")]
                            cx.waker().clone().wake();

                            Poll::Pending
                        }
                    })
                    .await;
                    // `Executor::run` doesn't return, so we notify that we got here and leave this thread parked.
                    let _ = tx.send(());
                    std::future::pending().await
                }) as _];

                let executor = Executor::new(&SHARED, source.as_ref(), futures);

                futures::executor::block_on(executor.run());
            }
        });

        assert!(rx.recv_timeout(std::time::Duration::from_secs(1)).is_ok());
    }
}
