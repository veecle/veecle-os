//! SOME/IP header de-/serialization.

use crate::parse::{ByteReader, Parse, ParseError};
use crate::serialize::{ByteWriter, Serialize, SerializeError};

/// Creates a new type wrapping a primitive. The new type implements conversion from and to the primitive as well as
/// [`Parse`].
macro_rules! create_new_type {
    (
        $(#[$($attributes:tt)*])*
        pub struct $name:ident($inner:ty);
    ) => {
        $(#[$($attributes)*])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name($inner);

        impl From<$name> for $inner {
            fn from(value: $name) -> $inner {
                value.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl<'a> Parse<'a> for $name {
            fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
                Parse::parse_partial(reader).map($name)
            }
        }

        impl<'a> Serialize for $name {
            fn required_length(&self) -> usize {
                self.0.required_length()
            }

            fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
                self.0.serialize_partial(byte_writer)
            }
        }
    };
}

create_new_type! {
    /// SOME/IP service ID.
    pub struct ServiceId(u16);
}

create_new_type! {
    /// SOME/IP method ID.
    pub struct MethodId(u16);
}

/// SOME/IP message ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct MessageId {
    service_id: ServiceId,
    method_id: MethodId,
}

impl MessageId {
    /// Creates a new message ID.
    pub fn new(service_id: ServiceId, method_id: MethodId) -> Self {
        Self {
            service_id,
            method_id,
        }
    }

    /// Returns the [`ServiceId`].
    pub fn service_id(&self) -> ServiceId {
        self.service_id
    }

    /// Sets the [`ServiceId`].
    pub fn set_service_id(&mut self, service_id: ServiceId) {
        self.service_id = service_id
    }

    /// Returns the [`MethodId`].
    pub fn method_id(&self) -> MethodId {
        self.method_id
    }

    /// Sets the [`MethodId`].
    pub fn set_method_id(&mut self, method_id: MethodId) {
        self.method_id = method_id
    }
}

create_new_type! {
    /// SOME/IP length header field.
    pub struct Length(u32);
}

impl Length {
    // Header fields included in the length.
    const REMAINING_HEADER_SIZE: u32 = 8;

    /// Calculates the length of the payload, not including any of the header.
    ///
    /// This does not take E2E protection into account.
    pub fn from_payload_length(length: u32) -> Self {
        Self(length + Self::REMAINING_HEADER_SIZE)
    }

    /// Calculates the length of the payload, not including any of the header.
    ///
    /// This does not take E2E protection into account.
    pub fn payload_length(&self) -> u32 {
        self.0.saturating_sub(Self::REMAINING_HEADER_SIZE)
    }
}

create_new_type! {
    /// SOME/IP client ID prefix.
    pub struct Prefix(u8);
}

create_new_type! {
    /// SOME/IP client ID inner ID.
    pub struct ClientIdInner(u8);
}

/// SOME/IP client ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct ClientId {
    prefix: Prefix,
    id: ClientIdInner,
}

impl ClientId {
    /// Creates a new client ID.
    pub fn new(prefix: Prefix, id: ClientIdInner) -> Self {
        Self { prefix, id }
    }

    /// Returns the prefix.
    pub fn prefix(&self) -> Prefix {
        self.prefix
    }

    /// Sets the prefix.
    pub fn set_prefix(&mut self, prefix: Prefix) {
        self.prefix = prefix
    }

    /// Returns the ID.
    pub fn id(&self) -> ClientIdInner {
        self.id
    }

    /// Sets the ID.
    pub fn set_id(&mut self, id: ClientIdInner) {
        self.id = id
    }
}

create_new_type! {
    /// SOME/IP session ID.
    pub struct SessionId(u16);
}

impl SessionId {
    /// Returns the next session ID.
    pub fn next(&self) -> Self {
        // Session handling is not active.
        if self.0 == 0 {
            return *self;
        }

        // The session ID needs to be in the range 0x1 - 0xFFFF.
        let next_id = self.0.checked_add(1).unwrap_or(1);

        Self(next_id)
    }
}

