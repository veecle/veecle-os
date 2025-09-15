//! FreeRTOS operating system abstraction layer.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
extern crate std;

pub mod log;
pub mod time;

mod error;

pub use veecle_osal_api::{Error, Result};

/// Utilities for working with FreeRTOS tasks.
pub mod task {
    pub use veecle_freertos_integration::task::block_on_future;
}
