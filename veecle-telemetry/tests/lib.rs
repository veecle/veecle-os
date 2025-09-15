#![expect(missing_docs, reason = "tests")]
#![cfg(not(miri))]

// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.
// Copyright 2025 Veecle GmbH.
//
// This file has been modified from the original TiKV implementation.

use std::time::Duration;

use indoc::indoc;
use serial_test::serial;
use tokio::runtime::Builder;
use veecle_telemetry::future::FutureExt;
use veecle_telemetry::protocol::Severity;
use veecle_telemetry::test_helpers::format_telemetry_tree;
use veecle_telemetry::{CurrentSpan, KeyValue, Span, SpanContext, instrument, span};

mod exporter {
    use std::sync::{Arc, LazyLock, Mutex};

    use veecle_telemetry::collector::TestExporter;
    use veecle_telemetry::protocol::{ExecutionId, InstanceMessage};

    /// Initializes the lazy lock which sets the exporter.
    pub fn set_exporter() -> ExporterHandle {
        static EXPORTER: LazyLock<Arc<Mutex<Vec<InstanceMessage<'static>>>>> =
            LazyLock::new(|| {
                let (reporter, collected_spans) = TestExporter::new();

                let execution_id = ExecutionId::random(&mut rand::rng());
                veecle_telemetry::collector::set_exporter(
                    execution_id,
                    Box::leak(Box::new(reporter)),
                )
                .expect("exporter was not set yet");

                collected_spans
            });

        ExporterHandle {
            message_buffer: EXPORTER.clone(),
        }
    }

    pub struct ExporterHandle {
        message_buffer: Arc<Mutex<Vec<InstanceMessage<'static>>>>,
    }

    impl ExporterHandle {
        pub fn take_messages(&self) -> Vec<InstanceMessage<'static>> {
            self.message_buffer.lock().unwrap().drain(..).collect()
        }
    }
}

use exporter::set_exporter;

