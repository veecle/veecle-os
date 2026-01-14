#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _: veecle_os_runtime::Reader<'_, Sensor>,
    veecle_os_runtime::Writer { .. }: veecle_os_runtime::Writer<'_, Sensor>,
    #[init_context] (a, [b, c], Sensor { 0: x }): (u8, [u8; 2], Sensor),
) -> veecle_os_runtime::Never {
    let _: u8 = a + b + c + x;
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [MacroTestActor: (1, [2, 3], Sensor(4))],
    };
}
