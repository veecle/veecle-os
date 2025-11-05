//! Store for tracing data.
//!
//! See [`Store`].

use std::collections::HashSet;
use std::fmt::Formatter;
use std::ops::{Add, Deref, Sub};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use anyhow::Context;
use egui::Color32;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use veecle_telemetry::protocol::{
    InstanceMessage, ProcessId, TelemetryMessage, ThreadId, TracingMessage,
};
use veecle_telemetry::{SpanContext, SpanId as TelemetrySpanId, Value as TelemetryValue};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

/// Unique identifier for a log.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LogId(usize);

impl From<usize> for LogId {
    fn from(value: usize) -> Self {
        LogId(value)
    }
}

/// The store parses trace files and collects all logs and traces from them for easy access.
#[derive(Debug)]
pub struct Store {
    spans: IndexMap<SpanContext, Span>,
    root_spans: IndexSet<SpanContext>,
    logs: Vec<Log>,
    actors: HashSet<String>,

    /// The earliest known timestamp.
    ///
    /// Gets initialized to [`Timestamp::MAX`] so the first real timestamp becomes the start.
    pub start: Timestamp,

    /// The latest known timestamp.
    ///
    /// Gets initialized to [`Timestamp::MIN`] so the first real timestamp becomes the end.
    pub end: Timestamp,

    /// When the latest data was recorded.
    ///
    /// Useful to e.g. predict what the "current" timestamp is along with `self.end` with a continuous data source.
    pub last_update: Instant,

    /// Whether we have a continuously updated data source.
    pub continuous: bool,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            spans: IndexMap::default(),
            root_spans: IndexSet::default(),
            logs: Vec::default(),
            actors: HashSet::default(),
            start: Timestamp::MAX,
            end: Timestamp::MIN,
            last_update: Instant::now(),
            continuous: false,
        }
    }
}

// Program span context for logs without a specific span.
const PROGRAM_SPAN_CONTEXT: SpanContext = SpanContext {
    process_id: ProcessId::from_raw(0),
    span_id: TelemetrySpanId(0),
};

