#[derive(Debug, Default, veecle_os_runtime::Storable)]
pub struct Command(pub u8);

#[veecle_os_runtime::actor]
async fn writer_actor<const N: usize>(
    mut _writer: veecle_os_runtime::mpsc::Writer<'_, Command, N>,
) -> veecle_os_runtime::Never {
    unreachable!();
}

#[veecle_os_runtime::actor]
async fn reader_actor<const N: usize>(
    _reader: veecle_os_runtime::mpsc::Reader<'_, Command, N>,
) -> veecle_os_runtime::Never {
    unreachable!();
}

fn main() {
    const N: usize = 2;

    let _ = veecle_os_runtime::execute! {
        actors: [WriterActor<N>, ReaderActor<N>],
    };
}
