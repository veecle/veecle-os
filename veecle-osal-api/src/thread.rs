//! Abstractions for thread-related operations.

/// `ThreadAbstraction` is used to query thread-related information in a platform-agnostic manner.
pub trait ThreadAbstraction {
    /// Returns a unique identifier for the current thread.
    ///
    /// The returned id is guaranteed to be unique within the lifetime of the process.
    /// Thread ids are not reused, even after a thread terminates.
    ///
    /// This is useful for telemetry and tracing, where thread ids can be included
    /// in spans and logs to help correlate events to specific threads of execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use veecle_osal_api::thread::ThreadAbstraction;
    /// use veecle_osal_std::thread::Thread;
    ///
    /// let thread_id = Thread::current_thread_id();
    /// println!("Current thread id: {}", thread_id);
    /// ```
    fn current_thread_id() -> u64;
}
