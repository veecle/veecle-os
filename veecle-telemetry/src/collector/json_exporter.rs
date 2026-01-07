use super::Export;

/// An exporter that outputs telemetry messages as JSON to stdout.
///
/// # Examples
///
/// ```rust
/// use veecle_osal_std::{time::Time, thread::Thread};
/// use veecle_telemetry::collector::{ConsoleJsonExporter, ProcessId};
///
/// let process_id = ProcessId::random(&mut rand::rng());
/// veecle_telemetry::collector::build()
///     .process_id(process_id)
///     .exporter(&ConsoleJsonExporter::DEFAULT)
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
