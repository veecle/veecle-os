//! Global collector state and initialization.

use core::sync::atomic::{AtomicUsize, Ordering};

use core::{error, fmt};

use super::{Collector, Export, InstanceMessage, ProcessId};

/// No-op exporter used when telemetry is disabled or not initialized.
#[derive(Debug)]
struct NopExporter;

impl Export for NopExporter {
    fn export(&self, _: InstanceMessage) {}
}

static NO_EXPORTER: NopExporter = NopExporter;

static NO_COLLECTOR: Collector = Collector::new(
    ProcessId::from_raw(0),
    &NO_EXPORTER,
    nop_timestamp,
    nop_thread_id,
);

/// The `GLOBAL_COLLECTOR` static holds the global collector instance. It is protected by
/// the `GLOBAL_INIT` static which determines whether `GLOBAL_COLLECTOR` has been initialized.
static mut GLOBAL_COLLECTOR: Collector = Collector::new(
    ProcessId::from_raw(0),
    &NO_EXPORTER,
    nop_timestamp,
    nop_thread_id,
);

fn nop_timestamp() -> u64 {
    0
}

fn nop_thread_id() -> core::num::NonZeroU64 {
    core::num::NonZeroU64::new(1).unwrap()
}

static GLOBAL_INIT: AtomicUsize = AtomicUsize::new(0);

// There are three different states that we care about:
// - the collector is uninitialized
// - the collector is initializing (`set_global` has been called but `GLOBAL_COLLECTOR` hasn't been set yet)
// - the collector is active
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

/// Set the global collector instance.
pub(super) fn set_collector(collector: Collector) -> Result<(), SetGlobalError> {
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
        unsafe { GLOBAL_COLLECTOR = collector }
        GLOBAL_INIT.store(INITIALIZED, Ordering::Release);
        Ok(())
    } else {
        Err(SetGlobalError(()))
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

    #[cfg(feature = "enable")]
    // Acquire memory ordering guarantees that current thread would see any
    // memory writes that happened before store of the value
    // into `GLOBAL_INIT` with memory ordering `Release` or stronger.
    //
    // Since the value `INITIALIZED` is written only after `GLOBAL_COLLECTOR` was
    // initialized, observing it after `Acquire` load here makes both
    // write to the `GLOBAL_COLLECTOR` static and initialization of the exporter
    // internal state synchronized with current thread.
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

/// The type returned by [`set_global`][super::Builder::set_global] if the collector has already been initialized.
#[derive(Debug)]
pub struct SetGlobalError(());

impl SetGlobalError {
    const MESSAGE: &'static str = "a global exporter has already been set";
}

impl fmt::Display for SetGlobalError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(Self::MESSAGE)
    }
}

impl error::Error for SetGlobalError {}
