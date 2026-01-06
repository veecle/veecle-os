use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::protocol::owned::{
    InstanceMessage, KeyValue, LogMessage, SpanAddEventMessage, SpanAddLinkMessage,
    SpanCreateMessage, SpanId, SpanSetAttributeMessage, TelemetryMessage, ThreadId, TracingMessage,
};

struct CreateAndParent {
    parent: Option<SpanId>,
    span_create: SpanCreateMessage,
}

struct TelemetryData {
    spans: Vec<CreateAndParent>,
    links: BTreeMap<Option<SpanId>, Vec<SpanAddLinkMessage>>,
    attributes: BTreeMap<Option<SpanId>, Vec<SpanSetAttributeMessage>>,
    events: BTreeMap<Option<SpanId>, Vec<SpanAddEventMessage>>,
    logs: BTreeMap<Option<SpanId>, Vec<LogMessage>>,
    execution_contexts: BTreeMap<ThreadId, Vec<SpanId>>,
}

impl TelemetryData {
    fn context_for(&mut self, thread_id: ThreadId) -> &mut Vec<SpanId> {
        self.execution_contexts.entry(thread_id).or_default()
    }

    fn current_span_for(&mut self, thread_id: ThreadId) -> Option<SpanId> {
        self.context_for(thread_id).last().cloned()
    }
}

pub fn format_telemetry_tree(messages: Vec<InstanceMessage>) -> String {
    let mut telemetry_data = TelemetryData {
        spans: Vec::new(),
        events: BTreeMap::new(),
        links: BTreeMap::new(),
        attributes: BTreeMap::new(),
        logs: BTreeMap::new(),
        execution_contexts: BTreeMap::new(),
    };

    for message in messages {
        match message.message {
            TelemetryMessage::Tracing(TracingMessage::CreateSpan(span_create)) => {
                let parent = telemetry_data.current_span_for(message.thread_id);
                telemetry_data.spans.push(CreateAndParent {
                    parent,
                    span_create,
                });
            }
            TelemetryMessage::Tracing(TracingMessage::EnterSpan(span_enter)) => {
                telemetry_data
                    .context_for(message.thread_id)
                    .push(span_enter.span_id);
            }
            TelemetryMessage::Tracing(TracingMessage::ExitSpan(span_exit)) => {
                let expected = telemetry_data.context_for(message.thread_id).pop();
                assert_eq!(Some(span_exit.span_id), expected);
            }
            TelemetryMessage::Tracing(TracingMessage::AddEvent(event)) => {
                let span_id = event
                    .span_id
                    .or_else(|| telemetry_data.current_span_for(message.thread_id));
                telemetry_data
                    .events
                    .entry(span_id)
                    .or_default()
                    .push(event);
            }
            TelemetryMessage::Tracing(TracingMessage::AddLink(link)) => {
                let span_id = link
                    .span_id
                    .or_else(|| telemetry_data.current_span_for(message.thread_id));
                telemetry_data.links.entry(span_id).or_default().push(link);
            }
            TelemetryMessage::Tracing(TracingMessage::SetAttribute(attr)) => {
                let span_id = attr
                    .span_id
                    .or_else(|| telemetry_data.current_span_for(message.thread_id));
                telemetry_data
                    .attributes
                    .entry(span_id)
                    .or_default()
                    .push(attr);
            }
            TelemetryMessage::Log(log_msg) => {
                let span_id = telemetry_data.current_span_for(message.thread_id);
                telemetry_data
                    .logs
                    .entry(span_id)
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

fn format_attributes(attrs: &[KeyValue], result: &mut String) {
    for (i, attr) in attrs.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        write!(result, "{attr}").unwrap();
    }
}

fn build_tree_string(
    data: &TelemetryData,
    parent_span_id: Option<SpanId>,
    depth: usize,
    result: &mut String,
) {
    // Find the span with the given `parent_span_id`.
    for span in data
        .spans
        .iter()
        .filter(|span| span.parent == parent_span_id)
    {
        // Add indentation.
        for _ in 0..depth {
            result.push_str("    ");
        }

        // Add span name.
        result.push_str(&span.span_create.name);

        // Add creation attributes in brackets.
        result.push_str(" [");
        format_attributes(&span.span_create.attributes, result);
        result.push_str("]\n");

        // Add span-specific attributes.
        if let Some(span_attrs) = data.attributes.get(&Some(span.span_create.span_id)) {
            for attr_msg in span_attrs {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                writeln!(result, "+ attr: {}", attr_msg.attribute).unwrap();
            }
        }

        // Add span links.
        if let Some(span_links) = data.links.get(&Some(span.span_create.span_id)) {
            for link_msg in span_links {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str(&format!("+ link: span={}\n", link_msg.link));
            }
        }

        // Add span events.
        if let Some(span_events) = data.events.get(&Some(span.span_create.span_id)) {
            for event in span_events {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str("+ event: ");
                result.push_str(event.name.as_ref());
                result.push_str(" [");
                format_attributes(&event.attributes, result);
                result.push_str("]\n");
            }
        }

        // Add span logs.
        if let Some(span_logs) = data.logs.get(&Some(span.span_create.span_id)) {
            for log_msg in span_logs {
                for _ in 0..=depth {
                    result.push_str("    ");
                }
                result.push_str(&format!(
                    "+ log: [{:?}] {} [",
                    log_msg.severity, &log_msg.body
                ));
                format_attributes(&log_msg.attributes, result);
                result.push_str("]\n");
            }
        }

        build_tree_string(data, Some(span.span_create.span_id), depth + 1, result);
    }

    // Add unattached logs (logs without trace/span context) at root level.
    if depth == 0
        && let Some(unattached_logs) = data.logs.get(&None)
    {
        for log_msg in unattached_logs {
            result.push_str(&format!(
                "+ log: [{:?}] {} [",
                log_msg.severity, &log_msg.body
            ));
            format_attributes(&log_msg.attributes, result);
            result.push_str("]\n");
        }
    }
}
