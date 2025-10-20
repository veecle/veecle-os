use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::time::Duration;

use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tempfile::{TempDir, TempPath};
use tokio::process::Child;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use veecle_ipc_protocol::EncodedStorable;
use veecle_orchestrator_protocol::{InstanceId, RuntimeInfo};

use crate::distributor::Distributor;
use crate::telemetry::Exporter;
use veecle_net_utils::AsyncUnixListener;

/// Represents the source of a runtime binary.
#[derive(Debug)]
pub enum BinarySource {
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

/// An instance of a runtime process registered on the [`Conductor`].
///
/// Each instance has a known binary path it will execute from, and may or may not have a currently
/// running process.
#[derive(Debug)]
struct RuntimeInstance {
    binary: BinarySource,
    process: Option<Child>,
    ipc_task: Option<tokio::task::JoinHandle<eyre::Result<()>>>,
    ipc_shutdown: CancellationToken,
    socket_path: Utf8PathBuf,
}

impl Drop for RuntimeInstance {
    fn drop(&mut self) {
        if let Some(task) = &self.ipc_task {
            task.abort();
        }
    }
}

/// Handles the IPC for a single runtime instance.
///
/// This expects to have the runtime instance connect using `veecle-ipc` to the provided `socket` (only one client at a
/// time, but maybe reconnecting if the instance is stopped and restarted).
/// Any messages arriving on `ipc_rx` will be encoded and sent to the instance.
/// Any `Storable` messages arriving from the instance will be decoded and forwarded to `ipc_tx`.
#[tracing::instrument(skip_all, fields(%id))]
async fn handle_instance_ipc(
    id: InstanceId,
    socket: tempfile::NamedTempFile<AsyncUnixListener>,
    ipc_tx: mpsc::Sender<EncodedStorable>,
    mut ipc_rx: mpsc::Receiver<EncodedStorable>,
    shutdown: CancellationToken,
    exporter: Option<Arc<Exporter>>,
) -> eyre::Result<()> {
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
                            let message = veecle_ipc_protocol::Message::Storable(storable);
                            stream.send(&message).await?;
                        }
                        message = stream.next() => {
                            let Some(message) = message.transpose()? else { break };
                            match message {
                                veecle_ipc_protocol::Message::Storable(storable) => {
                                    ipc_tx.send(storable).await?;
                                }
                                veecle_ipc_protocol::Message::Telemetry(message) => {
                                    if let Some(ref exporter) = exporter {
                                        exporter.export(message);
                                    }
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

impl RuntimeInstance {
    /// Returns a new `RuntimeInstance` instance.
    fn new(
        id: InstanceId,
        socket_dir: &Utf8Path,
        binary: BinarySource,
        ipc_tx: mpsc::Sender<EncodedStorable>,
        ipc_rx: mpsc::Receiver<EncodedStorable>,
        exporter: Option<Arc<Exporter>>,
    ) -> eyre::Result<Self> {
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
            .ok_or_else(|| eyre::eyre!("non-UTF-8 socket path"))?
            .to_owned();

        let ipc_shutdown = CancellationToken::new();
        let ipc_task = tokio::spawn(handle_instance_ipc(
            id,
            socket,
            ipc_tx,
            ipc_rx,
            ipc_shutdown.clone(),
            exporter,
        ));

        Ok(Self {
            binary,
            process: None,
            ipc_task: Some(ipc_task),
            ipc_shutdown,
            socket_path,
        })
    }
}

/// Operations sent to the actor.
#[derive(Debug)]
enum Command {
    AddInstance {
        id: InstanceId,
        binary: BinarySource,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    RemoveInstance {
        id: InstanceId,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    StartInstance {
        id: InstanceId,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    StopInstance {
        id: InstanceId,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    GetInfo {
        response_tx: oneshot::Sender<BTreeMap<InstanceId, RuntimeInfo>>,
    },

    Shutdown {
        response_tx: oneshot::Sender<()>,
    },

    Clear {
        response_tx: oneshot::Sender<()>,
    },
}

/// The `Conductor` manages a set of [`RuntimeInstance`]s.
pub(crate) struct Conductor {
    command_tx: mpsc::Sender<Command>,
    _task: tokio::task::JoinHandle<eyre::Result<()>>,
}

impl std::fmt::Debug for Conductor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conductor")
            .field("task", &self._task)
            .finish()
    }
}

impl Conductor {
    /// Returns a new `Conductor` instance.
    pub(crate) fn new(
        distributor: Arc<Distributor>,
        exporter: Option<Arc<Exporter>>,
    ) -> eyre::Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(crate::ARBITRARY_CHANNEL_BUFFER);

        let _task =
            tokio::task::spawn(
                async move { Inner::new(distributor, exporter)?.run(command_rx).await },
            );

        Ok(Self { command_tx, _task })
    }

    /// Adds a new runtime instance with the specified binary source.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn add(&self, id: InstanceId, binary: BinarySource) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::AddInstance {
                id,
                binary,
                response_tx,
            })
            .await?;

        response_rx.await?
    }

    /// Removes the runtime instance with the passed id.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn remove(&self, id: InstanceId) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::RemoveInstance { id, response_tx })
            .await?;

        response_rx.await?
    }

    /// Starts the runtime instance with the passed id.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn start(&self, id: InstanceId) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::StartInstance { id, response_tx })
            .await?;

        response_rx.await?
    }

