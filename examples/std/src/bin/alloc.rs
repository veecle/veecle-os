use std::alloc::System;

use examples_common::actors::alloc::BoxActor;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use veecle_os::osal::std::time::{Duration, Time, TimeAbstraction};
use veecle_os::runtime::Never;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[veecle_os::runtime::actor]
async fn alloc_stat_actor() -> Never {
    let region = Region::new(GLOBAL);

    loop {
        veecle_os::telemetry::debug!("Alloc stats", stats = format!("{:#?}", region.change()));

        Time::sleep(Duration::from_secs(4)).await.unwrap();
    }
}

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        actors: [
            AllocStatActor,
            BoxActor<veecle_os::osal::std::time::Time>,
        ],
    }
    .await;
}
