#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor_reader: veecle_os_runtime::Reader<'_, Sensor>,
    _unexpected: u32,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(_unexpected: u32) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(_unexpected: <Foo as Bar>::Ty) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(_unexpected: [u32; 0]) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
