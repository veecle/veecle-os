use veecle_os_examples_common::actors::ping_pong::{Ping, Pong, PongActor};

#[veecle_os::osal::std::main]
async fn main() {
    let connector = veecle_ipc::Connector::connect().await;

    veecle_os::telemetry::collector::build()
        .random_process_id()
        .leaked_exporter(connector.exporter())
        .system_time::<veecle_os::osal::std::time::Time>()
        .thread::<veecle_os::osal::std::thread::Thread>()
        .set_global()
        .unwrap();

    veecle_os::runtime::execute! {
        actors: [
            PongActor,
            veecle_ipc::Input::<Ping>: &connector,
            veecle_ipc::Output::<Pong>: (&connector).into(),
        ],
    }
    .await;
}
