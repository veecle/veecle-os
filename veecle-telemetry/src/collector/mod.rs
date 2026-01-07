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

mod collector;
mod global;

#[cfg(feature = "std")]
mod json_exporter;
#[cfg(feature = "std")]
mod pretty_exporter;
#[cfg(feature = "std")]
mod test_exporter;

use core::fmt::Debug;

#[cfg(feature = "std")]
pub use json_exporter::ConsoleJsonExporter;
#[cfg(feature = "std")]
pub use pretty_exporter::ConsolePrettyExporter;
#[cfg(feature = "std")]
#[doc(hidden)]
pub use test_exporter::TestExporter;

pub use self::collector::Collector;
pub use self::global::get_collector;
#[cfg(feature = "enable")]
pub use self::global::{SetExporterError, set_exporter};

pub use crate::protocol::base::ProcessId;
use crate::protocol::transient::InstanceMessage;

/// Trait for exporting telemetry data to external systems.
///
/// Implementors of this trait define how telemetry data should be exported,
/// whether to files, network endpoints, or other destinations.
///
/// # Examples
///
/// ```rust
/// use veecle_telemetry::collector::Export;
/// use veecle_telemetry::protocol::transient::InstanceMessage;
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
