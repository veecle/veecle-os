#![expect(missing_docs)]

use futures::future::FutureExt;
use veecle_osal_freertos::time::{Duration, Interval, Time, TimeAbstraction};

pub mod common;

#[test]
fn time_interval() {
    common::run_freertos_test(|| {
        let mut interval = Time::interval(Duration::from_millis(500));

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

        assert!(interval.tick().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(200),
        );

        assert!(interval.tick().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(400),
        );

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));

        assert!(interval.tick().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(200),
        );

        assert!(interval.tick().now_or_never().is_none());

        veecle_freertos_integration::CurrentTask::delay(
            veecle_freertos_integration::Duration::from_ms(300),
        );

        assert!(matches!(interval.tick().now_or_never(), Some(Ok(()))));
    });
}
