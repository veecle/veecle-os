#![expect(missing_docs, reason = "test")]
use veecle_osal_api::net::tcp::{Error, TcpSocket};
pub mod net_utils;

use embassy_net::Stack;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5);

#[test]
#[should_panic(expected = "success")]
fn tcp_new_invalid() {
    net_utils::embassy_test("127.0.0.1", |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let mut server = net_utils::tcp_socket(stack);
            let server_task = async {
                let _ = server.accept(ADDRESS).await.unwrap();
            };

            let client_task = async {
                let rx_buffer = Box::leak(Box::new([0u8; 4096]));
                let tx_buffer = Box::leak(Box::new([0u8; 4096]));

                let mut embassy_socket =
                    embassy_net::tcp::TcpSocket::new(stack, rx_buffer, tx_buffer);
                embassy_socket.connect(ADDRESS).await.unwrap();

                assert_eq!(
                    veecle_osal_embassy::net::tcp::TcpSocket::new(embassy_socket).unwrap_err(),
                    Error::InvalidState
                );
            };

            futures::join!(server_task, client_task);

            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
