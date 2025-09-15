//! `veecle-telemetry-server-protocol` contains message definition for the WebSocket tracing data protocol.

#![forbid(unsafe_code)]

use serde_derive::{Deserialize, Serialize};

/// Batched tracing data lines message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TracingMessage {
    /// Tracing data lines.
    pub lines: Vec<String>,

    /// Total number of tracing data lines the server has.
    pub total: usize,

    /// Will be true if the Veecle OS program has exited.
    pub done: bool,
}