/// SOME/IP request ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct RequestId {
    client_id: ClientId,
    session_id: SessionId,
}

impl RequestId {
    /// Creates a new message ID.
    pub fn new(client_id: ClientId, session_id: SessionId) -> Self {
        Self {
            client_id,
            session_id,
        }
    }

    /// Returns the [`ClientId`].
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Sets the [`ClientId`].
    pub fn set_client_id(&mut self, client_id: ClientId) {
        self.client_id = client_id
    }

    /// Returns the [`SessionId`].
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Sets the [`SessionId`].
    pub fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = session_id
    }
}

create_new_type! {
    /// SOME/IP protocol version.
    pub struct ProtocolVersion(u8);
}

create_new_type! {
    /// SOME/IP interface version.
    pub struct InterfaceVersion(u8);
}

/// SOME/IP message type version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// A request expecting a response (even void).
    Request,
    /// A fire&forget request
    RequestNoReturn,
    /// A request of a notification/event callback expecting no response.
    Notification,
    /// The response message.
    Response,
    /// The response containing an error.
    Error,
    /// A TP request expecting a response (even void).
    TpRequest,
    /// A TP fire&forget request.
    TpRequestNoReturn,
    /// A TP request of a notification/event call-back expecting no response.
    TpNotification,
    /// The TP response message.
    TpResponse,
    /// The TP response containing an error.
    TpError,
}

impl<'a> Parse<'a> for MessageType {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let byte = reader.read_byte()?;

        let message_type = match byte {
            0x00 => Self::Request,
            0x01 => Self::RequestNoReturn,
            0x02 => Self::Notification,
            0x80 => Self::Response,
            0x81 => Self::Error,
            0x20 => Self::TpRequest,
            0x21 => Self::TpRequestNoReturn,
            0x22 => Self::TpNotification,
            0xA0 => Self::TpResponse,
            0xA1 => Self::TpError,
            _ => {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }
        };

        Ok(message_type)
    }
}

impl Serialize for MessageType {
    fn required_length(&self) -> usize {
        1
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        match self {
            MessageType::Request => byte_writer.write_byte(0x00),
            MessageType::RequestNoReturn => byte_writer.write_byte(0x01),
            MessageType::Notification => byte_writer.write_byte(0x02),
            MessageType::Response => byte_writer.write_byte(0x80),
            MessageType::Error => byte_writer.write_byte(0x81),
            MessageType::TpRequest => byte_writer.write_byte(0x20),
            MessageType::TpRequestNoReturn => byte_writer.write_byte(0x21),
            MessageType::TpNotification => byte_writer.write_byte(0x22),
            MessageType::TpResponse => byte_writer.write_byte(0xA0),
            MessageType::TpError => byte_writer.write_byte(0xA1),
        }
    }
}

/// SOME/IP return code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnCode {
    /// No error occurred.
    Ok,
    /// An unspecified error occurred.
    NotOk,
    /// The requested Service ID is unknown.
    UnknownService,
    /// The requested Method ID is unknown. Service ID is known.
    UnknownMethod,
    /// Service ID and Method ID are known. Application not running.
    NotReady,
    /// System running the service is not reachable (internal error code only).
    NotReachable,
    /// A timeout occurred (internal error code only).
    Timeout,
    /// Version of SOME/IP protocol not supported.
    WrongProtocolVersion,
    /// Interface version mismatch.
    WrongInterfaceVersion,
    /// Deserialization error, so that payload cannot be de-serialized.
    MalformedMessage,
    /// An unexpected message type was received (e.g. REQUEST_NO_RETURN for a method defined as REQUEST).
    WrongMessageType,
    /// Repeated E2E calculation error.
    E2ERepeated,
    /// Wrong E2E sequence error.
    E2EWrongSequence,
    /// Not further specified E2E error.
    E2E,
    /// E2E not available.
    E2ENotAvailable,
    /// No new data for E2E calculation present.
    E2ENoNewData,
    /// Reserved for generic SOME/IP errors.
    Reserved0(u8),
    /// Reserved for specific errors of services and methods.
    Reserved1(u8),
}

