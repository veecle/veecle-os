use super::global::SetGlobalError;
use super::{Collector, Export, ProcessId};

use veecle_osal_api::thread::ThreadAbstraction;
use veecle_osal_api::time::{Instant, SystemTime, SystemTimeError, TimeAbstraction};

/// Returns [`<T as TimeAbstraction>::now`][TimeAbstraction::now] converted into nanoseconds since
/// [`Instant::MIN`].
fn timestamp_fn_monotonic<T>() -> u64
where
    T: TimeAbstraction,
{
    let timestamp_micros: u64 = T::now()
        .duration_since(Instant::MIN)
        .expect("now should be later than MIN")
        .as_micros();

    timestamp_micros * 1000
}

/// Returns [`<T as SystemTime>::duration_since_epoch`][SystemTime::duration_since_epoch] converted
/// into nanoseconds, falling back to [`timestamp_fn_monotonic`] if the system time is not
/// synchronized.
fn timestamp_fn_system_time<T>() -> u64
where
    T: TimeAbstraction + SystemTime,
{
    match T::duration_since_epoch() {
        Ok(duration) => duration.as_micros() * 1000,
        Err(SystemTimeError::Unsynchronized) => {
            // Fall back to monotonic time if not synchronized.
            timestamp_fn_monotonic::<T>()
        }
        Err(SystemTimeError::EpochIsLaterThanStartTime) => {
            panic!(
                "Failed to get duration since epoch: {:?}",
                SystemTimeError::EpochIsLaterThanStartTime
            );
        }
    }
}

/// Type-state markers for builder
mod state {
    #[derive(Debug)]
    pub struct NoProcessId;
    #[derive(Debug)]
    pub struct WithProcessId;
    #[derive(Debug)]
    pub struct NoExporter;
    #[derive(Debug)]
    pub struct WithExporter;
    #[derive(Debug)]
    pub struct NoTime;
    #[derive(Debug)]
    pub struct WithTime;
    #[derive(Debug)]
    pub struct NoThread;
    #[derive(Debug)]
    pub struct WithThread;
}

/// Builder for initializing the telemetry collector.
///
/// Uses type-state pattern to ensure all required components are configured at compile time.
/// Created via [`build()`] and finalized with [`set_global()`](Builder::set_global).
#[derive(Debug)]
#[must_use]
pub struct Builder<PID, EXP, TIME, THREAD> {
    process_id: Option<ProcessId>,
    exporter: Option<&'static (dyn Export + Sync)>,
    timestamp_fn: Option<fn() -> u64>,
    thread_id_fn: Option<fn() -> core::num::NonZeroU64>,
    _pid: core::marker::PhantomData<PID>,
    _exp: core::marker::PhantomData<EXP>,
    _time: core::marker::PhantomData<TIME>,
    _thread: core::marker::PhantomData<THREAD>,
}

/// Creates a new collector builder.
///
/// # Example
///
/// ```rust,no_run
/// use veecle_osal_std::{time::Time, thread::Thread};
/// use veecle_telemetry::collector;
///
/// collector::build()
///     .random_process_id()
///     .console_json_exporter()
///     .time::<Time>()
///     .thread::<Thread>()
///     .set_global().unwrap();
/// ```
pub fn build() -> Builder<state::NoProcessId, state::NoExporter, state::NoTime, state::NoThread> {
    Builder {
        process_id: None,
        exporter: None,
        timestamp_fn: None,
        thread_id_fn: None,
        _pid: core::marker::PhantomData,
        _exp: core::marker::PhantomData,
        _time: core::marker::PhantomData,
        _thread: core::marker::PhantomData,
    }
}

impl<PID, EXP, TIME, THREAD> Builder<PID, EXP, TIME, THREAD> {
    /// Sets the process id for this collector instance.
    pub fn process_id(
        self,
        process_id: ProcessId,
    ) -> Builder<state::WithProcessId, EXP, TIME, THREAD> {
        Builder {
            process_id: Some(process_id),
            exporter: self.exporter,
            timestamp_fn: self.timestamp_fn,
            thread_id_fn: self.thread_id_fn,
            _pid: core::marker::PhantomData,
            _exp: core::marker::PhantomData,
            _time: core::marker::PhantomData,
            _thread: core::marker::PhantomData,
        }
    }

    /// Sets the exporter for telemetry data.
    pub fn exporter(
        self,
        exporter: &'static (dyn Export + Sync),
    ) -> Builder<PID, state::WithExporter, TIME, THREAD> {
        Builder {
            process_id: self.process_id,
            exporter: Some(exporter),
            timestamp_fn: self.timestamp_fn,
            thread_id_fn: self.thread_id_fn,
            _pid: core::marker::PhantomData,
            _exp: core::marker::PhantomData,
            _time: core::marker::PhantomData,
            _thread: core::marker::PhantomData,
        }
    }

