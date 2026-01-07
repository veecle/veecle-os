#![expect(missing_docs, reason = "example")]

use veecle_osal_std::{thread::Thread, time::Time};
use veecle_telemetry::Span;
use veecle_telemetry::collector::{ConsoleJsonExporter, ProcessId};

fn main() {
    let process_id = ProcessId::random(&mut rand::rng());
    veecle_telemetry::collector::build()
        .process_id(process_id)
        .exporter(&ConsoleJsonExporter::DEFAULT)
        .time::<Time>()
        .thread::<Thread>()
        .set_global()
        .expect("exporter was not set yet");

    let _span = Span::new("main", &[]).entered();
    nested();
}

fn nested() {
    let _span = veecle_telemetry::span!("nested").entered();
    deeply_nested();
    std::thread::sleep(std::time::Duration::from_millis(10));
    deeply_nested();
}

fn deeply_nested() {
    let _span = veecle_telemetry::span!("deeply_nested", test = true).entered();
    std::thread::sleep(std::time::Duration::from_millis(10));
}
