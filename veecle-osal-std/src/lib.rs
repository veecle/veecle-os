//! Std basic operating system abstraction layer for Veecle OS.
//!
//! This provides the primitives that we need to use in Veecle OS, using the std library.

#![forbid(unsafe_code)]

pub mod log;
pub mod net;
pub mod thread;
pub mod time;

pub use veecle_osal_api::{Error, Result};
pub use veecle_osal_std_macros::main;

/// Do not use!
///
/// Reexported to enable the `veecle-osal-std` `main` macro.
///
/// This is exempted from SemVer versioning and may be changed or removed at any time without prior notice.
#[doc(hidden)]
pub mod reexports {
    pub use rand;
    pub use tokio;
}

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
