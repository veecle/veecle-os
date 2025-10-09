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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::num::NonZeroU64;
    use tokio::sync::mpsc;
    use veecle_telemetry::collector::Export;
    use veecle_telemetry::protocol::{
        InstanceMessage, LogMessage, ProcessId, Severity, TelemetryMessage, ThreadId,
    };

    use super::Exporter;

    const THREAD_ID: ThreadId =
        ThreadId::from_raw(ProcessId::from_raw(123), NonZeroU64::new(456).unwrap());

    #[tokio::test]
    async fn test_export_telemetry_message() {
        let (sender, mut receiver) = mpsc::channel(1);
        let exporter = Exporter::new(sender);

        let test_message = InstanceMessage {
            thread_id: THREAD_ID,
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: 1000000000,
                severity: Severity::Info,
                body: "test log message".into(),
                attributes: Default::default(),
            }),
        };

        exporter.export(test_message);

        let received = receiver.recv().await.expect("should receive message");
        match received {
            veecle_ipc_protocol::Message::Telemetry(message) => {
                assert_eq!(message.thread_id, THREAD_ID);
                match message.message {
                    TelemetryMessage::Log(message) => {
                        assert_eq!(message.time_unix_nano, 1000000000);
                        assert_eq!(message.severity, Severity::Info);
                        assert_eq!(message.body.as_ref(), "test log message");
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
            thread_id: THREAD_ID,
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: 2000000000,
                severity: Severity::Error,
                body: "error log message".into(),
                attributes: Default::default(),
            }),
        };

        exporter.export(test_message);
    }
}
