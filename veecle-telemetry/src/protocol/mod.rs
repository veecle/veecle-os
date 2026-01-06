//! Telemetry protocol types and message definitions.
//!
//! # Submodule split
//!
//! The [`base`] module defines the main message definitions, generic over how they actually store
//! their data. The [`transient`] and [`owned`] modules expose type aliases for the base definitions
//! with specific data storage families. The [`transient`] data is created with references and
//! thread-local data to avoid copying data when recording log events, this is what gets passed in
//! to a [`crate::collector::Export`] implementor. The [`owned`] data uses heap allocation to allow
//! passing the data around in-memory or across processes as needed.
//!
//! # Serialization
//!
//! Both [`transient`] and [`owned`] types implement [`serde::Serialize`], the [`owned`] types also
//! implement [`serde::Deserialize`]; the types from both modules are compatible, so you can
//! directly serialize a [`transient::LogMessage`] without any allocations then deserialize that as
//! an [`owned::LogMessage`].

pub mod base;
#[cfg(feature = "alloc")]
pub mod owned;
pub mod transient;

#[cfg(test)]
mod tests;