    /// Stops the runtime instance with the passed id.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn stop(&self, id: InstanceId) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::StopInstance { id, response_tx })
            .await?;

        response_rx.await?
    }

    /// Returns info about the current state.
    pub(crate) async fn info(&self) -> eyre::Result<BTreeMap<InstanceId, RuntimeInfo>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::GetInfo { response_tx })
            .await?;

        response_rx.await.map_err(Into::into)
    }

    /// Stops all runtime instances.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn shutdown(&self) {
        let (response_tx, response_rx) = oneshot::channel();

        let _ = self
            .command_tx
            .send(Command::Shutdown { response_tx })
            .await;

        let _ = response_rx.await;
    }

    /// Stops and removes all runtime instances.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn clear(&self) {
        let (response_tx, response_rx) = oneshot::channel();

        let _ = self.command_tx.send(Command::Clear { response_tx }).await;

        let _ = response_rx.await;
    }
}

/// The actual [`Conductor`] state machine, running in a background task and accepting commands over channels from its
/// façade.
struct Inner {
    ipc_socket_dir: TempDir,
    runtimes: HashMap<InstanceId, RuntimeInstance>,
    distributor: Arc<Distributor>,
    exporter: Option<Arc<Exporter>>,
}

impl Inner {
    fn new(distributor: Arc<Distributor>, exporter: Option<Arc<Exporter>>) -> eyre::Result<Self> {
        let ipc_socket_dir = tempfile::TempDir::with_prefix("veecle-orchestrator-ipc-sockets")?;
        let _ = Utf8Path::from_path(ipc_socket_dir.path())
            .ok_or_else(|| eyre::eyre!("non utf8 tempdir"))?
            .to_owned();
        Ok(Self {
            ipc_socket_dir,
            runtimes: HashMap::new(),
            distributor,
            exporter,
        })
    }

    fn ipc_socket_dir_utf8(&self) -> &Utf8Path {
        Utf8Path::from_path(self.ipc_socket_dir.path()).expect("checked in constructor")
    }

