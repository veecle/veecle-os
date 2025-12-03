#![recursion_limit = "256"]

use veecle_os::osal::std::time::Time;
use veecle_os_examples_common::actors::time::Tick as Tock;
use veecle_os_examples_common::actors::time::{Tick, TickerActor, TickerReader};

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        store: [Tick,Tock,
            // Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,
            // Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,Tick,
        ],
        actors: [
            TickerReader,
            TickerActor<Time>,
        ],
    }
    .await;
}
