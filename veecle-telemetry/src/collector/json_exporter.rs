use super::Export;
use crate::protocol::InstanceMessage;

/// An exporter that outputs telemetry messages as JSON to stdout.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::collector::{ConsoleJsonExporter, set_exporter, ProcessId};
///
/// let process_id = ProcessId::random(&mut rand::rng());
/// set_exporter(process_id, &ConsoleJsonExporter::DEFAULT).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ConsoleJsonExporter(());

impl ConsoleJsonExporter {
    /// A `const` version of `ConsoleJsonExporter::default()` to allow use as a `&'static`.
    pub const DEFAULT: Self = ConsoleJsonExporter(());
}

impl Export for ConsoleJsonExporter {
    fn export(&self, message: InstanceMessage) {
        std::println!("{}", serde_json::to_string(&message).unwrap());
    }
}
