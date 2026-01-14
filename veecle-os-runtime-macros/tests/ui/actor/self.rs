#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(&self) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(self) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(self: Self) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
