//! Logging related system utilities.

use std::io::Write;

pub use veecle_osal_api::log::LogTarget;

/// Implements the [`LogTarget`] trait by printing to standard output.
#[derive(Debug)]
pub struct Log;

impl LogTarget for Log {
    type Time = crate::time::Time;

    fn init() {
        // noöp
    }

    /// Prints to [`std::io::stdout`].
    fn println(args: core::fmt::Arguments<'_>) {
        // this is a logger, ignore any errors writing
        let _ = std::writeln!(std::io::stdout(), "{args}");
    }
}
