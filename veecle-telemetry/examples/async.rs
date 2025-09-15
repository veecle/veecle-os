#![expect(missing_docs, reason = "example")]

use std::time::Duration;

use veecle_telemetry::collector::ConsoleJsonExporter;
use veecle_telemetry::protocol::ExecutionId;
use veecle_telemetry::{CurrentSpan, Span, SpanContext};

#[tokio::main]
async fn main() {
    let execution_id = ExecutionId::random(&mut rand::rng());
    veecle_telemetry::collector::set_exporter(execution_id, &ConsoleJsonExporter)
        .expect("exporter was not set yet");

    let _span = Span::root("main", SpanContext::generate(), &[]).entered();

    tokio::join!(async_a(), async_b());
}

#[veecle_telemetry::instrument]
async fn async_a() {
    veecle_telemetry::info!("running nested function in `a`");

    nested().await;
}

#[veecle_telemetry::instrument]
async fn async_b() {
    veecle_telemetry::info!("running nested function in `b`");

    nested().await;
}

#[veecle_telemetry::instrument]
async fn nested() {
    tokio::time::sleep(Duration::from_millis(10)).await;
    CurrentSpan::add_event("nested_done", &[]);
}
