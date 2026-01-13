#![allow(missing_docs)]
//! Run via:
//! ```
//! # run example
//! cargo run --package veecle-telemetry-ui --example remote > spans.jsonl
//! # open veecle-telemetry-ui
//! cargo run --package veecle-telemetry-ui -- ./spans.jsonl
//! ```

use veecle_os_runtime::{Never, Reader, Writer};

use crate::common::{ConcreteTraceActor, Ping, Pong, PongActor, ping_loop};

mod common;

/// An actor that writes `Ping { i }` and waits for `Pong`.
/// Additionally, it validates that `Pong { value == i + 1 }` for `i = 0..`.
#[veecle_os_runtime::actor]
pub async fn ping_actor(mut ping: Writer<'_, Ping>, mut pong: Reader<'_, Pong>) -> Never {
    let mut value = 0;
    loop {
        ping_loop(&mut ping, &mut pong, &mut value).await;

        // Just to make execution stop in this example.
        if value > 2 {
            std::process::exit(0);
        }
    }
}

#[veecle_osal_std::main]
async fn main() {
    veecle_telemetry::collector::build()
        .random_process_id()
        .console_json_exporter()
        .time::<veecle_osal_std::time::Time>()
        .thread::<veecle_osal_std::thread::Thread>()
        .set_global()
        .expect("exporter was not set yet");

    veecle_os_runtime::execute! {
        store: [Ping, Pong],
        actors: [PingActor, PongActor, ConcreteTraceActor],
    }
    .await;
}
