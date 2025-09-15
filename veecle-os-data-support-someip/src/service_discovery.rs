//! Provides support for serialization and deserialization of SOME/IP service discovery payloads.

use bitflags::bitflags;

use crate::array::DynamicLengthArray;
use crate::parse::{ByteReader, Parse, ParseError};
use crate::serialize::{ByteWriter, Serialize, SerializeError};

/// Implements [`Parse`] and [`Serialize`] for types implementing [`Flags`](bitflags::Flags).
macro_rules! impl_for_bitflags {
    ($name:ident) => {
        impl<'a> Parse<'a> for $name {
            fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
                <Self as bitflags::Flags>::Bits::parse_partial(reader)
                    .map($name::from_bits_truncate)
            }
        }

        impl Serialize for $name {
            fn required_length(&self) -> usize {
                self.bits().required_length()
            }

            fn serialize_partial(
                &self,
                byte_writer: &mut ByteWriter,
            ) -> Result<(), SerializeError> {
                self.bits().serialize_partial(byte_writer)
            }
        }
    };
}

bitflags! {
    /// Service Discovery header flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HeaderFlags: u8 {
        /// Set to 1 after reboot until Session-ID wraps to 1, then 0.
        const REBOOT = 0b00000001;
        /// Set to 1 for all SD messages to indicate unicast support.
        const UNICAST = 0b00000010;
    }
}

impl_for_bitflags!(HeaderFlags);

/// SOME/IP service discovery header.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct Header<'a> {
    /// Service discovery flags.
    pub flags: HeaderFlags,

    /// Reserved bits.
    /// Should be set to 0.
    pub reserved: Reserved,

    /// Array of service [`Entry`]'s.
    pub entries: DynamicLengthArray<'a, Entry, u32, 32>,

    /// Array of [service_discovery::Option](`crate::service_discovery::Option`) for [`Self::entries`].
    pub options: DynamicLengthArray<'a, Option<'a>, u32, 32>,
}

/// SOME/IP service discovery header reserved bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reserved;

impl Parse<'_> for Reserved {
    fn parse_partial(reader: &mut ByteReader<'_>) -> Result<Self, ParseError> {
        let _ = reader.read_slice(3)?;
        Ok(Self)
    }
}

impl Serialize for Reserved {
    fn required_length(&self) -> usize {
        3
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(&[0; 3])
    }
}

/// SOME/IP service discovery entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// Service Entry with type 0x00.
    FindService(ServiceEntry),
    /// Service Entry with type 0x01.
    OfferService(ServiceEntry),
    /// Eventgroup entry with type 0x06.
    SubscribeEventgroup(EventgroupEntry),
    /// Eventgroup entry with type 0x07.
    SubscribeEventgroupAck(EventgroupEntry),
}

impl<'a> Parse<'a> for Entry {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let r#type = reader.read_byte()?;

        let entry = match r#type {
            0x00 => Entry::FindService(ServiceEntry::parse_partial(reader)?),
            0x01 => Entry::OfferService(ServiceEntry::parse_partial(reader)?),
            0x06 => Entry::SubscribeEventgroup(EventgroupEntry::parse_partial(reader)?),
            0x07 => Entry::SubscribeEventgroupAck(EventgroupEntry::parse_partial(reader)?),
            _invalid => {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }
        };

        Ok(entry)
    }
}

impl Serialize for Entry {
    fn required_length(&self) -> usize {
        1 + match self {
            Entry::FindService(service_entry) => service_entry.required_length(),
            Entry::OfferService(service_entry) => service_entry.required_length(),
            Entry::SubscribeEventgroup(eventgroup_entry) => eventgroup_entry.required_length(),
            Entry::SubscribeEventgroupAck(eventgroup_entry) => eventgroup_entry.required_length(),
        }
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        match self {
            Entry::FindService(service_entry) => {
                byte_writer.write_byte(0x00)?;
                service_entry.serialize_partial(byte_writer)
            }
            Entry::OfferService(service_entry) => {
                byte_writer.write_byte(0x01)?;
                service_entry.serialize_partial(byte_writer)
            }
            Entry::SubscribeEventgroup(eventgroup_entry) => {
                byte_writer.write_byte(0x06)?;
                eventgroup_entry.serialize_partial(byte_writer)
            }
            Entry::SubscribeEventgroupAck(eventgroup_entry) => {
                byte_writer.write_byte(0x07)?;
                eventgroup_entry.serialize_partial(byte_writer)
            }
        }
    }
}

