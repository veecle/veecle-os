#[derive(veecle_os_runtime_macros::Storable, Clone, Debug)]
pub struct Reference<'a>(&'a u8);

fn verify_static(_: &'static u8) {}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    reader: veecle_os_runtime::Reader<'_, Reference<'static>>,
    _writer: veecle_os_runtime::Writer<'_, Reference<'static>>,
) -> veecle_os_runtime::Never {
    verify_static(reader.read_cloned().unwrap().0);
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Reference<'static>],
        actors: [MacroTestActor],
    };
}
