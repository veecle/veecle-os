//! Roundtrip tests against external service.
//!
//! These tests verify that the SOME/IP correctly parses data types.
//! Each test sends a request with a specific data type to an external
//! test service and verifies that the response contains the same data.

#![cfg(target_os = "linux")]

use pretty_assertions::assert_eq;
use someip_test_service::TestService;
use someip_test_service_macro::test_with_test_service;
use veecle_os_data_support_someip::array::*;
use veecle_os_data_support_someip::header::*;
use veecle_os_data_support_someip::length::*;
use veecle_os_data_support_someip::parse::ParseExt;
use veecle_os_data_support_someip::string::*;

/// Makes a request to the method of the external test service,
/// asserts that the response has status "Ok" (by checking the response header),
/// and returns the full response bytes.
///
/// # Arguments
///
/// * `test_service` - The test service to send the request to.
/// * `method_id` - The method ID to call.
/// * `payload` - The payload data to send.
///
/// # Returns
///
/// The complete response bytes including header and payload.
fn make_request(test_service: &TestService, method_id: &MethodId, payload: &[u8]) -> Vec<u8> {
    // Calculate length: 8 bytes (remaining header size) + payload size.
    let length = Length::from(8 + payload.len() as u32);
    let request = create_raw_request(method_id, &length, payload);

    // Allocate response buffer: 16 bytes (full header) + payload size.
    let mut response = vec![u8::MIN; 16 + payload.len()];
    test_service
        .send_and_receive(request.as_slice(), &mut response)
        .expect("failed to send and/or receive SOME/IP message");

    let (header, _) =
        Header::parse_with_payload(&response).expect("failed to parse SOME/IP response");

    assert_eq!(
        header,
        Header::new(
            MessageId::new(ServiceId::from(0x04D2), *method_id),
            length,
            RequestId::new(
                ClientId::new(Prefix::from(0x0000), ClientIdInner::from(0x0001)),
                SessionId::from(0x01)
            ),
            ProtocolVersion::from(0x01),
            InterfaceVersion::from(0x00),
            MessageType::Response,
            ReturnCode::Ok,
        )
    );

    response
}

/// Creates a request that is ready to be sent to the test service via the network.
///
/// # Arguments
///
/// * `method_id` - The method ID to call.
/// * `length` - The length field of the header.
/// * `payload` - The payload data to send.
///
/// # Returns
///
/// The complete request bytes including header and payload.
fn create_raw_request(method_id: &MethodId, length: &Length, payload: &[u8]) -> Vec<u8> {
    // TODO: Replace with serializer.
    let mut header: Vec<u8> = vec![
        0x04, 0xD2, // Service ID: 1234 (0x04D2).
        0x00, 0x00, // Placeholder for method ID.
        0x00, 0x00, 0x00, 0x00, // Placeholder for length.
        0x00, 0x01, 0x00, 0x01, // Request ID: client id (0x0001) and session id (0x0001).
        0x01, // Protocol Version: 1.
        0x00, // Interface Version: 0.
        0x00, // Message Type: 0 (Request).
        0x00, // Return Code: 0.
    ];

    header[2..4].copy_from_slice(&u16::from(*method_id).to_be_bytes());
    header[4..8].copy_from_slice(&u32::from(*length).to_be_bytes());

    header.extend(payload);

    header
}

