//! Logging related system utilities.

pub use veecle_osal_api::log::LogTarget;

#[cfg(not(target_os = "none"))]
use std::io::Write;

/// Implements the [`LogTarget`] trait.
#[derive(Debug)]
pub struct Log;

impl LogTarget for Log {
    type Time = crate::time::Time;

    fn init() {
        #[cfg(target_os = "none")]
        rtt_target::rtt_init_print!();
    }

    fn println(args: core::fmt::Arguments<'_>) {
        #[cfg(not(target_os = "none"))]
        // this is a logger, ignore any errors writing
        let _ = std::writeln!(std::io::stdout(), "{args}");

        #[cfg(target_os = "none")]
        // `"{args}"` _would_ work, except `rtt_target::rprintln!` has a buggy macro arm that bypasses the formatting
        // infrastructure on a single expression.
        rtt_target::rprintln!("{}", args);
    }
}
