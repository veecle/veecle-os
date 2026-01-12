//! FreeRTOS-Linux example.
use core::convert::Infallible;

use veecle_freertos_integration::*;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

#[veecle_os::runtime::actor]
async fn queue_actor(
    #[init_context] init_context: (AsyncQueueReceiver<u32>, AsyncQueueSender<i32>),
) -> Infallible {
    const DATA: i32 = 77;

    let (mut queue_receiver, mut queue_sender) = init_context;

    loop {
        let data = queue_receiver.receive().await;
        veecle_os::telemetry::info!(
            "[Veecle OS task] Received some async data",
            data = i64::from(data)
        );

        queue_sender.send(DATA).await;
        veecle_os::telemetry::info!(
            "[Veecle OS Task] Sent response data",
            data = i64::from(DATA)
        );
    }
}

fn main() {
    veecle_os::telemetry::collector::build()
        .random_process_id()
        .console_json_exporter()
        .time::<veecle_os::osal::freertos::time::Time>()
        .thread::<veecle_os::osal::std::thread::Thread>()
        .set_global()
        .unwrap();

    let incoming_legacy_queue = Queue::<u32>::new(10).unwrap();
    let outgoing_legacy_queue = Queue::<i32>::new(10).unwrap();

    // Send data from C task to Veecle OS task.
    let async_queue_receiver = BlockingToAsyncQueueTaskBuilder::new(
        c"blocking_to_async",
        incoming_legacy_queue.clone(),
        2,
    )
    .priority(TaskPriority(2))
    .create()
    .unwrap();

    // Send data from Veecle OS task to C task.
    let async_queue_sender = AsyncToBlockingQueueTaskBuilder::new(
        c"async_to_blocking",
        outgoing_legacy_queue.clone(),
        2,
    )
    .priority(TaskPriority(2))
    .create()
    .unwrap();

    // Create garbage data.
    Task::new()
        .name(c"legacy_sender_task")
        .stack_size(8 * 1024)
        .priority(TaskPriority(1))
        .start(move |_| {
            const DATA: u32 = 99;

            loop {
                if incoming_legacy_queue
                    .send(DATA, Duration::from_ms(1000))
                    .is_ok()
                {
                    veecle_os::telemetry::info!(
                        "[Legacy task] Sent some data",
                        data = i64::from(DATA)
                    );
                }
                CurrentTask::delay(Duration::from_ms(1000));
            }
        })
        .unwrap();

    // Print return data.
    Task::new()
        .name(c"legacy_receiver_task")
        .stack_size(8 * 1024)
        .priority(TaskPriority(1))
        .start(move |_| {
            loop {
                if let Ok(data) = outgoing_legacy_queue.receive(Duration::from_ms(1000)) {
                    veecle_os::telemetry::info!(
                        "[Legacy task] Got data back",
                        data = i64::from(data)
                    );
                }
            }
        })
        .unwrap();

    // Main Veecle OS task.
    Task::new()
        .name(c"veecle_os_task")
        .stack_size(8 * 1024)
        .priority(TaskPriority(3))
        .start(move |_| {
            let init_data = (async_queue_receiver, async_queue_sender);

            veecle_os::osal::freertos::task::block_on_future(veecle_os::runtime::execute! {
                store: [],
                actors: [
                    QueueActor: init_data,
                ],
            });
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
}
