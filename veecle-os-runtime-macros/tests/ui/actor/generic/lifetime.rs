#[derive(Debug, PartialEq, Clone, Default, veecle_os_runtime::Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime::actor]
async fn with_lifetime<'a>(
    _reader: veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _writer: veecle_os_runtime::single_writer::Writer<'_, Sensor>,
    #[init_context] _context: &'a (),
) -> veecle_os_runtime::Never {
    core::future::pending().await
}

fn main() {
    let _ = veecle_os_runtime::execute! {
        actors: [WithLifetime<'static>: &()],
    };
}
