use veecle_freertos_integration::{FreeRtosAllocator, Task};

#[global_allocator]
static GLOBAL: FreeRtosAllocator =
    // SAFETY: The README.md requires one test per-binary, which should avoid any multi-threaded interactions with the
    // allocator.
    unsafe { FreeRtosAllocator::new() };

/// Runs `func` within a default-constructed [`Task`].
pub fn run_freertos_test(to_test_fn: impl FnOnce() + Send + 'static) {
    Task::new()
        .start(|_| {
            to_test_fn();

            end_scheduler();
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
}

/// Safe wrapper for [`vTaskEndScheduler`](veecle_freertos_sys::bindings::vTaskEndScheduler) for tests only.
pub fn end_scheduler() {
    // SAFETY: The README.md requires tests to be run using the FreeRTOS POSIX port.
    // On the FreeRTOS POSIX port, `vTaskEndScheduler` does not have any requirements on the caller.
    unsafe {
        veecle_freertos_sys::bindings::vTaskEndScheduler();
    }
}
