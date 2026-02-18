use examples_common::actors::time::{TickerActor, TickerReader};
use veecle_os::osal::std::time::Time;

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
