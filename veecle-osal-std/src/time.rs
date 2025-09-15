//! Reexport of `std::time` modules.

use std::future::{Future, IntoFuture};

use futures::future::{Either, TryFutureExt};
use veecle_osal_api::Error;
pub use veecle_osal_api::time::{
    Duration, Exceeded, Instant, Interval, SystemTime, SystemTimeError, SystemTimeSync,
    TimeAbstraction,
};

/// Implements the [`TimeAbstraction`] trait for standard Rust.
///
/// This implementation uses [`tokio`] under the hood and, therefore, all time-based
/// operations will rely on the Tokio runtime.
#[derive(Debug)]
pub struct Time;

impl TimeAbstraction for Time {
    fn now() -> Instant {
        use std::sync::LazyLock;
        static EPOCH: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
        Instant::MIN
            + Duration::try_from(EPOCH.elapsed())
                .expect("time elapsed since start is less than 2^64-1 microseconds")
    }

    async fn sleep(duration: Duration) -> Result<(), Error> {
        let duration = std::time::Duration::from_millis(duration.as_millis());
        tokio::time::sleep(duration).await;
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

    fn timeout_at<F>(
        deadline: Instant,
        future: F,
    ) -> impl Future<Output = Result<F::Output, Either<Exceeded, Error>>>
    where
        Self: Sized,
        F: IntoFuture,
    {
        let duration = deadline
            .duration_since(Self::now())
            .map(|duration| std::time::Duration::from_millis(duration.as_millis()))
            .unwrap_or(std::time::Duration::ZERO);

        tokio::time::timeout(duration, future).map_err(|_elapsed| Either::Left(Exceeded))
    }

    fn interval(period: Duration) -> impl Interval
    where
        Self: Sized,
    {
        struct IntervalInternal(tokio::time::Interval);

        impl Interval for IntervalInternal {
            async fn tick(&mut self) -> Result<(), Error> {
                self.0.tick().await;
                Ok(())
            }
        }

        let period = std::time::Duration::from_millis(period.as_millis());
        IntervalInternal(tokio::time::interval(period))
    }
}

impl SystemTime for Time {
    fn duration_since_epoch() -> Result<Duration, SystemTimeError> {
        let std_duration_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("SystemTime::now() should not be less then UNIX_EPOCH");
        Ok(Duration::from_millis(
            std_duration_since_epoch.as_millis() as u64
        ))
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use core::pin::pin;

    use futures::future::{Either, FutureExt};
    use veecle_osal_api::time::{Duration, Exceeded, Interval, SystemTime, TimeAbstraction};

    use crate::time::Time;

    #[test]
    fn test_std_system_time_duration_since_epoch() {
        let a = Time::duration_since_epoch().unwrap_or_default();
        let b = Time::duration_since_epoch().unwrap_or_default();
        assert!(b.as_millis() - a.as_millis() < 100);

        let sleep_time = 200;
        std::thread::sleep(std::time::Duration::from_millis(sleep_time));

        let c = Time::duration_since_epoch().unwrap_or_default();
        assert!(c.as_millis() - a.as_millis() >= sleep_time);
        assert!(c.as_millis() - a.as_millis() < sleep_time + 100);
    }

    #[tokio::test(start_paused = true)]
    async fn sleep_until_smoke_test() {
        let mut sleep = pin!(Time::sleep_until(Time::now() + Duration::from_secs(5)));

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(4)).await;

        assert!(matches!(sleep.as_mut().now_or_never(), Some(Ok(()))));
    }

    #[tokio::test(start_paused = true)]
    async fn sleep_smoke_test() {
        let mut sleep = pin!(Time::sleep(Duration::from_secs(5)));

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(4)).await;

        assert!(matches!(sleep.as_mut().now_or_never(), Some(Ok(()))));
    }

    #[tokio::test(start_paused = true)]
    async fn sleep_max_test() {
        let mut sleep = pin!(Time::sleep(Duration::MAX));

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(sleep.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(60 * 60 * 24 * 7 * 52 * 20)).await;

        assert!(sleep.as_mut().now_or_never().is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_at_smoke_test() {
        let mut future = pin!(Time::timeout_at(
            Time::now() + Duration::from_secs(10),
            Time::sleep_until(Time::now() + Duration::from_secs(5))
        ));

        assert!(future.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(future.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(4)).await;

        assert!(matches!(future.as_mut().now_or_never(), Some(Ok(Ok(())))));
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_at_smoke_test_exceeded() {
        let mut future = pin!(Time::timeout_at(
            Time::now() + Duration::from_secs(5),
            std::future::pending::<()>()
        ));

        assert!(future.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(future.as_mut().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(4)).await;

        assert!(matches!(
            future.as_mut().now_or_never(),
            Some(Err(Either::Left(Exceeded)))
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn interval_smoke_test() {
        let mut interval = Time::interval(Duration::from_secs(5));

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

        assert!(interval.tick().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(interval.tick().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(4)).await;

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

        assert!(interval.tick().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(2)).await;

        assert!(interval.tick().now_or_never().is_none());

        tokio::time::advance(std::time::Duration::from_secs(3)).await;

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));
    }
}
