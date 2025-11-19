use veecle_freertos_integration::{FreeRtosAllocator, Task, TaskPriority};
use veecle_os::osal::freertos::time::Time;
use veecle_os_examples_common::actors::time::{Tick, TickerActor, TickerReader};

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

/// An example of using the time abstraction within a FreeRTOS task.
pub fn main() -> ! {
    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::collector::ProcessId::random(&mut rand::rng()),
        &veecle_os::telemetry::collector::ConsoleJsonExporter::DEFAULT,
    )
    .unwrap();

    Task::new()
        .name(c"time example")
        .stack_size(8 * 1024)
        .priority(TaskPriority(2))
        .start(|_| {
            veecle_os::osal::freertos::task::block_on_future(veecle_os::runtime::execute! {
                store: [Tick],
                actors: [
                    TickerReader,
                    TickerActor<Time>,
                ],
            });
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
    panic!("FreeRTOS scheduler should not return");
}
