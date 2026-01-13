//! Tests whether the memory pool can be used to pass data through the datastore.

use std::convert::Infallible;
use std::future::poll_fn;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;
use veecle_os_runtime::Storable;
use veecle_os_runtime::memory_pool::{Chunk, MemoryPool};
use veecle_os_runtime::{ExclusiveReader, Writer};

#[test]
fn memory_pool() {
    static READ_COUNTER: AtomicUsize = AtomicUsize::new(0);

    static POOL: MemoryPool<u8, 5> = MemoryPool::new();

    #[derive(Debug)]
    pub struct Data;

    impl Storable for Data {
        type DataType = Chunk<'static, u8>;
    }

    #[veecle_os_runtime::actor]
    async fn exclusive_read_actor(mut reader: ExclusiveReader<'_, Data>) -> Infallible {
        loop {
            if let Some(value) = reader.take() {
                println!("Value received: {value:?}");
                println!("Value received: {:?}", *value);
                READ_COUNTER.fetch_add(1, Ordering::AcqRel);
            } else {
                reader.wait_for_update().await;
            }
        }
    }

    #[veecle_os_runtime::actor]
    async fn write_actor(
        mut writer: Writer<'_, Data>,
        #[init_context] pool: &'static MemoryPool<u8, 5>,
    ) -> Infallible {
        for index in 0..10 {
            writer.write(pool.chunk(index).unwrap()).await;
        }
        core::future::pending().await
    }

    veecle_os_test::block_on_future(veecle_os_test::execute! {
        store: [Data],
        actors: [
            ExclusiveReadActor,
            WriteActor: &POOL,
        ],
        validation: async || {
            poll_fn(|cx| {
                if READ_COUNTER.load(Ordering::SeqCst) == 10 {
                    println!("[VEECLE_OS_READ_SIDE]: Read counter at 10.");
                    return Poll::Ready(());
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });
}
