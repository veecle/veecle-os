use super::Export;
use crate::protocol::{InstanceMessage, LogMessage, TelemetryMessage};
use std::prelude::v1::String;

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
                .fold(String::new(), |mut formatted, key_value| {
                    formatted.push_str(", ");
                    formatted.push_str(&std::format!("{}", key_value));
                    formatted
                });
            std::println!("[{severity:?}:{time_unix_nano}] {body}: \"{attributes}\"");
        }
    }
}
