#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    Reader { .. }: veecle_os_runtime::Reader<'_, Sensor>,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