/// Service entry.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct ServiceEntry {
    /// Index of option in [`Header::options`] where first option run begins.
    pub first_option: u8,

    /// Index of option in [`Header::options`] where second option run begins.
    pub second_option: u8,

    /// Number of options in the first and second option runs.
    /// Split into two u4 (first and second option runs respectively).
    pub option_counts: u8,

    /// ID of the service this entry belongs to.
    pub service_id: u16,

    /// ID of the service instance this entry belongs to.
    /// When set to 0xffff, then this entry belongs to all service instances.
    pub instance_id: u16,

    /// Major version of the service and lifetime of this entry (in seconds).
    /// Split into 8 (major version) and 24 (TTL) bits.
    pub major_version_ttl: u32,

    /// Minor version of the service.
    pub minor_version: u32,
}

/// A wrapper type to gracefully parse the two `u4` option counts of the [`EventgroupEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct OptionsCount {
    inner: u8,
}

impl OptionsCount {
    /// Describes the number of options the first option run uses.
    ///
    /// Represents a u4 in the Payload.
    pub fn first(&self) -> u8 {
        self.inner & 0x0F
    }

    /// Describes the number of options the second option run uses.
    ///
    /// Represents a u4 in the Payload.
    pub fn second(&self) -> u8 {
        self.inner >> 4
    }
}

/// A wrapper type to gracefully parse the reserved `u12` and `u4` counter of the [`EventgroupEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Parse, Serialize)]
pub struct Counter {
    inner: u16,
}

impl Counter {
    /// Is used to differentiate identical Subscribe Eventgroups of the same subscriber. Set to 0x0 if not used.
    ///
    /// Represents a u4 in the Payload.
    pub fn counter(&self) -> u8 {
        (self.inner >> 12) as u8
    }
}

/// Eventgroup entry.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct EventgroupEntry {
    /// Index of this runs first option in the option array.
    pub first_option: u8,
    /// Index of this runs second option in the option array.
    pub second_option: u8,
    /// Describes the number of options the first and second option run uses.
    pub option_counts: OptionsCount,
    /// Describes the Service ID of the Service or Service-Instance this entry is concerned with.
    pub service_id: u16,
    /// Describes the Service Instance ID of the Service Instance this entry is concerned with or is set to 0xFFFF if
    /// all service instances of a service are meant.
    pub instance_id: u16,
    ///  Encodes the major version of the service (instance).
    pub major_version: u8,
    /// Describes the lifetime of the entry in seconds.
    pub ttl: Ttl,
    /// Is used to differentiate identical Subscribe Eventgroups of the same subscriber. Set to 0x0 if not used.
    ///
    /// This type also includes the reserved bytes.
    pub counter: Counter,
    /// Transports the ID of an Eventgroup.
    pub eventgroup_id: u16,
}

/// Lifetime of the entry in seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ttl {
    /// Lifetime of the entry in seconds.
    ///
    /// Represents a u24 in the payload.
    pub seconds: u32,
}

impl Parse<'_> for Ttl {
    fn parse_partial(reader: &mut ByteReader<'_>) -> Result<Self, ParseError> {
        let bytes = reader.read_slice(3)?;

        Ok(Self {
            seconds: u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]),
        })
    }
}

impl Serialize for Ttl {
    fn required_length(&self) -> usize {
        3
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(&self.seconds.to_be_bytes().as_slice()[1..])
    }
}

/// SOME/IP service discovery option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Option<'a> {
    /// Configuration Option.
    Configuration(ConfigurationOption<'a>),
    /// Load Balancing Option.
    LoadBalancing(LoadBalancingOption),
    /// IPv4 Endpoint Option.
    Ipv4Endpoint(IpV4Option),
    /// IPv6 Endpoint Option.
    Ipv6Endpoint(IpV6Option),
    /// IPv4 Multicast Option.
    Ipv4Multicast(IpV4Option),
    /// IPv6 Multicast Option.
    Ipv6Multicast(IpV6Option),
    /// IPv4 SD Endpoint Option.
    Ipv4SdEndpoint(IpV4Option),
    /// IPv6 SD Endpoint Option.
    Ipv6SdEndpoint(IpV6Option),
}