impl Store {
    /// Returns a [`SpanRef`] for the given [`SpanContext`].
    pub fn get_span(&self, context: SpanContext) -> Option<SpanRef<'_>> {
        SpanRef::new(self, context)
    }

    /// Returns a [`LogRef`] for the given [`LogId`].
    pub fn get_log(&self, id: LogId) -> Option<LogRef<'_>> {
        LogRef::new(self, id)
    }

    /// Returns an iterator over all root spans in the store.
    pub fn root_spans(&self) -> impl ExactSizeIterator<Item = SpanRef<'_>> {
        self.root_spans
            .iter()
            .map(|context| SpanRef::new(self, *context).expect("referenced span should exist"))
    }

    /// Returns an iterator over all logs in the store.
    pub fn logs(&self) -> impl ExactSizeIterator<Item = LogRef<'_>> {
        self.logs.iter().map(LogRef::new_with_log)
    }

    /// Returns an iterator over all actors that have been seen by the store.
    pub fn actors(&self) -> impl ExactSizeIterator<Item = &str> {
        self.actors.iter().map(String::as_str)
    }

    /// Process a single line from a trace file or piped input.
    pub fn process_line(&mut self, line: &str) -> anyhow::Result<()> {
        if line.is_empty() {
            return Ok(());
        }

        let message: InstanceMessage =
            serde_json::from_str(line).context("parsing instance message")?;
        self.process_message(message);

        Ok(())
    }

    /// Clear all data in the store.
    pub fn clear(&mut self) {
        self.spans.clear();
        self.root_spans.clear();
        self.logs.clear();
        self.actors.clear();

        self.start = Timestamp::MAX;
        self.end = Timestamp::MIN;

        self.continuous = false;
    }

    /// Inserts a program span which is used as a fallback span for logs which are not related to a span.
    ///
    /// This function is idempotent, it can be called at any time to ensure the program span exists.
    fn ensure_program_span(&mut self) {
        let unknown_actor = "None".to_string();

        if !self.spans.contains_key(&PROGRAM_SPAN_CONTEXT) {
            // program span to add logs without a span to.
            self.spans.insert(
                PROGRAM_SPAN_CONTEXT,
                Span {
                    context: PROGRAM_SPAN_CONTEXT,
                    fields: Default::default(),
                    parent: None,
                    children: vec![],
                    links: vec![],
                    logs: vec![],
                    start: Timestamp::from_ns(0),
                    end: Timestamp::from_ns(0),
                    actor: unknown_actor,
                    activity: vec![],
                    metadata: Metadata {
                        name: "program".to_string(),
                        target: "program".to_string(),
                        level: Level::Info,
                        file: None,
                    },
                },
            );
        }
    }

    /// Processes a single message from a trace file.
    ///
    /// Can also be used to process streaming data live.
    pub fn process_message(&mut self, instance_message: InstanceMessage) {
        // ensure the program span is inserted.
        self.ensure_program_span();

        let InstanceMessage {
            // TODO(DEV-605): support filtering by thread.
            thread,
            message,
        } = instance_message;

        match message {
            TelemetryMessage::Tracing(tracing_msg) => {
                self.process_tracing_message(thread, tracing_msg);
            }
            TelemetryMessage::Log(log_msg) => {
                self.process_log_message(thread, log_msg);
            }
            TelemetryMessage::TimeSync(_) => {
                // TODO(DEV-601): handle these messages.
            }
        }
    }

    /// Helper function to update timestamp bounds and last update time.
    fn update_timestamp(&mut self, time_unix_nano: u64) -> Timestamp {
        let timestamp = Timestamp::from_ns(time_unix_nano as i64);
        self.start = self.start.min(timestamp);
        self.end = self.end.max(timestamp);
        self.last_update = Instant::now();
        timestamp
    }

    /// Processes a single tracing message.
    fn process_tracing_message(&mut self, thread_id: ThreadId, tracing_msg: TracingMessage) {
        match tracing_msg {
            TracingMessage::CreateSpan(span_msg) => {
                let timestamp = self.update_timestamp(span_msg.start_time_unix_nano);

                let context = SpanContext::new(thread_id.process, span_msg.span_id);
                let parent_context = span_msg
                    .parent_span_id
                    .map(|parent_id| SpanContext::new(thread_id.process, parent_id));

                let mut parent_span = parent_context.map(|parent| {
                    self.spans
                        .get_mut(&parent)
                        .expect("parent span should exist")
                });

                if let Some(parent_span) = &mut parent_span {
                    parent_span.children.push(context);
                } else {
                    self.root_spans.insert(context);
                }

                let fields: IndexMap<String, Value> = span_msg
                    .attributes
                    .iter()
                    .map(|kv| (kv.key.as_ref().to_string(), Value::from(kv.value.clone())))
                    .collect();

                let actor = fields
                    .get("actor")
                    .and_then(|value| value.as_str())
                    .or_else(|| Some(&parent_span?.actor))
                    .unwrap_or("None")
                    .to_string();

                self.actors.insert(actor.clone());

                let metadata = Metadata {
                    name: span_msg.name.as_ref().to_string(),
                    target: "unknown".to_string(),
                    // Default level
                    level: Level::Info,
                    file: None,
                };

                let mut span = Span::new(context, parent_context, actor, metadata, fields);
                span.start = timestamp;
                span.end = Timestamp::MAX;
                self.spans.insert(context, span);
            }
            TracingMessage::EnterSpan(enter_msg) => {
                let timestamp = self.update_timestamp(enter_msg.time_unix_nano);

                let span_context = SpanContext::new(thread_id.process, enter_msg.span_id);

                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");

                span.start = span.start.min(timestamp);
                span.activity.push(ActivityPeriod {
                    start: timestamp,
                    end: Timestamp::MAX,
                });
            }
            TracingMessage::ExitSpan(exit_msg) => {
                let timestamp = self.update_timestamp(exit_msg.time_unix_nano);

                let span_context = SpanContext::new(thread_id.process, exit_msg.span_id);
                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");

                let activity = span
                    .activity
                    .last_mut()
                    .expect("an activity period should have been created");
                activity.end = timestamp;

                // Update the "fake" program span.
                let program_span = self
                    .spans
                    .get_mut(&PROGRAM_SPAN_CONTEXT)
                    .expect("program span should exist");
                program_span.end = program_span.end.max(timestamp);
            }
            TracingMessage::CloseSpan(close_msg) => {
                let timestamp = self.update_timestamp(close_msg.end_time_unix_nano);

                let span_context = SpanContext::new(thread_id.process, close_msg.span_id);
                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");

                // A span can be kept alive beyond the end of its used lifetime for various reasons. The end should
                // be left back at the last exit. This has a slight weirdness in a continuous UI that it will "snap
                // back" to the last exit, but most reasons for keeping a span alive should resolve within a few ms and
                // be unnoticeable.
                span.end = span
                    .activity
                    .last()
                    .map_or(timestamp, |activity| activity.end);
            }
            TracingMessage::AddEvent(event_msg) => {
                let timestamp = self.update_timestamp(event_msg.time_unix_nano);

                let span_context = SpanContext::new(thread_id.process, event_msg.span_id);
                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");

                let id = LogId(self.logs.len());
                span.logs.push(id);

                let fields: IndexMap<String, Value> = event_msg
                    .attributes
                    .iter()
                    .map(|kv| (kv.key.as_ref().to_string(), Value::from(kv.value.clone())))
                    .collect();

                let message = event_msg.name.as_ref().to_string();

                // TODO(DEV-584): add file or module path.
                let metadata = Metadata {
                    name: event_msg.name.as_ref().to_string(),
                    target: "unknown".to_string(),
                    level: Level::Info,
                    file: None,
                };

                self.logs.push(Log {
                    id,
                    span_context,
                    fields,
                    body: message,
                    actor: span.actor.clone(),
                    metadata,
                    timestamp,
                });
            }
            TracingMessage::AddLink(link_msg) => {
                let span_context = SpanContext::new(thread_id.process, link_msg.span_id);
                let linked_span_context = link_msg.link;

                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");
                span.links.push(linked_span_context);
            }
            TracingMessage::SetAttribute(attr_msg) => {
                let span_context = SpanContext::new(thread_id.process, attr_msg.span_id);
                let span = self
                    .spans
                    .get_mut(&span_context)
                    .expect("span should exist");

                let key = attr_msg.attribute.key.as_ref().to_string();
                let value = Value::from(attr_msg.attribute.value);
                span.fields.insert(key, value);
            }
        }
    }

    /// Processes a single log message.
    fn process_log_message(
        &mut self,
        thread_id: ThreadId,
        log_msg: veecle_telemetry::protocol::LogMessage,
    ) {
        let timestamp = self.update_timestamp(log_msg.time_unix_nano);

        // Find the span this log belongs to, or use the program span.
        let span_context = log_msg
            .span_id
            .map(|span_id| SpanContext::new(thread_id.process, span_id))
            .unwrap_or(PROGRAM_SPAN_CONTEXT);

        let span = if let Some(span) = self.spans.get_mut(&span_context) {
            span
        } else {
            self.spans
                .get_mut(&PROGRAM_SPAN_CONTEXT)
                .expect("program span should exist")
        };

        let id = LogId(self.logs.len());
        span.logs.push(id);

        let fields: IndexMap<String, Value> = log_msg
            .attributes
            .iter()
            .map(|kv| (kv.key.as_ref().to_string(), Value::from(kv.value.clone())))
            .collect();

        let message = log_msg.body.as_ref().to_string();

        // TODO(DEV-584): add file or module path.
        let metadata = Metadata {
            name: "log".to_string(),
            target: "unknown".to_string(),
            level: log_msg.severity.into(),
            file: None,
        };

        self.logs.push(Log {
            id,
            span_context,
            fields,
            body: message,
            actor: span.actor.clone(),
            metadata,
            timestamp,
        });
    }
}

