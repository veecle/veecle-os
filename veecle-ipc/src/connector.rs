use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;
use veecle_ipc_protocol::{Codec, ControlRequest, ControlResponse, EncodedStorable, Message, Uuid};

use crate::Exporter;

type Inputs = Arc<Mutex<HashMap<&'static str, mpsc::Sender<String>>>>;

/// Holds various output channel senders for the [`Connector`], separated so they have decoupled
/// buffering and prioritization.
#[derive(Debug)]
struct OutputTx {
    storable: mpsc::Sender<EncodedStorable>,
    telemetry: mpsc::Sender<veecle_telemetry::protocol::InstanceMessage<'static>>,
    control: mpsc::Sender<ControlRequest>,
}

/// The receivers for [`OutputTx`].
#[derive(Debug)]
struct OutputRx {
    storable: mpsc::Receiver<EncodedStorable>,
    telemetry: mpsc::Receiver<veecle_telemetry::protocol::InstanceMessage<'static>>,
    control: mpsc::Receiver<ControlRequest>,
}

impl OutputRx {
    /// Returns the first message available on any output channel.
    ///
    /// Purposefully prioritizes the more important channels to drain first.
    /// This may lead to the low priority channels never being serviced if we are not keeping up.
    async fn recv(&mut self) -> Option<Message<'static>> {
        Some(tokio::select! {
            biased; // Polls all branches in order to guarantee prioritization.
            Some(control) = self.control.recv() => Message::ControlRequest(control),
            Some(storable) = self.storable.recv() => Message::Storable(storable),
            Some(telemetry) = self.telemetry.recv() => Message::Telemetry(telemetry),
            else => return None, // Only reached when all channels are closed.
        })
    }
}

fn outputs() -> (OutputTx, OutputRx) {
    // Control requests are request-response so there should never be buffering as the sender will
    // be waiting on a response.
    let (control_tx, control_rx) = mpsc::channel(1);
    // The output channel capacity (128) determines buffering for IPC messages.
    // The `Output` actor uses `SendPolicy` to control behavior when this fills up:
    // - `SendPolicy::Panic` (default): panics to make buffer exhaustion visible
    // - `SendPolicy::Drop`: drops messages and logs a warning
    let (storable_tx, storable_rx) = mpsc::channel(128);
    // Telemetry can be quite chatty, so give it a large buffer, the `Exporter` will discard
    // messages if this is filled.
    let (telemetry_tx, telemetry_rx) = mpsc::channel(128);

    (
        OutputTx {
            storable: storable_tx,
            control: control_tx,
            telemetry: telemetry_tx,
        },
        OutputRx {
            storable: storable_rx,
            control: control_rx,
            telemetry: telemetry_rx,
        },
    )
}

/// Manages the connection to other runtimes via the `veecle-orchestrator`.
#[derive(Debug)]
pub struct Connector {
    runtime_id: Uuid,
    output_tx: OutputTx,
    inputs: Inputs,
    control_responses: Mutex<Option<mpsc::Receiver<ControlResponse>>>,
    _task: JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
}

impl Connector {
    /// Finds and connects to the `veecle-orchestrator`.
    ///
    /// See the [crate][`crate`] docs for an example.
    ///
    /// # Panics
    ///
    /// If the connection cannot be established.
    pub async fn connect() -> Self {
        let socket = std::env::var("VEECLE_IPC_SOCKET").unwrap();
        let runtime_id = std::env::var("VEECLE_RUNTIME_ID").unwrap();
        let runtime_id = Uuid::from_str(&runtime_id).unwrap();

        let stream = UnixStream::connect(&socket).await.unwrap();
        let mut stream = Framed::new(stream, Codec::new());

        let inputs = Inputs::default();
        let (output_tx, mut output_rx) = outputs();

        let (control_response_tx, control_response_rx) = mpsc::channel(16);
        let task = tokio::spawn({
            let inputs = inputs.clone();
            async move {
                loop {
                    tokio::select! {
                        message = output_rx.recv() => {
                            let Some(message) = message else { break };
                            stream.send(&message).await?;
                        }
                        message = stream.next() => {
                            let Some(message) = message else { break };
                            let message = match message {
                                Ok(message) => message,
                                Err(error) => {
                                    let error = anyhow::Error::new(error).context("invalid ipc message");
                                    veecle_telemetry::error!("error", error = format!("{error:?}"));
                                    continue
                                }
                            };
                            match message {
                                Message::Storable(storable) => {
                                    let Some(sender) = inputs.lock().unwrap().get(&*storable.type_name).cloned() else {
                                        continue
                                    };
                                    let _ = sender.send(storable.value).await;
                                }
                                Message::Telemetry(_) => {
                                    veecle_telemetry::error!("received unexpected ipc message variant", message = format!("{message:?}"));
                                }
                                Message::ControlRequest(_) => {
                                    veecle_telemetry::error!("received unexpected ipc message variant", message = format!("{message:?}"));
                                }
                                Message::ControlResponse(response) => {
                                    let _ = control_response_tx.send(response).await;
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        });

        Self {
            runtime_id,
            output_tx,
            inputs,
            control_responses: Mutex::new(Some(control_response_rx)),
            _task: task,
        }
    }

    /// Returns an [`Exporter`] that will forward [`veecle-telemetry`][veecle_telemetry] data over this IPC connection to
    /// be gathered by the `veecle-orchestrator`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "enable")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = veecle_ipc::Connector::connect().await;
    ///
    /// veecle_os::telemetry::collector::set_exporter(
    ///     veecle_os::telemetry::collector::ProcessId::random(&mut rand::rng()),
    ///     Box::leak(Box::new(connector.exporter())),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn exporter(&self) -> Exporter {
        Exporter::new(self.output_tx.telemetry.clone())
    }

    /// Returns this runtime's instance id.
    ///
    /// This id uniquely identifies this runtime instance within the orchestrator.
    pub fn runtime_id(&self) -> Uuid {
        self.runtime_id
    }

    /// Registers a new channel that will receive input from the `veecle-orchestrator` tagged with `type_name`.
    pub(crate) fn storable_input(&self, type_name: &'static str) -> mpsc::Receiver<String> {
        match self.inputs.lock().unwrap().entry(type_name) {
            Entry::Occupied(_) => panic!("type name {type_name} already registered"),
            Entry::Vacant(entry) => {
                let (sender, receiver) = mpsc::channel(16);
                entry.insert(sender);
                receiver
            }
        }
    }

    /// Gets a new sender to send values to the `veecle-orchestrator`.
    pub(crate) fn storable_output(&self) -> mpsc::Sender<EncodedStorable> {
        self.output_tx.storable.clone()
    }

    /// Gets the sender and receiver to send control messages and receive control responses from the `veecle-orchestrator`.
    ///
    /// This can only be called once, as there should only be one `ControlHandler` actor.
    pub(crate) fn control_channels(
        &self,
    ) -> (
        mpsc::Sender<ControlRequest>,
        mpsc::Receiver<ControlResponse>,
    ) {
        (
            self.output_tx.control.clone(),
            self.control_responses
                .lock()
                .unwrap()
                .take()
                .expect("control_channels can only be called once"),
        )
    }
}
