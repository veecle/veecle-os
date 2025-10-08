use veecle_os_examples_common::actors::ping_pong::{Ping, Pong, PongActor};

#[veecle_os::osal::std::main]
async fn main() {
    let connector = veecle_ipc::Connector::connect().await;

    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::collector::ProcessId::random(&mut rand::rng()),
        Box::leak(Box::new(connector.exporter())),
    )
    .unwrap();

    veecle_os::runtime::execute! {
        store: [Ping, Pong],
        actors: [
            PongActor,
            veecle_ipc::Input::<Ping>: &connector,
            veecle_ipc::Output::<Pong>: &connector,
        ],
    }
    .await;
}
