//! The Veecle OS operating system abstraction layer API.

#![no_std]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(any(test, feature = "test-suites"))]
extern crate std;

mod error;
pub mod log;
pub mod net;
pub mod time;

pub use error::{Error, Result};
