#![expect(missing_docs)]

use futures::FutureExt;
use veecle_osal_api::time::{Duration, Interval, TimeAbstraction};
use veecle_osal_embassy::time::Time;

#[test]
fn timer_interval() {
    let driver = embassy_time::MockDriver::get();

    let mut interval = Time::interval(Duration::from_secs(5));

    assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

    assert!(interval.tick().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(interval.tick().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(4));

    assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

    assert!(interval.tick().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(2));

    assert!(interval.tick().now_or_never().is_none());

    driver.advance(embassy_time::Duration::from_secs(3));

    assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));
}
