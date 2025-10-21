#![expect(missing_docs, reason = "example")]

use veecle_telemetry::Span;
use veecle_telemetry::collector::ConsoleJsonExporter;
use veecle_telemetry::protocol::ExecutionId;

fn main() {
    let execution_id = ExecutionId::random(&mut rand::rng());
    veecle_telemetry::collector::set_exporter(execution_id, &ConsoleJsonExporter)
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
