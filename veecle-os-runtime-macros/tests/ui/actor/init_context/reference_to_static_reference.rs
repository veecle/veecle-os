fn verify_static(_: &'static u8) {}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(#[init_context] init_context: &&'static u8) -> veecle_os_runtime::Never {
    verify_static(*init_context);
    unreachable!("We only care about the code compiling.");
}

fn main() {
    static VALUE: u8 = 5;
    let reference = &VALUE;
    let _ = veecle_os_runtime::execute! {
        actors: [
            MacroTestActor: &reference,
        ],
    };
}
