//! `veecle-telemetry-server` sends tracing data piped in over a WebSocket connection to `veecle-telemetry-ui`.

#![forbid(unsafe_code)]

mod store;

use std::io::{ErrorKind, IsTerminal};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::Parser;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use log::{LevelFilter, error, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use veecle_telemetry_server_protocol::TracingMessage;

use crate::store::{TracingLineData, TracingLineStore};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    #[arg(short, long, default_value_t = 9000)]
    port: u16,

    #[arg(long, env = "VEECLE_TELEMETRY_SOCKET")]
    telemetry_socket: Option<std::net::SocketAddr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    if std::io::stdin().is_terminal() && args.telemetry_socket.is_none() {
        error!(
            "veecle-telemetry-server needs data to be piped in directly or --telemetry-socket specified"
        );
        std::process::exit(1);
    }

    let bind_addr = std::net::SocketAddr::new(args.bind.parse()?, args.port);

    let server = TcpListener::bind(&bind_addr).await?;
    info!("Listening on: ws://{bind_addr}");

    let store: Arc<TracingLineStore> = Default::default();

    if !std::io::stdin().is_terminal() {
        tokio::spawn({
            let store = store.clone();
            async move {
                if let Err(error) = handle_stdin(store).await {
                    error!("stdin task failed: {error:?}");
                }
            }
        });
    }

    if let Some(address) = args.telemetry_socket {
        tokio::spawn({
            let store = store.clone();
            async move {
                if let Err(error) = handle_telemetry_socket(address, store).await {
                    error!("Telemetry socket task failed: {error:?}");
                }
            }
        });
    }

    loop {
        let (socket, _) = server.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_connection(socket, store.clone()).await {
                error!("connection task failed: {error:?}");
            }
        });
    }
}

async fn handle_stdin(store: Arc<TracingLineStore>) -> anyhow::Result<()> {
    let mut last_log = Instant::now();

    let mut reader = BufReader::new(tokio::io::stdin()).lines();

    while let Some(line) = reader.next_line().await.context("reading from stdin")? {
        let total_lines = store.push_line(line);

        if total_lines.is_multiple_of(10) || last_log.elapsed() >= Duration::from_millis(100) {
            print!("\rProcessed {total_lines} lines.\r");
            last_log = Instant::now();
        }
    }

    store.set_done();
    store.read(|data| {
        info!("Processed {} lines. Pipe closed.", data.lines.len());
    });

    Ok(())
}

const MAX_ITEMS: usize = 50;

async fn handle_connection(stream: TcpStream, store: Arc<TracingLineStore>) -> anyhow::Result<()> {
    /// Check if an error is a client disconnection error.
    fn is_client_disconnection_error(error: &anyhow::Error) -> bool {
        error.chain().any(|cause| {
            cause
                .downcast_ref::<std::io::Error>()
                .is_some_and(|io_err| {
                    matches!(
                        io_err.kind(),
                        ErrorKind::BrokenPipe
                            | ErrorKind::ConnectionReset
                            | ErrorKind::ConnectionAborted
                            | ErrorKind::UnexpectedEof
                    )
                })
        })
    }

    let address = stream.peer_addr().context("getting peer address")?;

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .context("websocket handshake")?;

    info!("New WebSocket connection: {address}");

    let (ws_sink, mut ws_stream) = ws_stream.split();

    let mut client = Client::new(ws_sink, store.clone());

    let result = async {
        client.send_tracing_messages().await?;

        loop {
            tokio::select! {
                message = ws_stream.next() => {
                    let Some(message) = message else {
                        break;
                    };

                    let message = message.context("reading from websocket")?;

                    if message.is_close() {
                        break;
                    }

                    client.handle_message(message).await?;
                }
                _ = store.wait_for_data() => {
                    client.send_tracing_messages().await?;
                }
            }
        }

        anyhow::Ok(())
    }
    .await;

    info!("Connection closed: {address}");

    match result {
        Ok(()) => Ok(()),
        Err(error) if is_client_disconnection_error(&error) => Ok(()),
        Err(error) => Err(error),
    }
}

#[derive(Debug)]
struct Client {
    sent: usize,

    ws_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    store: Arc<TracingLineStore>,
}

impl Client {
    fn new(
        ws_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
        store: Arc<TracingLineStore>,
    ) -> Self {
        Self {
            sent: 0,
            ws_sink,
            store,
        }
    }

    async fn send_tracing_messages(&mut self) -> anyhow::Result<()> {
        while self.sent < self.store.read(|data| data.lines.len()) {
            let message = self.store.read(|data| create_message(data, self.sent));

            self.sent += message.lines.len();

            self.ws_sink
                .send(Message::text(
                    serde_json::to_string(&message).context("serializing message")?,
                ))
                .await
                .context("sending message")?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> anyhow::Result<()> {
        warn!("Unknown message received: {message:?}");

        Ok(())
    }
}

fn create_message(data: &TracingLineData, offset: usize) -> TracingMessage {
    let lines: Vec<String> = data
        .lines
        .iter()
        .skip(offset)
        .take(MAX_ITEMS)
        .cloned()
        .collect();

    TracingMessage {
        total: data.lines.len(),
        done: data.done,

        lines,
    }
}

/// Listens for new connections to the telemetry socket and spawns a task to handle each one.
///
/// Only returns on error.
async fn handle_telemetry_socket(
    address: std::net::SocketAddr,
    store: Arc<TracingLineStore>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)
        .await
        .context("binding telemetry socket")?;

    info!("Listening for telemetry on: {address}");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        info!("New telemetry connection from: {peer_addr}");

        let store = store.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_telemetry_connection(stream, store).await {
                error!("Telemetry connection failed: {error:?}");
            }
        });
    }
}

/// Handles a telemetry socket connection, pushing any received lines into the `store`.
///
/// Returns when the remote disconnects, or there is an error.
async fn handle_telemetry_connection(
    stream: TcpStream,
    store: Arc<TracingLineStore>,
) -> anyhow::Result<()> {
    let mut lines = BufReader::new(stream).lines();

    while let Some(line) = lines.next_line().await.context("reading telemetry line")? {
        // Store the encoded line directly without deserializing
        store.push_line(line);
    }

    Ok(())
}