#[test_with_test_service]
fn bool(test_service: &TestService) {
    let method_id = MethodId::from(0x01A6);
    let input_data = vec![0x01]; // true.
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        bool::parse(payload.as_ref()).unwrap(),
        bool::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn i8(test_service: &TestService) {
    let method_id = MethodId::from(0x01A7);
    let input_data = vec![0x42];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        i8::parse(payload.as_ref()).unwrap(),
        i8::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn i16(test_service: &TestService) {
    let method_id = MethodId::from(0x01A8);
    let input_data = vec![0x11, 0x22];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        i16::parse(payload.as_ref()).unwrap(),
        i16::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn i32(test_service: &TestService) {
    let method_id = MethodId::from(0x01A9);
    let input_data = vec![0x11, 0x22, 0x33, 0x44];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        i32::parse(payload.as_ref()).unwrap(),
        i32::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn i64(test_service: &TestService) {
    let method_id = MethodId::from(0x01AA);
    let input_data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        i64::parse(payload.as_ref()).unwrap(),
        i64::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn u8(test_service: &TestService) {
    let method_id = MethodId::from(0x01AB);
    let input_data = vec![0x42];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        u8::parse(payload.as_ref()).unwrap(),
        u8::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn u16(test_service: &TestService) {
    let method_id = MethodId::from(0x01AC);
    let input_data = vec![0x11, 0x22];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        u16::parse(payload.as_ref()).unwrap(),
        u16::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn u32(test_service: &TestService) {
    let method_id = MethodId::from(0x01AD);
    let input_data = vec![0x11, 0x22, 0x33, 0x44];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        u32::parse(payload.as_ref()).unwrap(),
        u32::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn u64(test_service: &TestService) {
    let method_id = MethodId::from(0x01AE);
    let input_data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        u64::parse(payload.as_ref()).unwrap(),
        u64::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn f32(test_service: &TestService) {
    let method_id = MethodId::from(0x01B0);
    let input_data = vec![0x40, 0x48, 0xF5, 0xC3]; // 3.14.
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        f32::parse(payload.as_ref()).unwrap(),
        f32::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn f64(test_service: &TestService) {
    let method_id = MethodId::from(0x01AF);
    let input_data = vec![0x40, 0x09, 0x1E, 0xB8, 0x51, 0xEB, 0x85, 0x1F]; // 3.14.
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    assert_eq!(
        f64::parse(payload.as_ref()).unwrap(),
        f64::parse(&input_data).unwrap()
    );
}

#[test_with_test_service]
fn utf16le_dynamic(test_service: &TestService) {
    let method_id = MethodId::from(0x01B2);
    let input_data = vec![
        0x00, 0x00, 0x00, 0x0E, // Length.
        0xFF, 0xFE, // LE BOM.
        0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, // "Hello".
        0x00, 0x00, // Terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = DynamicLengthString::<'_, u32>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf16Le(encoded) = string.get_encoded() else {
        panic!("failed to decode dynamic length string as UTF-16LE");
    };
    assert_eq!(
        encoded.chars_lossy().collect::<String>(),
        String::from("Hello")
    );
}

#[test_with_test_service]
fn utf16be_dynamic(test_service: &TestService) {
    let method_id = MethodId::from(0x01B3);
    let input_data = vec![
        0x0, 0x0, 0x0, 0xE, // Length.
        0xFE, 0xFF, // BE BOM.
        0x0, 0x48, 0x0, 0x65, 0x0, 0x6C, 0x0, 0x6C, 0x0, 0x6F, // "Hello".
        0x0, 0x0, // Terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = DynamicLengthString::<'_, u32>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf16Be(encoded) = string.get_encoded() else {
        panic!("failed to decode dynamic length string as UTF-16BE");
    };
    assert_eq!(
        encoded.chars_lossy().collect::<String>(),
        String::from("Hello")
    );
}

#[test_with_test_service]
fn utf8_dynamic(test_service: &TestService) {
    let method_id = MethodId::from(0x01B4);
    let input_data = vec![
        0x0, 0x0, 0x0, 0x9, // Length.
        0xEF, 0xBB, 0xBF, // BOM.
        b'H', b'e', b'l', b'l', b'o', 0x0, // With terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = DynamicLengthString::<'_, u32>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf8(encoded) = string.get_encoded() else {
        panic!("failed to decode dynamic length string as UTF-8");
    };
    assert_eq!(encoded, &"Hello");
}

#[test_with_test_service]
fn utf16le_fixed(test_service: &TestService) {
    let method_id = MethodId::from(0x01B5);
    let input_data = vec![
        0xFF, 0xFE, // LE BOM.
        0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, // "Hello".
        0x00, 0x00, // Terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = FixedLengthString::<'_, 14>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf16Le(encoded) = string.get_encoded() else {
        panic!("failed to decode fixed length string as UTF-16LE");
    };
    assert_eq!(
        encoded.chars_lossy().collect::<String>(),
        String::from("Hello")
    );
}

#[test_with_test_service]
fn utf16be_fixed(test_service: &TestService) {
    let method_id = MethodId::from(0x01B6);
    let input_data = vec![
        0xFE, 0xFF, // BE BOM.
        0x00, 0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, // "Hello".
        0x00, 0x00, // Terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = FixedLengthString::<'_, 14>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf16Be(encoded) = string.get_encoded() else {
        panic!("failed to decode fixed length string as UTF-16BE");
    };
    assert_eq!(
        encoded.chars_lossy().collect::<String>(),
        String::from("Hello")
    );
}

#[test_with_test_service]
fn utf8_fixed(test_service: &TestService) {
    let method_id = MethodId::from(0x01B7);
    let input_data = vec![
        0xEF, 0xBB, 0xBF, // BOM.
        b'H', b'e', b'l', b'l', b'o', 0x0, // With terminator.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let string = FixedLengthString::<'_, 9>::parse(payload.as_ref()).unwrap();
    let EncodedString::Utf8(encoded) = string.get_encoded() else {
        panic!("failed to decode fixed length string as UTF-8");
    };
    assert_eq!(encoded, &"Hello");
}

#[test_with_test_service]
fn array_fixed_length(test_service: &TestService) {
    let method_id = MethodId::from(0x1B9);
    let input_data = vec![
        0, 0, 0, 0, // Item 0.
        1, 1, 1, 1, // Item 1.
        2, 2, 2, 2, // Item 2.
        3, 3, 3, 3, // Item 3.
        4, 4, 4, 4, // Item 4.
        5, 5, 5, 5, // Item 5.
        6, 6, 6, 6, // Item 6.
        7, 7, 7, 7, // Item 7.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let parsed_array =
        FixedLengthArray::<'_, u32, NoLengthField, 8>::parse(payload.as_ref()).unwrap();
    assert_eq!(
        parsed_array,
        FixedLengthArray::<'_, u32, NoLengthField, 8>::parse(input_data.as_ref()).unwrap()
    );
}

#[test_with_test_service]
fn array_dynamic_length_1_byte(test_service: &TestService) {
    let method_id = MethodId::from(0x1BA);
    let input_data = vec![
        8, // Length (in bytes).
        1, 1, 1, 1, // Item 0.
        2, 2, 2, 2, // Item 1.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let parsed_array = DynamicLengthArray::<'_, u32, u8, 2>::parse(payload.as_ref()).unwrap();
    assert_eq!(
        parsed_array,
        DynamicLengthArray::<'_, u32, u8, 2>::parse(input_data.as_ref()).unwrap()
    );
}

#[test_with_test_service]
fn array_dynamic_length_2_bytes(test_service: &TestService) {
    let method_id = MethodId::from(0x1BB);
    let input_data = vec![
        0, 8, // Length (in bytes).
        1, 1, 1, 1, // Item 0.
        2, 2, 2, 2, // Item 1.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let parsed_array = DynamicLengthArray::<'_, u32, u16, 2>::parse(payload.as_ref()).unwrap();
    assert_eq!(
        parsed_array,
        DynamicLengthArray::<'_, u32, u16, 2>::parse(input_data.as_ref()).unwrap()
    );
}

#[test_with_test_service]
fn array_dynamic_length_4_bytes(test_service: &TestService) {
    let method_id = MethodId::from(0x1BC);
    let input_data = vec![
        0, 0, 0, 8, // Length (in bytes).
        1, 1, 1, 1, // Item 0.
        2, 2, 2, 2, // Item 1.
    ];
    let response = make_request(test_service, &method_id, &input_data);
    let (_, payload) = Header::parse_with_payload(&response).unwrap();
    let parsed_array = DynamicLengthArray::<'_, u32, u32, 2>::parse(payload.as_ref()).unwrap();
    assert_eq!(
        parsed_array,
        DynamicLengthArray::<'_, u32, u32, 2>::parse(input_data.as_ref()).unwrap()
    );
}
