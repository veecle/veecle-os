#![expect(missing_docs)]

use veecle_osal_freertos::time::{Duration, Time, TimeAbstraction};

pub mod common;

#[test]
fn timer_sleep_until_async() {
    common::run_freertos_test(|| {
        veecle_osal_freertos::task::block_on_future(async move {
            let start = Time::now();

            Time::sleep_until(start + Duration::from_millis(200))
                .await
                .unwrap();

            let elapsed = Time::now().duration_since(start).unwrap();
            assert!(elapsed >= Duration::from_millis(200));
            assert!(elapsed < Duration::from_millis(250));
        });
    });
}
