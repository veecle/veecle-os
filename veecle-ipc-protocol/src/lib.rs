//! Defines the shared protocol used to communicate between runtime instances and the `veecle-orchestrator`.

#![forbid(unsafe_code)]

pub use uuid::Uuid;
use veecle_telemetry::to_static::ToStatic;

#[cfg(feature = "jsonl")]
mod jsonl;

#[cfg(feature = "jsonl")]
pub use jsonl::{Codec, CodecError, EncodedStorable};

/// A control request sent from a runtime to the orchestrator.
#[derive(Clone, Debug, veecle_os_runtime::Storable)]
#[cfg_attr(feature = "jsonl", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlRequest {
    /// Request to start a runtime instance.
    StartRuntime {
        /// The runtime instance to start.
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: Uuid,
    },

    /// Request to stop a runtime instance.
    StopRuntime {
        /// The runtime instance to stop.
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: Uuid,
    },
}

/// Response to a control request.
#[derive(Clone, Debug, veecle_os_runtime::Storable)]
#[cfg_attr(feature = "jsonl", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlResponse {
    /// Runtime started successfully.
    Started,

    /// Runtime stopped successfully.
    Stopped,

    /// Error occurred while processing the control request.
    Error(String),
}

/// A message between a runtime instance and the `veecle-orchestrator`.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "jsonl", derive(serde::Serialize, serde::Deserialize))]
pub enum Message<'a> {
    #[cfg(feature = "jsonl")]
    /// A data value going between the local instance and another runtime instance (both input and output).
    EncodedStorable(EncodedStorable),

    /// A telemetry message from `veecle-telemetry` system.
    #[cfg_attr(feature = "jsonl", serde(borrow))]
    Telemetry(veecle_telemetry::protocol::InstanceMessage<'a>),

    /// A control request sent from a runtime to the orchestrator.
    ControlRequest(ControlRequest),

    /// A response to a control request sent from the orchestrator to a runtime.
    ControlResponse(ControlResponse),
}

impl<'a> Message<'a> {
    /// Converts this message to have a static lifetime.
    pub fn to_static(self) -> Message<'static> {
        match self {
            #[cfg(feature = "jsonl")]
            Message::EncodedStorable(storable) => Message::EncodedStorable(storable),
            Message::Telemetry(message) => Message::Telemetry(message.to_static()),
            Message::ControlRequest(request) => Message::ControlRequest(request),
            Message::ControlResponse(response) => Message::ControlResponse(response),
        }
    }
}
