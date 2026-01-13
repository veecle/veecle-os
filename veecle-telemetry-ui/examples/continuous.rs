#![allow(missing_docs)]
//! Run via:
//! ```
//! # run example and pipe it into veecle-telemetry-ui
//! cargo run --package veecle-telemetry-ui --example continuous | cargo run --package veecle-telemetry-ui
//! ```

use veecle_os_runtime::{Never, Reader, Writer};
use veecle_osal_std::time::{Duration, Time, TimeAbstraction};

use crate::common::{ConcreteTraceActor, Ping, Pong, PongActor, ping_loop};

mod common;

/// An actor that writes `Ping { i }` and waits for `Pong`.
/// Additionally, it validates that `Pong { value == i + 1 }` for `i = 0..`.
#[veecle_os_runtime::actor]
async fn ping_actor(
    mut ping: Writer<'_, Ping>,
    mut pong: Reader<'_, Pong>,
) -> Result<Never, veecle_osal_std::Error> {
    let mut value = 0;

    loop {
        ping_loop(&mut ping, &mut pong, &mut value).await;

        Time::sleep(Duration::from_millis(100)).await?;
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
