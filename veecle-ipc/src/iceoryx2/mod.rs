//! TODO

mod actors;
mod telemetry;

use core::fmt::Debug;
use std::str::FromStr;

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::service::ipc;
use iceoryx2::service::port_factory::publish_subscribe;
use iceoryx2::service::service_name::ServiceName;
use veecle_ipc_protocol::Uuid;

pub use actors::{ControlHandler, Input, Output};
pub use telemetry::Exporter;

/// TODO
#[derive(Debug)]
pub struct Connector {
    node: Node<ipc::Service>,
    runtime_id: Uuid,
}

impl Connector {
    /// TODO
    pub async fn connect() -> Self {
        let node = NodeBuilder::new().create::<ipc::Service>().unwrap();
        let runtime_id = std::env::var("VEECLE_RUNTIME_ID").unwrap();
        let runtime_id = Uuid::from_str(&runtime_id).unwrap();

        Self { node, runtime_id }
    }

    pub(crate) fn storable<T: Debug + ZeroCopySend>(
        &self,
        type_name: &'static str,
    ) -> publish_subscribe::PortFactory<ipc::Service, T, ()> {
        self.node
            .service_builder(
                &ServiceName::new(&format!("veecle/ipc/storable/{type_name}")).unwrap(),
            )
            .publish_subscribe::<T>()
            .open_or_create()
            .unwrap()
    }

    /// TODO
    pub fn exporter(&self) -> Exporter {
        Exporter::new(&self.node, self.runtime_id)
    }

    /// Returns this runtime's instance id.
    ///
    /// This id uniquely identifies this runtime instance within the orchestrator.
    pub fn runtime_id(&self) -> Uuid {
        self.runtime_id
    }

    pub(crate) fn node(&self) -> &Node<ipc::Service> {
        &self.node
    }
}
