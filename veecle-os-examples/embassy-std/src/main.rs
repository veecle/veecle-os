use embassy_executor::Spawner;
use veecle_os::osal::embassy::time::Time;
use veecle_os_examples_common::actors::time::{Tick, TickerActor, TickerReader};

#[embassy_executor::task]
async fn run() {
    veecle_os::runtime::execute! {
        store: [Tick],
        actors: [
            TickerActor<Time>,
            TickerReader,
        ],
    }
    .await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    veecle_os::telemetry::collector::build()
        .process_id(veecle_os::telemetry::collector::ProcessId::random(
            &mut rand::rng(),
        ))
        .exporter(&veecle_os::telemetry::collector::ConsoleJsonExporter::DEFAULT)
        .time::<Time>()
        .thread::<veecle_os::osal::std::thread::Thread>()
        .set_global()
        .unwrap();

    spawner.spawn(run()).unwrap();
}
