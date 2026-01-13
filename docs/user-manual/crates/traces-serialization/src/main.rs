// ANCHOR: full
use core::convert::Infallible;
use core::fmt::Debug;

use veecle_os::runtime::{InitializedReader, Reader, Storable, Writer};

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
async fn ping_actor(mut ping: Writer<'_, Ping>, pong: Reader<'_, Pong>) -> Infallible {
    let mut value = 0;

    ping.write(value).await;
    value += 1;

    let mut pong = pong.wait_init().await;
    loop {
        pong.wait_for_update().await.read(|pong| {
            assert_eq!(pong, &value);
            if *pong == 5 {
                // Exit the application to allow doc-tests to complete.
                std::process::exit(0);
            }
        });

        ping.write(value).await;
        value += 1;
    }
}

#[veecle_os::runtime::actor]
async fn pong_actor(
    mut pong: Writer<'_, Pong>,
    mut ping: InitializedReader<'_, Ping>,
) -> Infallible {
    loop {
        let value = ping.wait_for_update().await.read(|ping| ping + 1);

        pong.write(value).await;
    }
}
// ANCHOR: setup
#[veecle_os::osal::std::main(telemetry = true)]
// ANCHOR_END: setup
async fn main() {
    veecle_os::runtime::execute! {
        store: [Ping, Pong],
        actors: [PingActor, PongActor],
    }
    .await;
}
// ANCHOR_END: full
