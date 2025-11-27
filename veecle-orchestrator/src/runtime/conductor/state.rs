use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use camino::Utf8Path;
use eyre::{OptionExt, Result, bail};
use futures::stream::StreamExt;
use tempfile::TempDir;
use tokio::sync::mpsc;
use veecle_orchestrator_protocol::{InstanceId, RuntimeInfo};

use crate::distributor::Distributor;
use crate::runtime::conductor::Command;
use crate::telemetry::Exporter;

use crate::runtime::{BinarySource, RuntimeInstance};

/// The actual state machine for managing runtime instances, running in a background task and accepting commands over channels from its
/// façade ([`super::Conductor`]).
pub(super) struct State {
    ipc_socket_dir: TempDir,
    runtimes: HashMap<InstanceId, RuntimeInstance>,
    distributor: Arc<Distributor>,
    exporter: Option<Arc<Exporter>>,
}

impl State {
    pub(super) fn new(
        distributor: Arc<Distributor>,
        exporter: Option<Arc<Exporter>>,
    ) -> Result<Self> {
        let ipc_socket_dir = tempfile::TempDir::with_prefix("veecle-orchestrator-ipc-sockets")?;
        let _ = Utf8Path::from_path(ipc_socket_dir.path())
            .ok_or_eyre("non utf8 tempdir")?
            .to_owned();
        Ok(Self {
            ipc_socket_dir,
            runtimes: HashMap::new(),
            distributor,
            exporter,
        })
    }

    pub(super) fn ipc_socket_dir_utf8(&self) -> &Utf8Path {
        Utf8Path::from_path(self.ipc_socket_dir.path()).expect("checked in constructor")
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn add_instance(
        &mut self,
        id: InstanceId,
        binary: BinarySource,
        privileged: bool,
        command_tx: mpsc::Sender<Command>,
    ) -> Result<()> {
        if self.runtimes.contains_key(&id) {
            bail!("instance id {id} already registered");
        }

        let ipc_tx = self.distributor.sender();
        let ipc_rx = self.distributor.channel(id).await?;
        let socket_dir = self.ipc_socket_dir_utf8();

        let instance = RuntimeInstance::new(
            id,
            socket_dir,
            binary,
            ipc_tx,
            ipc_rx,
            self.exporter.clone(),
            privileged,
            command_tx,
        )?;

        self.runtimes.insert(id, instance);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn remove_instance(&mut self, id: InstanceId) -> Result<()> {
        let Entry::Occupied(entry) = self.runtimes.entry(id) else {
            bail!("instance id {id} was not registered");
        };

        if entry.get().is_running() {
            bail!("instance id {id} is still running, you must stop it before removing");
        }

        entry.remove().cleanup().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(super) fn start_instance(&mut self, id: InstanceId) -> Result<()> {
        let Some(instance) = self.runtimes.get_mut(&id) else {
            bail!("instance id {id} was not registered");
        };

        instance.start()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn stop_instance(&mut self, id: InstanceId) -> Result<()> {
        let Some(instance) = self.runtimes.get_mut(&id) else {
            bail!("instance id {id} was not registered");
        };

        instance.stop().await?;

        Ok(())
    }

    pub(super) fn get_info(&self) -> BTreeMap<InstanceId, RuntimeInfo> {
        self.runtimes
            .iter()
            .map(|(&id, instance)| {
                (
                    id,
                    RuntimeInfo {
                        running: instance.is_running(),
                        binary: instance.binary().path().to_path_buf(),
                        privileged: instance.privileged(),
                    },
                )
            })
            .collect()
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn shutdown(&mut self) {
        futures::stream::iter(self.runtimes.iter_mut())
            .for_each_concurrent(None, async |(id, runtime)| {
                #[allow(
                    clippy::collapsible_if,
                    reason = "separate condition check from error handling"
                )]
                if runtime.is_running() {
                    if let Err(error) = runtime.stop().await {
                        tracing::info!("child {id} failed to stop: {error:?}");
                    }
                }
            })
            .await;
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn clear(&mut self) {
        futures::stream::iter(self.runtimes.drain())
            .for_each_concurrent(None, async |(id, runtime)| match runtime.cleanup().await {
                Ok(()) => tracing::debug!("instance {id} cleaned up"),
                Err(error) => tracing::warn!("instance {id} failed to cleanup: {error:?}"),
            })
            .await;
    }
}
