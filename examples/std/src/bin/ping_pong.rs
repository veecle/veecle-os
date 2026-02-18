//! std example for [examples_common::actors::ping_pong].
use examples_common::actors::ping_pong::{PingActor, PongActor, TraceActor};

#[veecle_os::osal::std::main(telemetry = true)]
async fn main() {
    veecle_os::runtime::execute! {
        actors: [
            PingActor,
            PongActor,
            TraceActor,
        ],
    }
    .await;
}