impl<'a> Parse<'a> for ReturnCode {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let byte = reader.read_byte()?;

        let return_code = match byte {
            0x00 => Self::Ok,
            0x01 => Self::NotOk,
            0x02 => Self::UnknownService,
            0x03 => Self::UnknownMethod,
            0x04 => Self::NotReady,
            0x05 => Self::NotReachable,
            0x06 => Self::Timeout,
            0x07 => Self::WrongProtocolVersion,
            0x08 => Self::WrongInterfaceVersion,
            0x09 => Self::MalformedMessage,
            0x0A => Self::WrongMessageType,
            0x0B => Self::E2ERepeated,
            0x0C => Self::E2EWrongSequence,
            0x0D => Self::E2E,
            0x0E => Self::E2ENotAvailable,
            0x0F => Self::E2ENoNewData,
            0x10..=0x1F => Self::Reserved0(byte),
            0x20..=0x5E => Self::Reserved1(byte),
            _ => {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }
        };

        Ok(return_code)
    }
}

impl Serialize for ReturnCode {
    fn required_length(&self) -> usize {
        1
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        match self {
            ReturnCode::Ok => byte_writer.write_byte(0x00),
            ReturnCode::NotOk => byte_writer.write_byte(0x01),
            ReturnCode::UnknownService => byte_writer.write_byte(0x02),
            ReturnCode::UnknownMethod => byte_writer.write_byte(0x03),
            ReturnCode::NotReady => byte_writer.write_byte(0x04),
            ReturnCode::NotReachable => byte_writer.write_byte(0x05),
            ReturnCode::Timeout => byte_writer.write_byte(0x06),
            ReturnCode::WrongProtocolVersion => byte_writer.write_byte(0x07),
            ReturnCode::WrongInterfaceVersion => byte_writer.write_byte(0x08),
            ReturnCode::MalformedMessage => byte_writer.write_byte(0x09),
            ReturnCode::WrongMessageType => byte_writer.write_byte(0x0A),
            ReturnCode::E2ERepeated => byte_writer.write_byte(0x0B),
            ReturnCode::E2EWrongSequence => byte_writer.write_byte(0x0C),
            ReturnCode::E2E => byte_writer.write_byte(0x0D),
            ReturnCode::E2ENotAvailable => byte_writer.write_byte(0x0E),
            ReturnCode::E2ENoNewData => byte_writer.write_byte(0x0F),
            ReturnCode::Reserved0(byte) => byte_writer.write_byte(*byte),
            ReturnCode::Reserved1(byte) => byte_writer.write_byte(*byte),
        }
    }
}

/// SOME/IP packet payload.
#[derive(Debug, PartialEq)]
pub struct Payload<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Payload<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Payload::new(bytes)
    }
}

impl AsRef<[u8]> for Payload<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> Payload<'a> {
    /// Creates a new [`Payload`] from the given bytes.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Returns the internal payload slice.
    pub fn into_inner(self) -> &'a [u8] {
        self.0
    }
}

/// SOME/IP header.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct Header {
    message_id: MessageId,
    length: Length,
    request_id: RequestId,
    protocol_version: ProtocolVersion,
    interface_version: InterfaceVersion,
    message_type: MessageType,
    return_code: ReturnCode,
}

impl Header {
    /// Creates a new [`Header`].
    pub fn new(
        message_id: MessageId,
        length: Length,
        request_id: RequestId,
        protocol_version: ProtocolVersion,
        interface_version: InterfaceVersion,
        message_type: MessageType,
        return_code: ReturnCode,
    ) -> Self {
        Self {
            message_id,
            length,
            request_id,
            protocol_version,
            interface_version,
            message_type,
            return_code,
        }
    }

