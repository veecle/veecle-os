use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::time::Duration;

use camino::{Utf8Path, Utf8PathBuf};
use eyre::{OptionExt, Result, WrapErr, bail};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use iceoryx2::node::NodeBuilder;
use iceoryx2::port::server::Server;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::port::unable_to_deliver_strategy::UnableToDeliverStrategy;
use iceoryx2::service::ipc;
use iceoryx2::service::service_name::ServiceName;
use tempfile::TempPath;
use tokio::process::Child;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use veecle_ipc_protocol::{ControlRequest, ControlResponse, EncodedStorable};
use veecle_orchestrator_protocol::InstanceId;

use crate::runtime::conductor::Command;
use crate::telemetry::Exporter;
use veecle_net_utils::AsyncUnixListener;

/// Represents the source of a runtime binary.
#[derive(Debug)]
pub(crate) enum BinarySource {
    /// A regular file path.
    Path(Utf8PathBuf),
    /// A temporary file path that will be cleaned up when dropped.
    Temporary(TempPath),
}

impl BinarySource {
    /// Gets the path to the binary file.
    pub fn path(&self) -> &Utf8Path {
        match self {
            Self::Path(path) => path,
            Self::Temporary(temp_path) => Utf8Path::from_path(temp_path.as_ref())
                .expect("temporary file path should be valid UTF-8"),
        }
    }
}

