//! std example for [veecle_os_examples_common::actors::ping_pong].
use veecle_os_examples_common::actors::ping_pong::{Ping, PingActor, Pong, PongActor, TraceActor};

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        store: [Ping, Pong],
        actors: [
            PingActor,
            PongActor,
            TraceActor,
        ],
    }
    .await;
}
