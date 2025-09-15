#![expect(missing_docs, reason = "test")]
// Embassy specific test.

pub mod net_utils;

use embassy_net::Stack;
use embassy_net::udp::PacketMetadata;
use std::net::IpAddr;
use veecle_osal_api::net::udp::Error;

// Checks whether invalid input to `UdpSocket::new` correctly returns an error.
#[test]
#[should_panic(expected = "success")]
fn udp_new_invalid() {
    const IP_ADDRESS: &str = "127.0.0.1";
    net_utils::embassy_test(IP_ADDRESS, |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let rx_meta_buffer = Box::leak(Box::new([PacketMetadata::EMPTY; 1024]));
            let rx_buffer = Box::leak(Box::new([0u8; 1024]));
            let tx_meta_buffer = Box::leak(Box::new([PacketMetadata::EMPTY; 1024]));
            let tx_buffer = Box::leak(Box::new([0u8; 1024]));
            let mut embassy_socket = embassy_net::udp::UdpSocket::new(
                stack,
                rx_meta_buffer,
                rx_buffer,
                tx_meta_buffer,
                tx_buffer,
            );
            embassy_socket
                .bind((IP_ADDRESS.parse::<IpAddr>().unwrap(), 10))
                .unwrap();
            assert_eq!(
                veecle_osal_embassy::net::udp::UdpSocket::new(embassy_socket).unwrap_err(),
                Error::InvalidState
            );
            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
