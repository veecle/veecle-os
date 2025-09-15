pub struct Foo<'a>(&'a u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(#[init_context] _init_context: Foo<'_>) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let value = 5;
    let _ = veecle_os_runtime::execute! {
        store: [],
        actors: [
            MacroTestActor: Foo(&value),
        ],
    };
}
