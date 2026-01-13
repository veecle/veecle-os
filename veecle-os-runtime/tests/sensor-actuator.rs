#![expect(missing_docs)]

use core::fmt::Debug;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::future::poll_fn;
use std::sync::Mutex;
use std::task::Poll;
use veecle_os_runtime::Never;

use veecle_os_runtime::{CombineReaders, InitializedReader, Storable, Writer};

static SENSOR_VALIDATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static PRINTER_OUTPUT: Mutex<String> = Mutex::new(String::new());

#[derive(Debug, PartialEq, Clone)]
pub struct Sensor;

impl Storable for Sensor {
    type DataType = u8;
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct ActuatorData;

impl Storable for ActuatorData {
    type DataType = u8;
}

#[veecle_os_runtime::actor]
async fn sensor_actor(mut sensor_writer: Writer<'_, Sensor>) -> Never {
    for sensor in (0..).cycle() {
        sensor_writer.write(sensor).await;
    }
    unreachable!("The endless loop should never end.");
}

#[veecle_os_runtime::actor]
async fn sensor_validation(
    mut sensor_reader: InitializedReader<'_, Sensor>,
    mut actuator_data_reader: InitializedReader<'_, ActuatorData>,
) -> Never {
    for expected_sensor in (0..).cycle() {
        let combined_reader = (&mut sensor_reader, &mut actuator_data_reader);
        combined_reader.read(|(a, b)| {
            println!("{:?}, {:?}", a, b);
        });
        sensor_reader.wait_for_update().await.read(|&sensor| {
            assert_eq!(sensor, expected_sensor);
            SENSOR_VALIDATION_COUNT.fetch_add(1, Ordering::SeqCst);
        })
    }
    unreachable!("The endless loop should never end.");
}

#[veecle_os_runtime::actor]
async fn actuator(
    mut sensor_reader: InitializedReader<'_, Sensor>,
    mut actuator_data_writer: Writer<'_, ActuatorData>,
) -> Never {
    loop {
        let value = sensor_reader
            .wait_for_update()
            .await
            .read(|sensor| sensor + 10);
        actuator_data_writer.write(value).await
    }
}

#[veecle_os_runtime::actor]
async fn actuator_validation(
    mut actuator_data_reader: InitializedReader<'_, ActuatorData>,
) -> Never {
    for expected_actuator in (10..).cycle() {
        actuator_data_reader
            .wait_for_update()
            .await
            .read(|&actuator| {
                assert_eq!(actuator, expected_actuator);
            });
    }
    unreachable!("The endless loop should never end.");
}

#[veecle_os_runtime::actor]
async fn printer(mut actuator_data_reader: InitializedReader<'_, ActuatorData>) -> Never {
    use std::fmt::Write;

    loop {
        actuator_data_reader.wait_for_update().await.read(|value| {
            writeln!(&mut PRINTER_OUTPUT.lock().unwrap(), "{value:?}").unwrap();
        });
    }
}

#[test]
fn main() {
    veecle_os_test::block_on_future(veecle_os_test::execute! {
        store: [Sensor, ActuatorData],
        actors: [
            Printer,
            SensorValidation,
            SensorActor,
            Actuator,
            ActuatorValidation,
        ],
        validation: async || {
            poll_fn(|cx| {
                if SENSOR_VALIDATION_COUNT.load(Ordering::SeqCst) == 2 {
                    return Poll::Ready(());
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });

    assert_eq!(&**PRINTER_OUTPUT.lock().unwrap(), "10\n11\n");
}
