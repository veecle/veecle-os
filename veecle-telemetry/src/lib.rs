//! # `veecle-telemetry`
//!
//! A telemetry library for collecting and exporting observability data including traces, logs, and metrics.
//!
//! This crate provides telemetry collection capabilities with support for both `std` and `no_std`
//! environments, including FreeRTOS targets.
//!
//! ## Features
//!
//! - **Tracing**: Distributed tracing with spans, events, and context propagation
//! - **Logging**: Structured logging with multiple severity levels
//! - **Zero-cost abstractions**: When telemetry is disabled, operations compile to no-ops
//! - **Cross-platform**: Works on `std`, `no_std`, and FreeRTOS environments
//! - **Exporters**: Multiple export formats including JSON console output
//!
//! ## Feature Flags
//!
//! - `enable` - Enable collecting and exporting telemetry data, should only be set in binary crates
//! - `std` - Enable standard library support
//! - `alloc` - Enable allocator support for dynamic data structures
//! - `freertos` - Enable FreeRTOS support
//! - `system_time` - Enable system time synchronization
//!
//! ## Basic Usage
//!
//! First, set up an exporter in your application:
//!
//! ```rust
//! use veecle_telemetry::collector::{ConsoleJsonExporter, set_exporter};
//! use veecle_telemetry::ProcessId;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let process_id = ProcessId::random(&mut rand::rng());
//! set_exporter(process_id, &ConsoleJsonExporter)?;
//! # Ok(())
//! # }
//! ```
//!
//! Then use the telemetry macros and functions:
//!
//! ```rust
//! use veecle_telemetry::{Span, info, instrument, span};
//!
//! // Structured logging
//! info!("Server started", port = 8080, version = "1.0.0");
//!
//! // Manual span creation
//! let span = span!("process_request", user_id = 123);
//! let _guard = span.entered();
//!
//! // Automatic instrumentation
//! #[instrument]
//! fn process_data(input: &str) -> String {
//!     // Function body is automatically wrapped in a span
//!     format!("processed: {}", input)
//! }
//! ```
//!
//! ## Span Management
//!
//! Spans represent units of work and can be nested to show relationships:
//!
//! ```rust
//! use veecle_telemetry::{CurrentSpan, span};
//!
//! let parent_span = span!("parent_operation");
//! let _guard = parent_span.entered();
//!
//! // Child spans automatically inherit the parent context
//! let child_span = span!("child_operation", step = 1);
//! let _child_guard = child_span.entered();
//!
//! // Add events to the current span
//! CurrentSpan::add_event("milestone_reached", &[]);
//! ```
//!
//! ## Conditional Compilation
//!
//! When the `enable` feature is disabled, all telemetry operations compile to no-ops,
//! ensuring zero runtime overhead in production builds where telemetry is not needed.

#![no_std]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "enable", not(any(feature = "std", feature = "freertos"))))]
compile_error! {
    "veecle_telemetry requires that either the `std` (default) or the `freertos` feature is enabled to work"
}

pub mod collector;
pub mod future;
pub mod id;
pub mod log;
#[doc(hidden)]
pub mod macro_helpers;
pub mod macros;
pub mod protocol;
mod span;
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub mod test_helpers;
#[cfg(feature = "enable")]
mod time;
pub mod to_static;
pub mod types;
pub mod value;

pub use id::{ProcessId, SpanContext, SpanId};
pub use span::{CurrentSpan, Span, SpanGuard, SpanGuardRef};
pub use value::{KeyValue, Value};
pub use veecle_telemetry_macros::instrument;