impl From<Utf8PathBuf> for BinarySource {
    fn from(path: Utf8PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<TempPath> for BinarySource {
    fn from(temp_path: TempPath) -> Self {
        Self::Temporary(temp_path)
    }
}

/// An instance of a runtime process registered on this orchestrator.
///
/// Each instance has a known binary path it will execute from, and may or may not have a currently
/// running process.
#[derive(Debug)]
pub(crate) struct RuntimeInstance {
    id: InstanceId,
    binary: BinarySource,
    process: Option<Child>,
    ipc_task: Option<tokio::task::JoinHandle<Result<()>>>,
    ipc_shutdown: CancellationToken,
    iceoryx2_task: Option<tokio::task::JoinHandle<Result<()>>>,
    socket_path: Utf8PathBuf,
    privileged: bool,
}

impl Drop for RuntimeInstance {
    fn drop(&mut self) {
        self.ipc_shutdown.cancel();
        if let Some(task) = &self.ipc_task {
            task.abort();
        }
        if let Some(task) = &self.iceoryx2_task {
            task.abort();
        }
    }
}

/// Helps send a [`Command`] over the given channel and receive the response `T` over the
/// command-specific one-shot channel.
async fn send_command<T>(
    command_tx: &mpsc::Sender<Command>,
    make_command: impl FnOnce(oneshot::Sender<eyre::Result<T>>) -> Command,
) -> eyre::Result<T> {
    let (response_tx, response_rx) = oneshot::channel();
    command_tx
        .send(make_command(response_tx))
        .await
        .map_err(|_| eyre::Error::msg("conductor unavailable"))?;

    response_rx
        .await
        .map_err(|_| eyre::Error::msg("conductor unavailable"))?
}

/// Handles a single [`ControlRequest`].
async fn handle_control_request(
    request: veecle_ipc_protocol::ControlRequest,
    command_tx: &mpsc::Sender<Command>,
) -> veecle_ipc_protocol::ControlResponse {
    let response: eyre::Result<_> = async {
        match request {
            ControlRequest::StartRuntime { id } => {
                let id = InstanceId(id);
                send_command(command_tx, |response_tx| Command::StartInstance {
                    id,
                    response_tx,
                })
                .await?;
                Ok(ControlResponse::Started)
            }
            ControlRequest::StopRuntime { id } => {
                let id = InstanceId(id);
                send_command(command_tx, |response_tx| Command::StopInstance {
                    id,
                    response_tx,
                })
                .await?;
                Ok(ControlResponse::Stopped)
            }
        }
    }
    .await;

    match response {
        Ok(response) => response,
        Err(error) => ControlResponse::Error(error.to_string()),
    }
}

/// Handles the IPC for a single runtime instance.
///
/// This expects to have the runtime instance connect using `veecle-ipc` to the provided `socket` (only one client at a
/// time, but maybe reconnecting if the instance is stopped and restarted).
/// Any messages arriving on `ipc_rx` will be encoded and sent to the instance.
/// Any `Storable` messages arriving from the instance will be decoded and forwarded to `ipc_tx`.
#[tracing::instrument(skip_all, fields(%id))]
#[expect(clippy::too_many_arguments)]
async fn handle_instance_ipc(
    id: InstanceId,
    socket: tempfile::NamedTempFile<AsyncUnixListener>,
    ipc_tx: mpsc::Sender<EncodedStorable>,
    mut ipc_rx: mpsc::Receiver<EncodedStorable>,
    shutdown: CancellationToken,
    exporter: Option<Arc<Exporter>>,
    privileged: bool,
    command_tx: mpsc::Sender<Command>,
) -> Result<()> {
    let socket = socket.as_file();
    loop {
        tokio::select! {
            accept_result = socket.accept() => {
                let (stream, _address) = accept_result?;
                let mut stream = Framed::new(stream, veecle_ipc_protocol::Codec::new());
                loop {
                    tokio::select! {
                        storable = ipc_rx.recv() => {
                            let Some(storable) = storable else { break };
                            let message = veecle_ipc_protocol::Message::EncodedStorable(storable);
                            stream.send(&message).await?;
                        }
                        message = stream.next() => {
                            let Some(message) = message.transpose()? else { break };
                            match message {
                                veecle_ipc_protocol::Message::EncodedStorable(storable) => {
                                    ipc_tx.send(storable).await?;
                                }
                                veecle_ipc_protocol::Message::Telemetry(message) => {
                                    if let Some(ref exporter) = exporter {
                                        exporter.export(message);
                                    }
                                }
                                veecle_ipc_protocol::Message::ControlRequest(request) => {
                                    let response = if privileged {
                                        handle_control_request(request, &command_tx).await
                                    } else {
                                        tracing::warn!("non-privileged runtime attempted to send control request");
                                        veecle_ipc_protocol::ControlResponse::Error("no control privileges".to_owned())
                                    };

                                    stream.send(&veecle_ipc_protocol::Message::ControlResponse(response)).await?;
                                }
                                veecle_ipc_protocol::Message::ControlResponse(_) | _ => {
                                    tracing::warn!(?message, "received unexpected ipc message variant");
                                }
                            }
                        }
                    }
                }
            }
            _ = shutdown.cancelled() => {
                return Ok(());
            }
        }
    }
}

/// Handles telemetry messages from a runtime instance via iceoryx2.
///
/// This subscribes to the telemetry topic for the given runtime instance and forwards
/// any received telemetry messages to the exporter.
///
/// TODO: This could be implemented via `async`, but it's `!Send` so would need to run a
/// `LocalSet`/`LocalRuntime`.
#[tracing::instrument(skip_all, fields(%id))]
fn handle_iceoryx2(
    id: InstanceId,
    exporter: Option<Arc<Exporter>>,
    privileged: bool,
    command_tx: mpsc::Sender<Command>,
    shutdown: CancellationToken,
) -> Result<()> {
    let node = NodeBuilder::new()
        .create::<ipc::Service>()
        .wrap_err("failed to create iceoryx2 node")?;

    let telemetry_service_name = ServiceName::new(&format!("veecle/runtime/{id}/telemetry"))
        .wrap_err("invalid telemetry service name")?;
    let telemetry_service = node
        .service_builder(&telemetry_service_name)
        .publish_subscribe::<[u8]>()
        .open_or_create()
        .wrap_err("failed to create telemetry service")?;
    let telemetry_subscriber = telemetry_service
        .subscriber_builder()
        .create()
        .wrap_err("failed to create telemetry subscriber")?;

    let control_service_name = ServiceName::new(&format!("veecle/runtime/{id}/control"))
        .wrap_err("invalid control service name")?;
    let control_service = node
        .service_builder(&control_service_name)
        .request_response::<[u8], [u8]>()
        .open_or_create()
        .wrap_err("failed to create control service")?;
    let control_server = control_service
        .server_builder()
        .initial_max_slice_len(4096)
        .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
        .create()
        .wrap_err("failed to create control server")?;

    let runtime = tokio::runtime::Handle::current();

    loop {
        if shutdown.is_cancelled() {
            return Ok(());
        }

        while let Ok(Some(request)) = Server::<_, [u8], (), [u8], ()>::receive(&control_server) {
            if shutdown.is_cancelled() {
                return Ok(());
            }

            match serde_json::from_slice::<ControlRequest>(&request) {
                Ok(control_request) => {
                    tracing::info!(?control_request, "received control request");
                    let response = if privileged {
                        runtime.block_on(handle_control_request(control_request, &command_tx))
                    } else {
                        tracing::warn!("non-privileged runtime attempted to send control request");
                        ControlResponse::Error("no control privileges".to_owned())
                    };

                    match serde_json::to_vec(&response) {
                        Ok(json) => {
                            if let Ok(resp) = request.loan_slice_uninit(json.len()) {
                                let resp = resp.write_from_slice(&json);
                                if let Err(error) = resp.send() {
                                    tracing::error!(?error, "failed to send control response");
                                }
                            } else {
                                tracing::error!("failed to loan buffer for control response");
                            }
                        }
                        Err(error) => {
                            tracing::error!(?error, "failed to serialize control response");
                        }
                    }
                }
                Err(error) => {
                    tracing::error!(?error, "failed to deserialize control request");
                }
            }
        }

        while let Ok(Some(sample)) = Subscriber::<_, [u8], _>::receive(&telemetry_subscriber) {
            if shutdown.is_cancelled() {
                return Ok(());
            }

            let Some(exporter) = &exporter else {
                continue;
            };

            match serde_json::from_slice::<veecle_telemetry::protocol::InstanceMessage<'_>>(&sample)
            {
                Ok(message) => {
                    exporter.export(message);
                }
                Err(error) => {
                    tracing::error!(?error, "failed to deserialize telemetry message");
                }
            }
        }

        // There is no way to register interest for new values, so we just busy-poll with a slight
        // delay.
        std::thread::sleep(Duration::from_millis(1));
    }
}

impl RuntimeInstance {
    /// Returns a new `RuntimeInstance` instance.
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: InstanceId,
        socket_dir: &Utf8Path,
        binary: BinarySource,
        ipc_tx: mpsc::Sender<EncodedStorable>,
        ipc_rx: mpsc::Receiver<EncodedStorable>,
        exporter: Option<Arc<Exporter>>,
        privileged: bool,
        command_tx: mpsc::Sender<Command>,
    ) -> Result<Self> {
        let socket = tempfile::Builder::new()
            .prefix(&format!("{id}-"))
            .suffix(".sock")
            .make_in(socket_dir, |path| {
                let socket_path = Utf8Path::from_path(path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "non-UTF-8 socket path")
                })?;
                AsyncUnixListener::bind(socket_path)
            })?;

