mod inner {
    #[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime_macros::Storable)]
    pub struct Sensor(pub u8);

    #[veecle_os_runtime_macros::actor]
    async fn macro_test_actor(
        _reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    ) -> veecle_os_runtime::Never {
        unreachable!("We only care about the code compiling.");
    }
}

fn main() {
    let _: self::inner::MacroTestActor;
}
