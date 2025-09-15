#![expect(missing_docs)]

use futures::FutureExt;
use futures::future::Either;
use std::pin::pin;
use veecle_osal_api::time::Duration;
use veecle_osal_api::time::TimeAbstraction;
use veecle_osal_embassy::time::Time;

#[test]
fn timer_timeout_at_exceeded() {
    let driver = embassy_time::MockDriver::get();
    let mut future = pin!(Time::timeout_at(
        Time::now() + Duration::from_secs(5),
        std::future::pending::<()>()
    ));

    assert!(future.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(future.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(4));

    assert!(matches!(
        future.as_mut().now_or_never(),
        Some(Err(Either::Left(veecle_osal_api::time::Exceeded)))
    ));
}
