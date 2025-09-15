#![expect(missing_docs)]

use futures::FutureExt;
use std::pin::pin;
use veecle_osal_api::time::Duration;
use veecle_osal_api::time::TimeAbstraction;
use veecle_osal_embassy::time::Time;

#[test]
fn timer_sleep_max() {
    let driver = embassy_time::MockDriver::get();
    let mut sleep = pin!(Time::sleep(Duration::MAX));

    assert!(sleep.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(sleep.as_mut().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(
        60 * 60 * 24 * 7 * 52 * 20,
    ));

    assert!(sleep.as_mut().now_or_never().is_none());
}
