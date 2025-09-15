//! Time related system utilities.

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use core::task::Poll;

use futures::task::AtomicWaker;
use veecle_freertos_integration::{
    Duration as FreeRtosDuration, TickType_t, Timer as FreeRtosTimer,
};
use veecle_osal_api::Error;
pub use veecle_osal_api::time::{
    Duration, Exceeded, Instant, Interval, SystemTime, SystemTimeError, SystemTimeSync,
    TimeAbstraction,
};

use crate::error::into_veecle_os_error;

/// Implements the [`TimeAbstraction`] trait for FreeRTOS.
///
/// ## Details
///
/// - Before using [`Self::duration_since_epoch`], you are expected to synchronize system time via
///   [`Self::set_system_time`].
/// - Maximum precision of system time synchronization is limited to seconds.
#[derive(Debug)]
pub struct Time;

impl TimeAbstraction for Time {
    fn now() -> Instant {
        Instant::MIN
            + Duration::from_millis(
                veecle_freertos_integration::scheduler::get_tick_count_duration().ms() as u64,
            )
    }

    async fn sleep(duration: Duration) -> Result<(), Error> {
        let duration = FreeRtosDuration::from_ms(duration.as_millis() as TickType_t);

        if duration == FreeRtosDuration::zero() {
            return Ok(());
        }

        /// Data shared between the timer managed by FreeRTOS and the future we're returning.
        #[derive(Default)]
        struct Context {
            /// Tracks the waker currently associated to the future.
            waker: AtomicWaker,

            /// Set by the timer once the deadline is hit, to avoid spurious wakeups resolving early.
            done: AtomicBool,
        }

        let context = Arc::new(Context::default());

        let callback = Box::new({
            let context = context.clone();
            move |_| {
                context.done.store(true, Ordering::Relaxed);
                context.waker.wake();
            }
        });

        let timer =
            FreeRtosTimer::periodic(None, duration, callback).map_err(into_veecle_os_error)?;

        timer.handle().start().map_err(into_veecle_os_error)?;

        core::future::poll_fn(move |cx| {
            context.waker.register(cx.waker());
            if context.done.load(Ordering::Relaxed) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        Ok(())
    }

    async fn sleep_until(deadline: Instant) -> Result<(), Error> {
        Self::sleep(
            deadline
                .duration_since(Self::now())
                .unwrap_or(Duration::ZERO),
        )
        .await
    }

    fn interval(period: Duration) -> impl Interval
    where
        Self: Sized,
    {
        /// Data shared between the timer managed by FreeRTOS and the stream we're returning.
        struct Context {
            /// Tracks the waker currently associated to the stream.
            waker: AtomicWaker,

            /// How many more times the timer has fired than the stream has yielded.
            ///
            /// This is kept as a count so that when the stream misses ticks it will obey the trait docs.
            count: AtomicUsize,
        }

        struct IntervalInternal<F>
        where
            F: Fn(veecle_freertos_integration::TimerHandle) + Send + 'static,
        {
            // Owned but unused so it gets dropped and de-registered when this is dropped.
            _timer: Option<FreeRtosTimer<F>>,
            error: Option<veecle_freertos_integration::FreeRtosError>,
            context: Arc<Context>,
        }

        impl<F> Interval for IntervalInternal<F>
        where
            F: Fn(veecle_freertos_integration::TimerHandle) + Send + 'static,
        {
            async fn tick(&mut self) -> Result<(), Error> {
                if let Some(error) = self.error.take() {
                    return Err(into_veecle_os_error(error));
                }

                core::future::poll_fn(|cx| {
                    self.context.waker.register(cx.waker());

                    // It's ok to split the load and sub here because this is the only place that subtracts, we want to
                    // saturate at 0 rather than wrapping over to `usize::MAX`, and we don't care about the exact count.
                    if self.context.count.load(Ordering::Relaxed) != 0 {
                        self.context.count.fetch_sub(1, Ordering::Relaxed);
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                })
                .await
            }
        }

        let period = FreeRtosDuration::from_ms(period.as_millis() as TickType_t);

        let context = Arc::new(Context {
            waker: AtomicWaker::new(),
            count: AtomicUsize::new(1),
        });

        let callback = Box::new({
            let context = context.clone();
            move |_| {
                // This could technically wrap around, but at that point with a 1 millisecond period the stream is
                // already behind by 49 days and I don't think we care.
                context.count.fetch_add(1, Ordering::Relaxed);
                context.waker.wake();
            }
        });

        let (mut error, timer) = match FreeRtosTimer::periodic(None, period, callback) {
            Ok(timer) => (None, Some(timer)),
            Err(error) => (Some(error), None),
        };

        if let Some(timer) = &timer {
            error = timer.handle().start().err();
        }

        IntervalInternal {
            _timer: timer,
            error,
            context,
        }
    }
}

static SYSTEM_TIME_OFFSET_SECONDS: AtomicU32 = AtomicU32::new(0);

impl SystemTime for Time {
    fn duration_since_epoch() -> Result<Duration, SystemTimeError> {
        let offset = SYSTEM_TIME_OFFSET_SECONDS.load(Ordering::Relaxed);
        if offset == 0 {
            Err(SystemTimeError::Unsynchronized)
        } else {
            let duration_since_start = Self::now()
                .duration_since(Instant::MIN)
                .expect("now can't be less than min time");
            Ok(duration_since_start + Duration::from_secs(offset as u64))
        }
    }
}

impl SystemTimeSync for Time {
    fn set_system_time(duration_since_epoch: Duration) -> Result<(), SystemTimeError> {
        let duration_since_start = Duration::from_millis(
            veecle_freertos_integration::scheduler::get_tick_count_duration().ms() as u64,
        );
        if duration_since_epoch < duration_since_start {
            Err(SystemTimeError::EpochIsLaterThanStartTime)
        } else {
            SYSTEM_TIME_OFFSET_SECONDS.store(
                (duration_since_epoch - duration_since_start).as_secs() as u32,
                Ordering::Relaxed,
            );
            Ok(())
        }
    }
}
