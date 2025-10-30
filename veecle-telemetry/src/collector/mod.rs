//! Telemetry data collection and export infrastructure.
//!
//! This module provides the core infrastructure for collecting telemetry data and exporting it
//! to various backends.
//! It includes the global collector singleton, export trait, and various
//! built-in exporters.
//!
//! # Global Collector
//!
//! The collector uses a global singleton pattern to ensure telemetry data is collected
//! consistently across the entire application.
//! The collector must be initialized once
//! using [`set_exporter`] before any telemetry data can be collected.
//!
//! # Export Trait
//!
//! The [`Export`] trait defines the interface for exporting telemetry data.
//! Custom exporters can be implemented by providing an implementation of this trait.
//!
//! # Built-in Exporters
//!
//! - [`ConsoleJsonExporter`] - Exports telemetry data as JSON to stdout
//! - [`TestExporter`] - Collects telemetry data in memory for testing purposes

#[cfg(feature = "std")]
mod json_exporter;
#[cfg(feature = "std")]
mod pretty_exporter;
#[cfg(feature = "std")]
mod test_exporter;

use core::fmt::Debug;
#[cfg(feature = "enable")]
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use core::{error, fmt};

#[cfg(feature = "std")]
pub use json_exporter::ConsoleJsonExporter;
#[cfg(feature = "std")]
pub use pretty_exporter::ConsolePrettyExporter;
#[cfg(feature = "std")]
#[doc(hidden)]
pub use test_exporter::TestExporter;

use crate::TraceId;
#[cfg(feature = "enable")]
use crate::protocol::ExecutionId;
use crate::protocol::InstanceMessage;
#[cfg(feature = "enable")]
use crate::protocol::{
    LogMessage, SpanAddEventMessage, SpanAddLinkMessage, SpanCloseMessage, SpanCreateMessage,
    SpanEnterMessage, SpanExitMessage, SpanSetAttributeMessage, TelemetryMessage, TracingMessage,
};

/// Trait for exporting telemetry data to external systems.
///
/// Implementors of this trait define how telemetry data should be exported,
/// whether to files, network endpoints, or other destinations.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::collector::Export;
/// use veecle_telemetry::protocol::InstanceMessage;
///
/// #[derive(Debug)]
/// struct CustomExporter;
///
/// impl Export for CustomExporter {
///     fn export(&self, message: InstanceMessage<'_>) {
///         // Custom export logic here
///         println!("Exporting: {:?}", message);
///     }
/// }
/// ```
pub trait Export: Debug {
    /// Exports a telemetry message.
    ///
    /// This method is called for each telemetry message that needs to be exported.
    /// The implementation should handle the message appropriately based on its type.
    fn export(&self, message: InstanceMessage<'_>);
}

/// The global telemetry collector.
///
/// This structure manages the collection and export of telemetry data.
/// It maintains a unique execution ID, handles trace ID generation, and coordinates with the
/// configured exporter.
///
/// The collector is typically accessed through the [`get_collector`] function rather
/// than being constructed directly.
#[derive(Debug)]
pub struct Collector {
    #[cfg(feature = "enable")]
    inner: CollectorInner,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct CollectorInner {
    execution_id: ExecutionId,

    exporter: &'static (dyn Export + Sync),

    trace_id_prefix: u64,
    trace_id_counter: AtomicU64,
}

#[cfg(feature = "enable")]
#[derive(Debug)]
struct NopExporter;

#[cfg(feature = "enable")]
impl Export for NopExporter {
    fn export(&self, _: InstanceMessage) {}
}

// The GLOBAL_COLLECTOR static holds a pointer to the global exporter. It is protected by
// the GLOBAL_INIT static which determines whether GLOBAL_EXPORTER has been initialized.
#[cfg(feature = "enable")]
static mut GLOBAL_COLLECTOR: Collector = Collector {
    inner: CollectorInner {
        execution_id: ExecutionId::from_raw(0),
        exporter: &NO_EXPORTER,

        trace_id_prefix: 0,
        trace_id_counter: AtomicU64::new(0),
    },
};
static NO_COLLECTOR: Collector = Collector {
    #[cfg(feature = "enable")]
    inner: CollectorInner {
        execution_id: ExecutionId::from_raw(0),
        exporter: &NO_EXPORTER,

        trace_id_prefix: 0,
        trace_id_counter: AtomicU64::new(0),
    },
};
#[cfg(feature = "enable")]
static NO_EXPORTER: NopExporter = NopExporter;

#[cfg(feature = "enable")]
static GLOBAL_INIT: AtomicUsize = AtomicUsize::new(0);

// There are three different states that we care about:
// - the collector is uninitialized
// - the collector is initializing (set_exporter has been called but GLOBAL_COLLECTOR hasn't been set yet)
// - the collector is active
#[cfg(feature = "enable")]
const UNINITIALIZED: usize = 0;
#[cfg(feature = "enable")]
const INITIALIZING: usize = 1;
#[cfg(feature = "enable")]
const INITIALIZED: usize = 2;

/// Initializes the collector with the given Exporter and [`ExecutionId`].
///
/// An [`ExecutionId`] should never be re-used as it's used to collect metadata about the execution and to generate
/// [`TraceId`]s which need to be globally unique.
#[cfg(feature = "enable")]
pub fn set_exporter(
    execution_id: ExecutionId,
    exporter: &'static (dyn Export + Sync),
) -> Result<(), SetExporterError> {
    if GLOBAL_INIT
        .compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
        .is_ok()
    {
        // SAFETY: this is guarded by the atomic
        unsafe { GLOBAL_COLLECTOR = Collector::new(execution_id, exporter) }
        GLOBAL_INIT.store(INITIALIZED, Ordering::Release);

        Ok(())
    } else {
        Err(SetExporterError(()))
    }
}

/// Returns a reference to the collector.
///
/// If an exporter has not been set, a no-op implementation is returned.
pub fn get_collector() -> &'static Collector {
    #[cfg(not(feature = "enable"))]
    {
        &NO_COLLECTOR
    }

