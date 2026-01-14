use embassy_executor::Spawner;
use veecle_os::osal::embassy::time::Time;
use veecle_os_examples_common::actors::time::{TickerActor, TickerReader};

#[embassy_executor::task]
async fn run() {
    veecle_os::runtime::execute! {
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
        .random_process_id()
        .console_json_exporter()
        .time::<Time>()
        .thread::<veecle_os::osal::std::thread::Thread>()
        .set_global()
        .unwrap();

    spawner.spawn(run()).unwrap();
}
