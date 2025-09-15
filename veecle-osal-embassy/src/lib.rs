//! Embassy operating system abstraction layer.
//!
//! This OSAL requires binaries to depend on the [`embassy-time`](https://docs.rs/embassy-time/latest/embassy_time/)
//! crate with one of the `generic-queue-*` features enabled.
//! See <https://docs.rs/embassy-time/latest/embassy_time/#generic-queue> for more information.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

#[cfg(not(target_os = "none"))]
extern crate std;

pub mod log;
pub mod net;
pub mod time;

pub use veecle_osal_api::{Error, Result};

/// Helper trait to convert errors into osal errors.
///
/// We cannot implement `From` as that would be part of the public API.
pub(crate) trait IntoOsalError<E>
where
    E: core::error::Error,
{
    /// Converts the error into an OSAL error.
    fn into_osal_error(self) -> E;
}
