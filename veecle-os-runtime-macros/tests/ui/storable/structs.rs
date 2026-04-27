#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor0<T>
where
    T: std::fmt::Debug,
{
    test: u8,
    test0: u8,
    test1: u8,
    test2: T,
}

impl<T: std::fmt::Debug> veecle_os_runtime::Flatten for Sensor0<T> {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor1(u8, u8, u8);

impl veecle_os_runtime::Flatten for Sensor1 {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor2;

impl veecle_os_runtime::Flatten for Sensor2 {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor3(Sensor1);

impl veecle_os_runtime::Flatten for Sensor3 {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[derive(Debug, PartialEq, veecle_os_runtime_macros::Storable)]
pub struct Sensor4(std::string::String);

impl veecle_os_runtime::Flatten for Sensor4 {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

fn value() -> u8 {
    1
}
#[veecle_os_runtime_macros::actor]
async fn macro_test_actor(
    _sensor0_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor0<usize>>,
    _sensor0_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor0<usize>>,
    _sensor1_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor1>,
    _sensor1_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor1>,
    _sensor2_reader: veecle_os_runtime::single_writer::Reader<'_, Sensor2>,
    _sensor2_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor2>,
    _sensor3_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor3>,
    _sensor4_writer: veecle_os_runtime::single_writer::Writer<'_, Sensor4>,
) -> veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [MacroTestActor],
    };
}
