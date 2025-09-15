#![expect(missing_docs)]

use veecle_osal_api::time::{SystemTime, SystemTimeError};
use veecle_osal_embassy::time::Time;

#[test]
fn system_time_duration_since_epoch_unsynced() {
    assert_eq!(
        Time::duration_since_epoch(),
        Err(SystemTimeError::Unsynchronized)
    );
}