    // Acquire memory ordering guarantees that current thread would see any
    // memory writes that happened before store of the value
    // into `GLOBAL_INIT` with memory ordering `Release` or stronger.
    //
    // Since the value `INITIALIZED` is written only after `GLOBAL_COLLECTOR` was
    // initialized, observing it after `Acquire` load here makes both
    // write to the `GLOBAL_COLLECTOR` static and initialization of the exporter
    // internal state synchronized with current thread.
    #[cfg(feature = "enable")]
    if GLOBAL_INIT.load(Ordering::Acquire) != INITIALIZED {
        &NO_COLLECTOR
    } else {
        // SAFETY: this is guarded by the atomic
        unsafe {
            #[expect(clippy::deref_addrof, reason = "false positive")]
            &*&raw const GLOBAL_COLLECTOR
        }
    }
}

/// The type returned by [`set_exporter`] if [`set_exporter`] has already been called.
///
/// [`set_exporter`]: fn.set_exporter.html
#[derive(Debug)]
pub struct SetExporterError(());

impl SetExporterError {
    const MESSAGE: &'static str = "a global exporter has already been set";
}

impl fmt::Display for SetExporterError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(Self::MESSAGE)
    }
}

impl error::Error for SetExporterError {}

impl Collector {
    #[cfg(feature = "enable")]
    fn new(execution_id: ExecutionId, exporter: &'static (dyn Export + Sync)) -> Self {
        let execution_id_raw = *execution_id;
        let trace_id_prefix = (execution_id_raw >> 64) as u64;
        let initial_counter_value = execution_id_raw as u64;

        Self {
            inner: CollectorInner {
                execution_id,
                exporter,
                trace_id_prefix,
                trace_id_counter: AtomicU64::new(initial_counter_value),
            },
        }
    }

    #[inline]
    pub(crate) fn generate_trace_id(&self) -> TraceId {
        #[cfg(not(feature = "enable"))]
        {
            TraceId(0)
        }

        #[cfg(feature = "enable")]
        if self.inner.trace_id_prefix == 0 {
            TraceId(0)
        } else {
            let suffix = self.inner.trace_id_counter.fetch_add(1, Ordering::Relaxed);

            TraceId(((self.inner.trace_id_prefix as u128) << 32) | (suffix as u128))
        }
    }
}

#[cfg(feature = "enable")]
impl Collector {
    /// Collects and exports an external telemetry message.
    ///
    /// This method allows external systems to inject telemetry messages into the
    /// collector pipeline.
    /// The message will be exported using the configured exporter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use veecle_telemetry::collector::get_collector;
    /// use veecle_telemetry::protocol::{
    ///     ExecutionId,
    ///     InstanceMessage,
    ///     TelemetryMessage,
    ///     TimeSyncMessage,
    /// };
    ///
    /// let collector = get_collector();
    /// let message = InstanceMessage {
    ///     execution: ExecutionId::from_raw(1),
    ///     message: TelemetryMessage::TimeSync(TimeSyncMessage {
    ///         local_timestamp: 0,
    ///         since_epoch: 0,
    ///     }),
    /// };
    /// collector.collect_external(message);
    /// ```
    #[inline]
    pub fn collect_external(&self, message: InstanceMessage<'_>) {
        self.inner.exporter.export(message);
    }

    #[inline]
    pub(crate) fn new_span(&self, span: SpanCreateMessage<'_>) {
        self.tracing_message(TracingMessage::CreateSpan(span));
    }

    #[inline]
    pub(crate) fn enter_span(&self, enter: SpanEnterMessage) {
        self.tracing_message(TracingMessage::EnterSpan(enter));
    }

    #[inline]
    pub(crate) fn exit_span(&self, exit: SpanExitMessage) {
        self.tracing_message(TracingMessage::ExitSpan(exit));
    }

    #[inline]
    pub(crate) fn close_span(&self, span: SpanCloseMessage) {
        self.tracing_message(TracingMessage::CloseSpan(span));
    }

    #[inline]
    pub(crate) fn span_event(&self, event: SpanAddEventMessage<'_>) {
        self.tracing_message(TracingMessage::AddEvent(event));
    }

    #[inline]
    pub(crate) fn span_link(&self, link: SpanAddLinkMessage) {
        self.tracing_message(TracingMessage::AddLink(link));
    }

    #[inline]
    pub(crate) fn span_attribute(&self, attribute: SpanSetAttributeMessage<'_>) {
        self.tracing_message(TracingMessage::SetAttribute(attribute));
    }

    #[inline]
    pub(crate) fn log_message(&self, log: LogMessage<'_>) {
        self.inner.exporter.export(InstanceMessage {
            execution: self.inner.execution_id,
            message: TelemetryMessage::Log(log),
        });
    }

    #[inline]
    fn tracing_message(&self, message: TracingMessage<'_>) {
        self.inner.exporter.export(InstanceMessage {
            execution: self.inner.execution_id,
            message: TelemetryMessage::Tracing(message),
        });
    }
}
