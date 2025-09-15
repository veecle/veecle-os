#![expect(missing_docs)]

use futures::FutureExt;
use std::pin::pin;
use veecle_osal_api::time::Duration;
use veecle_osal_api::time::TimeAbstraction;
use veecle_osal_embassy::time::Time;

#[test]
fn timer_timeout_at() {
    let driver = embassy_time::MockDriver::get();
    let mut future = pin!(Time::timeout_at(
        Time::now() + Duration::from_secs(10),
        Time::sleep_until(Time::now() + Duration::from_secs(5))
    ));

    assert!(future.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(future.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(4));

    assert!(matches!(future.as_mut().now_or_never(), Some(Ok(Ok(())))));
}
