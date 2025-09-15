#![expect(missing_docs)]

use futures::FutureExt;
use std::pin::pin;
use veecle_osal_api::time::Duration;
use veecle_osal_api::time::TimeAbstraction;
use veecle_osal_embassy::time::Time;

#[test]
fn timer_sleep() {
    let driver = embassy_time::MockDriver::get();
    let mut sleep = pin!(Time::sleep(Duration::from_secs(5)));

    assert!(sleep.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(sleep.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(4));

    assert!(matches!(sleep.as_mut().now_or_never(), Some(Ok(()))));
}
