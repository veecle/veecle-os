use tokio::sync::mpsc;
use veecle_telemetry::collector::Export;
use veecle_telemetry::protocol::InstanceMessage;
use veecle_telemetry::to_static::ToStatic;

/// An [`Export`] implementer that forwards telemetry messages via IPC.
#[derive(Debug)]
pub struct Exporter {
    sender: mpsc::Sender<veecle_ipc_protocol::Message<'static>>,
}

impl Exporter {
    /// Creates a new IPC telemetry exporter.
    pub fn new(sender: mpsc::Sender<veecle_ipc_protocol::Message<'static>>) -> Self {
        Self { sender }
    }
}

impl Export for Exporter {
    /// Exports a telemetry message by forwarding it via IPC.
    ///
    /// This method converts the telemetry message to a static lifetime
    /// and sends it through the IPC channel. If the channel is full or closed,
    /// the message is dropped to avoid blocking telemetry collection.
    fn export(&self, message: InstanceMessage<'_>) {
        let message = veecle_ipc_protocol::Message::Telemetry(message.to_static());
        let _ = self.sender.try_send(message);
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;
    use veecle_telemetry::collector::Export;
    use veecle_telemetry::protocol::{
        ExecutionId, InstanceMessage, LogMessage, Severity, TelemetryMessage,
    };
    use veecle_telemetry::{SpanId, TraceId};

    use super::Exporter;

    #[tokio::test]
    async fn test_export_telemetry_message() {
        let (sender, mut receiver) = mpsc::channel(1);
        let exporter = Exporter::new(sender);

        let test_message = InstanceMessage {
            execution: ExecutionId::from_raw(123),
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: 1000000000,
                severity: Severity::Info,
                body: "test log message".into(),
                attributes: Default::default(),
                trace_id: Some(TraceId(0x1234)),
                span_id: Some(SpanId(0x5678)),
            }),
        };

        exporter.export(test_message);

        let received = receiver.recv().await.expect("should receive message");
        match received {
            veecle_ipc_protocol::Message::Telemetry(message) => {
                assert_eq!(*message.execution, 123);
                match message.message {
                    TelemetryMessage::Log(message) => {
                        assert_eq!(message.time_unix_nano, 1000000000);
                        assert_eq!(message.severity, Severity::Info);
                        assert_eq!(message.body.as_ref(), "test log message");
                        assert_eq!(message.trace_id, Some(TraceId(0x1234)));
                        assert_eq!(message.span_id, Some(SpanId(0x5678)));
                    }
                    _ => panic!("Expected Log message"),
                }
            }
            _ => panic!("Expected Telemetry"),
        }
    }

    #[tokio::test]
    async fn test_export_with_closed_channel() {
        let (sender, receiver) = mpsc::channel(1);
        let exporter = Exporter::new(sender);

        drop(receiver);

        let test_message = InstanceMessage {
            execution: ExecutionId::from_raw(456),
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: 2000000000,
                severity: Severity::Error,
                body: "error log message".into(),
                attributes: Default::default(),
                trace_id: Some(TraceId(0xabcd)),
                span_id: Some(SpanId(0xef01)),
            }),
        };

        exporter.export(test_message);
    }
}
