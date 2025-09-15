#[veecle_os_runtime_macros::actor(foo = "bar")]
async fn macro_test_actor() -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
