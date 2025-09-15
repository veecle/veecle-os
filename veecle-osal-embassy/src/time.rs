//! Time related system utilities.

use core::sync::atomic::{AtomicU32, Ordering};
use veecle_osal_api::Error;
pub use veecle_osal_api::time::{
    Duration, Instant, Interval, SystemTime, SystemTimeError, SystemTimeSync, TimeAbstraction,
};

/// Implements the [`TimeAbstraction`] trait for embassy.
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
        Instant::MIN + Duration::from_millis(embassy_time::Instant::now().as_millis())
    }

    async fn sleep_until(deadline: Instant) -> Result<(), Error> {
        Self::sleep(
            deadline
                .duration_since(Self::now())
                .unwrap_or(Duration::ZERO),
        )
        .await
    }

    async fn sleep(duration: Duration) -> Result<(), Error> {
        // `embassy_time::Timer::after` panics on overflow, so we reduce the duration until it doesn't overflow.
        // This does only reduce the chance of overflow and thus a panic.
        // We cannot prevent all situations in which an overflow might occur.
        let mut duration = duration;
        while Self::now().checked_add(duration).is_none() {
            duration = duration / 2;
        }
        embassy_time::Timer::after(embassy_time::Duration::from_millis(duration.as_millis())).await;
        Ok(())
    }

    fn interval(period: Duration) -> impl Interval
    where
        Self: Sized,
    {
        struct IntervalInternal {
            ticker: embassy_time::Ticker,
            first_poll: bool,
        }

        impl Interval for IntervalInternal {
            async fn tick(&mut self) -> Result<(), Error> {
                // Embassy's ticker doesn't immediately yield an item, so we need to do that to conform to the trait contract.
                if self.first_poll {
                    self.first_poll = false;
                    return Ok(());
                }

                self.ticker.next().await;
                Ok(())
            }
        }

        IntervalInternal {
            ticker: embassy_time::Ticker::every(embassy_time::Duration::from_millis(
                period.as_millis(),
            )),
            first_poll: true,
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
        let duration_since_start = Self::now()
            .duration_since(Instant::MIN)
            .expect("now can't be less than min time");

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
