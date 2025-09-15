#![expect(missing_docs, reason = "test")]
pub mod net_utils;

use embassy_net::Stack;

#[test]
#[should_panic(expected = "success")]
fn udp_bind_specific_port() {
    const IP_ADDRESS: &str = "127.0.0.1";
    net_utils::embassy_test(IP_ADDRESS, |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let socket = net_utils::udp_socket(stack);
            veecle_osal_api::net::udp::test_suite::test_bind_specific_port(socket, IP_ADDRESS)
                .await;
            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
