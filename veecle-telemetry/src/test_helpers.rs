use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::protocol::{
    InstanceMessage, LogMessage, SpanAddEventMessage, SpanAddLinkMessage, SpanCreateMessage,
    SpanSetAttributeMessage, TelemetryMessage, TracingMessage,
};
use crate::value::Value;
use crate::{SpanId, TraceId};

struct TelemetryData<'a> {
    spans: Vec<SpanCreateMessage<'a>>,
    links: BTreeMap<(TraceId, SpanId), Vec<SpanAddLinkMessage>>,
    attributes: BTreeMap<(TraceId, SpanId), Vec<SpanSetAttributeMessage<'a>>>,
    events: BTreeMap<(TraceId, SpanId), Vec<SpanAddEventMessage<'a>>>,
    logs: BTreeMap<(Option<TraceId>, Option<SpanId>), Vec<LogMessage<'a>>>,
}

pub fn format_telemetry_tree(messages: Vec<InstanceMessage>) -> String {
    let mut telemetry_data = TelemetryData {
        spans: Vec::new(),
        events: BTreeMap::new(),
        links: BTreeMap::new(),
        attributes: BTreeMap::new(),
        logs: BTreeMap::new(),
    };

    for message in messages {
        match message.message {
            TelemetryMessage::Tracing(TracingMessage::CreateSpan(span_create)) => {
                telemetry_data.spans.push(span_create);
            }
            TelemetryMessage::Tracing(TracingMessage::AddEvent(event)) => {
                telemetry_data
                    .events
                    .entry((event.trace_id, event.span_id))
                    .or_default()
                    .push(event);
            }
            TelemetryMessage::Tracing(TracingMessage::AddLink(link)) => {
                telemetry_data
                    .links
                    .entry((link.trace_id, link.span_id))
                    .or_default()
                    .push(link);
            }
            TelemetryMessage::Tracing(TracingMessage::SetAttribute(attr)) => {
                telemetry_data
                    .attributes
                    .entry((attr.trace_id, attr.span_id))
                    .or_default()
                    .push(attr);
            }
            TelemetryMessage::Log(log_msg) => {
                telemetry_data
                    .logs
                    .entry((log_msg.trace_id, log_msg.span_id))
                    .or_default()
                    .push(log_msg);
            }
            _ => {}
        }
    }

    let mut result = String::new();
    build_tree_string(&telemetry_data, None, 0, &mut result);
    result
}

fn format_attribute_value(value: &Value, result: &mut String) {
    match value {
        Value::String(s) => {
            result.push('"');
            result.push_str(s);
            result.push('"');
        }
        Value::Bool(b) => {
            result.push_str(&format!("{b}"));
        }
        Value::I64(i) => {
            result.push_str(&format!("{i}"));
        }
        Value::F64(f) => {
            result.push_str(&format!("{f}"));
        }
    }
}

fn format_attributes<'a, I>(attrs: I, result: &mut String)
where
    I: Iterator<Item = &'a crate::value::KeyValue<'a>>,
{
    for (i, attr) in attrs.enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(&attr.key);
        result.push('=');
        format_attribute_value(&attr.value, result);
    }
}

fn build_tree_string(
    data: &TelemetryData,
    parent_span_id: Option<SpanId>,
    depth: usize,
    result: &mut String,
) {
    // Find the span with the given span_id
    for span in data
        .spans
        .iter()
        .filter(|s| s.parent_span_id == parent_span_id)
    {
        // Add indentation
        for _ in 0..depth {
            result.push_str("    ");
        }

        // Add span name
        result.push_str(&span.name);

        // Add creation attributes in brackets
        result.push_str(" [");
        format_attributes(span.attributes.iter(), result);
        result.push_str("]\n");

        // Add span-specific attributes
        if let Some(span_attrs) = data.attributes.get(&(span.trace_id, span.span_id)) {
            for attr_msg in span_attrs {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str("+ attr: ");
                result.push_str(&attr_msg.attribute.key);
                result.push('=');
                format_attribute_value(&attr_msg.attribute.value, result);
                result.push('\n');
            }
        }

        // Add span links
        if let Some(span_links) = data.links.get(&(span.trace_id, span.span_id)) {
            for link_msg in span_links {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str(&format!(
                    "+ link: trace={:x}, span={:x}\n",
                    link_msg.link.trace_id.0, link_msg.link.span_id.0
                ));
            }
        }

        // Add span events
        if let Some(span_events) = data.events.get(&(span.trace_id, span.span_id)) {
            for event in span_events {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str("+ event: ");
                result.push_str(&event.name);
                result.push_str(" [");
                format_attributes(event.attributes.iter(), result);
                result.push_str("]\n");
            }
        }

        // Add span logs
        if let Some(span_logs) = data.logs.get(&(Some(span.trace_id), Some(span.span_id))) {
            for log_msg in span_logs {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str(&format!(
                    "+ log: [{:?}] {} [",
                    log_msg.severity, log_msg.body
                ));
                format_attributes(log_msg.attributes.iter(), result);
                result.push_str("]\n");
            }
        }

        build_tree_string(data, Some(span.span_id), depth + 1, result);
    }

    // Add unattached logs (logs without trace/span context) at root level
    if depth == 0
        && let Some(unattached_logs) = data.logs.get(&(None, None))
    {
        for log_msg in unattached_logs {
            result.push_str(&format!(
                "+ log: [{:?}] {} [",
                log_msg.severity, log_msg.body
            ));
            format_attributes(log_msg.attributes.iter(), result);
            result.push_str("]\n");
        }
    }
}
