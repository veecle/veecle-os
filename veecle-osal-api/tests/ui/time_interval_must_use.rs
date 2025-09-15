#![deny(unused_must_use)]

use veecle_osal_api::time::{Duration, TimeAbstraction};

pub async fn test<T>() where T: TimeAbstraction {
    T::interval(Duration::from_secs(1));
}

fn main() {}
