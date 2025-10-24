//! An example of how to use the orchestrator control messages.
//!
//! This implements a [useless machine](https://en.wikipedia.org/wiki/Useless_machine) in the form
//! of a runtime process that just shuts itself down.
use core::convert::Infallible;

use veecle_ipc::{ControlRequest, ControlResponse, Uuid};
use veecle_os::osal::std::time::{Duration, Time, TimeAbstraction};
use veecle_os::runtime::{Reader, Writer};

#[veecle_os::runtime::actor]
async fn useless_machine_actor(
    #[init_context] id: Uuid,
    mut request: Writer<'_, ControlRequest>,
    response: Reader<'_, ControlResponse>,
) -> Infallible {
    veecle_os::telemetry::info!(
        "useless machine starting, will shut down soon",
        id = id.to_string()
    );

    Time::sleep(Duration::from_secs(2)).await.unwrap();

    veecle_os::telemetry::info!("sending stop request", id = id.to_string());

    request.write(ControlRequest::StopRuntime { id }).await;

    let response = response.wait_init().await;
    let response = response.read_cloned();

    veecle_os::telemetry::error!(
        "runtime still executing after stop request—either lacks privileges or an error occurred",
        response = format!("{response:?}")
    );
    loop {
        Time::sleep(Duration::from_secs(2)).await.unwrap();
    }
}

/// A delayed [useless machine](https://en.wikipedia.org/wiki/Useless_machine) demonstrating usage
/// of the orchestrator privileged control messages.
#[veecle_os::osal::std::main]
async fn main() {
    let connector = veecle_ipc::jsonl::Connector::connect().await;

    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::protocol::ProcessId::random(&mut rand::rng()),
        Box::leak(Box::new(connector.exporter())),
    )
    .unwrap();

    veecle_os::runtime::execute! {
        store: [ControlRequest, ControlResponse],
        actors: [
            UselessMachineActor: connector.runtime_id(),
            veecle_ipc::jsonl::ControlHandler: &connector,
        ],
    }
    .await;
}
