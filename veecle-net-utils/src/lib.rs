//! Networking utilities for Veecle Services.
//!
//! Generic socket addressing and stream handling for both Unix domain sockets and TCP sockets,
//! with both async and blocking variants.
//!
//! # Features
//!
//! - `tokio`: Enable async networking support (requires Tokio). Default: disabled.

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod address;
#[cfg(feature = "tokio")]
mod async_io;
mod blocking;

pub use address::{
    MultiSocketAddress, UnresolvedMultiSocketAddress, UnresolvedMultiSocketAddressParseError,
    UnresolvedSocketAddress, UnresolvedSocketAddressParseError,
};

pub use blocking::BlockingSocketStream;

#[cfg(feature = "tokio")]
pub use async_io::{AsyncSocketListener, AsyncSocketStream, AsyncUnixListener};
