#![expect(missing_docs)]

use core::fmt::Debug;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::future::poll_fn;
use std::sync::atomic::Ordering::SeqCst;
use std::task::Poll;
use veecle_os_runtime::{ExclusiveReader, Never, Reader, Storable, Writer};

static READ_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq, Clone, Storable)]
pub struct Sensor(pub u8);

#[veecle_os_runtime::actor]
async fn exclusive_read_actor(mut reader: ExclusiveReader<'_, Sensor>) -> Never {
    loop {
        if let Some(value) = reader.take() {
            println!("Value received: {value:?}");
            READ_COUNTER.fetch_add(1, Ordering::AcqRel);
        } else {
            reader.wait_for_update().await;
        }
    }
}

#[veecle_os_runtime::actor]
async fn write_actor(mut writer: Writer<'_, Sensor>) -> Never {
    for index in 0..10 {
        writer.write(Sensor(index)).await;
    }
    core::future::pending().await
}

#[test]
fn main() {
    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [
            ExclusiveReadActor,
            WriteActor,
        ],
        validation: async || {
            poll_fn(|cx| {
                if READ_COUNTER.load(SeqCst) == 10 {
                    return Poll::Ready(());
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });
    assert_eq!(READ_COUNTER.load(SeqCst), 10);
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `exclusive_reader::Sensor`")]
fn not_exclusive_first() {
    #[veecle_os_runtime::actor]
    async fn non_excl_read_actor(_reader: Reader<'_, Sensor>) -> Never {
        core::future::pending().await
    }

    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            ExclusiveReadActor,
            WriteActor,
            NonExclReadActor,
        ],
    });
}

#[test]
#[should_panic(expected = "conflict with exclusive reader for `exclusive_reader::Sensor`")]
fn not_exclusive_last() {
    #[veecle_os_runtime::actor]
    async fn non_excl_read_actor(_reader: Reader<'_, Sensor>) -> Never {
        core::future::pending().await
    }

    futures::executor::block_on(veecle_os_runtime::execute! {
        actors: [
            WriteActor,
            NonExclReadActor,
            ExclusiveReadActor,
        ],
    });
}
