#![expect(missing_docs, reason = "Integration test")]
#![cfg(target_os = "linux")]

use someip_test_service::TestService;
use someip_test_service_macro::test_with_test_service;

#[test_with_test_service]
fn smoke_test(test_service: &TestService) {
    let request = &[
        0x04, 0xD2, // Service ID: 1234 (0x04D2).
        0x01, 0xA7, // Method ID: 0x0423.
        0x00, 0x00, 0x00,
        0x09, // Length: 9 bytes (8 bytes header after length + 1 byte payload).
        0x00, 0x01, 0x00, 0x01, // Request ID: client id (0x0001) and session id (0x0001).
        0x01, // Protocol Version: 1.
        0x00, // Interface Version: 1.
        0x00, // Message Type: 0 (Request).
        0x00, // Return Code: 0.
        0x12, // Single int8.
    ];

    let mut response = [0u8; 17];
    test_service
        .send_and_receive(request, &mut response)
        .unwrap();

    assert_eq!(
        response,
        [
            0x04, 0xD2, // Service ID: 1234 (0x04D2).
            0x01, 0xA7, // Method ID: 423 (0x01A7).
            0x00, 0x00, 0x00,
            0x09, // Length: 9 bytes (8 bytes header after length + 1 byte payload).
            0x00, 0x01, 0x00, 0x01, // Request ID: Client ID (0x0001) and Session ID (0x0001).
            0x01, // Protocol Version: 1.
            0x00, // Interface Version: 0.
            0x80, // Message Type: 128 (Response).
            0x00, // Return Code: 0 (OK).
            0x12, // Payload: 12 (same as we sent).
        ]
    );
}
