#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(&self) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(self) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(self: Self) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
