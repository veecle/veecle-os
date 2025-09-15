//! The Veecle OS operating system abstraction layer API.

#![no_std]
// Set by `cargo-llvm-cov` https://github.com/taiki-e/cargo-llvm-cov?tab=readme-ov-file#exclude-code-from-coverage
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(any(test, feature = "test-suites"))]
extern crate std;

mod error;
pub mod log;
pub mod net;
pub mod time;

pub use error::{Error, Result};
