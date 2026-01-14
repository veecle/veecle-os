#[veecle_os_runtime_macros::actor]
async fn macro_test_actor() -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [],
        actors: [MacroTestActor],
    };
}
