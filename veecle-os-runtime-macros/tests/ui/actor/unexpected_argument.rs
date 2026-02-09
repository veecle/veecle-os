#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor1(
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _unexpected: u32,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor2(_unexpected: u32) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

pub trait Bar {
    type Ty;
}
pub struct Foo;
impl Bar for Foo {
    type Ty = ();
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor3(_unexpected: <Foo as Bar>::Ty) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor4(_unexpected: [u32; 0]) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor5(
    _unexpected: u32,
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor6(
    _unexpected: u32,
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _unexpected1: usize,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
