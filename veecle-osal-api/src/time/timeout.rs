use core::future::Future;
use core::pin::pin;

use futures::future::{Either, select};

use super::{Error, Instant, TimeAbstraction};

/// A [`TimeAbstraction::timeout_at`] reached the deadline before the future resolved.
#[derive(Debug)]
pub struct Exceeded;

/// Implementation for [`TimeAbstraction::timeout_at`].
pub async fn timeout_at<T, F>(
    deadline: Instant,
    future: F,
) -> Result<F::Output, Either<Exceeded, Error>>
where
    T: TimeAbstraction,
    F: Future,
{
    match select(pin!(T::sleep_until(deadline)), pin!(future)).await {
        Either::Left((Ok(_), _)) => Err(Either::Left(Exceeded)),
        Either::Left((Err(error), _)) => Err(Either::Right(error)),
        Either::Right((output, _)) => Ok(output),
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use crate::time::{Duration, Error, Instant, Interval, TimeAbstraction};

    /// A mock implementation for [TimeAbstraction].
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    struct TimeMock<const NOW: u64>;

    impl<const NOW: u64> TimeAbstraction for TimeMock<NOW> {
        fn now() -> Instant {
            Instant::MIN + Duration::from_secs(NOW)
        }

        async fn sleep_until(deadline: Instant) -> Result<(), Error> {
            if Self::now() < deadline {
                // Never resolves.
                core::future::pending::<()>().await;
            }
            Ok(())
        }

        fn interval(_: Duration) -> impl Interval {
            struct IntervalInternal;
            impl Interval for IntervalInternal {
                async fn tick(&mut self) -> Result<(), Error> {
                    unimplemented!()
                }
            }
            unimplemented!();
            #[allow(unreachable_code)] // Used for type hinting.
            IntervalInternal
        }
    }

    #[test]
    fn timeout_with_future_that_completes_in_time_should_not_fail() {
        async fn should_complete_on_time() {}

        let result = block_on(TimeMock::<0>::timeout_at(
            Instant::MIN + Duration::from_secs(123),
            should_complete_on_time(),
        ));
        assert!(result.is_ok(), "the future did complete out of time");
    }

    #[test]
    fn timeout_with_future_that_completes_out_of_time_should_fail() {
        async fn should_complete_out_of_time() {}

        let result = block_on(TimeMock::<123>::timeout_at(
            Instant::MIN + Duration::from_secs(0),
            should_complete_out_of_time(),
        ));
        assert!(result.is_err(), "the future did complete in time");
    }
}
