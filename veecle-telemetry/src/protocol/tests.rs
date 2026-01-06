#![cfg_attr(coverage_nightly, coverage(off))]

use core::num::NonZeroU64;

use crate::protocol::base::{ProcessId, SpanId, ThreadId};
use crate::protocol::{owned, transient};

#[cfg(feature = "alloc")]
#[test]
fn transient_to_owned_conversion() {
    use alloc::string::String;

    // Create some data with non-static lifetime
    let borrowed_name_str = "test_span";

    let owned_key = String::from("test_key");
    let owned_value = String::from("test_value");
    let attribute = transient::KeyValue {
        key: owned_key.as_str(),
        value: transient::Value::String(owned_value.as_str()),
    };

    let attributes = [attribute];
    let span_event = transient::SpanAddEventMessage {
        span_id: Some(SpanId(0)),
        name: borrowed_name_str,
        time_unix_nano: 0,
        attributes: &attributes[..],
    };

    let tracing_message = transient::TracingMessage::AddEvent(span_event);
    let telemetry_message = transient::TelemetryMessage::Tracing(tracing_message);
    let instance_message = transient::InstanceMessage {
        thread_id: ThreadId::from_raw(ProcessId::from_raw(999), NonZeroU64::new(111).unwrap()),
        message: telemetry_message,
    };

    let static_message: owned::InstanceMessage = instance_message.into();

    // Verify the conversion worked - the static message should have the same data
    if let owned::TelemetryMessage::Tracing(owned::TracingMessage::AddEvent(span_event)) =
        &static_message.message
    {
        assert_eq!(&span_event.name, "test_span");
    } else {
        panic!("Expected AddEvent message");
    }
}

#[cfg(feature = "alloc")]
#[test]
fn serde_roundtrip_owned_types() {
    use alloc::string::String;

    // Create an owned message
    let attribute = owned::KeyValue {
        key: String::from("test_key"),
        value: owned::Value::String(String::from("test_value")),
    };

    let span_event = owned::SpanAddEventMessage {
        span_id: Some(SpanId(42)),
        name: String::from("test_event"),
        time_unix_nano: 123456789,
        attributes: alloc::vec![attribute],
    };

    let tracing_message = owned::TracingMessage::AddEvent(span_event);
    let telemetry_message = owned::TelemetryMessage::Tracing(tracing_message);
    let instance_message = owned::InstanceMessage {
        thread_id: ThreadId::from_raw(ProcessId::from_raw(999), NonZeroU64::new(111).unwrap()),
        message: telemetry_message,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&instance_message).expect("serialization failed");

    // Deserialize back (owned types support DeserializeOwned)
    let deserialized: owned::InstanceMessage =
        serde_json::from_str(&json).expect("deserialization failed");

    // Verify the roundtrip
    if let owned::TelemetryMessage::Tracing(owned::TracingMessage::AddEvent(event)) =
        &deserialized.message
    {
        assert_eq!(event.span_id, Some(SpanId(42)));
        assert_eq!(&event.name, "test_event");
        assert_eq!(event.time_unix_nano, 123456789);
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(&event.attributes[0].key, "test_key");
    } else {
        panic!("Expected AddEvent message");
    }
}

#[cfg(feature = "alloc")]
#[test]
fn serde_transient_serialize_owned_deserialize() {
    use alloc::string::String;

    // Create some transient data
    let borrowed_name_str = "test_event";
    let owned_key = String::from("test_key");
    let owned_value = String::from("test_value");

    let attribute = transient::KeyValue {
        key: owned_key.as_str(),
        value: transient::Value::String(owned_value.as_str()),
    };

    let attributes = [attribute];
    let span_event = transient::SpanAddEventMessage {
        span_id: Some(SpanId(42)),
        name: borrowed_name_str,
        time_unix_nano: 123456789,
        attributes: &attributes[..],
    };

    let tracing_message = transient::TracingMessage::AddEvent(span_event);
    let telemetry_message = transient::TelemetryMessage::Tracing(tracing_message);
    let instance_message = transient::InstanceMessage {
        thread_id: ThreadId::from_raw(ProcessId::from_raw(999), NonZeroU64::new(111).unwrap()),
        message: telemetry_message,
    };

    // Serialize transient message to JSON
    let json = serde_json::to_string(&instance_message).expect("serialization failed");

    // Deserialize as owned message
    let deserialized: owned::InstanceMessage =
        serde_json::from_str(&json).expect("deserialization failed");

    // Verify the deserialization
    if let owned::TelemetryMessage::Tracing(owned::TracingMessage::AddEvent(event)) =
        &deserialized.message
    {
        assert_eq!(event.span_id, Some(SpanId(42)));
        assert_eq!(&event.name, "test_event");
        assert_eq!(event.time_unix_nano, 123456789);
        assert_eq!(event.attributes.len(), 1);
        assert_eq!(&event.attributes[0].key, "test_key");
    } else {
        panic!("Expected AddEvent message");
    }
}
