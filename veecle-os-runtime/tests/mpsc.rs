#![expect(missing_docs)]

use core::fmt::Debug;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::future::poll_fn;
use std::task::Poll;

use veecle_os_runtime::mpsc::{Reader, Writer};
use veecle_os_runtime::single_writer;
use veecle_os_runtime::{Never, Storable};

#[derive(Debug, PartialEq, Eq, Clone, Storable)]
pub struct Event(pub u8);

static READ_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[veecle_os_runtime::actor]
async fn writer_a(mut writer: Writer<'_, Event, 3>) -> Never {
    for index in 0..5 {
        writer.write(Event(index)).await;
    }
    core::future::pending().await
}

#[veecle_os_runtime::actor]
async fn writer_b(mut writer: Writer<'_, Event, 3>) -> Never {
    for index in 10..15 {
        writer.write(Event(index)).await;
    }
    core::future::pending().await
}

#[veecle_os_runtime::actor]
async fn collector(mut reader: Reader<'_, Event, 3>) -> Never {
    loop {
        reader.wait_for_update().await;
        let _ = reader.take_one();
        READ_COUNTER.fetch_add(1, Ordering::AcqRel);
    }
}

#[test]
fn two_writers_one_reader() {
    READ_COUNTER.store(0, Ordering::SeqCst);

    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [
            WriterA,
            WriterB,
            Collector,
        ],
        validation: async || {
            poll_fn(|context| {
                if READ_COUNTER.load(Ordering::SeqCst) >= 10 {
                    return Poll::Ready(());
                }
                context.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });
    assert!(READ_COUNTER.load(Ordering::SeqCst) >= 10);
}

/// Tests that mpsc can coexist with single_writer in the same system.
#[derive(Debug, PartialEq, Eq, Clone, Storable)]
pub struct Trigger;

static MIXED_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[veecle_os_runtime::actor]
async fn trigger_writer(mut writer: single_writer::Writer<'_, Trigger>) -> Never {
    for _ in 0..3 {
        writer.write(Trigger).await;
    }
    core::future::pending().await
}

#[veecle_os_runtime::actor]
async fn mpsc_writer_a(
    mut writer: Writer<'_, Event, 2>,
    mut trigger: single_writer::Reader<'_, Trigger>,
) -> Never {
    loop {
        trigger.wait_for_update().await;
        writer.write(Event(100)).await;
    }
}

#[veecle_os_runtime::actor]
async fn mixed_collector(mut reader: Reader<'_, Event, 2>) -> Never {
    loop {
        reader.wait_for_update().await;
        let _ = reader.take_one();
        MIXED_COUNTER.fetch_add(1, Ordering::AcqRel);
    }
}

#[test]
fn mixed_single_and_mpsc() {
    MIXED_COUNTER.store(0, Ordering::SeqCst);

    veecle_os_test::block_on_future(veecle_os_test::execute! {
        actors: [
            TriggerWriter,
            MpscWriterA,
            MixedCollector,
        ],
        validation: async || {
            poll_fn(|context| {
                if MIXED_COUNTER.load(Ordering::SeqCst) >= 3 {
                    return Poll::Ready(());
                }
                context.waker().wake_by_ref();
                Poll::Pending
            }).await;
        }
    });
    assert!(MIXED_COUNTER.load(Ordering::SeqCst) >= 3);
}