        let socket_path = Utf8Path::from_path(socket.path())
            .ok_or_eyre("non-UTF-8 socket path")?
            .to_owned();

        let ipc_shutdown = CancellationToken::new();
        let ipc_task = tokio::spawn(handle_instance_ipc(
            id,
            socket,
            ipc_tx,
            ipc_rx,
            ipc_shutdown.clone(),
            exporter.clone(),
            privileged,
            command_tx.clone(),
        ));

        let iceoryx2_task = tokio::task::spawn_blocking({
            let shutdown = ipc_shutdown.clone();
            move || handle_iceoryx2(id, exporter, privileged, command_tx, shutdown)
        });

        Ok(Self {
            id,
            binary,
            process: None,
            ipc_task: Some(ipc_task),
            ipc_shutdown,
            iceoryx2_task: Some(iceoryx2_task),
            socket_path,
            privileged,
        })
    }

    /// Returns whether this instance has a currently running process.
    pub(crate) fn is_running(&self) -> bool {
        self.process.is_some()
    }

    /// Returns the binary source used for this instance.
    pub(crate) fn binary(&self) -> &BinarySource {
        &self.binary
    }

    /// Returns whether this instance has control privileges.
    pub(crate) fn privileged(&self) -> bool {
        self.privileged
    }

    /// Starts the process for this instance.
    pub(crate) fn start(&mut self) -> Result<()> {
        if self.process.is_some() {
            bail!("instance id {} is already running", self.id);
        }

        let binary = self.binary.path();
        let process = tokio::process::Command::new(binary)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("VEECLE_IPC_SOCKET", &self.socket_path)
            .env("VEECLE_RUNTIME_ID", self.id.to_string())
            .spawn()
            .wrap_err_with(|| format!("starting runtime process '{binary}'"))?;

        self.process = Some(process);

        Ok(())
    }

    /// Stops the process for this instance (but allows it to be started again later).
    pub(crate) async fn stop(&mut self) -> Result<()> {
        let Some(process) = self.process.take() else {
            bail!("instance id {} is not running", self.id);
        };

        let status = kill_child(process).await?;

        tracing::info!("child stop exit status {status:?}");

        Ok(())
    }

    /// Stops all processing for this instance and cleans up any associated temporary files.
    pub(crate) async fn cleanup(mut self) -> Result<()> {
        if self.is_running() {
            self.stop().await?;
        }
        self.ipc_shutdown.cancel();
        self.ipc_task
            .take()
            .ok_or_eyre("IPC task missing")?
            .await
            .wrap_err("joining IPC task failed")?
            .wrap_err("IPC task failed")?;
        self.iceoryx2_task
            .take()
            .ok_or_eyre("iceoryx2 task missing")?
            .await
            .wrap_err("joining iceoryx2 task failed")?
            .wrap_err("iceoryx2 task failed")?;
        Ok(())
    }
}

/// Attempts to nicely kill a child process, first attempting to interrupt it and give it 100ms to shutdown before
/// killing it.
async fn kill_child(mut process: Child) -> Result<ExitStatus> {
    if let Some(id) = process.id() {
        let pid = nix::unistd::Pid::from_raw(libc::pid_t::try_from(id).unwrap());
        nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGINT)
            .wrap_err("interrupting child")?;
        if let Ok(status) = timeout(Duration::from_millis(100), process.wait()).await {
            return Ok(status?);
        }
        tracing::warn!("child did not stop in time after interrupt");
    }

    // `Child::kill` does not return the `ExitStatus`, for consistency with the process exiting itself we manually
    // `start_kill` and `wait` to grab the returned status code.
    process.start_kill().wrap_err("killing child")?;
    let status = timeout(Duration::from_millis(100), process.wait())
        .await
        .wrap_err("waiting for child to be killed")??;

    Ok(status)
}
