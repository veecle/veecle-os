#![allow(missing_docs)]

use core::fmt::Debug;
use veecle_os_runtime::Never;

use futures_test::future::FutureTestExt;
use veecle_os_runtime::{InitializedReader, Reader, Storable, Writer};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Storable)]
pub struct Signal(usize);

#[derive(Debug, Default, Clone, Storable)]
pub struct UpToDateSignal(Signal);

#[veecle_os_runtime::actor]
async fn filter_actor(
    mut up_to_date: Writer<'_, UpToDateSignal>,
    mut source: InitializedReader<'_, Signal>,
) -> Never {
    let mut latest = Signal(0);

    loop {
        let signal = source.wait_for_update().await.read_cloned();

        if latest < signal {
            up_to_date.write(UpToDateSignal(signal)).await;
            latest = signal;
        }
    }
}

#[test]
fn outdated_signals_should_be_discarded() {
    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [FilterActor],

        validation: async |mut reader: Reader<'_, UpToDateSignal>, mut writer: Writer<'_, Signal>| {
            writer.write(Signal(1)).await;

            reader.wait_for_update().await.read(|value| {
                assert_eq!(
                    &value.unwrap().0,
                    &Signal(1),
                    "up-to-date signal should be written"
                );
            });

            writer.write(Signal(0)).await;

            core::future::ready(()).pending_once().await;

            reader.read(|value| {
                assert_eq!(
                    &value.unwrap().0,
                    &Signal(1),
                    "outdated signal should not be written"
                );
            });
        }
    });
}
