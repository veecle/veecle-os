//! Defines the shared protocol used to communicate between runtime instances and the `veecle-orchestrator`.

#![forbid(unsafe_code)]

use std::borrow::Cow;

use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec, LinesCodecError};
use veecle_telemetry::to_static::ToStatic;

/// A message between a runtime instance and the `veecle-orchestrator`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Message<'a> {
    /// A data value going between the local instance and another runtime instance (both input and output).
    Storable(EncodedStorable),

    /// A telemetry message from `veecle-telemetry` system.
    #[serde(borrow)]
    Telemetry(veecle_telemetry::protocol::InstanceMessage<'a>),
}

impl<'a> Message<'a> {
    /// Converts this message to have a static lifetime.
    pub fn to_static(self) -> Message<'static> {
        match self {
            Message::Storable(storable) => Message::Storable(storable),
            Message::Telemetry(message) => Message::Telemetry(message.to_static()),
        }
    }
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
    type Item = Message<'static>;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(line) = self.lines.decode(src)? else {
            return Ok(None);
        };
        Ok(Some(
            serde_json::from_str::<Message>(&line).map(|message| message.to_static())?,
        ))
    }

    fn decode_eof(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(line) = self.lines.decode_eof(buffer)? else {
            return Ok(None);
        };
        Ok(Some(
            serde_json::from_str::<Message>(&line).map(|message| message.to_static())?,
        ))
    }
}

impl Encoder<&Message<'_>> for Codec {
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
