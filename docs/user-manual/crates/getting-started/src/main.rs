// ANCHOR: full
// ANCHOR: init
//! Getting started example.
use core::convert::Infallible;
use core::fmt::Debug;

use veecle_os::runtime::{InitializedReader, Storable, Writer};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Value;

impl Storable for Value {
    type DataType = u32;
}
// ANCHOR_END: init

// ANCHOR: sender
/// An actor that writes `Value { i++ }` continuously.
#[veecle_os::runtime::actor]
async fn sender_actor(mut writer: Writer<'_, Value>) -> Infallible {
    let mut value = 0;
    loop {
        println!("[sender] Sending {:?}", value);
        writer.write(value).await;
        value += 1;
    }
}
// ANCHOR_END: sender

// ANCHOR: receiver
/// An actor that reads `Value` and terminates when the value is 5.
#[veecle_os::runtime::actor]
async fn receiver_actor(mut reader: InitializedReader<'_, Value>) -> Infallible {
    loop {
        println!("[receiver] Waiting for value");
        reader.wait_for_update().await.read(|value| {
            println!("[receiver] Received: {value:?}");
            if *value == 5 {
                println!("[receiver] Exiting because value is 5");
                // Actors should never terminate. This program ends so it always generates the same short output that is
                // easy to verify.
                std::process::exit(0);
            }
        });
    }
}
// ANCHOR_END: receiver

// ANCHOR: main
#[veecle_os::osal::std::main]
async fn main() {
    veecle_os::runtime::execute! {
        store: [Value],
        actors: [ReceiverActor, SenderActor],
    }
    .await;
}
// ANCHOR_END: main
// ANCHOR_END: full
