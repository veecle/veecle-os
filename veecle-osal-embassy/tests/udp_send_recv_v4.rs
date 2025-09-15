#![expect(missing_docs, reason = "test")]
pub mod net_utils;

use embassy_net::Stack;

#[test]
#[should_panic(expected = "success")]
fn udp_send_recv_v4() {
    const IP_ADDRESS: &str = "127.0.0.1";
    net_utils::embassy_test(IP_ADDRESS, |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let socket1 = net_utils::udp_socket(stack);
            let socket2 = net_utils::udp_socket(stack);
            veecle_osal_api::net::udp::test_suite::test_send_recv(socket1, socket2, IP_ADDRESS)
                .await;
            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
