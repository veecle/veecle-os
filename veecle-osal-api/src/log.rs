//! Traits for outputting log strings.

use crate::time::TimeAbstraction;

/// `LogTarget` is used to perform log-related operations in a platform-agnostic manner.
pub trait LogTarget: Send + Sync + 'static {
    /// A source of time to add into log messages.
    type Time: TimeAbstraction;

    /// Initializes global state necessary for this type.
    fn init();

    /// Outputs a line of text through this log target.
    fn println(args: core::fmt::Arguments<'_>);
}
