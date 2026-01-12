use super::Export;

/// An exporter that outputs telemetry messages as JSON to stdout.
///
/// # Examples
///
/// ```rust
/// use veecle_osal_std::{time::Time, thread::Thread};
///
/// veecle_telemetry::collector::build()
///     .random_process_id()
///     .console_json_exporter()
///     .time::<Time>()
///     .thread::<Thread>()
///     .set_global()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ConsoleJsonExporter(());

impl ConsoleJsonExporter {
    /// A `const` version of `ConsoleJsonExporter::default()` to allow use as a `&'static`.
    pub const DEFAULT: Self = ConsoleJsonExporter(());
}

impl Export for ConsoleJsonExporter {
    fn export(&self, message: crate::protocol::transient::InstanceMessage) {
        std::println!("{}", serde_json::to_string(&message).unwrap());
    }
}