impl<'a> Parse<'a> for Option<'a> {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let length = u16::parse_partial(reader)? as usize;
        let r#type = u8::parse_partial(reader)?;

        let mut option_reader = reader.sub_reader(length)?;
        let option = match r#type {
            0x01 => Option::Configuration(ConfigurationOption::parse_partial(&mut option_reader)?),
            0x02 => Option::LoadBalancing(
                LoadBalancingOption::parse_partial(&mut option_reader).unwrap(),
            ),
            0x04 => Option::Ipv4Endpoint(
                IpOption::<Ipv4Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            0x06 => Option::Ipv6Endpoint(
                IpOption::<Ipv6Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            0x14 => Option::Ipv4Multicast(
                IpOption::<Ipv4Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            0x16 => Option::Ipv6Multicast(
                IpOption::<Ipv6Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            0x24 => Option::Ipv4SdEndpoint(
                IpOption::<Ipv4Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            0x26 => Option::Ipv6SdEndpoint(
                IpOption::<Ipv6Address>::parse_partial(&mut option_reader).unwrap(),
            ),
            _invalid => {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }
        };

        Ok(option)
    }
}

impl Serialize for Option<'_> {
    fn required_length(&self) -> usize {
        3 + match self {
            Option::Configuration(configuration_option) => configuration_option.required_length(),
            Option::LoadBalancing(load_balancing_option) => load_balancing_option.required_length(),
            Option::Ipv4Endpoint(ip_option) => ip_option.required_length(),
            Option::Ipv6Endpoint(ip_option) => ip_option.required_length(),
            Option::Ipv4Multicast(ip_option) => ip_option.required_length(),
            Option::Ipv6Multicast(ip_option) => ip_option.required_length(),
            Option::Ipv4SdEndpoint(ip_option) => ip_option.required_length(),
            Option::Ipv6SdEndpoint(ip_option) => ip_option.required_length(),
        }
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        let reserved_length = byte_writer.reserve_length()?;

        let length = match self {
            Option::Configuration(configuration_option) => {
                byte_writer.write_byte(0x01)?;
                byte_writer.write_counted(|byte_writer| {
                    configuration_option.serialize_partial(byte_writer)
                })?
            }
            Option::LoadBalancing(load_balancing_option) => {
                byte_writer.write_byte(0x02)?;
                byte_writer.write_counted(|byte_writer| {
                    load_balancing_option.serialize_partial(byte_writer)
                })?
            }
            Option::Ipv4Endpoint(ip_option) => {
                byte_writer.write_byte(0x04)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
            Option::Ipv6Endpoint(ip_option) => {
                byte_writer.write_byte(0x06)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
            Option::Ipv4Multicast(ip_option) => {
                byte_writer.write_byte(0x14)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
            Option::Ipv6Multicast(ip_option) => {
                byte_writer.write_byte(0x16)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
            Option::Ipv4SdEndpoint(ip_option) => {
                byte_writer.write_byte(0x24)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
            Option::Ipv6SdEndpoint(ip_option) => {
                byte_writer.write_byte(0x26)?;
                byte_writer.write_counted(|byte_writer| ip_option.serialize_partial(byte_writer))?
            }
        };

        byte_writer.write_length(reserved_length, &(length as u16))
    }
}

/// Array of [`ConfigurationString`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationArray<'a> {
    /// Reader for the configuration array items.
    reader: ByteReader<'a>,
}

impl<'a> ConfigurationArray<'a> {
    /// Creates a new configuration array.
    pub fn create<'b, I>(
        mut strings: I,
        buffer: &'a mut [u8],
    ) -> Result<ConfigurationArray<'a>, SerializeError>
    where
        I: Iterator<Item = &'b ConfigurationString<'b>>,
    {
        let mut byte_writer = ByteWriter::new(buffer);

        let used_bytes = byte_writer.write_counted(move |byte_writer| {
            for string in strings.by_ref() {
                string.serialize_partial(byte_writer)?;
            }

            // Zero termination.
            byte_writer.write_byte(0x0)
        })?;

        let reader = ByteReader::new(&buffer[..used_bytes]);

        Ok(Self { reader })
    }

    /// Returns an iterator over all [`ConfigurationString`]s.
    pub fn iter(&'a self) -> ConfigurationArrayIter<'a> {
        ConfigurationArrayIter {
            reader: self.reader.clone(),
        }
    }
}

impl<'a> Parse<'a> for ConfigurationArray<'a> {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let reader = reader.take_remaining();

        // Ensure that all the items are valid and the array is zero-terminated.
        {
            let mut item_reader = reader.clone();

            while item_reader.len() > 1 {
                let _ = ConfigurationString::parse_partial(&mut item_reader)?;
            }

            if item_reader.read_byte().unwrap() != 0 {
                return Err(ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                });
            }
        }

        Ok(Self { reader })
    }
}

impl Serialize for ConfigurationArray<'_> {
    fn required_length(&self) -> usize {
        self.reader.len()
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.reader.remaining_slice())
    }
}

