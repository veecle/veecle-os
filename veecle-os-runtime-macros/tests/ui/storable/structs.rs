#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor0<T>
where
    T: std::fmt::Debug,
{
    test: u8,
    test0: u8,
    test1: u8,
    test2: T,
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor1(u8, u8, u8);

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor2;

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor3(Sensor1);

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor4(std::string::String);

fn value() -> u8 {
    1
}
#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor0_reader: veecle_os_runtime::Reader<'_, Sensor0<usize>>,
    _sensor0_writer: veecle_os_runtime::Writer<'_, Sensor0<usize>>,
    _sensor1_reader: veecle_os_runtime::Reader<'_, Sensor1>,
    _sensor1_writer: veecle_os_runtime::Writer<'_, Sensor1>,
    _sensor2_reader: veecle_os_runtime::Reader<'_, Sensor2>,
    _sensor2_writer: veecle_os_runtime::Writer<'_, Sensor2>,
    _sensor3_writer: veecle_os_runtime::Writer<'_, Sensor3>,
    _sensor4_writer: veecle_os_runtime::Writer<'_, Sensor4>,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor0<usize>, Sensor1, Sensor2, Sensor3, Sensor4],
        actors: [MacroTestActor],
    };
}
