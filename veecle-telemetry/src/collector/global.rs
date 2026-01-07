//! Global collector state and initialization.

#[cfg(feature = "enable")]
use core::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "enable")]
use core::{error, fmt};

use super::Export;
use super::collector::Collector;
use crate::protocol::base::ProcessId;
use crate::protocol::transient::InstanceMessage;

// No-op exporter used when telemetry is disabled or not initialized.
#[derive(Debug)]
struct NopExporter;

impl Export for NopExporter {
    fn export(&self, _: InstanceMessage) {}
}

static NO_EXPORTER: NopExporter = NopExporter;

static NO_COLLECTOR: Collector = Collector::new(ProcessId::from_raw(0), &NO_EXPORTER);

/// The `GLOBAL_COLLECTOR` static holds the global collector instance. It is protected by
/// the `GLOBAL_INIT` static which determines whether `GLOBAL_COLLECTOR` has been initialized.
#[cfg(feature = "enable")]
static mut GLOBAL_COLLECTOR: Collector = Collector::new(ProcessId::from_raw(0), &NO_EXPORTER);

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

/// Initializes the collector with the given Exporter and [`ProcessId`].
///
/// A [`ProcessId`] should never be re-used as it's used to collect metadata about the execution and to generate
/// [`SpanContext`]s which need to be globally unique.
///
/// [`SpanContext`]: crate::SpanContext
#[cfg(feature = "enable")]
pub fn set_exporter(
    process_id: ProcessId,
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
        unsafe { GLOBAL_COLLECTOR = Collector::new(process_id, exporter) }
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
#[cfg(feature = "enable")]
#[derive(Debug)]
pub struct SetExporterError(());

#[cfg(feature = "enable")]
impl SetExporterError {
    const MESSAGE: &'static str = "a global exporter has already been set";
}

#[cfg(feature = "enable")]
impl fmt::Display for SetExporterError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(Self::MESSAGE)
    }
}

#[cfg(feature = "enable")]
impl error::Error for SetExporterError {}
