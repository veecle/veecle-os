#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(#[init_context] foo: u32) -> std::convert::Infallible {
    let _ = foo;
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [],
        actors: [
            MacroTestActor: 1,
        ],
    };
}
