//! Thread-related abstractions.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

pub use veecle_osal_api::thread::ThreadAbstraction;

/// Implements the [`ThreadAbstraction`] trait for standard Rust.
///
/// This implementation uses a thread-local counter initialized from a static atomic counter
/// to assign unique IDs to threads.
#[derive(Debug)]
pub struct Thread;

/// Global counter for generating unique thread IDs.
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Thread-local storage for the current thread's ID.
    static THREAD_ID: Cell<u64> = const { Cell::new(0) };
}

impl ThreadAbstraction for Thread {
    fn current_thread_id() -> u64 {
        THREAD_ID.with(|id| {
            let current = id.get();
            if current == 0 {
                let new_id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
                id.set(new_id);
                new_id
            } else {
                current
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_id_consistency() {
        let id1 = Thread::current_thread_id();
        let id2 = Thread::current_thread_id();
        assert_eq!(
            id1, id2,
            "Thread ID should be consistent within the same thread"
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
            "Main thread and thread 1 should have different IDs"
        );
        assert_ne!(
            main_id, thread2_id,
            "Main thread and thread 2 should have different IDs"
        );
        assert_ne!(
            thread1_id, thread2_id,
            "Thread 1 and thread 2 should have different IDs"
        );
    }

    #[test]
    fn test_thread_id_non_zero() {
        let id = Thread::current_thread_id();
        assert_ne!(id, 0, "Thread ID should never be zero");
    }
}