    /// Returns the [`MessageId`].
    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    /// Returns the [`Length`].
    pub fn length(&self) -> Length {
        self.length
    }

    /// Returns the [`RequestId`].
    pub fn request_id(&self) -> RequestId {
        self.request_id
    }

    /// Returns the [`ProtocolVersion`].
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    /// Returns the [`InterfaceVersion`].
    pub fn interface_version(&self) -> InterfaceVersion {
        self.interface_version
    }

    /// Returns the [`MessageType`].
    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    /// Returns the [`ReturnCode`].
    pub fn return_code(&self) -> ReturnCode {
        self.return_code
    }

    /// Returns the [`MessageId`].
    pub fn set_message_id(&mut self, message_id: MessageId) {
        self.message_id = message_id;
    }

    /// Sets the [`Length`].
    pub fn set_length(&mut self, length: Length) {
        self.length = length;
    }

    /// Sets the [`RequestId`].
    pub fn set_request_id(&mut self, request_id: RequestId) {
        self.request_id = request_id;
    }

    /// Sets the [`ProtocolVersion`].
    pub fn set_protocol_version(&mut self, protocol_version: ProtocolVersion) {
        self.protocol_version = protocol_version;
    }

    /// Sets the [`InterfaceVersion`].
    pub fn set_interface_version(&mut self, interface_version: InterfaceVersion) {
        self.interface_version = interface_version;
    }

    /// Sets the [`MessageType`].
    pub fn set_message_type(&mut self, message_type: MessageType) {
        self.message_type = message_type;
    }

    /// Sets the [`ReturnCode`].
    pub fn set_return_code(&mut self, return_code: ReturnCode) {
        self.return_code = return_code;
    }

    /// Splits the bytes into header and payload and returns the header as a [`Header`].
    pub fn parse_with_payload(bytes: &[u8]) -> Result<(Header, Payload<'_>), ParseError> {
        let mut reader = ByteReader::new(bytes);

        let header = Header::parse_partial(&mut reader)?;
        let payload = Payload(reader.remaining_slice());

        Ok((header, payload))
    }

