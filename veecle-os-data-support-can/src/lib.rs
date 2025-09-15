//! Support for working with CAN messages within a runtime instance.
#![no_std]
#![forbid(unsafe_code)]

#[cfg(test)]
extern crate std;

mod error;
mod frame;
mod generate;
mod id;

#[doc(hidden)]
/// Private API, do not use.
pub mod bits;

pub use self::error::CanDecodeError;
pub use self::frame::{Frame, FrameSize};
pub use self::id::{ExtendedId, Id, StandardId};

#[doc(hidden)]
/// Private API, do not use.
// Re-exports used in generated code.
// The non-ascii name is used as another signal to try and avoid dependents accessing this private API directly.
pub mod reëxports {
    pub use ::{serde, tinyvec, veecle_os_data_support_can_macros, veecle_os_runtime};
    #[cfg(feature = "arbitrary")]
    pub use ::arbitrary;

    pub use crate::bits;
}
