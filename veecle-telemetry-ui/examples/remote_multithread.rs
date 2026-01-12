#![allow(missing_docs)]
//! Example with actual multiple OS threads generating telemetry data.
//!
//! This example spawns 8 OS threads to demonstrate multi-threaded telemetry
//! capabilities of the `veecle-telemetry-ui`.
//!
//! The threads are grouped into 4 groups (2 threads per group)
//! - "group-A": workers 1-2
//! - "group-B": workers 3-4
//! - ... etc.
//!
//! Run via:
//!
//! ```
//! # run example
//! cargo run --package veecle-telemetry-ui --example remote_multithread > spans.jsonl
//! # open veecle-telemetry-ui
//! cargo run --package veecle-telemetry-ui -- ./spans.jsonl
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Worker function that runs on a dedicated OS thread.
fn worker_thread(worker_id: i64, done: Arc<AtomicBool>) {
    for index in 0..3 {
        worker_task(worker_id, index);
        thread::sleep(Duration::from_millis(10));
    }
    done.store(true, Ordering::Relaxed);
}

fn worker_task(worker_id: i64, index: i64) {
    let group = match worker_id {
        1..=2 => "group-A",
        3..=4 => "group-B",
        5..=6 => "group-C",
        _ => "group-D",
    };

    // Create spans with actor names where multiple threads share the same actor.
    let span = veecle_telemetry::span!("worker_task", actor = group, worker_id, index);
    let _guard = span.entered();

    veecle_telemetry::info!("Starting task", worker_id = worker_id, index = index);

    // Simulate simple work.
    compute_fibonacci(worker_id, index);
    compute_factorial(worker_id, index);

    veecle_telemetry::info!("Task completed", worker_id = worker_id, index = index);
}

#[veecle_telemetry::instrument]
fn compute_fibonacci(worker_id: i64, n: i64) {
    veecle_telemetry::debug!("Computing fibonacci", worker_id = worker_id);
    let mut a = 0i64;
    let mut b = 1i64;
    for index in 0..(n + 10).min(20) {
        let c = a.saturating_add(b);
        a = b;
        b = c;
        if index % 5 == 0 {
            veecle_telemetry::trace!(
                "Fibonacci progress",
                worker_id = worker_id,
                index = index,
                value = b
            );
        }
    }
    veecle_telemetry::debug!("Fibonacci done", worker_id = worker_id, result = b);
}

#[veecle_telemetry::instrument]
fn compute_factorial(worker_id: i64, n: i64) {
    veecle_telemetry::debug!("Computing factorial", worker_id = worker_id);
    let mut result = 1i64;
    for index in 1..=(n + 5).min(15) {
        result = result.saturating_mul(index);
        if index % 3 == 0 {
            veecle_telemetry::trace!(
                "Factorial progress",
                worker_id = worker_id,
                index = index,
                value = result
            );
        }
    }
    veecle_telemetry::debug!("Factorial done", worker_id = worker_id, result = result);
}

fn main() {
    veecle_telemetry::collector::build()
        .random_process_id()
        .console_json_exporter()
        .time::<veecle_osal_std::time::Time>()
        .thread::<veecle_osal_std::thread::Thread>()
        .set_global()
        .expect("exporter was not set yet");

    eprintln!("Starting multi-threaded telemetry example with 8 OS threads...");

    // Spawn 8 workers.
    // TODO(?): allow to customize this in an example?
    let mut handles = vec![];
    let mut done_flags = vec![];

    for worker_id in 1..=8 {
        let done = Arc::new(AtomicBool::new(false));
        done_flags.push(done.clone());

        let handle = thread::Builder::new()
            .name(format!("worker-{}", worker_id))
            .spawn(move || {
                worker_thread(worker_id, done);
            })
            .expect("Failed to spawn thread");

        handles.push(handle);
    }

    // Wait for workers.
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(done_flags.iter().all(|d| d.load(Ordering::Relaxed)));

    eprintln!("All worker threads completed!");
}