#[test]
#[serial]
fn trace_macro() {
    trait Foo {
        async fn run(&self, millis: &u64);
    }

    struct Bar;

    impl Foo for Bar {
        #[instrument(name = "run")]
        async fn run(&self, millis: &u64) {
            work_inner().await;
            work(millis).await;
            let _g = span!("local-span").entered();
        }
    }

    #[instrument(short_name = true)]
    async fn work(millis: &u64) {
        work_inner().await;
        tokio::time::sleep(Duration::from_millis(*millis))
            .with_span(span!("sleep"))
            .await;
    }

    #[instrument(short_name = true)]
    async fn work_inner() {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    impl Bar {
        #[instrument(short_name = true)]
        async fn work2(&self, millis: &u64) {
            work_inner().await;
            tokio::time::sleep(Duration::from_millis(*millis))
                .with_span(span!("sleep"))
                .await;
        }
    }

    #[instrument(short_name = true)]
    async fn work3(millis1: &u64, millis2: &u64) {
        work_inner().await;
        tokio::time::sleep(Duration::from_millis(*millis1))
            .with_span(span!("sleep"))
            .await;
        tokio::time::sleep(Duration::from_millis(*millis2))
            .with_span(span!("sleep"))
            .await;
    }

    let exporter = set_exporter();

    {
        let context = SpanContext::generate();
        let root = Span::root("root", context, &[]);

        let runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(
            async {
                Bar.run(&100).await;
                Bar.work2(&100).await;
                work3(&100, &100).await;
            }
            .with_span(root),
        );
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {"
            root []
                run []
                    work_inner []
                    work []
                        work_inner []
                        sleep []
                    local-span []
                work2 []
                    work_inner []
                    sleep []
                work3 []
                    work_inner []
                    sleep []
                    sleep []
        "}
    );
}

#[test]
#[serial]
fn trace_macro_example() {
    #[instrument(short_name = true)]
    fn do_something_short_name(i: u64) {
        std::thread::sleep(Duration::from_millis(i));
    }

    #[instrument(short_name = true)]
    async fn do_something_async_short_name(i: u64) {
        tokio::time::sleep(Duration::from_millis(i)).await;
    }

    #[instrument]
    fn do_something(i: u64) {
        std::thread::sleep(Duration::from_millis(i));
    }

    #[instrument]
    async fn do_something_async(i: u64) {
        tokio::time::sleep(Duration::from_millis(i)).await;
    }

    let exporter = set_exporter();

    {
        let context = SpanContext::generate();
        let _root_guard = Span::root("root", context, &[]).entered();

        let runtime = Builder::new_current_thread().enable_all().build().unwrap();

        do_something(100);
        runtime.block_on(do_something_async(100));
        do_something_short_name(100);
        runtime.block_on(do_something_async_short_name(100));
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {"
            root []
                lib::trace_macro_example::{{closure}}::do_something []
                lib::trace_macro_example::{{closure}}::do_something_async []
                do_something_short_name []
                do_something_async_short_name []
        "}
    );
}

#[test]
#[serial]
fn span_property() {
    let exporter = set_exporter();

    {
        let context = SpanContext::generate();
        let root = Span::root(
            "root",
            context,
            &[KeyValue::new("k1", "v1"), KeyValue::new("k2", 2)],
        );
        root.set_attribute(KeyValue::new("k3", "v3"));

        let _g = root.enter();

        root.set_attribute(KeyValue::new("k4", "v4"));
        root.set_attribute(KeyValue::new("k5", 5));

        let _span = span!("span").entered();
        CurrentSpan::set_attribute(KeyValue::new("k6", "v6"));
        CurrentSpan::set_attribute(KeyValue::new("k7", 7));
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {r#"
            root [k1="v1", k2=2]
                + attr: k3="v3"
                + attr: k4="v4"
                + attr: k5=5
                span []
                    + attr: k6="v6"
                    + attr: k7=7
        "#}
    );
}

#[test]
#[serial]
fn current_span_integration() {
    use veecle_telemetry::{CurrentSpan, SpanId, TraceId};

    let exporter = set_exporter();

    {
        let context = SpanContext::generate();
        let root = Span::root("root", context, &[]);

        let _guard = root.entered();

        // Test CurrentSpan::event
        CurrentSpan::add_event(
            "test_event",
            &[
                KeyValue::new("event_key", "event_value"),
                KeyValue::new("event_num", 42),
            ],
        );

        // Test CurrentSpan::link
        let external_context =
            SpanContext::new(TraceId(0x123456789ABCDEF0), SpanId(0xFEDCBA9876543210));
        CurrentSpan::add_link(external_context);

        // Test CurrentSpan::attribute
        CurrentSpan::set_attribute(KeyValue::new("runtime_attr", "added_later"));

        // Create a child span to test nested behavior
        let child_span = span!("child", { child_attr = true });
        let _child_guard = child_span.entered();

        // Test CurrentSpan methods on child span
        CurrentSpan::add_event(
            "child_event",
            &[KeyValue::new("child_event_data", "nested")],
        );
        CurrentSpan::set_attribute(KeyValue::new("child_runtime_attr", 100));

        let another_external =
            SpanContext::new(TraceId(0x1111111111111111), SpanId(0x2222222222222222));
        CurrentSpan::add_link(another_external);
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {r#"
            root []
                + attr: runtime_attr="added_later"
                + link: trace=123456789abcdef0, span=fedcba9876543210
                + event: test_event [event_key="event_value", event_num=42]
                child [child_attr=true]
                    + attr: child_runtime_attr=100
                    + link: trace=1111111111111111, span=2222222222222222
                    + event: child_event [child_event_data="nested"]
        "#}
    );
}

#[test]
#[serial]
fn log_without_span_context() {
    let exporter = set_exporter();

    {
        veecle_telemetry::info!("test message");
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {"
            + log: [Info] test message []
        "}
    );
}

#[test]
#[serial]
fn log_with_span_context() {
    use veecle_telemetry::{SpanId, TraceId};

    let exporter = set_exporter();

    {
        let trace_id = TraceId(0x123);
        let span_id = SpanId(0);
        let span_context = SpanContext::new(trace_id, span_id);
        let span = Span::root("test_span", span_context, &[]);

        let _guard = span.entered();
        veecle_telemetry::log::log(Severity::Error, "error message", &[]);
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {"
            test_span []
                + log: [Error] error message []
        "}
    );
}

#[test]
#[serial]
fn log_with_attributes() {
    let exporter = set_exporter();

    {
        veecle_telemetry::debug!("debug message", key1 = "value1", key2 = 42);
        veecle_telemetry::info!("info message");
        veecle_telemetry::warn!("warning message", error_code = 404);
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {r#"
            + log: [Debug] debug message [key1="value1", key2=42]
            + log: [Info] info message []
            + log: [Warn] warning message [error_code=404]
        "#}
    );
}

#[test]
#[serial]
fn log_attribute_syntax_variations() {
    let exporter = set_exporter();

    {
        let my_var = "test_value";

        // Test identifier only syntax
        veecle_telemetry::log!(Severity::Info, "test identifier", my_var);
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {r#"
            + log: [Info] test identifier [my_var="test_value"]
        "#}
    );

    // Test literal key syntax
    let exporter = set_exporter();
    {
        veecle_telemetry::log!(Severity::Warn, "test literal key", "literal_key" = 123);
    }

    let graph = format_telemetry_tree(exporter.take_messages());
    assert_eq!(
        graph,
        indoc! {"
            + log: [Warn] test literal key [literal_key=123]
        "}
    );
}
