#[derive(Debug, PartialEq, Clone, Default)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor(crate = "::veecle_os_runtime")]
async fn macro_test_actor1() -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor(crate())]
async fn macro_test_actor2() -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor(crate = ::veecle_os_runtime, crate = ::veecle_os_runtime2)]
async fn macro_test_actor3() -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
}