/// Iterator over [`ConfigurationString`]s.
#[derive(Debug)]
pub struct ConfigurationArrayIter<'a> {
    reader: ByteReader<'a>,
}

impl<'a> Iterator for ConfigurationArrayIter<'a> {
    type Item = ConfigurationString<'a>;

    fn next(&mut self) -> core::option::Option<Self::Item> {
        if self.reader.len() == 1 {
            return None;
        }

        Some(ConfigurationString::parse_partial(&mut self.reader).unwrap())
    }
}

bitflags! {
    /// Service Discovery configuration option flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConfigurationOptionFlags: u8 {
        /// Shall be set to 1 if the Option can be discarded by the receiver.
        const DISCARDABLE = 0b00000001;
    }
}

impl_for_bitflags!(ConfigurationOptionFlags);

/// Configuration Option.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct ConfigurationOption<'a> {
    /// Configuration option flags.
    pub flag_reserved: ConfigurationOptionFlags,
    /// Array of all configuration strings.
    pub configuration_strings: ConfigurationArray<'a>,
}

/// Value of a [`ConfigurationString`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationStringValue<'a> {
    /// A key without a value (e.g. "foo")
    None,
    /// A key with an empty value (e.g. "foo=")
    Empty,
    /// A key with a value (e.g. "foo=bar")
    Value(&'a str),
}

// Length is a one byte field.
// The last configuration string has a length of 0, which indicates the end of the configuration option.
// Every string has at least a key, optionally a value.
// Strings with '=', shall be key-value if characters follow the '=', if not the value shall be considered empty.
// Strings without '=' shall be considered present.
// Key ends with '=' and shall only contain the ASCII characters 0x20-0x7E excluding 0x3D ('=').
// The key must contain at least one non-whitespace character.
/// Configuration string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationString<'a> {
    /// Key of a configuration string.
    pub key: &'a str,
    /// Value of a configuration string.
    pub value: ConfigurationStringValue<'a>,
}

impl<'a> Parse<'a> for ConfigurationString<'a> {
    fn parse_partial(reader: &mut ByteReader<'a>) -> Result<Self, ParseError> {
        let length = reader.read_byte()? as usize;
        let string_slice = reader.read_slice(length)?;

        // Find position of '='.
        // Everything before the `=` is the key, the rest will be deserialized by `ConfigStringValue`.
        let equal_position = string_slice.iter().position(|byte| *byte == b'=');
        let key_length = equal_position.unwrap_or(length);
        let (key_slice, value_slice) = string_slice.split_at(key_length);

        // Check for forbidden characters.
        if key_slice
            .iter()
            // 0x3D ('=') is impossible due it being the key-value delimiter and used to derive `key_length`.
            .any(|byte| !matches!(byte, 0x20..0x7E))
        {
            return Err(ParseError::MalformedMessage {
                failed_at: core::any::type_name::<Self>(),
            });
        }

        // Check for at least one non-whitespace character.
        if !key_slice.iter().any(|byte| !byte.is_ascii_whitespace()) {
            return Err(ParseError::MalformedMessage {
                failed_at: core::any::type_name::<Self>(),
            });
        }

        let key = core::str::from_utf8(key_slice).unwrap();

        let value = if key_length == length {
            ConfigurationStringValue::None
        } else if key_length == (length) - 1 {
            ConfigurationStringValue::Empty
        } else {
            let value = core::str::from_utf8(&value_slice[1..]).map_err(|_| {
                ParseError::MalformedMessage {
                    failed_at: core::any::type_name::<Self>(),
                }
            })?;

            ConfigurationStringValue::Value(value)
        };

        Ok(Self { key, value })
    }
}