/// A reference to a span in the store with accessors for parent, children and logs.
#[derive(Debug, Copy, Clone)]
pub struct SpanRef<'a> {
    store: &'a Store,
    span: &'a Span,
}

impl<'a> SpanRef<'a> {
    fn new(store: &'a Store, context: SpanContext) -> Option<Self> {
        Some(Self {
            store,
            span: store.spans.get(&context)?,
        })
    }

    /// Returns an iterator with [`SpanRef`]s of all child spans.
    pub fn children(&self) -> impl ExactSizeIterator<Item = SpanRef<'a>> {
        self.span
            .children
            .iter()
            .map(|context| Self::new(self.store, *context).expect("referenced child should exist"))
    }

    /// Returns an iterator with [`LogRef`]s of all logs.
    pub fn logs(&self) -> impl ExactSizeIterator<Item = LogRef<'a>> {
        self.span
            .logs
            .iter()
            .map(|id| LogRef::new(self.store, *id).expect("referenced log should exist"))
    }
}

impl Deref for SpanRef<'_> {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        self.span
    }
}

/// A reference to a log in the store with an accessor for the containing span.
#[derive(Debug, Copy, Clone)]
pub struct LogRef<'a> {
    log: &'a Log,
}

impl<'a> LogRef<'a> {
    fn new(store: &'a Store, id: LogId) -> Option<Self> {
        Some(Self {
            log: store.logs.get(id.0)?,
        })
    }

