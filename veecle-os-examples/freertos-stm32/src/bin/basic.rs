//! Example on STM32F767ZI
#![no_std]
#![no_main]
#![allow(non_snake_case)]

use core::ffi::c_void;

use cortex_m::asm;
use cortex_m_rt::{ExceptionFrame, entry, exception};
use stm32f7xx_hal::gpio::*;
use stm32f7xx_hal::pac::Peripherals;
use veecle_freertos_integration::*;
use veecle_os::osal::freertos::log::{Log, LogTarget};
use veecle_os::osal::freertos::time::{SystemTime, SystemTimeSync, Time};

extern crate panic_halt;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

#[entry]
fn main() -> ! {
    // TODO(DEV-533/DEV-559): Initialize `veecle_os::telemetry` here, and use it for later `println`s.

    // In real-world app, you are expected to sync with a real server.
    Time::set_system_time(veecle_os::osal::freertos::time::Duration::from_secs(42))
        .expect("Unable to set system time.");

    veecle_freertos_integration::hooks::set_on_assert(|file_name, line| {
        Log::println(format_args!("{file_name}: {line}"));
    });

    Log::println(format_args!("Configuring peripherals"));
    let dp = Peripherals::take().unwrap();
    let mut led = dp.GPIOB.split().pb0.into_push_pull_output();
    led.set_low();

    Task::new()
        .name(c"basic example")
        .stack_size(4096)
        .priority(TaskPriority(2))
        .start(move |_| {
            loop {
                veecle_freertos_integration::CurrentTask::delay(Duration::from_ms(1000));
                led.set_high();
                Log::println(format_args!(
                    "[{:?}] Led ON",
                    Time::duration_since_epoch().unwrap_or_default()
                ));

                veecle_freertos_integration::CurrentTask::delay(Duration::from_ms(1000));
                led.set_low();
                Log::println(format_args!(
                    "[{:?}] Led OFF",
                    Time::duration_since_epoch().unwrap_or_default()
                ));
            }
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
