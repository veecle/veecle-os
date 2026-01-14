#![no_std]
#![no_main]
#![allow(non_snake_case)]

use core::ffi::c_void;

use cortex_m::asm;
use cortex_m_rt::{ExceptionFrame, entry, exception};
use stm32f7xx_hal as _;
use veecle_freertos_integration::*;
use veecle_os::osal::freertos::log::{Log, LogTarget};
use veecle_os::osal::freertos::time::Time;
use veecle_os_examples_common::actors::time::{TickerActor, TickerReader};

extern crate panic_halt;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

#[entry]
fn main() -> ! {
    // TODO(DEV-533/DEV-559): Initialize `veecle_os::telemetry` here, and use it for later `println`s.

    Log::println(format_args!("Configuring tasks"));

    veecle_freertos_integration::hooks::set_on_assert(|file_name, line| {
        Log::println(format_args!("{file_name}: {line}"));
    });

    Task::new()
        .name(c"time example")
        .stack_size(4096)
        .priority(TaskPriority(2))
        .start(|_| {
            veecle_os::osal::freertos::task::block_on_future(veecle_os::runtime::execute! {
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

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {}

#[exception]
#[allow(clippy::empty_loop)]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    asm::bkpt();
    loop {}
}

// SAFETY: `vApplicationStackOverflowHook` matches the signature in `task.h` of the FreeRTOS kernel source.
// FreeRTOS itself doesn't provide a definition for this function.
// Neither does this example, which means this is the only instance of this function being exported.
#[unsafe(no_mangle)]
fn vApplicationStackOverflowHook(_pxTask: *const c_void, _pcTaskName: *const u8) {
    asm::bkpt();
}