    #[tracing::instrument(skip(self))]
    async fn add_instance(&mut self, id: InstanceId, binary: BinarySource) -> eyre::Result<()> {
        if self.runtimes.contains_key(&id) {
            eyre::bail!("instance id {id} already registered");
        }

        let ipc_tx = self.distributor.sender();
        let ipc_rx = self.distributor.channel(id).await?;
        let socket_dir = self.ipc_socket_dir_utf8();
        let exporter = self.exporter.clone();

        let instance = RuntimeInstance::new(id, socket_dir, binary, ipc_tx, ipc_rx, exporter)?;

        self.runtimes.insert(id, instance);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn remove_instance(&mut self, id: InstanceId) -> eyre::Result<()> {
        let Entry::Occupied(entry) = self.runtimes.entry(id) else {
            eyre::bail!("instance id {id} was not registered");
        };

        if entry.get().process.is_some() {
            eyre::bail!("instance id {id} is still running, you must stop it before removing");
        }

        entry.remove();

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn start_instance(&mut self, id: InstanceId) -> eyre::Result<()> {
        let Some(instance) = self.runtimes.get_mut(&id) else {
            eyre::bail!("instance id {id} was not registered");
        };

        if instance.process.is_some() {
            eyre::bail!("instance id {id} is already running");
        }

        let binary = instance.binary.path();
        let process = tokio::process::Command::new(binary)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env("VEECLE_IPC_SOCKET", &instance.socket_path)
            .spawn()
            .wrap_err_with(|| format!("starting runtime process '{binary}'"))?;

        instance.process = Some(process);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn stop_instance(&mut self, id: InstanceId) -> eyre::Result<()> {
        let Some(instance) = self.runtimes.get_mut(&id) else {
            eyre::bail!("instance id {id} was not registered");
        };

        let Some(process) = instance.process.take() else {
            eyre::bail!("instance id {id} is not running");
        };

        let status = kill_child(process).await?;

        tracing::info!("child stop exit status {status:?}");

        Ok(())
    }

    fn get_info(&self) -> BTreeMap<InstanceId, RuntimeInfo> {
        self.runtimes
            .iter()
            .map(|(&id, instance)| {
                (
                    id,
                    RuntimeInfo {
                        running: instance.process.is_some(),
                        binary: instance.binary.path().to_path_buf(),
                    },
                )
            })
            .collect()
    }

    #[tracing::instrument(skip(self))]
    async fn shutdown(&mut self) {
        let processes = Vec::from_iter(
            self.runtimes
                .iter_mut()
                .filter_map(|(&id, runtime)| runtime.process.take().map(|process| (id, process))),
        );

        futures::stream::iter(processes)
            .for_each_concurrent(None, async |(id, process)| {
                let status = kill_child(process).await;
                tracing::info!("child {id} exit status {status:?}");
            })
            .await;
    }

    #[tracing::instrument(skip(self))]
    async fn clear(&mut self) {
        self.shutdown().await;

        let ipc_tasks = Vec::from_iter(self.runtimes.drain().filter_map(|(id, mut runtime)| {
            runtime.ipc_shutdown.cancel();
            runtime.ipc_task.take().map(|task| (id, task))
        }));

        for (id, task) in ipc_tasks {
            match task.await {
                Ok(Ok(())) => tracing::debug!("IPC task {id} completed successfully"),
                Ok(Err(error)) => tracing::warn!("IPC task {id} failed: {error}"),
                Err(error) => tracing::warn!("IPC task {id} join failed: {error}"),
            }
        }
    }

    /// Applies the given command to this state.
    async fn apply_command(&mut self, command: Command) {
        match command {
            Command::AddInstance {
                id,
                binary,
                response_tx,
            } => {
                let response = self.add_instance(id, binary).await;
                let _ = response_tx.send(response);
            }
            Command::RemoveInstance { id, response_tx } => {
                let response = self.remove_instance(id);
                let _ = response_tx.send(response);
            }
            Command::StartInstance { id, response_tx } => {
                let response = self.start_instance(id);
                let _ = response_tx.send(response);
            }
            Command::StopInstance { id, response_tx } => {
                let response = self.stop_instance(id).await;
                let _ = response_tx.send(response);
            }
            Command::GetInfo { response_tx } => {
                let _ = response_tx.send(self.get_info());
            }
            Command::Shutdown { response_tx } => {
                self.shutdown().await;
                let _ = response_tx.send(());
            }
            Command::Clear { response_tx } => {
                self.clear().await;
                let _ = response_tx.send(());
            }
        }
    }

    /// Runs a loop applying all received commands to this state.
    async fn run(&mut self, mut command_rx: mpsc::Receiver<Command>) -> eyre::Result<()> {
        while let Some(command) = command_rx.recv().await {
            self.apply_command(command).await;
        }

        Ok(())
    }
}

/// Attempts to nicely kill a child process, first attempting to interrupt it and give it 100ms to shutdown before
/// killing it.
async fn kill_child(mut process: Child) -> eyre::Result<ExitStatus> {
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
