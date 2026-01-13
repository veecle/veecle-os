#![expect(missing_docs)]

#[derive(Eq, PartialEq, Debug, Clone, veecle_os_runtime::Storable)]
pub struct Sensor(u8);

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct GenericData<T>(T);

impl<T> veecle_os_runtime::Storable for GenericData<T>
where
    T: core::fmt::Debug + 'static,
{
    type DataType = Self;
}

#[derive(Eq, PartialEq, Debug, Clone, veecle_os_runtime::Storable)]
pub struct Other(u8);

#[derive(Eq, PartialEq, Debug, Clone, veecle_os_runtime::Storable)]
pub struct Data(u8);

async fn yield_once() {
    let mut yielded = false;
    core::future::poll_fn(|cx| {
        if core::mem::replace(&mut yielded, true) {
            core::task::Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    })
    .await
}

#[veecle_os_runtime::actor]
async fn sensor_reader_writer(
    _sensor_reader: veecle_os_runtime::Reader<'_, Sensor>,
    _sensor_writer: veecle_os_runtime::Writer<'_, Sensor>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn sensor_reader(
    _sensor_reader: veecle_os_runtime::Reader<'_, Sensor>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_reader(
    _other_reader: veecle_os_runtime::Reader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_exclusive_reader(
    _other_reader: veecle_os_runtime::ExclusiveReader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_writer(
    _other_writer: veecle_os_runtime::Writer<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn data_writer(
    _data_writer: veecle_os_runtime::Writer<'_, Data>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn exclusive_data_reader(
    _reader: veecle_os_runtime::ExclusiveReader<'_, Data>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn generic_reader<T: veecle_os_runtime::Storable + 'static>(
    _reader: veecle_os_runtime::Reader<'_, T>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn generic_writer<T: veecle_os_runtime::Storable + 'static>(
    _writer: veecle_os_runtime::Writer<'_, T>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn contextual_actor<T: core::fmt::Debug>(
    #[init_context] context: T,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done {context:?}")
}

#[veecle_os_runtime::actor]
async fn referencing_actor(#[init_context] context: &i32) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done {context}")
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test1() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Sensor],

        actors: [
            SensorReaderWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test2() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Sensor, Other],

        actors: [
            SensorReaderWriter, SensorReader, OtherReader, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "missing writer for `execute_macro::Data`")]
fn make_executor_smoke_test3() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Sensor, Data],

        actors: [
            SensorReaderWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "missing reader for `execute_macro::Data`")]
fn make_executor_smoke_test4() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Sensor, Data],

        actors: [
            SensorReaderWriter, DataWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "no slot available for `execute_macro::Other`")]
fn make_executor_smoke_test5() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [],

        actors: [
            OtherReader, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "multiple writers for `execute_macro::Other`")]
fn make_executor_smoke_test6() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Other],

        actors: [
            OtherReader, OtherWriter, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test7() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Data],

        actors: [
            GenericReader<Data>, GenericWriter<Data>,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test8() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [GenericData<bool>, GenericData<i32>],

        actors: [
            GenericReader<GenericData<bool>>,
            GenericWriter<GenericData<bool>>,
            GenericReader<GenericData<i32>>,
            GenericWriter<GenericData<i32>>,
        ],
    });
}

#[test]
#[should_panic(expected = "done true")]
fn make_executor_smoke_test9() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [],

        actors: [
            ContextualActor<bool>: true,
        ],
    });
}

#[test]
#[should_panic(expected = "done 5")]
fn make_executor_smoke_test10() {
    let local = 5;
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [],

        actors: [
            ReferencingActor: &local,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test11() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Data],

        actors: [
            DataWriter,
            ExclusiveDataReader
        ],
    });
}

#[test]
#[should_panic(expected = "done [5]")]
fn make_executor_smoke_test12() {
    let non_copyable = vec![5];
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [],

        actors: [
            ContextualActor<Vec<i32>>: non_copyable,
        ],
    });
}

#[test]
#[should_panic(expected = "done true")]
fn make_executor_smoke_test13() {
    let local = true;
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [],

        actors: [
            ContextualActor<_>: true,
            ContextualActor<_>: const { &true },
            ContextualActor<_>: &local,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`")]
fn make_executor_smoke_test14() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Other],

        actors: [
            OtherExclusiveReader, OtherWriter, OtherReader,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`")]
fn make_executor_smoke_test15() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        store: [Other],

        actors: [
            OtherExclusiveReader, OtherWriter, OtherExclusiveReader,
        ],
    });
}
