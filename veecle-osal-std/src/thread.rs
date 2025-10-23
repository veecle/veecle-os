//! Thread-related abstractions.

use std::cell::LazyCell;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

pub use veecle_osal_api::thread::ThreadAbstraction;

/// Implements the [`ThreadAbstraction`] trait for standard Rust.
#[derive(Debug)]
pub struct Thread;

/// Global counter for generating unique thread ids.
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Thread-local storage for the current thread's id.
    static THREAD_ID: LazyCell<u64> = const { LazyCell::new(||{
        // `Relaxed` is enough, we don't care about what specific value a thread sees.
        // We just ensure that every value is unique.
        // This assumes that creating 2^64 threads is impractical and no overflow occurs.
        NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed)
    }) };
}

impl ThreadAbstraction for Thread {
    fn current_thread_id() -> NonZeroU64 {
        NonZeroU64::new(THREAD_ID.with(|thread_id| **thread_id)).expect("overflow should not occur")
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_thread_id_consistency() {
        let id1 = Thread::current_thread_id();
        let id2 = Thread::current_thread_id();
        assert_eq!(
            id1, id2,
            "Thread id should be consistent within the same thread"
        );
    }

    #[test]
    fn test_thread_id_uniqueness() {
        let main_id = Thread::current_thread_id();

        let handle1 = std::thread::spawn(Thread::current_thread_id);
        let handle2 = std::thread::spawn(Thread::current_thread_id);

        let thread1_id = handle1.join().unwrap();
        let thread2_id = handle2.join().unwrap();

        assert_ne!(
            main_id, thread1_id,
            "Main thread and thread 1 should have different ids"
        );
        assert_ne!(
            main_id, thread2_id,
            "Main thread and thread 2 should have different ids"
        );
        assert_ne!(
            thread1_id, thread2_id,
            "Thread 1 and thread 2 should have different ids"
        );
    }
}
