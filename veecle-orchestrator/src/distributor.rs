use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::net::SocketAddr;

use tokio::sync::{mpsc, oneshot};
use veecle_ipc_protocol::EncodedStorable;
use veecle_orchestrator_protocol::{InstanceId, LinkTarget};

/// Operations sent to the actor.
#[derive(Debug)]
enum Command {
    AddInstance {
        id: InstanceId,
        response_tx: oneshot::Sender<eyre::Result<mpsc::Receiver<EncodedStorable>>>,
    },

    AddLink {
        type_name: String,
        target: LinkTarget,
        response_tx: oneshot::Sender<eyre::Result<()>>,
    },

    GetInfo {
        response_tx: oneshot::Sender<BTreeMap<String, Vec<LinkTarget>>>,
    },
}

/// Handles routing `EncodedStorable` messages between different instances based on the configured links.
pub struct Distributor {
    input_tx: mpsc::Sender<EncodedStorable>,
    command_tx: mpsc::Sender<Command>,
    _task: tokio::task::JoinHandle<eyre::Result<()>>,
}

impl std::fmt::Debug for Distributor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Distributor")
            .field("task", &self._task)
            .finish()
    }
}

impl Distributor {
    /// Creates a new `Distributor` with no predefined links.
    pub fn new(external_output_tx: mpsc::Sender<(SocketAddr, EncodedStorable)>) -> Self {
        let (input_tx, input_rx) =
            mpsc::channel::<EncodedStorable>(crate::ARBITRARY_CHANNEL_BUFFER);
        let (command_tx, command_rx) = mpsc::channel(crate::ARBITRARY_CHANNEL_BUFFER);

        // This is using an actor model, a single task owns the configuration and receives both the messages to
        // route and updates to the configuration.
        let _task = tokio::task::spawn(async move {
            Inner::new(input_rx, command_rx, external_output_tx)
                .run()
                .await
        });

        Self {
            input_tx,
            command_tx,
            _task,
        }
    }

    /// Returns a sender that can be used to distribute a new incoming message from an instance.
    pub fn sender(&self) -> mpsc::Sender<EncodedStorable> {
        self.input_tx.clone()
    }

    /// Registers a new known runtime instance and returns a channel that will receive any messages routed to it.
    pub async fn channel(&self, id: InstanceId) -> eyre::Result<mpsc::Receiver<EncodedStorable>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::AddInstance { id, response_tx })
            .await?;

        let rx = response_rx.await??;

        Ok(rx)
    }

    /// Adds a link to instance `target` for any IPC messages tagged with `type_name`.
    pub async fn link(&self, type_name: String, target: LinkTarget) -> eyre::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::AddLink {
                type_name,
                target,
                response_tx,
            })
            .await?;

        response_rx.await??;

        Ok(())
    }

    /// Returns info about the current state.
    pub async fn info(&self) -> eyre::Result<BTreeMap<String, Vec<LinkTarget>>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(Command::GetInfo { response_tx })
            .await?;

        let info = response_rx.await?;

        Ok(info)
    }
}

/// The actual [`Distributor`] state machine, running in a background task and accepting commands over channels from its
/// façade.
struct Inner {
    /// Input messages from both local and remote instances.
    input_rx: mpsc::Receiver<EncodedStorable>,

    /// Input commands to reconfigure the links.
    command_rx: mpsc::Receiver<Command>,

    /// Output messages to any remote instance.
    external_output_tx: mpsc::Sender<(SocketAddr, EncodedStorable)>,

    /// The links, for a specific data type, to a list of target instances.
    links: BTreeMap<String, Vec<LinkTarget>>,

    /// How to actually send a message to the chosen target instances.
    instance_txs: BTreeMap<InstanceId, mpsc::Sender<EncodedStorable>>,
}

impl Inner {
    fn new(
        input_rx: mpsc::Receiver<EncodedStorable>,
        command_rx: mpsc::Receiver<Command>,
        external_output_tx: mpsc::Sender<(SocketAddr, EncodedStorable)>,
    ) -> Self {
        Self {
            input_rx,
            command_rx,
            external_output_tx,
            links: BTreeMap::new(),
            instance_txs: BTreeMap::new(),
        }
    }

    async fn route_message(&mut self, storable: EncodedStorable) -> eyre::Result<()> {
        let type_name = &storable.type_name;
        let Some(targets) = self.links.get(&**type_name) else {
            tracing::warn!(%type_name, "no registered ipc link");
            return Ok(());
        };

        for target in targets {
            match target {
                LinkTarget::Local(id) => {
                    let Some(sender) = self.instance_txs.get(id) else {
                        tracing::warn!(%type_name, %id, "no instance");
                        continue;
                    };
                    sender.send(storable.clone()).await?;
                }
                &LinkTarget::Remote(address) => {
                    self.external_output_tx
                        .send((address, storable.clone()))
                        .await?;
                }
            }
        }

        Ok(())
    }

    fn add_instance(&mut self, id: InstanceId) -> eyre::Result<mpsc::Receiver<EncodedStorable>> {
        let Entry::Vacant(entry) = self.instance_txs.entry(id) else {
            eyre::bail!("instance id {id} already registered");
        };

        let (tx, rx) = mpsc::channel(crate::ARBITRARY_CHANNEL_BUFFER);
        entry.insert(tx);
        Ok(rx)
    }

    fn add_link(&mut self, type_name: String, target: LinkTarget) -> eyre::Result<()> {
        if let LinkTarget::Local(id) = &target {
            eyre::ensure!(
                self.instance_txs.contains_key(id),
                "instance id {target} was not registered"
            );
        }

        self.links.entry(type_name).or_default().push(target);

        Ok(())
    }

    fn apply_command(&mut self, command: Command) {
        match command {
            Command::AddInstance { id, response_tx } => {
                let response = self.add_instance(id);
                let _ = response_tx.send(response);
            }
            Command::AddLink {
                type_name,
                target,
                response_tx,
            } => {
                let response = self.add_link(type_name, target);
                let _ = response_tx.send(response);
            }
            Command::GetInfo { response_tx } => {
                let _ = response_tx.send(self.links.clone());
            }
        }
    }

    async fn run(&mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
                data = self.input_rx.recv() => {
                    let Some(storable) = data else { break };
                    self.route_message(storable).await?;
                }

                command = self.command_rx.recv() => {
                    let Some(command) = command else { break };
                    self.apply_command(command);
                }
            }
        }

        Ok(())
    }
}
