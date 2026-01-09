use core::fmt::Debug;

use super::{Export, ProcessId};

#[cfg(feature = "enable")]
use crate::protocol::transient::{
    InstanceMessage, KeyValue, LogMessage, Severity, SpanAddEventMessage, SpanAddLinkMessage,
    SpanCloseMessage, SpanContext, SpanCreateMessage, SpanEnterMessage, SpanExitMessage, SpanId,
    SpanSetAttributeMessage, TelemetryMessage, ThreadId, TracingMessage,
};

/// The global telemetry collector.
///
/// This structure manages the collection and export of telemetry data.
/// It maintains a unique execution ID, handles trace ID generation, and coordinates with the
/// configured exporter.
///
/// The collector is typically accessed through the [`get_collector`][super::get_collector] function rather
/// than being constructed directly.
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
    now_fn: fn() -> u64,
    thread_id_fn: fn() -> core::num::NonZeroU64,
}

impl Collector {
    pub(super) const fn new(
        process_id: ProcessId,
        exporter: &'static (dyn Export + Sync),
        now_fn: fn() -> u64,
        thread_id_fn: fn() -> core::num::NonZeroU64,
    ) -> Self {
        #[cfg(not(feature = "enable"))]
        let _ = (process_id, exporter, now_fn, thread_id_fn);

        Self {
            #[cfg(feature = "enable")]
            inner: CollectorInner {
                process_id,
                exporter,
                now_fn,
                thread_id_fn,
            },
        }
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn process_id(&self) -> ProcessId {
        self.inner.process_id
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn now(&self) -> u64 {
        (self.inner.now_fn)()
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn thread_id(&self) -> ThreadId {
        ThreadId::from_raw(self.inner.process_id, (self.inner.thread_id_fn)())
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
            start_time_unix_nano: self.now(),
            attributes,
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn enter_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::EnterSpan(SpanEnterMessage {
            span_id,
            time_unix_nano: self.now(),
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn exit_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::ExitSpan(SpanExitMessage {
            span_id,
            time_unix_nano: self.now(),
        }));
    }

    #[inline]
    #[cfg(feature = "enable")]
    pub(crate) fn close_span(&self, span_id: SpanId) {
        self.tracing_message(TracingMessage::CloseSpan(SpanCloseMessage {
            span_id,
            end_time_unix_nano: self.now(),
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
            time_unix_nano: self.now(),
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
            thread_id: self.thread_id(),
            message: TelemetryMessage::Log(LogMessage {
                time_unix_nano: self.now(),
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
            thread_id: self.thread_id(),
            message: TelemetryMessage::Tracing(message),
        });
    }
}
