use std::alloc::System;
use std::convert::Infallible;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use veecle_os::osal::std::time::{Duration, Time, TimeAbstraction};

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

#[veecle_os::runtime::actor]
async fn box_actor() -> Infallible {
    const BOX_COUNT: usize = 5;
    let mut box_counter = 0;
    let mut boxes: [Option<Box<u64>>; BOX_COUNT] = [const { None }; BOX_COUNT];

    loop {
        match boxes.iter_mut().find(|slot| slot.is_none()) {
            // Allocate a new box.
            Some(slot) => {
                *slot = Some(Box::new(box_counter));
                box_counter += 1;
            }
            // Drop all boxes.
            None => boxes = [const { None }; BOX_COUNT],
        }
        veecle_os::telemetry::info!("Boxes", boxes = format!("{:?}", boxes));

        Time::sleep(Duration::from_secs(1)).await.unwrap();
    }
}

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        store: [],
        actors: [
            AllocStatActor,
            BoxActor,
        ],
    }
    .await;
}
