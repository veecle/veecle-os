#![expect(missing_docs)]

use std::future::poll_fn;
use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd, RawFd};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::thread;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::pipe;
use tokio::net::unix::pipe::{Receiver, Sender};
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use veecle_os_runtime::{Reader, Storable, Writer};

// This test showcases how two runtime instances can communicate via I/O runtime-actors (using anonymous unix pipes).
// The runtime-actors use their own Tokio runtimes.
// This test does not represent any production-ready setup.

// To run this test with output, run: `cargo test -p veecle-os-runtime --test tokio-runtime-actor -- --nocapture`

const BUFFER_SIZE: usize = 16;

/// Message sent between runtime instances via unix pipe.
#[derive(Debug, Default, Storable)]
pub struct PipeMessage(pub [u8; BUFFER_SIZE]);

#[veecle_os_runtime::actor]
async fn write_pipe_runtime_actor(#[init_context] pipe_tx_fd: RawFd) -> veecle_os_runtime::Never {
    let (pipe_message_channel_tx, mut pipe_message_channel_rx) = mpsc::channel::<PipeMessage>(100);
    thread::spawn(move || {
        let tokio_runtime = Builder::new_current_thread().enable_io().build().unwrap();

        tokio_runtime.block_on(async {
            // We don't want a duplicated OwnedFd, which will free the resource on drop. This is why we need to
            // store a RawFd.
            //
            // Safety:
            // The file descriptor is valid as it is taken directly from the creation of an OwnedFd and the resource
            // pointed to is open.
            let owned_fd = unsafe { OwnedFd::from_raw_fd(pipe_tx_fd) };

            let mut pipe_tx = Sender::from_owned_fd(owned_fd).unwrap();

            for _ in 0..10 {
                if let Some(pipe_message) = pipe_message_channel_rx.recv().await {
                    println!("[WRITE_RUNTIME_ACTOR]: Writing to pipe.");
                    pipe_tx.write_all(&pipe_message.0).await.unwrap();
                } else {
                    eprint!("[WRITE_RUNTIME_ACTOR]: Received None from pipe_message_channel_rx.");
                }
            }
        })
    });

    for index in 0..10 {
        let message = PipeMessage([index; BUFFER_SIZE]);
        pipe_message_channel_tx.send(message).await.unwrap();
    }

    core::future::pending().await
}

#[veecle_os_runtime::actor]
async fn read_pipe_runtime_actor(
    mut pipe_message_writer: Writer<'_, PipeMessage>,
    #[init_context] pipe_rx_fd: RawFd,
) -> veecle_os_runtime::Never {
    let (tx, mut rx) = mpsc::channel(100);
    thread::spawn(move || {
        let tokio_runtime = Builder::new_current_thread().enable_io().build().unwrap();

        tokio_runtime.block_on(async {
            // We don't want a duplicated OwnedFd, which will free the resource on drop. This is why we need to
            // store a RawFd.
            //
            // Safety:
            // The file descriptor is valid as it is taken directly from the creation of an OwnedFd and the resource
            // pointed to is open.
            let owned_fd = unsafe { OwnedFd::from_raw_fd(pipe_rx_fd) };
            let mut pipe_rx = Receiver::from_owned_fd(owned_fd).unwrap();

            let mut pipe_message = vec![0; BUFFER_SIZE];

            for _ in 0..10 {
                println!("[READ_RUNTIME_ACTOR]: Reading from pipe");
                pipe_rx.read_exact(&mut pipe_message).await.unwrap();
                if let Err(error) = tx
                    .send(PipeMessage(pipe_message.as_slice().try_into().unwrap()))
                    .await
                {
                    eprintln!("[READ_RUNTIME_ACTOR]: Error: {error:?}");
                }
            }
        })
    });

    while let Some(pipe_message) = rx.recv().await {
        pipe_message_writer.write(pipe_message).await;
    }

    core::future::pending().await
}

#[veecle_os_runtime::actor]
async fn read_printer(
    mut pipe_message_reader: Reader<'_, PipeMessage>,
    #[init_context] read_counter: &'static AtomicUsize,
) -> veecle_os_runtime::Never {
    loop {
        pipe_message_reader
            .read_updated(|pipe_message: &PipeMessage| {
                println!("[PRINTER]: PipeMessage: {pipe_message:?}");
                read_counter.fetch_add(1, Ordering::AcqRel);
            })
            .await;
    }
}

fn veecle_os_executor_read(pipe_rx: OwnedFd) {
    static READ_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let pipe_rx = pipe_rx.into_raw_fd();

    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [
            ReadPipeRuntimeActor: pipe_rx,
            ReadPrinter: &READ_COUNTER,
        ],
        validation: async || {
            poll_fn(|cx| {
                if READ_COUNTER.load(Ordering::Acquire) == 10 {
                    println!("[VEECLE_OS_READ_SIDE]: Read counter at 10.");
                    return Poll::Ready(());
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });
}

fn veecle_os_executor_write(pipe_tx: OwnedFd) {
    let pipe_tx = pipe_tx.into_raw_fd();

    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            WritePipeRuntimeActor: pipe_tx,
        ]
    });
}
// Miri does not support the pipe creation:
// error: unsupported operation: can't call foreign function `pipe2` on OS `linux`
#[cfg_attr(miri, ignore)]
#[test]
fn main() {
    // Set up the pipe for the runtime-actors.
    let (pipe_tx, pipe_rx) = create_pipe();

    let read_join_handle = thread::spawn(move || {
        veecle_os_executor_read(pipe_rx);
    });
    thread::spawn(move || {
        veecle_os_executor_write(pipe_tx);
    });
    read_join_handle.join().unwrap();
}

/// Creates the pipe used by the runtime-actors.
///
/// In a real scenario, this should all be done within the runtime-actor itself. For ease of use, we create an
/// anonymous pipe here and provide it to the runtime-actors.
fn create_pipe() -> (OwnedFd, OwnedFd) {
    let pipe_fd_store: Arc<Mutex<Option<(OwnedFd, OwnedFd)>>> = Arc::new(Mutex::new(None));
    // Cloned because otherwise the Tokio runtime will move `pipe_fd_store`.
    let pipe_fs_store_clone = pipe_fd_store.clone();

    let tokio_runtime = Builder::new_current_thread().enable_io().build().unwrap();
    tokio_runtime.block_on(async move {
        let (pipe_tx, pipe_rx) = pipe::pipe().unwrap();
        pipe_fs_store_clone.lock().unwrap().replace((
            pipe_tx.into_blocking_fd().unwrap(),
            pipe_rx.into_blocking_fd().unwrap(),
        ));
    });
    tokio_runtime.shutdown_background();

    let (pipe_tx, pipe_rx) = pipe_fd_store.lock().unwrap().take().unwrap();
    (pipe_tx, pipe_rx)
}
