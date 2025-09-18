//! Allocation example actor.

use alloc::boxed::Box;
use alloc::format;
use core::convert::Infallible;
use veecle_os::osal::api::time::{Duration, TimeAbstraction};

#[veecle_os::runtime::actor]
pub async fn box_actor<T: TimeAbstraction>() -> Infallible {
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

        T::sleep(Duration::from_secs(1)).await.unwrap();
    }
}
