use examples_common::actors::alloc::BoxActor;
use veecle_freertos_integration::{FreeRtosAllocator, Task, TaskPriority, vPortGetHeapStats};
use veecle_os::osal::api::time::{Duration, TimeAbstraction};
use veecle_os::osal::freertos::time::Time;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

#[veecle_os::runtime::actor]
async fn alloc_stat_actor() -> veecle_os::runtime::Never {
    loop {
        Time::sleep(Duration::from_secs(4)).await.unwrap();

        veecle_os::telemetry::debug!("Alloc", stats = format!("{:#?}", vPortGetHeapStats()));
    }
}

pub fn main() -> ! {
    veecle_os::telemetry::collector::build()
        .random_process_id()
        .console_json_exporter()
        .time::<Time>()
        .thread::<veecle_os::osal::std::thread::Thread>()
        .set_global()
        .unwrap();

    Task::new()
        .name(c"alloc example")
        .stack_size(1024 * 8)
        .priority(TaskPriority(2))
        .start(|_| {
            veecle_os::osal::freertos::task::block_on_future(veecle_os::runtime::execute! {
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
