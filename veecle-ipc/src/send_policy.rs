//! Policies for handling IPC message sending when the channel is full.

/// Policy for handling messages when the IPC output channel is full.
///
/// The default is [`Panic`](SendPolicy::Panic) to make buffer exhaustion immediately visible
/// during development and testing. For production deployments where message loss is acceptable
/// (e.g., telemetry data), explicitly use [`Drop`](SendPolicy::Drop).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendPolicy {
    /// Drop messages when the output channel is full and log a warning.
    Drop,

    /// Panic when the output channel is full.
    Panic,
}

impl Default for SendPolicy {
    /// Returns [`Panic`](SendPolicy::Panic) to make buffer exhaustion visible by default.
    fn default() -> Self {
        Self::Panic
    }
}
