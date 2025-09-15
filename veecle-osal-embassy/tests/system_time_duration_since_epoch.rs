#![expect(missing_docs)]

use veecle_osal_api::time::{Duration, SystemTime, SystemTimeSync};
use veecle_osal_embassy::time::Time;

// Miri does not support `std::time::SystemTime::now()`.
#[cfg(not(miri))]
#[test]
fn system_time_duration_since_epoch() {
    let driver = embassy_time::MockDriver::get();
    Time::set_system_time(Duration::from_secs(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("SystemTime::now() should not be less then UNIX_EPOCH")
            .as_secs(),
    ))
    .expect("Unable to set system time.");

    let a = Time::duration_since_epoch().unwrap_or_default();
    let b = Time::duration_since_epoch().unwrap_or_default();
    assert!(b.as_millis() - a.as_millis() < 100);

    let sleep_time = 200;
    driver.advance(embassy_time::Duration::from_millis(sleep_time));

    let c = Time::duration_since_epoch().unwrap_or_default();
    assert!(c.as_millis() - a.as_millis() >= sleep_time);
    assert!(c.as_millis() - a.as_millis() < sleep_time + 100);
}
