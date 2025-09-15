use super::Export;
use crate::protocol::InstanceMessage;

/// An exporter that outputs telemetry messages as JSON to stdout.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::collector::{ConsoleJsonExporter, set_exporter};
/// use veecle_telemetry::protocol::ExecutionId;
///
/// let execution_id = ExecutionId::random(&mut rand::rng());
/// set_exporter(execution_id, &ConsoleJsonExporter).unwrap();
/// ```
#[derive(Debug)]
pub struct ConsoleJsonExporter;

impl Export for ConsoleJsonExporter {
    fn export(&self, message: InstanceMessage) {
        std::println!("{}", serde_json::to_string(&message).unwrap());
    }
}
