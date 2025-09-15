#![expect(missing_docs)]

use veecle_osal_freertos::time::{Duration, Interval, Time, TimeAbstraction};

pub mod common;

#[test]
fn timer_interval_async() {
    common::run_freertos_test(|| {
        veecle_osal_freertos::task::block_on_future(async move {
            let start = Time::now();
            let mut interval = Time::interval(Duration::from_millis(100));

            interval.tick().await.unwrap();
            interval.tick().await.unwrap();
            interval.tick().await.unwrap();

            let elapsed = Time::now().duration_since(start).unwrap();
            assert!(elapsed >= Duration::from_millis(200));
            assert!(elapsed < Duration::from_millis(250));
        });
    });
}
