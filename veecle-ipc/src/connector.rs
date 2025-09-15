use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;

use crate::Exporter;

type Inputs = Arc<Mutex<HashMap<&'static str, mpsc::Sender<String>>>>;

/// Manages the connection to other runtimes via the `veecle-orchestrator`.
#[derive(Debug)]
pub struct Connector {
    output: mpsc::Sender<veecle_ipc_protocol::Message<'static>>,
    inputs: Inputs,
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

        let stream = UnixStream::connect(&socket).await.unwrap();
        let mut stream = Framed::new(stream, veecle_ipc_protocol::Codec::new());

        // TODO: if this fills up then we currently panic when trying to write to it, we need to
        // make some decisions around reliability guarantees for whether we can just drop messages
        // instead.
        let (output, mut output_rx) = mpsc::channel(128);
        let inputs = Inputs::default();
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
                                veecle_ipc_protocol::Message::Storable(storable) => {
                                    let Some(sender) = inputs.lock().unwrap().get(&*storable.type_name).cloned() else {
                                        continue
                                    };
                                    let _ = sender.send(storable.value).await;
                                }
                                veecle_ipc_protocol::Message::Telemetry(_) => {
                                    veecle_telemetry::error!("received unexpected ipc message variant", message = format!("{message:?}"));
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        });

        Self {
            output,
            inputs,
            _task: task,
        }
    }

    /// Returns an [`Exporter`] that will forward [`veecle-telemetry`][veecle_telemetry] data over this IPC connection to
    /// be gathered by the `veecle-orchestrator`.
    ///
    /// ```no_run
    /// # async move {
    /// let connector = veecle_ipc::Connector::connect().await;
    ///
    /// veecle_os::telemetry::collector::set_exporter(
    ///     veecle_os::telemetry::protocol::ExecutionId::random(&mut rand::rng()),
    ///     Box::leak(Box::new(connector.exporter())),
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # };
    /// ```
    pub fn exporter(&self) -> Exporter {
        Exporter::new(self.output.clone())
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
    pub(crate) fn output(&self) -> mpsc::Sender<veecle_ipc_protocol::Message<'static>> {
        self.output.clone()
    }
}
