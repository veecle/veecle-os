#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub enum Sensor0 {
    Variant1,
    Variant2(u8),
    Variant3 { test: u8 },
}

#[repr(usize)]
#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub enum Sensor1 {
    Variant1 = 5,
    Variant2(u8),
    Variant3 { test: u8 },
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor0_reader: veecle_os_runtime::Reader<'_, Sensor0>,
    _sensor0_writer: veecle_os_runtime::Writer<'_, Sensor0>,
    _sensor1_reader: veecle_os_runtime::Reader<'_, Sensor1>,
    _sensor1_writer: veecle_os_runtime::Writer<'_, Sensor1>,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor0, Sensor1],
        actors: [MacroTestActor],
    };
}
