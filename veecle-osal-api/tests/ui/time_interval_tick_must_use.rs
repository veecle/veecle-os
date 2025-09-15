#![deny(unused_must_use)]

use veecle_osal_api::time::Interval;

pub async fn test(mut interval: impl Interval) {
    interval.tick().await;
}

fn main() {}
