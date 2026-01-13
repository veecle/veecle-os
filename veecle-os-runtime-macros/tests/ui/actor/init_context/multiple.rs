#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor_reader: veecle_os_runtime::Reader<'_, Sensor>,
    _sensor_writer: veecle_os_runtime::Writer<'_, Sensor>,
    #[init_context] _init_context1: u8,
    #[init_context] _init_context2: String,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
