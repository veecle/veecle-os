#[derive(veecle_os_runtime_macros::Storable, Clone, Debug)]
pub struct Reference<'a>(&'a u8);

fn verify_static(_: &'static u8) {}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    mut reader: veecle_os_runtime::single_writer::Reader<'_, Reference<'static>>,
    _writer: veecle_os_runtime::single_writer::Writer<'_, Reference<'static>>,
) -> veecle_os_runtime::Never {
    verify_static(reader.read_cloned().unwrap().0);
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [MacroTestActor],
    };
}
