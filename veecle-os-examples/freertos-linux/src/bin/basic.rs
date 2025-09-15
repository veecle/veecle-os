//! FreeRTOS-Linux example.
use veecle_freertos_integration::*;
use veecle_os::osal::freertos::time::{SystemTime, SystemTimeSync, Time};

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

fn main() {
    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::protocol::ExecutionId::random(&mut rand::rng()),
        &veecle_os::telemetry::collector::ConsoleJsonExporter,
    )
    .unwrap();

    Time::set_system_time(veecle_os::osal::freertos::time::Duration::from_secs(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("SystemTime::now() should not be less then UNIX_EPOCH")
            .as_secs(),
    ))
    .expect("Unable to set system time.");

    #[repr(align(128))]
    #[derive(Debug)]
    struct Test {
        _content: usize,
    }

    let x = Box::new(Test { _content: 42 });
    veecle_os::telemetry::debug!(
        "Boxed Test (allocator large alignment test)",
        x = format!("{x:?}")
    );
    assert!(core::ptr::addr_of!(*x).is_aligned());

    let x = Box::new(15);
    veecle_os::telemetry::debug!("Boxed int (allocator test)", x = *x);

    hooks::set_on_assert(|file_name, line| {
        println!("file name and line: {file_name}:{line}",);
    });

    veecle_os::telemetry::debug!("Starting FreeRTOS app ...");
    Task::new()
        .name(c"hello")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(|_this_task| {
            let mut i = 0;
            loop {
                let duration_since_epoch = Time::duration_since_epoch().unwrap_or_default();
                veecle_os::telemetry::debug!(
                    "Hello from Task!",
                    i,
                    timestamp = format!("{duration_since_epoch:?}")
                );
                CurrentTask::delay(Duration::from_ms(1000));
                i += 1;
            }
        })
        .unwrap();

    veecle_os::telemetry::debug!("Starting scheduler");
    veecle_freertos_integration::scheduler::start_scheduler();

    #[allow(unreachable_code)]
    loop {
        veecle_os::telemetry::error!("Loop forever!");
    }
}
