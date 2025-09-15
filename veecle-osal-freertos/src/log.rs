//! Logging related system utilities.

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
use std::io::Write;

pub use veecle_osal_api::log::LogTarget;

/// Implements the [`LogTarget`] trait.
#[derive(Debug)]
pub struct Log;

impl LogTarget for Log {
    type Time = crate::time::Time;

    fn init() {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        rtt_target::rtt_init_print!();
    }

    fn println(args: core::fmt::Arguments<'_>) {
        // TODO: How should this sort of `cfg` be handled.
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        // `"{args}"` _would_ work, except `rtt_target::rprintln!` has a buggy macro arm that bypasses the formatting
        // infrastructure on a single expression.
        rtt_target::rprintln!("{}", args);

        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        // this is a logger, ignore any errors writing
        let _ = std::writeln!(std::io::stdout(), "{args}");
    }
}
