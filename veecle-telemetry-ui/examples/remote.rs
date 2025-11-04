#![allow(missing_docs)]
//! Run via:
//! ```
//! # run example
//! cargo run --package veecle-telemetry-ui --example remote > spans.jsonl
//! # open veecle-telemetry-ui
//! cargo run --package veecle-telemetry-ui -- ./spans.jsonl
//! ```

use std::convert::Infallible;

use veecle_os_runtime::{Reader, Writer};
use veecle_telemetry::collector::{ConsoleJsonExporter, ProcessId};

use crate::common::{ConcreteTraceActor, Ping, Pong, PongActor, ping_loop};

mod common;

/// An actor that writes `Ping { i }` and waits for `Pong`.
/// Additionally, it validates that `Pong { value == i + 1 }` for `i = 0..`.
#[veecle_os_runtime::actor]
pub async fn ping_actor(mut ping: Writer<'_, Ping>, mut pong: Reader<'_, Pong>) -> Infallible {
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
    let process_id = ProcessId::random(&mut rand::rng());
    veecle_telemetry::collector::set_exporter(process_id, &ConsoleJsonExporter)
        .expect("exporter was not set yet");

    veecle_os_runtime::execute! {
        store: [Ping, Pong],
        actors: [PingActor, PongActor, ConcreteTraceActor],
    }
    .await;
}
