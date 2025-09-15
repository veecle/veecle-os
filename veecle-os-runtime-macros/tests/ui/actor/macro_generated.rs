use ::{veecle_os_runtime, veecle_os_runtime_macros};

#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor1(pub u8);

macro_rules! make_actor {
    () => {
        #[$crate::veecle_os_runtime_macros::actor]
        async fn macro_test_actor(
            _sensor_reader: $crate::veecle_os_runtime::Reader<'_, Sensor>,
            _sensor1_reader_exlc: $crate::veecle_os_runtime::ExclusiveReader<'_, Sensor1>,
            _sensor_writer: $crate::veecle_os_runtime::Writer<'_, Sensor>,
            _sensor1_writer: $crate::veecle_os_runtime::Writer<'_, Sensor1>,
        ) -> std::convert::Infallible {
            unreachable!("We only care about the code compiling.");
        }
    };
}

make_actor!();

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor, Sensor1],
        actors: [MacroTestActor],
    };
}
