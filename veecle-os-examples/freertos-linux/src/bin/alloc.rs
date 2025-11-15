use std::convert::Infallible;

use veecle_freertos_integration::{FreeRtosAllocator, Task, TaskPriority, vPortGetHeapStats};
use veecle_os::osal::api::time::{Duration, TimeAbstraction};
use veecle_os::osal::freertos::time::Time;
use veecle_os_examples_common::actors::alloc::BoxActor;

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

pub fn main() -> ! {
    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::collector::ProcessId::random(&mut rand::rng()),
        &veecle_os::telemetry::collector::ConsoleJsonExporter::DEFAULT,
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
                    BoxActor<veecle_os::osal::freertos::time::Time>,
                ],
            });
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
    panic!("FreeRTOS scheduler should not return");
}
