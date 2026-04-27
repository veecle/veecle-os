#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

impl veecle_os_runtime::Flatten for Sensor {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[veecle_os_runtime::actor]
async fn const_generic<const N: usize>(
    _reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _writer: veecle_os_runtime::single_writer::Writer<'_, Sensor>,
) -> veecle_os_runtime::Never {
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [ConstGeneric<2>],
    };
}