    /// Configures the time abstraction to use (monotonic time only).
    pub fn time<T>(self) -> Builder<PID, EXP, state::WithTime, THREAD>
    where
        T: TimeAbstraction,
    {
        Builder {
            process_id: self.process_id,
            exporter: self.exporter,
            timestamp_fn: Some(timestamp_fn_monotonic::<T>),
            thread_id_fn: self.thread_id_fn,
            _pid: core::marker::PhantomData,
            _exp: core::marker::PhantomData,
            _time: core::marker::PhantomData,
            _thread: core::marker::PhantomData,
        }
    }

    /// Configures the time abstraction with system time to use (Unix epoch synchronization).
    pub fn system_time<T>(self) -> Builder<PID, EXP, state::WithTime, THREAD>
    where
        T: TimeAbstraction + SystemTime,
    {
        Builder {
            process_id: self.process_id,
            exporter: self.exporter,
            timestamp_fn: Some(timestamp_fn_system_time::<T>),
            thread_id_fn: self.thread_id_fn,
            _pid: core::marker::PhantomData,
            _exp: core::marker::PhantomData,
            _time: core::marker::PhantomData,
            _thread: core::marker::PhantomData,
        }
    }

    /// Configures the thread abstraction to use.
    pub fn thread<Th>(self) -> Builder<PID, EXP, TIME, state::WithThread>
    where
        Th: ThreadAbstraction,
    {
        Builder {
            process_id: self.process_id,
            exporter: self.exporter,
            timestamp_fn: self.timestamp_fn,
            thread_id_fn: Some(Th::current_thread_id),
            _pid: core::marker::PhantomData,
            _exp: core::marker::PhantomData,
            _time: core::marker::PhantomData,
            _thread: core::marker::PhantomData,
        }
    }
}

impl<EXP, TIME, THREAD> Builder<state::NoProcessId, EXP, TIME, THREAD> {
    /// Sets a randomly generated process id.
    ///
    /// Equivalent to `.process_id(ProcessId::random(&mut rand::rng()))`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use veecle_osal_std::{time::Time, thread::Thread};
    /// use veecle_telemetry::collector;
    ///
    /// # let exporter = &veecle_telemetry::collector::ConsoleJsonExporter::DEFAULT;
    /// collector::build()
    ///     .random_process_id()
    ///     .exporter(exporter)
    ///     .time::<Time>()
    ///     .thread::<Thread>()
    ///     .set_global().unwrap();
    /// ```
    #[cfg(feature = "std")]
    pub fn random_process_id(self) -> Builder<state::WithProcessId, EXP, TIME, THREAD> {
        self.process_id(ProcessId::random(&mut rand::rng()))
    }
}

impl<PID, TIME, THREAD> Builder<PID, state::NoExporter, TIME, THREAD> {
    /// Sets the given exporter by leaking it to obtain a static reference.
    ///
    /// This is a convenience method for dynamic exporters that need to be boxed
    /// and leaked. Equivalent to `.exporter(Box::leak(Box::new(exporter)))`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use veecle_osal_std::{time::Time, thread::Thread};
    /// use veecle_telemetry::collector::TestExporter;
    ///
    /// let (exporter, _collected) = TestExporter::new();
    /// veecle_telemetry::collector::build()
    ///     .random_process_id()
    ///     .leaked_exporter(exporter)
    ///     .time::<Time>()
    ///     .thread::<Thread>()
    ///     .set_global().unwrap();
    /// ```
    #[cfg(feature = "alloc")]
    pub fn leaked_exporter(
        self,
        exporter: impl Export + Sync + 'static,
    ) -> Builder<PID, state::WithExporter, TIME, THREAD> {
        self.exporter(alloc::boxed::Box::leak(alloc::boxed::Box::new(exporter)))
    }

    /// Sets the exporter to be the [`ConsoleJsonExporter`][super::ConsoleJsonExporter].
    ///
    /// Equivalent to `.exporter(&ConsoleJsonExporter::DEFAULT)`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use veecle_osal_std::{time::Time, thread::Thread};
    /// use veecle_telemetry::collector;
    ///
    /// collector::build()
    ///     .random_process_id()
    ///     .console_json_exporter()
    ///     .time::<Time>()
    ///     .thread::<Thread>()
    ///     .set_global().unwrap();
    /// ```
    #[cfg(feature = "std")]
    pub fn console_json_exporter(self) -> Builder<PID, state::WithExporter, TIME, THREAD> {
        self.exporter(&super::ConsoleJsonExporter::DEFAULT)
    }
}

impl Builder<state::WithProcessId, state::WithExporter, state::WithTime, state::WithThread> {
    /// Builds this configuration into a [`Collector`] instance.
    pub fn build(self) -> Collector {
        Collector::new(
            self.process_id.unwrap(),
            self.exporter.unwrap(),
            self.timestamp_fn.unwrap(),
            self.thread_id_fn.unwrap(),
        )
    }

    /// Sets this collector as the global collector instance.
    ///
    /// This can only be called once per process.
    pub fn set_global(self) -> Result<(), SetGlobalError> {
        super::global::set_collector(self.build())
    }
}
