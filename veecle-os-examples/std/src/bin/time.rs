use veecle_os::osal::std::time::Time;
use veecle_os_examples_common::actors::time::{TickerActor, TickerReader};

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        actors: [
            TickerReader,
            TickerActor<Time>,
        ],
    }
    .await;
}