impl Serialize for ConfigurationString<'_> {
    fn required_length(&self) -> usize {
        1 + self.key.len()
            + match self.value {
                ConfigurationStringValue::None => 0,
                ConfigurationStringValue::Empty => 1,
                ConfigurationStringValue::Value(value) => value.len() + 1,
            }
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        let reserved_length = byte_writer.reserve_length()?;

        let length = byte_writer.write_counted(|byte_writer| {
            byte_writer.write_slice(self.key.as_bytes())?;

            match self.value {
                ConfigurationStringValue::None => Ok(()),
                ConfigurationStringValue::Empty => byte_writer.write_byte(b'='),
                ConfigurationStringValue::Value(value) => {
                    byte_writer.write_byte(b'=')?;
                    byte_writer.write_slice(value.as_bytes())
                }
            }
        })?;

        let length = u8::try_from(length).map_err(|_| SerializeError::LengthOverflow)?;
        byte_writer.write_length(reserved_length, &length)
    }
}

bitflags! {
    /// Service Discovery load balancing option flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LoadBalancingOptionFlags: u8 {
        /// Shall be set to 1 if the Option can be discarded by the receiver.
        const DISCARDABLE = 0b00000001;
    }
}

impl_for_bitflags!(LoadBalancingOptionFlags);

/// Load Balancing Option.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct LoadBalancingOption {
    /// Load balancing option flags.
    pub flag_reserved: LoadBalancingOptionFlags,
    /// Carries the Priority of this instance. Lower value means higher priority.
    pub priority: u16,
    /// Carries the Weight of this instance. Large value means higher probability to be chosen.
    pub weight: u16,
}

/// An arbitrary IP Option.
///
/// This type is a single representation for all the Ipv4 and Ipv6 options.
#[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
pub struct IpOption<T>
where
    T: for<'p> Parse<'p> + Serialize,
{
    /// Discardable flag and reserved bits.
    /// Split into 1 bit (discardable) and 7 bits (reserved). Both shall be set to 0.
    pub flag_reserved: u8,

    /// IPv4-address.
    /// Shall contain the unicast IP-address.
    pub address: T,

    /// Reserved bytes.
    /// Shall be set to 0x00.
    pub reserved: u8,

    /// Transport protocol type.
    /// Shall be set to the transport layer protocol (ISO/OSI layer 4)
    /// based on the IANA/IETF types (0x06: TCP, 0x11: UDP).
    pub l4_proto: u8,

    /// Transport protocol port number.
    /// Shall be set to the port of the layer 4 protocol.
    pub port_number: u16,
}

/// An IPv4 address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Address {
    /// IP address octets.
    pub octets: [u8; 4],
}

impl Parse<'_> for Ipv4Address {
    fn parse_partial(reader: &mut ByteReader<'_>) -> Result<Self, ParseError> {
        Ok(Self {
            octets: reader.read_array()?,
        })
    }
}

impl Serialize for Ipv4Address {
    fn required_length(&self) -> usize {
        4
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.octets.as_slice())
    }
}

/// An IPv6 address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Address {
    /// IP address octets.
    pub octets: [u8; 16],
}

impl Parse<'_> for Ipv6Address {
    fn parse_partial(reader: &mut ByteReader<'_>) -> Result<Self, ParseError> {
        Ok(Self {
            octets: reader.read_array()?,
        })
    }
}

impl Serialize for Ipv6Address {
    fn required_length(&self) -> usize {
        16
    }

    fn serialize_partial(&self, byte_writer: &mut ByteWriter) -> Result<(), SerializeError> {
        byte_writer.write_slice(self.octets.as_slice())
    }
}

/// An IPv4 Option.
pub type IpV4Option = IpOption<Ipv4Address>;

