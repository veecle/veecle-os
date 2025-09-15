#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime::actor]
async fn discard<T>(
    _reader: veecle_os_runtime::Reader<'_, T>,
    _writer: veecle_os_runtime::Writer<'_, T>,
) -> core::convert::Infallible
where
    T: veecle_os_runtime::Storable + 'static,
{
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor],
        actors: [Discard<Sensor>],
    };
}
