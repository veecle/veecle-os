//! Defines the shared protocol used to communicate between runtime instances and the `veecle-orchestrator`.

#![forbid(unsafe_code)]

pub use uuid::Uuid;
use veecle_telemetry::to_static::ToStatic;

#[cfg(feature = "jsonl")]
mod jsonl;

#[cfg(feature = "jsonl")]
pub use jsonl::{Codec, CodecError, EncodedStorable};

#[cfg(feature = "iceoryx2")]
use iceoryx2::prelude::ZeroCopySend;

/// A control request sent from a runtime to the orchestrator.
#[derive(Clone, Debug, veecle_os_runtime::Storable)]
#[cfg_attr(feature = "iceoryx2", derive(ZeroCopySend))]
#[cfg_attr(feature = "iceoryx2", repr(C))]
pub enum ControlRequest {
    /// Request to start a runtime instance.
    StartRuntime {
        /// The runtime instance to start (16 bytes representing UUID).
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: [u8; 16],
    },

    /// Request to stop a runtime instance.
    StopRuntime {
        /// The runtime instance to stop (16 bytes representing UUID).
        // This is `veecle_orchestrator_protocol::InstanceId` but we don't want the dependency.
        id: [u8; 16],
    },
}

#[cfg(feature = "jsonl")]
impl serde::Serialize for ControlRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStructVariant;
        match self {
            ControlRequest::StartRuntime { id } => {
                let uuid = Uuid::from_bytes(*id);
                let mut state =
                    serializer.serialize_struct_variant("ControlRequest", 0, "StartRuntime", 1)?;
                state.serialize_field("id", &uuid)?;
                state.end()
            }
            ControlRequest::StopRuntime { id } => {
                let uuid = Uuid::from_bytes(*id);
                let mut state =
                    serializer.serialize_struct_variant("ControlRequest", 1, "StopRuntime", 1)?;
                state.serialize_field("id", &uuid)?;
                state.end()
            }
        }
    }
}

#[cfg(feature = "jsonl")]
impl<'de> serde::Deserialize<'de> for ControlRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "PascalCase")]
        enum Variant {
            StartRuntime,
            StopRuntime,
        }

        struct ControlRequestVisitor;

        impl<'de> serde::de::Visitor<'de> for ControlRequestVisitor {
            type Value = ControlRequest;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("enum ControlRequest")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                use serde::de::{MapAccess, VariantAccess};
                let (variant, variant_access) = data.variant()?;
                match variant {
                    Variant::StartRuntime => {
                        struct StartRuntimeVisitor;
                        impl<'de> serde::de::Visitor<'de> for StartRuntimeVisitor {
                            type Value = ControlRequest;

                            fn expecting(
                                &self,
                                formatter: &mut core::fmt::Formatter,
                            ) -> core::fmt::Result {
                                formatter.write_str("struct StartRuntime")
                            }

                            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                            where
                                A: MapAccess<'de>,
                            {
                                let mut id: Option<Uuid> = None;
                                while let Some(key) = map.next_key::<String>()? {
                                    if key == "id" {
                                        id = Some(map.next_value()?);
                                    }
                                }
                                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                                Ok(ControlRequest::StartRuntime { id: *id.as_bytes() })
                            }
                        }
                        variant_access.struct_variant(&["id"], StartRuntimeVisitor)
                    }
                    Variant::StopRuntime => {
                        struct StopRuntimeVisitor;
                        impl<'de> serde::de::Visitor<'de> for StopRuntimeVisitor {
                            type Value = ControlRequest;

                            fn expecting(
                                &self,
                                formatter: &mut core::fmt::Formatter,
                            ) -> core::fmt::Result {
                                formatter.write_str("struct StopRuntime")
                            }

                            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                            where
                                A: MapAccess<'de>,
                            {
                                let mut id: Option<Uuid> = None;
                                while let Some(key) = map.next_key::<String>()? {
                                    if key == "id" {
                                        id = Some(map.next_value()?);
                                    }
                                }
                                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                                Ok(ControlRequest::StopRuntime { id: *id.as_bytes() })
                            }
                        }
                        variant_access.struct_variant(&["id"], StopRuntimeVisitor)
                    }
                }
            }
        }

        deserializer.deserialize_enum(
            "ControlRequest",
            &["StartRuntime", "StopRuntime"],
            ControlRequestVisitor,
        )
    }
}

/// Response to a control request.
#[derive(Clone, Debug, veecle_os_runtime::Storable)]
#[cfg_attr(feature = "iceoryx2", derive(ZeroCopySend))]
#[cfg_attr(feature = "iceoryx2", repr(C))]
#[allow(clippy::large_enum_variant)] // "Zero Copy" :clueless:
pub enum ControlResponse {
    /// Runtime started successfully.
    Started,

    /// Runtime stopped successfully.
    Stopped,

    /// Error occurred while processing the control request.
    Error {
        /// Error message as UTF-8 bytes.
        message: [u8; 256],
        /// Number of valid bytes in the message.
        length: u8,
    },
}

impl ControlResponse {
    pub fn error(message: &str) -> Self {
        let mut bytes = [0; 256];

        assert!(message.len() <= bytes.len());

        bytes[..message.len()].copy_from_slice(&message.as_bytes()[..message.len()]);

        ControlResponse::Error {
            message: bytes,
            length: message.len() as u8,
        }
    }

    pub fn as_error(&self) -> Option<&str> {
        match self {
            Self::Error { message, length } => {
                Some(std::str::from_utf8(&message[..*length as usize]).unwrap())
            }
            _ => None,
        }
    }
}

#[cfg(feature = "jsonl")]
impl serde::Serialize for ControlResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStructVariant;
        match self {
            ControlResponse::Started => {
                serializer.serialize_unit_variant("ControlResponse", 0, "Started")
            }
            ControlResponse::Stopped => {
                serializer.serialize_unit_variant("ControlResponse", 1, "Stopped")
            }
            ControlResponse::Error { message, length } => {
                let msg_str = core::str::from_utf8(&message[..*length as usize])
                    .map_err(|_| serde::ser::Error::custom("invalid utf8"))?;
                let mut state =
                    serializer.serialize_struct_variant("ControlResponse", 2, "Error", 1)?;
                state.serialize_field("0", msg_str)?;
                state.end()
            }
        }
    }
}

#[cfg(feature = "jsonl")]
impl<'de> serde::Deserialize<'de> for ControlResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "PascalCase")]
        enum Variant {
            Started,
            Stopped,
            Error,
        }

        struct ControlResponseVisitor;

        impl<'de> serde::de::Visitor<'de> for ControlResponseVisitor {
            type Value = ControlResponse;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("enum ControlResponse")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                use serde::de::VariantAccess;
                let (variant, variant_access) = data.variant()?;
                match variant {
                    Variant::Started => {
                        variant_access.unit_variant()?;
                        Ok(ControlResponse::Started)
                    }
                    Variant::Stopped => {
                        variant_access.unit_variant()?;
                        Ok(ControlResponse::Stopped)
                    }
                    Variant::Error => {
                        let msg: String = variant_access.newtype_variant()?;
                        let msg_bytes = msg.as_bytes();
                        let length = msg_bytes.len().min(256);
                        let mut message = [0u8; 256];
                        message[..length].copy_from_slice(&msg_bytes[..length]);
                        Ok(ControlResponse::Error {
                            message,
                            length: length as u8,
                        })
                    }
                }
            }
        }

        deserializer.deserialize_enum(
            "ControlResponse",
            &["Started", "Stopped", "Error"],
            ControlResponseVisitor,
        )
    }
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
