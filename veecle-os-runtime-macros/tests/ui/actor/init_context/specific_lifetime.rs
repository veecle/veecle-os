#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    #[init_context] _init_context: &'a u8,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {}