    /// Serializes the header and the payload into one packet.
    pub fn serialize_with_payload<'a>(
        &mut self,
        payload: Payload,
        buffer: &'a mut [u8],
    ) -> Result<&'a [u8], SerializeError> {
        let mut byte_writer = ByteWriter::new(buffer);

        self.length = Length::from_payload_length(payload.as_ref().len() as u32);

        let written = byte_writer.write_counted(|byte_writer| {
            self.serialize_partial(byte_writer)?;
            byte_writer.write_slice(payload.as_ref())
        })?;

        Ok(&buffer[..written])
    }

    /// Serializes the header and the payload into one packet.
    pub fn serialize_with_serializable<'a>(
        &mut self,
        payload: &impl Serialize,
        buffer: &'a mut [u8],
    ) -> Result<&'a [u8], SerializeError> {
        let mut byte_writer = ByteWriter::new(buffer);

        self.length = Length::from_payload_length(payload.required_length() as u32);

        let written = byte_writer.write_counted(|byte_writer| {
            self.serialize_partial(byte_writer)?;
            payload.serialize_partial(byte_writer)
        })?;

        Ok(&buffer[..written])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {

    use pretty_assertions::assert_eq;

    use super::{
        ClientId, Header, InterfaceVersion, Length, MessageId, MessageType, MethodId, Payload,
        ProtocolVersion, RequestId, ReturnCode, ServiceId, SessionId,
    };
    use crate::header::{ClientIdInner, Prefix};
    use crate::parse::{Parse, ParseError, ParseExt};
    use crate::serialize::{Serialize, SerializeError};

    const SOMEIP_PACKET_BYTES: &[u8] = &[
        0x12, 0x34, // Service ID
        0x56, 0x78, // Method ID
        0x00, 0x00, 0x00, 0x12, // Length (18 bytes)
        0x9A, 0xBC, // Client ID
        0xDE, 0xF0, // Session ID
        0x01, // Protocol Version
        0x02, // Interface Version
        0x01, // Message Type
        0x00, // Return Code
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, // Payload
    ];

    #[test]
    fn conversion() {
        const EXPECTED_DATA: &[u8] = &[
            0, 1, // Service ID
            0, 2, // Method ID
            0, 0, 0, 3, // Length
            4, 5, // CLient ID
            0, 6, // Session ID
            7, // Protocol Version
            8, // Interface Version
            0, // Message Type
            0, // Return Code
        ];

        let header = Header {
            message_id: MessageId::new(ServiceId(1), MethodId(2)),
            length: Length(3),
            request_id: RequestId::new(ClientId::new(4.into(), 5.into()), SessionId(6)),
            protocol_version: ProtocolVersion(7),
            interface_version: InterfaceVersion(8),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };

        test_round_trip!(Header, header, EXPECTED_DATA);
    }

    #[test]
    fn parse_with_payload_cut_off() {
        for cut_off in 0..16 {
            assert_eq!(
                Header::parse_with_payload(&SOMEIP_PACKET_BYTES[..cut_off]),
                Err(crate::parse::ParseError::PayloadTooShort)
            );
        }
    }

    #[test]
    fn payload_from() {
        let payload_data = [10, 20, 30];

        let payload = Payload::new(payload_data.as_slice());
        assert_eq!(payload.as_ref(), payload_data);

        let payload = Payload::from(payload_data.as_slice());
        assert_eq!(payload.as_ref(), payload_data);
    }

    #[test]
    fn payload_into_inner() {
        let payload_data = [10, 20, 30];

        let payload = Payload::new(payload_data.as_slice());
        assert_eq!(payload.into_inner(), payload_data);
    }

    #[test]
    fn payload_length() {
        let (header, payload) = Header::parse_with_payload(SOMEIP_PACKET_BYTES).unwrap();

        assert_eq!(payload.as_ref(), &SOMEIP_PACKET_BYTES[16..]);
        assert_eq!(
            header.length.payload_length() as usize,
            payload.as_ref().len()
        );
    }

    #[test]
    fn set_header_length_field() {
        let mut header = Header {
            message_id: MessageId::new(ServiceId(0), MethodId(0)),
            length: Length(0),
            request_id: RequestId::new(ClientId::new(0.into(), 0.into()), SessionId(0)),
            protocol_version: ProtocolVersion(0),
            interface_version: InterfaceVersion(0),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };
        let payload = [1, 2, 3, 4, 5];

        let mut buffer = [0u8; 128];
        let serialized = header
            .serialize_with_payload(Payload(&payload), &mut buffer)
            .unwrap();

        let (parsed_header, parsed_payload) = Header::parse_with_payload(serialized).unwrap();

        assert_eq!(
            parsed_header.length.payload_length() as usize,
            payload.len()
        );
        assert_eq!(parsed_payload.as_ref(), &payload);
    }

    #[test]
    fn serialize_with_payload_buffer_too_small() {
        let mut header = Header {
            message_id: MessageId::new(ServiceId(0), MethodId(0)),
            length: Length(0),
            request_id: RequestId::new(ClientId::new(0.into(), 0.into()), SessionId(0)),
            protocol_version: ProtocolVersion(0),
            interface_version: InterfaceVersion(0),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };
        let payload = [1, 2, 3, 4, 5];

        let mut buffer = [0u8; 128];

        for buffer_length in 0..header.required_length() + payload.len() {
            assert_eq!(
                header.serialize_with_payload(Payload(&payload), &mut buffer[..buffer_length]),
                Err(SerializeError::BufferTooSmall)
            );
        }
    }

    #[test]
    fn getters_setters() {
        let mut header = Header {
            message_id: MessageId::new(ServiceId(0), MethodId(0)),
            length: Length(0),
            request_id: RequestId::new(ClientId::new(0.into(), 0.into()), SessionId(0)),
            protocol_version: ProtocolVersion(0),
            interface_version: InterfaceVersion(0),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };

        let mut message_id = MessageId::new(ServiceId(0), MethodId(0));

        let service_id = ServiceId(8);
        let method_id = MethodId(9);
        message_id.set_service_id(service_id);
        message_id.set_method_id(method_id);

        let mut client_id = ClientId::new(Prefix(0), ClientIdInner(0));

        let prefix = Prefix(8);
        let client_id_inner = ClientIdInner(9);
        client_id.set_prefix(prefix);
        client_id.set_id(client_id_inner);

        let length = Length(20);

        let mut request_id =
            RequestId::new(ClientId::new(Prefix(0), ClientIdInner(0)), SessionId(0));

        let session_id = SessionId(10);
        request_id.set_client_id(client_id);
        request_id.set_session_id(session_id);

        let protocol_version = ProtocolVersion(8);
        let interface_version = InterfaceVersion(9);
        let message_type = MessageType::Response;
        let return_code = ReturnCode::NotOk;

        header.set_message_id(message_id);
        header.set_length(length);
        header.set_request_id(request_id);
        header.set_protocol_version(protocol_version);
        header.set_interface_version(interface_version);
        header.set_message_type(message_type);
        header.set_return_code(return_code);

        assert_eq!(header.message_id().service_id(), service_id);
        assert_eq!(header.message_id().method_id(), method_id);

        assert_eq!(header.length(), length);

        assert_eq!(header.request_id().client_id().prefix(), prefix);
        assert_eq!(header.request_id().client_id().id(), client_id_inner);
        assert_eq!(header.request_id().session_id(), session_id);

        assert_eq!(header.protocol_version(), protocol_version);
        assert_eq!(header.interface_version(), interface_version);
        assert_eq!(header.message_type(), message_type);
        assert_eq!(header.return_code(), return_code);
    }

    #[test]
    fn message_id_from_u32() {
        const BYTES: [u8; 4] = [0x1, 0x2, 0x3, 0x4];

        let parsed_message_id = MessageId::parse(&BYTES).unwrap();
        let created_message_id = MessageId::new(
            ServiceId::from(u16::from_be_bytes(BYTES[..2].try_into().unwrap())),
            MethodId::from(u16::from_be_bytes(BYTES[2..].try_into().unwrap())),
        );

        assert_eq!(
            parsed_message_id.service_id(),
            created_message_id.service_id()
        );
        assert_eq!(
            parsed_message_id.method_id(),
            created_message_id.method_id()
        );
    }

    #[test]
    fn session_id_next() {
        assert_eq!(SessionId(0).next(), SessionId(0));
        assert_eq!(SessionId(1).next(), SessionId(2));
        assert_eq!(SessionId(0xFFFF).next(), SessionId(1));
    }

    #[test]
    fn message_types() {
        const EXPECTED_DATA: &[u8] = &[0, 1, 2, 128, 129, 32, 33, 34, 160, 161];

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
        struct Test(
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
            MessageType,
        );

        let message_types = Test(
            MessageType::Request,
            MessageType::RequestNoReturn,
            MessageType::Notification,
            MessageType::Response,
            MessageType::Error,
            MessageType::TpRequest,
            MessageType::TpRequestNoReturn,
            MessageType::TpNotification,
            MessageType::TpResponse,
            MessageType::TpError,
        );

        test_round_trip!(Test, message_types, EXPECTED_DATA);
    }

    #[test]
    fn invalid_message_type() {
        const USED_VALUES: &[u8] = &[0x00, 0x01, 0x02, 0x80, 0x81, 0x20, 0x21, 0x22, 0xA0, 0xA1];

        for byte in 0x00..0xFF {
            if !USED_VALUES.contains(&byte) {
                assert!(matches!(
                    MessageType::parse(&[byte]),
                    Err(ParseError::MalformedMessage { .. })
                ));
            }
        }
    }

    #[test]
    fn return_codes() {
        const EXPECTED_DATA: &[u8] =
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 32];

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
        struct Test(
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
            ReturnCode,
        );

        let return_codes = Test(
            ReturnCode::Ok,
            ReturnCode::NotOk,
            ReturnCode::UnknownService,
            ReturnCode::UnknownMethod,
            ReturnCode::NotReady,
            ReturnCode::NotReachable,
            ReturnCode::Timeout,
            ReturnCode::WrongProtocolVersion,
            ReturnCode::WrongInterfaceVersion,
            ReturnCode::MalformedMessage,
            ReturnCode::WrongMessageType,
            ReturnCode::E2ERepeated,
            ReturnCode::E2EWrongSequence,
            ReturnCode::E2E,
            ReturnCode::E2ENotAvailable,
            ReturnCode::E2ENoNewData,
            ReturnCode::Reserved0(0x10),
            ReturnCode::Reserved1(0x20),
        );

        test_round_trip!(Test, return_codes, EXPECTED_DATA);
    }

    #[test]
    fn invalid_return_code() {
        for byte in 0x5F..0xFF {
            assert!(matches!(
                ReturnCode::parse(&[byte]),
                Err(ParseError::MalformedMessage { .. })
            ));
        }
    }

    #[test]
    fn serialize_with_serializable() {
        #[derive(Debug, Parse, Serialize, Eq, PartialEq)]
        pub struct SerializablePayload {
            pub data: u32,
            pub boolean: bool,
        }

        let mut header = Header {
            message_id: MessageId::new(ServiceId(0), MethodId(0)),
            length: Length(0),
            request_id: RequestId::new(ClientId::new(0.into(), 0.into()), SessionId(0)),
            protocol_version: ProtocolVersion(0),
            interface_version: InterfaceVersion(0),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };
        let payload = SerializablePayload {
            data: 1,
            boolean: true,
        };

        let mut buffer = [0u8; 128];
        let serialized = header
            .serialize_with_serializable(&payload, &mut buffer)
            .unwrap();

        let (parsed_header, parsed_payload) = Header::parse_with_payload(serialized).unwrap();

        assert_eq!(
            parsed_header.length.payload_length() as usize,
            payload.required_length()
        );

        let parsed_payload = SerializablePayload::parse(parsed_payload.into_inner()).unwrap();
        assert_eq!(&parsed_payload, &payload);
    }

    #[test]
    fn serialize_with_serializable_buffer_too_small() {
        #[derive(Debug, Parse, Serialize, Eq, PartialEq)]
        pub struct SerializablePayload {
            pub data: u32,
            pub boolean: bool,
        }

        let mut header = Header {
            message_id: MessageId::new(ServiceId(0), MethodId(0)),
            length: Length(0),
            request_id: RequestId::new(ClientId::new(0.into(), 0.into()), SessionId(0)),
            protocol_version: ProtocolVersion(0),
            interface_version: InterfaceVersion(0),
            message_type: MessageType::Request,
            return_code: ReturnCode::Ok,
        };
        let payload = SerializablePayload {
            data: 1,
            boolean: true,
        };

        let mut buffer_header_fail = [0u8; 0];
        assert_eq!(
            header.serialize_with_serializable(&payload, &mut buffer_header_fail),
            Err(SerializeError::BufferTooSmall)
        );

        let mut buffer_payload_fail = [0u8; 17];
        assert_eq!(
            header.serialize_with_serializable(&payload, &mut buffer_payload_fail),
            Err(SerializeError::BufferTooSmall)
        );
    }
}
