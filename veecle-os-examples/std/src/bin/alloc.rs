use std::alloc::System;
use std::convert::Infallible;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use veecle_os::osal::std::time::{Duration, Time, TimeAbstraction};
use veecle_os_examples_common::actors::alloc::BoxActor;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[veecle_os::runtime::actor]
async fn alloc_stat_actor() -> Infallible {
    let region = Region::new(GLOBAL);

    loop {
        veecle_os::telemetry::debug!("Alloc stats", stats = format!("{:#?}", region.change()));

        Time::sleep(Duration::from_secs(4)).await.unwrap();
    }
}

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        store: [],
        actors: [
            AllocStatActor,
            BoxActor<veecle_os::osal::std::time::Time>,
        ],
    }
    .await;
}
