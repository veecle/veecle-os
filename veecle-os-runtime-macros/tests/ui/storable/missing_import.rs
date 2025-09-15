mod inner {
    pub struct Inner;
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
#[storable(data_type = "Vec<Inner>")]
pub struct Sensor;

fn value() -> u8 {
    1
}
#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor_reader: veecle_os_runtime::Reader<'_, Sensor>,
    _sensor_writer: veecle_os_runtime::Writer<'_, Sensor>,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor],
        actors: [MacroTestActor],
    };
}
