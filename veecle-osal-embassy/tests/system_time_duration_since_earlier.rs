#![expect(missing_docs)]

use veecle_osal_api::time::{Duration, SystemTimeError, SystemTimeSync};
use veecle_osal_embassy::time::Time;

#[test]
fn system_time_duration_since_epoch_earlier() {
    let driver = embassy_time::MockDriver::get();
    driver.advance(embassy_time::Duration::from_secs(100));

    assert_eq!(
        Time::set_system_time(Duration::from_secs(10)),
        Err(SystemTimeError::EpochIsLaterThanStartTime)
    );
}
