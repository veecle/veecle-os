// ANCHOR: full
use core::fmt::Debug;

use veecle_os::runtime::single_writer::{Reader, Writer};
use veecle_os::runtime::{Never, Storable};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ping;

impl Storable for Ping {
    type DataType = u32;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pong;

impl Storable for Pong {
    type DataType = u32;
}

#[veecle_os::runtime::actor]
async fn ping_actor(mut ping: Writer<'_, Ping>, mut pong: Reader<'_, Pong>) -> Never {
    let mut value = 0;

    ping.write(value).await;
    value += 1;

    loop {
        pong.read_updated(|pong| {
            assert_eq!(pong, &value);
            if *pong == 5 {
                // Exit the application to allow doc-tests to complete.
                std::process::exit(0);
            }
        })
        .await;

        ping.write(value).await;
        value += 1;
    }
}

#[veecle_os::runtime::actor]
async fn pong_actor(mut pong: Writer<'_, Pong>, mut ping: Reader<'_, Ping>) -> Never {
    loop {
        let value = ping.read_updated(|ping| ping + 1).await;

        pong.write(value).await;
    }
}
// ANCHOR: setup
#[veecle_os::osal::std::main(telemetry = true)]
// ANCHOR_END: setup
async fn main() {
    veecle_os::runtime::execute! {
        actors: [PingActor, PongActor],
    }
    .await;
}
// ANCHOR_END: full
