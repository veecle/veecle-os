#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime::actor]
async fn discard<T>(
    _reader: veecle_os_runtime::Reader<'_, Sensor>,
    _writer: veecle_os_runtime::Writer<'_, Sensor>,
    #[init_context] _context: T,
) -> core::convert::Infallible
where
    T: core::fmt::Debug + 'static,
{
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        store: [Sensor],
        actors: [Discard<()>: ()],
    };
}
