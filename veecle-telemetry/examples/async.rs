#![expect(missing_docs, reason = "example")]

use std::time::Duration;

use veecle_osal_std::{thread::Thread, time::Time};
use veecle_telemetry::{CurrentSpan, Span};

#[tokio::main]
async fn main() {
    veecle_telemetry::collector::build()
        .random_process_id()
        .console_json_exporter()
        .time::<Time>()
        .thread::<Thread>()
        .set_global()
        .expect("exporter was not set yet");

    let _span = Span::new("main", &[]).entered();

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
