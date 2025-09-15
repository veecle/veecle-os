#![expect(missing_docs)]

use pretty_assertions::assert_eq;
use veecle_os_data_support_someip::header::*;
use veecle_os_data_support_someip::parse::ParseExt;
use veecle_os_data_support_someip::service_discovery::{
    self, Entry, HeaderFlags, IpV4Option, Ipv4Address, Option, ServiceEntry,
};

/// Test that SOME/IP header can be deserialized
/// from bytes into the struct and serialized back into bytes.
/// See EV-55 for details.
#[test]
fn message_header() {
    use veecle_os_data_support_someip::parse::ParseExt;

    // Raw SOME/IP message.
    const BYTES: &[u8] = &[5, 102, 128, 2, 0, 0, 0, 12, 0, 0, 104, 55, 1, 0, 2, 0];

    let header = Header::parse(BYTES).expect("failed to parse SOME/IP message");
    assert_eq!(
        header,
        Header::new(
            MessageId::new(ServiceId::from(0x0566), MethodId::from(0x8002)),
            Length::from(12),
            RequestId::new(
                ClientId::new(Prefix::from(0x0000), ClientIdInner::from(0x0000)),
                SessionId::from(0x6837)
            ),
            ProtocolVersion::from(0x01),
            InterfaceVersion::from(0x00),
            MessageType::Notification,
            ReturnCode::Ok,
        )
    );
}

/// Test that SOME/IP service discovery header can be deserialized
/// from bytes into the struct and serialized back into bytes.
/// See EV-55 for details.
#[test]
fn service_discovery_header() {
    // Raw SOME/IP service discovery message.
    // First 16 bytes is a header, rest is a service discovery header.
    const BYTES: &[u8] = &[
        255, 128, 129, 0, 0, 0, 0, 76, 0, 0, 20, 147, 1, 1, 2, 0, 64, 0, 0, 0, 0, 0, 0, 32, 1, 0,
        0, 16, 3, 232, 0, 10, 1, 0, 0, 128, 0, 0, 0, 0, 1, 1, 0, 16, 3, 235, 0, 10, 1, 0, 0, 128,
        0, 0, 0, 0, 0, 0, 0, 24, 0, 9, 4, 0, 192, 0, 2, 0, 0, 17, 0, 24, 0, 9, 4, 0, 192, 0, 2, 0,
        0, 17, 0, 26,
    ];

    let (header, payload) =
        Header::parse_with_payload(BYTES).expect("failed to parse SOME/IP message");

    assert_eq!(
        header,
        Header::new(
            MessageId::new(ServiceId::from(0xFF80), MethodId::from(0x8100)),
            Length::from(76),
            RequestId::new(
                ClientId::new(Prefix::from(0x0000), ClientIdInner::from(0x0000)),
                SessionId::from(0x1493)
            ),
            ProtocolVersion::from(0x01),
            InterfaceVersion::from(0x01),
            MessageType::Notification,
            ReturnCode::Ok,
        )
    );

    let service_discovery_header = service_discovery::Header::parse(payload.as_ref())
        .expect("failed to deserialize SOME/IP service discovery header");

    assert_eq!(service_discovery_header.flags, HeaderFlags::empty());

    let expected_entries = [
        Entry::OfferService(ServiceEntry {
            first_option: 0x00,
            second_option: 0x00,
            option_counts: 16,
            service_id: 0x03E8,
            instance_id: 0x000A,
            major_version_ttl: 0x1000080,
            minor_version: 0,
        }),
        Entry::OfferService(ServiceEntry {
            first_option: 0x01,
            second_option: 0x00,
            option_counts: 16,
            service_id: 0x03EB,
            instance_id: 0x000A,
            major_version_ttl: 0x1000080,
            minor_version: 0,
        }),
    ];

    let mut entry_iter = service_discovery_header.entries.iter();
    expected_entries
        .into_iter()
        .for_each(|entry| assert_eq!(entry, entry_iter.next().unwrap()));
    assert_eq!(entry_iter.next(), None);

    let expected_options = [
        Option::Ipv4Endpoint(IpV4Option {
            flag_reserved: 00,
            address: Ipv4Address {
                octets: [192, 0, 2, 0],
            },
            reserved: 00,
            l4_proto: 17,
            port_number: 24,
        }),
        Option::Ipv4Endpoint(IpV4Option {
            flag_reserved: 00,
            address: Ipv4Address {
                octets: [192, 0, 2, 0],
            },
            reserved: 00,
            l4_proto: 17,
            port_number: 26,
        }),
    ];

    let mut option_iter = service_discovery_header.options.iter();
    expected_options
        .into_iter()
        .for_each(|option| assert_eq!(option, option_iter.next().unwrap()));
    assert_eq!(option_iter.next(), None);
}
