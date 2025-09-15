#![expect(missing_docs)]

use futures::future::FutureExt;
use veecle_osal_freertos::time::{Duration, Time, TimeAbstraction};

pub mod common;

#[test]
fn timer_sleep_until() {
    common::run_freertos_test(|| {
        let mut sleep =
            core::pin::pin!(Time::sleep_until(Time::now() + Duration::from_millis(500)));

        assert!(sleep.as_mut().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(200),
        );

        assert!(sleep.as_mut().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(400),
        );

        assert!(matches!(sleep.as_mut().now_or_never(), Some(Ok(()))));
    });
}
