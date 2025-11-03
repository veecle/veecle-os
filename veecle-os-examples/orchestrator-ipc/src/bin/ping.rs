use veecle_os_examples_common::actors::ping_pong::{Ping, PingActor, Pong};

#[veecle_os::osal::std::main]
async fn main() {
    let connector = veecle_ipc::Connector::connect().await;

    veecle_os::telemetry::collector::set_exporter(
        veecle_os::telemetry::protocol::ExecutionId::random(&mut rand::rng()),
        Box::leak(Box::new(connector.exporter())),
    )
    .unwrap();

    veecle_os::runtime::execute! {
        store: [Ping, Pong],
        actors: [
            PingActor,
            veecle_ipc::Output::<Ping>: (&connector).into(),
            veecle_ipc::Input::<Pong>: &connector,
        ],
    }
    .await;
}
