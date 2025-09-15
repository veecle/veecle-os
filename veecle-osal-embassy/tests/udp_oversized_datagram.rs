#![expect(missing_docs, reason = "test")]
// Embassy specific test.

pub mod net_utils;

use crate::net_utils::UDP_MAX_PACKET_SIZE;
use embassy_net::Stack;
use std::net::SocketAddr;
use veecle_osal_api::net::udp::{Error, UdpSocket};

#[test]
#[should_panic(expected = "success")]
fn udp_oversized_datagram() {
    const IP_ADDRESS: &str = "127.0.0.1";
    net_utils::embassy_test(IP_ADDRESS, |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let mut socket = net_utils::udp_socket(stack);
            let ip_address = IP_ADDRESS.parse().unwrap();
            let address = SocketAddr::new(ip_address, 58090);
            socket.bind(address).await.unwrap();
            let send_data = [0u8; UDP_MAX_PACKET_SIZE + 10];
            assert_eq!(
                socket.send_to(&send_data, address).await,
                Err(Error::BufferTooLarge)
            );

            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
