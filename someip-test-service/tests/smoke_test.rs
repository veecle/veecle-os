//! Sanity check test suite.

use someip_test_service::{Config, TestService};

#[test]
#[ntest_timeout::timeout(240000)]
#[cfg(target_os = "linux")]
fn smoke_test() {
    let config = Config::default();
    let test_service = TestService::new(&config);

    let request = &[
        0x04, 0xD2, // Service ID: 1234 (0x04D2).
        0x01, 0xA7, // Method ID: 0x0423.
        0x00, 0x00, 0x00,
        0x09, // Length: 9 bytes (8 bytes header after length + 1 bytes payload).
        0x00, 0x01, 0x00, 0x01, // Request ID: client id (0x0001) and session id (0x0001).
        0x01, // Protocol Version: 1.
        0x00, // Interface Version: 1.
        0x00, // Message Type: 0 (Request).
        0x00, // Return Code: 0.
        0x12, // Single int8.
    ];
    let mut response = [0u8; 17];
    assert!(
        test_service
            .send_and_receive(request, &mut response)
            .is_ok(),
        "failed to send and receive message"
    );

    assert_eq!(
        response,
        [
            0x04, 0xD2, // Service ID: 1234 (0x04D2).
            0x01, 0xA7, // Method ID: 423 (0x01A7).
            0x00, 0x00, 0x00,
            0x09, // Length: 9 bytes (8 bytes header after length + 1 bytes payload).
            0x00, 0x01, 0x00, 0x01, // Request ID: Client ID (0x0001) and Session ID (0x0001).
            0x01, // Protocol Version: 1.
            0x00, // Interface Version: 0.
            0x80, // Message Type: 128 (Response).
            0x00, // Return Code: 0 (OK).
            0x12, // Payload: 12 (same as we sent).
        ]
    );
}

#[test]
#[cfg(not(target_os = "linux"))]
#[should_panic]
fn someip_test_service_should_fail_on_non_linux_platforms() {
    TestService::new(&Config::default());
}