    fn new_with_log(log: &'a Log) -> Self {
        Self { log }
    }
}

impl Deref for LogRef<'_> {
    type Target = Log;

    fn deref(&self) -> &Self::Target {
        self.log
    }
}

/// Represents a [`veecle_telemetry`] Span.
#[derive(Debug, Clone)]
pub struct Span {
    /// See [`SpanContext`].
    pub context: SpanContext,

    /// Fields added to the span.
    pub fields: IndexMap<String, Value>,

    /// The parent span's context.
    pub parent: Option<SpanContext>,
    /// The span's children.
    pub children: Vec<SpanContext>,
    /// Related spans this span is linked to.
    pub links: Vec<SpanContext>,
    /// Logs that happened during this span.
    pub logs: Vec<LogId>,

    /// The timestamp when this span started.
    pub start: Timestamp,
    /// The timestamp when this span ended.
    pub end: Timestamp,

    /// The actor this span is in.
    pub actor: String,

    /// List of activity periods for this span.
    ///
    /// If this span was part of an async operation, it may have multiple periods separated by suspensions.
    pub activity: Vec<ActivityPeriod>,

    /// Span metadata.
    pub metadata: Metadata,
}

impl Span {
    fn new(
        context: SpanContext,
        parent: Option<SpanContext>,
        actor: String,
        metadata: impl Into<Metadata>,
        fields: IndexMap<String, Value>,
    ) -> Self {
        let metadata = metadata.into();

        Self {
            context,
            fields,
            parent,
            children: vec![],
            links: vec![],
            logs: vec![],
            start: Timestamp::MAX,
            end: Timestamp::MIN,
            actor,
            activity: vec![],
            metadata,
        }
    }

    /// Returns the total duration of the spans.
    ///
    /// Note: this does not subtract periods of inactivity.
    pub fn duration(&self) -> Timestamp {
        if self.start == Timestamp::MAX && self.end == Timestamp::MIN {
            // No data loaded yet, return zero duration.
            Timestamp::from_ns(0)
        } else {
            self.end - self.start
        }
    }

    /// Returns the total duration of the span in ms.
    ///
    /// Note: this does not subtract periods of inactivity.
    pub fn duration_ms(&self) -> f64 {
        self.duration().as_ms()
    }
}

/// The possible values a span field may contain.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// A [`String`].
    Str(String),
    /// A [`f64`].
    F64(f64),
    /// An [`i64`].
    I64(i64),
    /// An [`u64`].
    U64(u64),
    /// An [`i128`].
    I128(i128),
    /// An [`u128`].
    U128(u128),
    /// A [`bool`].
    Bool(bool),
}

impl Value {
    /// Returns a string slice if the value is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => std::fmt::Display::fmt(value, f),
            Value::F64(value) => std::fmt::Display::fmt(value, f),
            Value::I64(value) => std::fmt::Display::fmt(value, f),
            Value::U64(value) => std::fmt::Display::fmt(value, f),
            Value::I128(value) => std::fmt::Display::fmt(value, f),
            Value::U128(value) => std::fmt::Display::fmt(value, f),
            Value::Bool(value) => std::fmt::Display::fmt(value, f),
        }
    }
}

impl From<TelemetryValue<'_>> for Value {
    fn from(value: TelemetryValue) -> Self {
        match value {
            TelemetryValue::String(s) => Value::Str(s.as_ref().to_string()),
            TelemetryValue::Bool(b) => Value::Bool(b),
            TelemetryValue::I64(i) => Value::I64(i),
            TelemetryValue::F64(f) => Value::F64(f),
        }
    }
}

/// Represents a period of activity within a span.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ActivityPeriod {
    /// The timestamp when this period started.
    pub start: Timestamp,
    /// The timestamp when this period ended.
    pub end: Timestamp,
}

