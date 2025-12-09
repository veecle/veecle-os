use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use veecle_orchestrator_protocol::{InstanceId, Priority, RuntimeInfo};

use crate::distributor::Distributor;
use crate::telemetry::Exporter;

use crate::runtime::BinarySource;
use crate::runtime::conductor::State;

/// Manages a set of [`crate::runtime::RuntimeInstance`]s.
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

/// Operations sent to the actor.
#[derive(Debug)]
pub(crate) enum Command {
    AddInstance {
        id: InstanceId,
        binary: BinarySource,
        privileged: bool,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    RemoveInstance {
        id: InstanceId,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    StartInstance {
        id: InstanceId,
        priority: Option<Priority>,
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

impl Conductor {
    /// Returns a new `Conductor` instance.
    pub(crate) fn new(
        distributor: Arc<Distributor>,
        exporter: Option<Arc<Exporter>>,
    ) -> eyre::Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(crate::ARBITRARY_CHANNEL_BUFFER);

        let command_tx_weak = command_tx.downgrade();
        let _task = tokio::task::spawn(async move {
            let state = State::new(distributor, exporter)?;
            run(state, command_rx, command_tx_weak).await
        });

        Ok(Self { command_tx, _task })
    }

    /// Adds a new runtime instance with the specified binary source.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn add(
        &self,
        id: InstanceId,
        binary: BinarySource,
        privileged: bool,
    ) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::AddInstance {
                id,
                binary,
                privileged,
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
    pub(crate) async fn start(
        &self,
        id: InstanceId,
        priority: Option<Priority>,
    ) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::StartInstance {
                id,
                priority,
                response_tx,
            })
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

/// Runs a loop applying all received commands to the state.
async fn run(
    mut state: State,
    mut command_rx: mpsc::Receiver<Command>,
    command_tx_weak: mpsc::WeakSender<Command>,
) -> eyre::Result<()> {
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::AddInstance {
                id,
                binary,
                privileged,
                response_tx,
            } => {
                let response = match command_tx_weak.upgrade() {
                    Some(command_tx) => {
                        state.add_instance(id, binary, privileged, command_tx).await
                    }
                    None => Err(eyre::eyre!("conductor has been dropped")),
                };
                let _ = response_tx.send(response);
            }
            Command::RemoveInstance { id, response_tx } => {
                let response = state.remove_instance(id).await;
                let _ = response_tx.send(response);
            }
            Command::StartInstance {
                id,
                priority,
                response_tx,
            } => {
                let response = state.start_instance(id, priority);
                let _ = response_tx.send(response);
            }
            Command::StopInstance { id, response_tx } => {
                let response = state.stop_instance(id).await;
                let _ = response_tx.send(response);
            }
            Command::GetInfo { response_tx } => {
                let _ = response_tx.send(state.get_info());
            }
            Command::Shutdown { response_tx } => {
                state.shutdown().await;
                let _ = response_tx.send(());
            }
            Command::Clear { response_tx } => {
                state.clear().await;
                let _ = response_tx.send(());
            }
        }
    }

    Ok(())
}
