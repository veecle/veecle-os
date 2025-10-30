use super::Export;
use crate::protocol::{InstanceMessage, LogMessage, TelemetryMessage};

/// Exporter that pretty prints telemetry messages to stdout.
///
/// This exporter only supports log messages (e.g. `error!("foo")`).
///
/// <div class="warning">
/// Only intended for experimentation and examples.
/// `telemetry-ui` is strongly recommended for anything beyond experimentation.
/// </div>
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::collector::{ConsolePrettyExporter, set_exporter};
/// use veecle_telemetry::protocol::ExecutionId;
///
/// let execution_id = ExecutionId::random(&mut rand::rng());
/// set_exporter(execution_id, &ConsolePrettyExporter).unwrap();
/// ```
#[derive(Debug)]
pub struct ConsolePrettyExporter;

impl Export for ConsolePrettyExporter {
    fn export(
        &self,
        InstanceMessage {
            execution: _,
            message,
        }: InstanceMessage,
    ) {
        if let TelemetryMessage::Log(LogMessage {
            time_unix_nano,
            severity,
            body,
            attributes,
            ..
        }) = message
        {
            let attributes = attributes
                .iter()
                .map(|key_value| std::format!("{}", key_value))
                .reduce(|mut formatted, attribute| {
                    formatted.push_str(&attribute);
                    formatted
                })
                .unwrap_or_default();
            std::println!("[{severity:?}:{time_unix_nano}] {body}: \"{attributes}\"");
        }
    }
}
