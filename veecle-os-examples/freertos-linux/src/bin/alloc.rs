use std::convert::Infallible;

use veecle_freertos_integration::{FreeRtosAllocator, Task, TaskPriority, vPortGetHeapStats};
use veecle_os::osal::api::time::{Duration, TimeAbstraction};
use veecle_os::osal::freertos::time::Time;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

#[veecle_os::runtime::actor]
async fn alloc_stat_actor() -> Infallible {
    loop {
        Time::sleep(Duration::from_secs(4)).await.unwrap();

        veecle_os::telemetry::debug!("Alloc", stats = format!("{:#?}", vPortGetHeapStats()));
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

pub fn main() -> ! {
    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::protocol::ExecutionId::random(&mut rand::rng()),
        &veecle_os::telemetry::collector::ConsoleJsonExporter,
    )
    .unwrap();

    Task::new()
        .name(c"alloc example")
        .stack_size(1024 * 8)
        .priority(TaskPriority(2))
        .start(|_| {
            veecle_os::osal::freertos::task::block_on_future(veecle_os::runtime::execute! {
                store: [],
                actors: [
                    AllocStatActor,
                    BoxActor,
                ],
            });
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
    panic!("FreeRTOS scheduler should not return");
}
