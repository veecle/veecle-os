#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime::actor]
async fn discard<T>(
    _reader: veecle_os_runtime::single_writer::Reader<'_, T>,
    _writer: veecle_os_runtime::single_writer::Writer<'_, T>,
) -> veecle_os_runtime::Never
where
    T: veecle_os_runtime::Storable + 'static,
{
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [Discard<Sensor>],
    };
}
