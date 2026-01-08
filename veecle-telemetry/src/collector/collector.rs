//! Core collector types and implementation.

use core::fmt::Debug;

use super::Export;
use crate::protocol::base::ProcessId;
#[cfg(feature = "enable")]
use crate::protocol::transient::InstanceMessage;
#[cfg(feature = "enable")]
use crate::protocol::transient::{
    KeyValue, LogMessage, Severity, SpanAddEventMessage, SpanAddLinkMessage, SpanCloseMessage,
    SpanContext, SpanCreateMessage, SpanEnterMessage, SpanExitMessage, SpanId,
    SpanSetAttributeMessage, TelemetryMessage, ThreadId, TracingMessage,
};
#[cfg(feature = "enable")]
use crate::time::now;

/// The global telemetry collector.
///
/// This structure manages the collection and export of telemetry data.
/// It maintains a unique execution ID, handles trace ID generation, and coordinates with the
/// configured exporter.
///
/// The collector is typically accessed through the [`get_collector`] function rather
/// than being constructed directly.
///
/// [`get_collector`]: super::get_collector
#[derive(Debug)]
pub struct Collector {
    #[cfg(feature = "enable")]
    inner: CollectorInner,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct CollectorInner {
    process_id: ProcessId,

    exporter: &'static (dyn Export + Sync),
}

impl Collector {
    pub(super) const fn new(process_id: ProcessId, exporter: &'static (dyn Export + Sync)) -> Self {
        #[cfg(not(feature = "enable"))]
        {
            let _ = process_id;
            let _ = exporter;
        }
        Self {
            #[cfg(feature = "enable")]
            inner: CollectorInner {
                process_id,
                exporter,
            },
        }
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn process_id(&self) -> ProcessId {
        self.inner.process_id
    }

    /// Collects and exports an external telemetry message.
    ///
    /// This method allows external systems to inject telemetry messages into the
    /// collector pipeline.
    /// The message will be exported using the configured exporter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use core::num::NonZeroU64;
    /// use veecle_telemetry::collector::get_collector;
    /// use veecle_telemetry::protocol::transient::{
    ///     ThreadId,
    ///     ProcessId,
    ///     InstanceMessage,
    ///     TelemetryMessage,
    ///     TimeSyncMessage,
    /// };
    ///
    /// let collector = get_collector();
    /// let message = InstanceMessage {
    ///     thread_id: ThreadId::from_raw(ProcessId::from_raw(1), NonZeroU64::new(1).unwrap()),
    ///     message: TelemetryMessage::TimeSync(TimeSyncMessage {
    ///         local_timestamp: 0,
    ///         since_epoch: 0,
    ///     }),
    /// };
    /// collector.collect_external(message);
    /// ```
    #[inline]
    #[cfg(feature = "enable")]
    pub fn collect_external(&self, message: InstanceMessage<'_>) {
        self.inner.exporter.export(message);
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn new_span<'a>(
        &self,
        span_id: SpanId,
        name: &'a str,
        attributes: &'a [KeyValue<'a>],
    ) {
        self.tracing_message(TracingMessage::CreateSpan(SpanCreateMessage {
            span_id,
            name,
            start_time_unix_nano: now().as_nanos(),
            attributes,
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn enter_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::EnterSpan(SpanEnterMessage {
            span_id,
            time_unix_nano: now().as_nanos(),
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn exit_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::ExitSpan(SpanExitMessage {
            span_id,
            time_unix_nano: now().as_nanos(),
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn close_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::CloseSpan(SpanCloseMessage {
            span_id,
            end_time_unix_nano: now().as_nanos(),
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn span_event<'a>(
        &self,
        span_id: Option<SpanId>,
        name: &'a str,
        attributes: &'a [KeyValue<'a>],
    ) {
        self.tracing_message(TracingMessage::AddEvent(SpanAddEventMessage {
            span_id,
            name,
            time_unix_nano: now().as_nanos(),
            attributes,
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn span_link(&self, span_id: Option<SpanId>, link: SpanContext) {
        self.tracing_message(TracingMessage::AddLink(SpanAddLinkMessage {
            span_id,
            link,
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn span_attribute<'a>(&self, span_id: Option<SpanId>, attribute: KeyValue<'a>) {
        self.tracing_message(TracingMessage::SetAttribute(SpanSetAttributeMessage {
            span_id,
            attribute,
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn log_message<'a>(
        &self,
        severity: Severity,
        body: &'a str,
        attributes: &'a [KeyValue<'a>],
    ) {
        self.inner.exporter.export(InstanceMessage {
            thread_id: ThreadId::current(self.inner.process_id),
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: now().as_nanos(),
                severity,
                body,
                attributes,
            }),
        });
    }

    #[inline]
    #[cfg(feature = "enable")]
    fn tracing_message(&self, message: TracingMessage<'_>) {
        self.inner.exporter.export(InstanceMessage {
            thread_id: ThreadId::current(self.inner.process_id),
            message: TelemetryMessage::Tracing(message),
        });
    }
}
