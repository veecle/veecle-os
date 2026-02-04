use std::fmt::Debug;

use serde::Serialize;
use veecle_os_runtime::{CombineReaders, InitializedReader, Never, Reader, Storable, Writer};

#[derive(Debug, Default, Storable, Serialize)]
pub struct Ping {
    pub value: u32,
}

#[derive(Debug, Default, Storable, Serialize)]
pub struct Pong {
    value: u32,
}

#[veecle_telemetry::instrument]
pub async fn ping_loop(ping: &mut Writer<'_, Ping>, pong: &mut Reader<'_, Pong>, value: &mut u32) {
    veecle_telemetry::info!(
        "Sending ping",
        ping = format!("{:?}", Ping { value: *value })
    );
    ping.write(Ping { value: *value }).await;
    *value += 1;

    veecle_telemetry::info!("Waiting for pong");
    pong.read_updated(|pong| {
        veecle_telemetry::info!("Pong received", pong = format!("{:?}", pong));
        assert_eq!(pong.value, *value);
    })
    .await;
}

/// An actor that reads `Ping`, replies with `Pong { ping + 1 }` and waits for the next `Ping`.
#[veecle_os_runtime::actor]
pub async fn pong_actor(
    mut pong: Writer<'_, Pong>,
    mut ping: InitializedReader<'_, Ping>,
) -> Never {
    loop {
        pong_loop(&mut pong, &mut ping).await
    }
}

#[veecle_telemetry::instrument]
async fn pong_loop(pong: &mut Writer<'_, Pong>, ping: &mut InitializedReader<'_, Ping>) {
    veecle_telemetry::info!("Waiting for ping");
    let value = ping.wait_for_update().await.read(|ping| {
        veecle_telemetry::info!("Ping received", ping = format!("{:?}", ping));
        ping.value + 1
    });

    let data = Pong { value };
    veecle_telemetry::info!("Sending pong", pong = format!("{:?}", data));
    pong.write(data).await;
}

/// An actor that continuously reads and logs updates from all debuggable types using a pair of concrete readers.
#[veecle_os_runtime::actor]
pub async fn concrete_trace_actor(mut ping: Reader<'_, Ping>, mut pong: Reader<'_, Pong>) -> Never {
    loop {
        concrete_trace_loop(&mut ping, &mut pong).await;
    }
}

#[veecle_telemetry::instrument]
async fn concrete_trace_loop(ping: &mut Reader<'_, Ping>, pong: &mut Reader<'_, Pong>) {
    let mut pair = (ping, pong);
    veecle_telemetry::info!("Waiting for pong");
    pair.wait_for_update().await.read(|(ping, pong)| {
        veecle_telemetry::error!(
            "Concrete Trace",
            ping = format!("{:?}", ping),
            pong = format!("{:?}", pong)
        );
    });
}
