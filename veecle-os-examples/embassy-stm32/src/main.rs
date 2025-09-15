#![no_std]
#![no_main]

use core::convert::Infallible;
use core::num::Wrapping;
use embassy_executor::Spawner;
use veecle_os::osal::api::log::LogTarget;
use veecle_os::osal::api::time::{Duration, TimeAbstraction};
use veecle_os::runtime::{Reader, Storable, Writer};

use panic_halt as _;

#[derive(Debug, Storable)]
pub struct Value(pub usize);

/// An actor that continuously reads and logs updates from `Value`.
#[veecle_os::runtime::actor]
pub async fn print_actor<L: LogTarget>(mut reader: Reader<'_, Value>) -> Infallible {
    loop {
        reader.wait_for_update().await.read(|value| {
            if let Some(value) = value {
                L::println(format_args!("{value:?}"));
            }
        });
    }
}

/// An actor that continuously increments `Value`.
#[veecle_os::runtime::actor]
pub async fn increment_actor<T: TimeAbstraction>(mut writer: Writer<'_, Value>) -> Infallible {
    let mut counter = Wrapping(0);
    loop {
        writer.write(Value(counter.0)).await;
        counter += 1;
        T::sleep(Duration::from_millis(500)).await.unwrap();
    }
}

#[embassy_executor::task]
async fn run() {
    veecle_os::runtime::execute! {
        store: [Value],

        actors: [
            PrintActor<veecle_os::osal::embassy::log::Log>,
            IncrementActor<veecle_os::osal::embassy::time::Time>,
        ],
    }
    .await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Required to initialize the chip.
    // If removed, the chip will freeze when trying to wait for a timer.
    let _ = embassy_stm32::init(Default::default());
    veecle_os::osal::embassy::log::Log::init();
    spawner.spawn(run()).unwrap();
}
