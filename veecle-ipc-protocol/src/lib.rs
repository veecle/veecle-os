//! Defines the shared protocol used to communicate between runtime instances and the `veecle-orchestrator`.

#![forbid(unsafe_code)]

use std::borrow::Cow;

use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec, LinesCodecError};
pub use uuid::Uuid;
use veecle_telemetry::protocol::owned;

/// Priority level for a runtime process.
#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// High priority (nice value -10).
    High,
    /// Normal priority (nice value 0).
    #[default]
    Normal,
    /// Low priority (nice value 10).
    Low,
}

/// A control request sent from a runtime to the orchestrator.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, veecle_os_runtime::Storable)]
pub enum ControlRequest {
    /// Request to start a runtime instance.
    StartRuntime {
        /// The runtime instance to start.
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: Uuid,

        /// The priority level for the runtime process.
        ///
        /// If not specified, defaults to [`Priority::Normal`].
        #[serde(default)]
        priority: Option<Priority>,
    },

    /// Request to stop a runtime instance.
    StopRuntime {
        /// The runtime instance to stop.
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: Uuid,
    },
}

/// Response to a control request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, veecle_os_runtime::Storable)]
pub enum ControlResponse {
    /// Runtime started successfully.
    Started,

    /// Runtime stopped successfully.
    Stopped,

    /// Error occurred while processing the control request.
    Error(String),
}

/// A message between a runtime instance and the `veecle-orchestrator`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Message {
    /// A data value going between the local instance and another runtime instance (both input and output).
    Storable(EncodedStorable),

    /// A telemetry message from `veecle-telemetry` system.
    #[serde(deserialize_with = "deserialize_instance_message")]
    Telemetry(owned::InstanceMessage),

    /// A control request sent from a runtime to the orchestrator.
    ControlRequest(ControlRequest),

    /// A response to a control request sent from the orchestrator to a runtime.
    ControlResponse(ControlResponse),
}

/// Deserializes into a fully owned [`owned::InstanceMessage`].
///
/// This is necessary to avoid lifetimes in `Message` from the `Cow` inside `InstanceMessage`.
///
/// TODO(#185): should not be needed once #185 is implemented.
fn deserialize_instance_message<'de, D>(deserializer: D) -> Result<owned::InstanceMessage, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    use veecle_telemetry::protocol::InstanceMessage;
    use veecle_telemetry::to_static::ToStatic;
    use veecle_telemetry::value::OwnedValue;

    InstanceMessage::<OwnedValue>::deserialize(deserializer).map(|message| message.to_static())
}

/// A data value going between the local instance and another runtime instance (both input and output).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncodedStorable {
    /// Type name of the data, used as a routing tag and to know how to deserialize the inner `value`.
    // TODO: using the type-name to tag messages doesn't guarantee uniqueness.
    pub type_name: Cow<'static, str>,

    /// JSON-encoded instance of a `type_name` value.
    pub value: String,
}

impl EncodedStorable {
    /// Encodes the given value into a [`EncodedStorable`] instance.
    pub fn new<T>(value: &T) -> serde_json::Result<Self>
    where
        T: serde::Serialize + 'static,
    {
        Ok(Self {
            type_name: Cow::Borrowed(std::any::type_name::<T>()),
            value: serde_json::to_string(&value)?,
        })
    }
}

#[derive(Debug, thiserror::Error, displaydoc::Display)]
/// An error occurred while encoding or decoding a [`Message`] with [`Codec`].
pub enum CodecError {
    /// The maximum line length was exceeded.
    MaxLineLengthExceeded,

    /// An IO error occurred.
    Io(#[from] std::io::Error),

    /// A JSON error occurred.
    Json(#[from] serde_json::Error),
}

impl From<LinesCodecError> for CodecError {
    fn from(error: LinesCodecError) -> Self {
        match error {
            LinesCodecError::MaxLineLengthExceeded => Self::MaxLineLengthExceeded,
            LinesCodecError::Io(error) => Self::Io(error),
        }
    }
}

/// A [`Decoder`] and [`Encoder`] implementation that reads JSONL encoded [`Message`]s from a byte stream.
#[derive(Debug)]
pub struct Codec {
    lines: LinesCodec,
}

impl Codec {
    /// Returns a new `Codec`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            // TODO: Arbitrary limit, but we should switch away from JSONL anyway so this can be bettered later.
            lines: LinesCodec::new_with_max_length(2048),
        }
    }
}

impl Decoder for Codec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(line) = self.lines.decode(src)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str::<Message>(&line)?))
    }

    fn decode_eof(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(line) = self.lines.decode_eof(buffer)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str::<Message>(&line)?))
    }
}

impl Encoder<&Message> for Codec {
    type Error = CodecError;

    fn encode(&mut self, item: &Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let line = serde_json::to_string(&item)?;

        // `LinesCodec` only applies the maximum when decoding, we want to also avoid sending messages that would fail
        // on the receiving side.
        if line.len() > self.lines.max_length() {
            return Err(CodecError::MaxLineLengthExceeded);
        }

        self.lines.encode(line, dst)?;
        Ok(())
    }
}
