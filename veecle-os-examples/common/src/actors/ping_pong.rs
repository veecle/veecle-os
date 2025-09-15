//! Ping-pong example actors.

use core::convert::Infallible;
use core::fmt::Debug;
use futures::future::FutureExt;
use serde::{Deserialize, Serialize};
use veecle_os::runtime::{InitializedReader, Reader, Storable, Writer};

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
pub async fn ping_actor(mut ping: Writer<'_, Ping>, pong: Reader<'_, Pong>) -> Infallible {
    let mut value = 0;
    #[cfg(feature = "telemetry")]
    veecle_os::telemetry::info!("[PING TASK] Sending initial", ping = i64::from(value));
    ping.write(Ping { value }).await;
    value += 1;

    #[cfg(feature = "telemetry")]
    veecle_os::telemetry::info!("[PING TASK] Waiting for pong");
    let mut pong = pong.wait_init().await;
    loop {
        #[cfg(feature = "telemetry")]
        veecle_os::telemetry::info!("[PING TASK] Waiting for pong");
        pong.wait_for_update().await.read(|pong| {
            #[cfg(feature = "telemetry")]
            veecle_os::telemetry::info!("[PING TASK] Pong received", pong = i64::from(pong.value));
            assert_eq!(pong.value, value);
        });
        #[cfg(feature = "telemetry")]
        veecle_os::telemetry::info!("[PING TASK] Sending", ping = i64::from(value));
        ping.write(Ping { value }).await;
        value += 1;
    }
}

/// An actor that reads `Ping`, replies with `Pong { ping + 1 }` and waits for the next `Ping`.
#[veecle_os::runtime::actor]
pub async fn pong_actor(
    mut pong: Writer<'_, Pong>,
    mut ping: InitializedReader<'_, Ping>,
) -> Infallible {
    loop {
        #[cfg(feature = "telemetry")]
        veecle_os::telemetry::info!("[PONG TASK] Waiting for ping");

        let value = ping.wait_for_update().await.read(|ping| {
            #[cfg(feature = "telemetry")]
            veecle_os::telemetry::info!("[PONG TASK] Ping received", ping = i64::from(ping.value));
            ping.value + 1
        });

        let data = Pong { value };
        #[cfg(feature = "telemetry")]
        veecle_os::telemetry::info!("[PONG TASK] Sending", pong = i64::from(data.value));
        pong.write(data).await;
    }
}

/// An actor that continuously reads and logs updates from ping and pong.
#[veecle_os::runtime::actor]
pub async fn trace_actor(
    mut ping_reader: Reader<'_, Ping>,
    mut pong_reader: Reader<'_, Pong>,
) -> Infallible {
    loop {
        futures::select_biased! {
            _ = ping_reader.wait_for_update().fuse() => {
                ping_reader.read(|ping| {
                    if let Some(ping) = ping {
                        let _ = ping;
                        #[cfg(feature = "telemetry")]
                        veecle_os::telemetry::error!("[TRACE TASK]", ping = i64::from(ping.value));
                    }
                });
            }
            _ = pong_reader.wait_for_update().fuse() => {
                pong_reader.read(|pong| {
                    if let Some(pong) = pong {
                        let _ = pong;
                        #[cfg(feature = "telemetry")]
                        veecle_os::telemetry::error!("[TRACE TASK]", pong = i64::from(pong.value));
                    }
                });
            }
        }
    }
}
