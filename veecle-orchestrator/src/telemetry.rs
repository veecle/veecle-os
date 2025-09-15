//! Telemetry forwarding for sending telemetry data to `veecle-telemetry-server`.

use std::net::SocketAddr;
use std::sync::Mutex;

use eyre::WrapErr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::{error, info, warn};
use veecle_telemetry::protocol::InstanceMessage;
use veecle_telemetry::to_static::ToStatic;

#[derive(Debug)]
struct ExporterState {
    sender: mpsc::UnboundedSender<InstanceMessage<'static>>,
    task: tokio::task::JoinHandle<()>,
}

/// Telemetry exporter that forwards InstanceMessage to `veecle-telemetry-server` via TCP.
#[derive(Debug)]
pub struct Exporter {
    state: Mutex<Option<ExporterState>>,
}

impl Exporter {
    /// Creates a new telemetry exporter.
    pub fn new(server_address: SocketAddr) -> eyre::Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            telemetry_forwarding_task(server_address, receiver).await;
        });

        Ok(Self {
            state: Mutex::new(Some(ExporterState { sender, task })),
        })
    }

    /// Exports a telemetry message by forwarding it via TCP.
    pub fn export(&self, message: InstanceMessage<'_>) {
        let static_message = message.to_static();
        if let Some(state) = &*self.state.lock().unwrap()
            && let Err(error) = state.sender.send(static_message)
        {
            warn!("Failed to send telemetry message to forwarding task: {error}");
        }
    }

    /// Shuts down the telemetry exporter, ensuring all messages are flushed.
    pub async fn shutdown(&self) {
        let state = self.state.lock().unwrap().take();

        if let Some(mut state) = state {
            drop(state.sender);
            // If messages aren't flushed within a second, just abort processing them.
            if tokio::time::timeout(Duration::from_secs(1), &mut state.task)
                .await
                .is_err()
            {
                state.task.abort();
            }
        }
    }
}

/// If `connection` is empty will attempt to connect to `server_address` to fill it, with exponential
/// backoff, returning the resulting connection.
#[tracing::instrument(skip(connection))]
async fn ensure_connection(
    server_address: SocketAddr,
    connection: &mut Option<TcpStream>,
) -> &mut TcpStream {
    if let Some(stream) = connection {
        return stream;
    }

    let mut reconnect_delay = Duration::from_millis(100);
    const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);

    loop {
        match TcpStream::connect(server_address).await {
            Ok(stream) => {
                info!("Connected to telemetry server");
                return connection.insert(stream);
            }
            Err(error) => {
                warn!(
                    "Failed to connect to telemetry server: {error}, \
                    reconnecting in {reconnect_delay:?}"
                );
                sleep(reconnect_delay).await;
                reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
            }
        }
    }
}

/// Forwards any messages from `receiver` JSONL encoded to the server at `server_address`,
/// automatically reconnecting as needed.
async fn telemetry_forwarding_task(
    server_address: SocketAddr,
    receiver: mpsc::UnboundedReceiver<InstanceMessage<'static>>,
) {
    let mut connection: Option<TcpStream> = None;
    let mut pending_line: Option<String> = None;

    let mut lines = UnboundedReceiverStream::new(receiver).filter_map(|message| {
        serde_json::to_string(&message)
            .wrap_err("failed to serialize telemetry message")
            .inspect_err(|error| error!(?error))
            .map(|json| json + "\n")
            .ok()
    });

    loop {
        if pending_line.is_none() {
            pending_line = lines.next().await;
        }
        let Some(line) = pending_line.take() else {
            break;
        };

        let stream = ensure_connection(server_address, &mut connection).await;
        if let Err(error) = stream.write_all(line.as_bytes()).await {
            warn!("Failed to send telemetry message, connection lost: {error}");
            connection = None;
            pending_line = Some(line);
        }
    }
}
