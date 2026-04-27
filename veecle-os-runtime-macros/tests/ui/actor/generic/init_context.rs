#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

impl veecle_os_runtime::Flatten for Sensor {
    fn flatten(&self, _buffer: &mut impl veecle_os_runtime::MetricBuffer) {}
}

#[veecle_os_runtime::actor]
async fn discard<T>(
    _reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _writer: veecle_os_runtime::single_writer::Writer<'_, Sensor>,
    #[init_context] _context: T,
) -> veecle_os_runtime::Never
where
    T: core::fmt::Debug + 'static,
{
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [Discard<()>: ()],
    };
}
