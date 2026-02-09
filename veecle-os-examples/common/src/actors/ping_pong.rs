//! Ping-pong example actors.

use core::fmt::Debug;
use futures::future::FutureExt;
use serde::{Deserialize, Serialize};
use veecle_os::runtime::{Never, Reader, Storable, Writer};
use veecle_os::telemetry::{error, info};

#[derive(Debug, Clone, PartialEq, Eq, Default, Storable, Deserialize, Serialize)]
pub struct Ping {
    value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Storable, Deserialize, Serialize)]
pub struct Pong {
    value: u32,
}

/// An actor that writes `Ping { i }` and waits for `Pong`.
/// Additionally, it validates that `Pong { value == i + 1 }` for `i = 0..`.
#[veecle_os::runtime::actor]
pub async fn ping_actor(mut ping: Writer<'_, Ping>, mut pong: Reader<'_, Pong>) -> Never {
    let mut value = 0;
    info!("[PING TASK] Sending initial", ping = i64::from(value));
    ping.write(Ping { value }).await;
    value += 1;

    loop {
        info!("[PING TASK] Waiting for pong");
        pong.read_updated(|pong| {
            info!("[PING TASK] Pong received", pong = i64::from(pong.value));
            assert_eq!(pong.value, value);
        })
        .await;
        info!("[PING TASK] Sending", ping = i64::from(value));
        ping.write(Ping { value }).await;
        value += 1;
    }
}

/// An actor that reads `Ping`, replies with `Pong { ping + 1 }` and waits for the next `Ping`.
#[veecle_os::runtime::actor]
pub async fn pong_actor(mut pong: Writer<'_, Pong>, mut ping: Reader<'_, Ping>) -> Never {
    loop {
        info!("[PONG TASK] Waiting for ping");

        let value = ping
            .read_updated(|ping| {
                info!("[PONG TASK] Ping received", ping = i64::from(ping.value));
                ping.value + 1
            })
            .await;

        let data = Pong { value };
        info!("[PONG TASK] Sending", pong = i64::from(data.value));
        pong.write(data).await;
    }
}

/// An actor that continuously reads and logs updates from ping and pong.
#[veecle_os::runtime::actor]
pub async fn trace_actor(
    mut ping_reader: Reader<'_, Ping>,
    mut pong_reader: Reader<'_, Pong>,
) -> Never {
    loop {
        futures::select_biased! {
            _ = ping_reader.wait_for_update().fuse() => {
                ping_reader.read(|ping| {
                    if let Some(ping) = ping {
                        let _ = ping;
                        error!("[TRACE TASK]", ping = i64::from(ping.value));
                    }
                });
            }
            _ = pong_reader.wait_for_update().fuse() => {
                pong_reader.read(|pong| {
                    if let Some(pong) = pong {
                        let _ = pong;
                        error!("[TRACE TASK]", pong = i64::from(pong.value));
                    }
                });
            }
        }
    }
}
