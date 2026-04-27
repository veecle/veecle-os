#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

impl veecle_os_runtime::Flatten for Sensor {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Actuator(pub u8);

impl veecle_os_runtime::Flatten for Actuator {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _actuator_writer: veecle_os_runtime::single_writer::Writer<'_, Actuator>,
    _sensor_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _sensor_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor>,
    _actuator_reader: veecle_os_runtime::single_writer::ExclusiveReader<'_, Actuator>,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [MacroTestActor],
    };
}
