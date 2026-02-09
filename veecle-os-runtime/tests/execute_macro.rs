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
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _sensor_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn sensor_reader(
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_reader(
    _other_reader: veecle_os_runtime::single_writer::Reader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_exclusive_reader(
    _other_reader: veecle_os_runtime::single_writer::ExclusiveReader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_double_exclusive_reader(
    _other_reader: veecle_os_runtime::single_writer::ExclusiveReader<'_, Other>,
    _other_reader_2: veecle_os_runtime::single_writer::ExclusiveReader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_dual_reader(
    _other_reader: veecle_os_runtime::single_writer::ExclusiveReader<'_, Other>,
    _other_reader_2: veecle_os_runtime::single_writer::Reader<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn other_writer(
    _other_writer: veecle_os_runtime::single_writer::Writer<'_, Other>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn data_writer(
    _data_writer: veecle_os_runtime::single_writer::Writer<'_, Data>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn exclusive_data_reader(
    _reader: veecle_os_runtime::single_writer::ExclusiveReader<'_, Data>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn generic_reader<T: veecle_os_runtime::Storable + 'static>(
    _reader: veecle_os_runtime::single_writer::Reader<'_, T>,
) -> veecle_os_runtime::Never {
    yield_once().await;
    panic!("done")
}

#[veecle_os_runtime::actor]
async fn generic_writer<T: veecle_os_runtime::Storable + 'static>(
    _writer: veecle_os_runtime::single_writer::Writer<'_, T>,
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
        actors: [
            SensorReaderWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test2() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            SensorReaderWriter, SensorReader, OtherReader, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(
    expected = "missing reader for `execute_macro::Data`, written by: `execute_macro::DataWriter<'_>`"
)]
fn make_executor_smoke_test3() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            SensorReaderWriter, DataWriter,
        ],
    });
}

#[test]
#[should_panic(
    expected = "multiple writers for `execute_macro::Other`: `execute_macro::OtherWriter<'_>`, `execute_macro::OtherWriter<'_>`"
)]
fn make_executor_smoke_test4() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            OtherReader, OtherWriter, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test5() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            GenericReader<Data>, GenericWriter<Data>,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test6() {
    futures::executor::block_on(veecle_os_runtime::execute! {
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
fn make_executor_smoke_test7() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            ContextualActor<bool>: true,
        ],
    });
}

#[test]
#[should_panic(expected = "done 5")]
fn make_executor_smoke_test8() {
    let local = 5;
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            ReferencingActor: &local,
        ],
    });
}

#[test]
#[should_panic(expected = "done")]
fn make_executor_smoke_test9() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            DataWriter,
            ExclusiveDataReader
        ],
    });
}

#[test]
#[should_panic(expected = "done [5]")]
fn make_executor_smoke_test10() {
    let non_copyable = vec![5];
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            ContextualActor<Vec<i32>>: non_copyable,
        ],
    });
}

#[test]
#[should_panic(expected = "done true")]
fn make_executor_smoke_test11() {
    let local = true;
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            ContextualActor<bool>: true,
            ContextualActor<&bool>: const { &true },
            ContextualActor<&bool>: &local,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`:
exclusive readers: `execute_macro::OtherExclusiveReader<'_>`
    other readers: `execute_macro::OtherReader<'_>`")]
fn make_executor_smoke_test12() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            OtherExclusiveReader, OtherWriter, OtherReader,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`:
exclusive readers: `execute_macro::OtherExclusiveReader<'_>`, `execute_macro::OtherExclusiveReader<'_>`
    other readers: nothing")]
fn make_executor_smoke_test13() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            OtherExclusiveReader, OtherWriter, OtherExclusiveReader,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`:
exclusive readers: `execute_macro::OtherDoubleExclusiveReader<'_>`, `execute_macro::OtherDoubleExclusiveReader<'_>`
    other readers: nothing")]
fn make_executor_smoke_test14() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            OtherDoubleExclusiveReader, OtherWriter,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `execute_macro::Other`:
exclusive readers: `execute_macro::OtherDualReader<'_>`
    other readers: `execute_macro::OtherDualReader<'_>`")]
fn make_executor_smoke_test15() {
    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            OtherDualReader, OtherWriter,
        ],
    });
}