/// Represents a log entry in the trace.
///
/// A log entry is associated with a span and contains a message
/// along with metadata such as level and timestamp.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Log {
    /// See [LogId].
    pub id: LogId,
    /// The span this log is part of.
    pub span_context: SpanContext,

    /// Fields added to the span.
    pub fields: IndexMap<String, Value>,

    /// Log message.
    pub body: String,
    /// Actor this was logged in.
    pub actor: String,

    /// Span metadata.
    pub metadata: Metadata,

    /// Timestamp.
    pub timestamp: Timestamp,
}

/// Metadata associated with a span or log entry.
///
/// Contains information about where and how the span or log was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// The name of the span described by this metadata.
    pub name: String,

    /// The part of the system that the span that this metadata describes
    /// occurred in.
    pub target: String,

    /// The verbosity level of the described span.
    pub level: Level,

    /// The name of the source code file where the span occurred, or `None` if
    /// this could not be determined.
    pub file: Option<String>,
}

/// Describes the level of verbosity of a log, span or event.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl From<veecle_telemetry::protocol::Severity> for Level {
    fn from(value: veecle_telemetry::protocol::Severity) -> Self {
        Level::from(&value)
    }
}

impl From<&veecle_telemetry::protocol::Severity> for Level {
    fn from(value: &veecle_telemetry::protocol::Severity) -> Self {
        match *value {
            veecle_telemetry::protocol::Severity::Error => Level::Error,
            veecle_telemetry::protocol::Severity::Warn => Level::Warn,
            veecle_telemetry::protocol::Severity::Info => Level::Info,
            veecle_telemetry::protocol::Severity::Debug => Level::Debug,
            veecle_telemetry::protocol::Severity::Trace => Level::Trace,
            veecle_telemetry::protocol::Severity::Fatal => Level::Error,
        }
    }
}

impl Level {
    /// Returns an ANSI string representation of the level for logging purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    /// Returns an egui color representation of the log level.
    pub fn color(&self) -> Color32 {
        match self {
            Level::Error => Color32::from_rgb(0xDD, 0x2D, 0x35),
            Level::Warn => Color32::from_rgb(0xBC, 0x5F, 0x00),
            Level::Info => Color32::from_rgb(0x40, 0x92, 0x0A),
            Level::Debug => Color32::from_rgb(0x38, 0x73, 0xEE),
            Level::Trace => Color32::from_rgb(0x00, 0x8B, 0xA1),
        }
    }
}

/// A timestamp internally represented as nanoseconds.
// TODO: investigate if i64 is big enough.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// The largest value that can be represented by this timestamp type.
    pub const MAX: Timestamp = Timestamp(i64::MAX);
    /// The smallest value that can be represented by this timestamp type.
    pub const MIN: Timestamp = Timestamp(i64::MIN);

    /// Create a timestamp from a raw value in nanoseconds.
    pub fn from_ns(value: i64) -> Self {
        Self(value)
    }

    /// Returns the timestamp as milliseconds.
    pub fn as_ms(&self) -> f64 {
        self.0 as f64 * 1e-6
    }

    /// Returns the timestamp as nanoseconds.
    pub fn as_ns(&self) -> i64 {
        self.0
    }
}

impl Add<Timestamp> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Timestamp) -> Self::Output {
        Timestamp(self.0 + rhs.0)
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

impl From<TimestampF> for Timestamp {
    fn from(value: TimestampF) -> Self {
        Timestamp(value.as_ns() as i64)
    }
}

/// A timestamp internally represented as nanoseconds.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct TimestampF(f64);

impl TimestampF {
    /// Create a timestamp from a raw value in nanoseconds.
    pub fn from_ns(value: f64) -> Self {
        Self(value)
    }

    /// Returns the timestamp as nanoseconds.
    pub fn as_ns(&self) -> f64 {
        self.0
    }
}

impl Add<TimestampF> for TimestampF {
    type Output = TimestampF;

    fn add(self, rhs: TimestampF) -> Self::Output {
        TimestampF(self.0 + rhs.0)
    }
}

impl Sub<TimestampF> for TimestampF {
    type Output = TimestampF;

    fn sub(self, rhs: TimestampF) -> Self::Output {
        TimestampF(self.0 - rhs.0)
    }
}

impl From<Timestamp> for TimestampF {
    fn from(value: Timestamp) -> Self {
        TimestampF(value.as_ns() as f64)
    }
}