/// An IPv6 Option.
pub type IpV6Option = IpOption<Ipv6Address>;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod configuration_array {

    use crate::parse::{ParseError, ParseExt};
    use crate::serialize::SerializeError;
    use crate::service_discovery::{
        ConfigurationArray, ConfigurationString, ConfigurationStringValue,
    };

    #[test]
    fn create() {
        let strings = [ConfigurationString {
            key: "key",
            value: ConfigurationStringValue::Value("test"),
        }];

        let mut buffer = [0u8; 32];
        let array = ConfigurationArray::create(strings.iter(), &mut buffer).unwrap();

        assert!(array.iter().eq(strings.into_iter()));
    }

    #[test]
    fn create_buffer_too_small() {
        let strings = [ConfigurationString {
            key: "key",
            value: ConfigurationStringValue::Value("test"),
        }];

        let mut buffer = [0u8; 6];
        assert!(matches!(
            ConfigurationArray::create(strings.iter(), &mut buffer),
            Err(SerializeError::BufferTooSmall)
        ));
    }

    #[test]
    fn parse_not_null_terminated() {
        const TEST_DATA: &[u8] = &[4, 111, 111, 110, 101, 0xFF];

        assert!(matches!(
            ConfigurationArray::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod configuration_string {

    use crate::parse::{Parse, ParseError, ParseExt};
    use crate::serialize::{Serialize, SerializeError, SerializeExt};
    use crate::service_discovery::{ConfigurationString, ConfigurationStringValue};

    #[test]
    fn conversion() {
        const EXPECTED_DATA: &[u8] = b"\
        \x03key\
        \x04key=\
        \x09key=value\
        \x09yek=value";

        #[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
        struct Test<'a>(
            ConfigurationString<'a>,
            ConfigurationString<'a>,
            ConfigurationString<'a>,
            ConfigurationString<'a>,
        );

        let strings = Test(
            ConfigurationString {
                key: "key",
                value: ConfigurationStringValue::None,
            },
            ConfigurationString {
                key: "key",
                value: ConfigurationStringValue::Empty,
            },
            ConfigurationString {
                key: "key",
                value: ConfigurationStringValue::Value("value"),
            },
            ConfigurationString {
                key: "yek",
                value: ConfigurationStringValue::Value("value"),
            },
        );

        test_round_trip!(Test, strings, EXPECTED_DATA);
    }

    #[test]
    fn parse_zero_length() {
        const TEST_DATA: &[u8] = b"\x00";
        assert!(matches!(
            ConfigurationString::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn parse_no_key() {
        const TEST_DATA: &[u8] = b"\x01=";
        assert!(matches!(
            ConfigurationString::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn parse_key_invalid_characters() {
        const TEST_DATA: &[u8] = &[1, 0x10];
        assert!(matches!(
            ConfigurationString::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn parse_key_only_whitespaces() {
        const TEST_DATA: &[u8] = b"\x01 ";
        assert!(matches!(
            ConfigurationString::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn parse_value_invalid_utf8() {
        const TEST_DATA: &[u8] = &[5, b'k', b'=', 0xE2, 0x28, 0xA1];
        assert!(matches!(
            ConfigurationString::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }

    #[test]
    fn serialize_length_overflow() {
        let long_key = core::str::from_utf8(&[b'a'; 256]).unwrap();

        let string = ConfigurationString {
            key: long_key,
            value: ConfigurationStringValue::None,
        };

        assert_eq!(
            string.serialize(&mut [0; 512]),
            Err(SerializeError::LengthOverflow)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod header {
    use crate::array::DynamicLengthArray;
    use crate::service_discovery::{Entry, Header, HeaderFlags, Option, Reserved};

    #[test]
    fn reserved() {
        const EXPECTED_DATA: &[u8] = &[0; 3];

        let reserved = Reserved;

        test_round_trip!(Reserved, reserved, EXPECTED_DATA);
    }

    #[test]
    fn conversion() {
        const EXPECTED_DATA: &[u8] = &[
            3, // Header Flags
            0, 0, 0, // Reserved
            0, 0, 0, 0, // Entries
            0, 0, 0, 0, // Options
        ];

        let mut buffer = [0u8; 64];
        let entries =
            DynamicLengthArray::<'_, Entry, u32, 32>::create(core::iter::empty(), &mut buffer)
                .unwrap();

        let mut buffer = [0u8; 64];
        let options =
            DynamicLengthArray::<'_, Option, u32, 32>::create(core::iter::empty(), &mut buffer)
                .unwrap();

        let header = Header {
            flags: HeaderFlags::all(),
            reserved: Reserved,
            entries,
            options,
        };

        test_round_trip!(Header, header, EXPECTED_DATA);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod entry {

    use super::{Counter, Entry, EventgroupEntry, OptionsCount, ServiceEntry};
    use crate::parse::{Parse, ParseError, ParseExt};
    use crate::serialize::Serialize;
    use crate::service_discovery::Ttl;

    #[test]
    fn ttl() {
        const EXPECTED_DATA: &[u8] = &[5, 59, 176];

        let ttl = Ttl { seconds: 342960 };

        test_round_trip!(Ttl, ttl, EXPECTED_DATA);
    }

    #[test]
    fn entry() {
        const EXPECTED_DATA: &[u8] = &[
            0, 1, 2, 3, 0, 4, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, // Find service
            1, 1, 2, 3, 0, 4, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, // Offers service
            6, 1, 2, 3, 0, 4, 0, 5, 6, 0, 0, 7, 0, 8, 0, 9, // Subscribe eventgroup
            7, 1, 2, 3, 0, 4, 0, 5, 6, 0, 0, 7, 0, 8, 0, 9, // Subscribe eventgroup ack
        ];

        #[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
        struct Test(Entry, Entry, Entry, Entry);

        let entries = Test(
            Entry::FindService(ServiceEntry {
                first_option: 1,
                second_option: 2,
                option_counts: 3,
                service_id: 4,
                instance_id: 5,
                major_version_ttl: 6,
                minor_version: 7,
            }),
            Entry::OfferService(ServiceEntry {
                first_option: 1,
                second_option: 2,
                option_counts: 3,
                service_id: 4,
                instance_id: 5,
                major_version_ttl: 6,
                minor_version: 7,
            }),
            Entry::SubscribeEventgroup(EventgroupEntry {
                first_option: 1,
                second_option: 2,
                option_counts: OptionsCount { inner: 3 },
                service_id: 4,
                instance_id: 5,
                major_version: 6,
                ttl: Ttl { seconds: 7 },
                counter: Counter { inner: 8 },
                eventgroup_id: 9,
            }),
            Entry::SubscribeEventgroupAck(EventgroupEntry {
                first_option: 1,
                second_option: 2,
                option_counts: OptionsCount { inner: 3 },
                service_id: 4,
                instance_id: 5,
                major_version: 6,
                ttl: Ttl { seconds: 7 },
                counter: Counter { inner: 8 },
                eventgroup_id: 9,
            }),
        );

        test_round_trip!(Test, entries, EXPECTED_DATA);
    }

    #[test]
    fn invalid_entry() {
        const USED_VALUES: &[u8] = &[0x00, 0x01, 0x06, 0x07];

        for byte in 0x00..0xFF {
            if !USED_VALUES.contains(&byte) {
                assert!(matches!(
                    Entry::parse(&[byte]),
                    Err(ParseError::MalformedMessage { .. })
                ));
            }
        }
    }

    #[test]
    fn options_count() {
        let count = OptionsCount { inner: 0xAF };

        assert_eq!(count.first(), 0xF);
        assert_eq!(count.second(), 0xA);
    }

    #[test]
    fn counter() {
        let counter = Counter { inner: 0x9000 };
        assert_eq!(counter.counter(), 0x9);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod option {

    use super::Option;
    use crate::parse::{Parse, ParseError, ParseExt};
    use crate::serialize::Serialize;
    use crate::service_discovery::{
        ConfigurationArray, ConfigurationOption, ConfigurationOptionFlags, ConfigurationString,
        ConfigurationStringValue, IpV4Option, IpV6Option, Ipv4Address, Ipv6Address,
        LoadBalancingOption, LoadBalancingOptionFlags,
    };

    #[test]
    fn ipv4_address() {
        const EXPECTED_DATA: &[u8] = &[192, 1, 2, 3];

        let ip_address = Ipv4Address {
            octets: [192, 1, 2, 3],
        };

        test_round_trip!(Ipv4Address, ip_address, EXPECTED_DATA);
    }

    #[test]
    fn ipv6_address() {
        const EXPECTED_DATA: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        let ip_address = Ipv6Address {
            octets: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        };

        test_round_trip!(Ipv6Address, ip_address, EXPECTED_DATA);
    }

    #[test]
    fn options() {
        const EXPECTED_DATA: &[u8] = &[
            0, 25, 1, 0, 4, 110, 111, 110, 101, 6, 101, 109, 112, 116, 121, 61, 10, 118, 97, 108,
            117, 101, 61, 116, 101, 115, 116, 0, // ConfigurationOption
            0, 5, 2, 0, 0, 1, 0, 2, // LoadBalancingOption
            0, 9, 4, 1, 2, 2, 2, 2, 3, 4, 0, 5, // Ipv4Endpoint
            0, 21, 6, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4, 0,
            5, // Ipv6Endpoint
            0, 9, 20, 1, 2, 2, 2, 2, 3, 4, 0, 5, //  Ipv4Multicast
            0, 21, 22, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4, 0,
            5, // Ipv6Multicast
            0, 9, 36, 1, 2, 2, 2, 2, 3, 4, 0, 5, // Ipv4SdEndpoint
            0, 21, 38, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 4, 0,
            5, // Ipv6SdEndpoint
        ];

        let strings = [
            ConfigurationString {
                key: "none",
                value: ConfigurationStringValue::None,
            },
            ConfigurationString {
                key: "empty",
                value: ConfigurationStringValue::Empty,
            },
            ConfigurationString {
                key: "value",
                value: ConfigurationStringValue::Value("test"),
            },
        ];

        let mut buffer = [0u8; 32];
        let configuration_strings =
            ConfigurationArray::create(strings.iter(), &mut buffer).unwrap();

        #[derive(Debug, Clone, PartialEq, Eq, Parse, Serialize)]
        struct Test<'a>(
            Option<'a>,
            Option<'a>,
            Option<'a>,
            Option<'a>,
            Option<'a>,
            Option<'a>,
            Option<'a>,
            Option<'a>,
        );

        let options = Test(
            Option::Configuration(ConfigurationOption {
                flag_reserved: ConfigurationOptionFlags::empty(),
                configuration_strings,
            }),
            Option::LoadBalancing(LoadBalancingOption {
                flag_reserved: LoadBalancingOptionFlags::empty(),
                priority: 1,
                weight: 2,
            }),
            Option::Ipv4Endpoint(IpV4Option {
                flag_reserved: 1,
                address: Ipv4Address { octets: [2; 4] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
            Option::Ipv6Endpoint(IpV6Option {
                flag_reserved: 1,
                address: Ipv6Address { octets: [2; 16] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
            Option::Ipv4Multicast(IpV4Option {
                flag_reserved: 1,
                address: Ipv4Address { octets: [2; 4] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
            Option::Ipv6Multicast(IpV6Option {
                flag_reserved: 1,
                address: Ipv6Address { octets: [2; 16] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
            Option::Ipv4SdEndpoint(IpV4Option {
                flag_reserved: 1,
                address: Ipv4Address { octets: [2; 4] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
            Option::Ipv6SdEndpoint(IpV6Option {
                flag_reserved: 1,
                address: Ipv6Address { octets: [2; 16] },
                reserved: 3,
                l4_proto: 4,
                port_number: 5,
            }),
        );

        test_round_trip!(Test, options, EXPECTED_DATA);
    }

    #[test]
    fn invalid_option() {
        const USED_VALUES: &[u8] = &[0x01, 0x02, 0x04, 0x06, 0x14, 0x16, 0x24, 0x26];

        for byte in 0x00..0xFF {
            if !USED_VALUES.contains(&byte) {
                assert!(matches!(
                    Option::parse(&[0x0, 0x0, byte]),
                    Err(ParseError::MalformedMessage { .. })
                ));
            }
        }
    }

    #[test]
    fn invalid_configuration_option() {
        const TEST_DATA: &[u8] = &[
            0, 25, 1, 0, 4, 0x7E, 111, 110, 101, 6, 101, 109, 112, 116, 121, 61, 10, 118, 97, 108,
            117, 101, 61, 116, 101, 115, 116, 0, // ConfigurationOption
        ];

        assert!(matches!(
            Option::parse(TEST_DATA),
            Err(ParseError::MalformedMessage { .. })
        ));
    }
}
