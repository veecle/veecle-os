#![expect(missing_docs, reason = "test")]
pub mod net_utils;

use embassy_net::Stack;

#[test]
#[should_panic(expected = "success")]
fn tcp_accept_all_zero_ip_v4() {
    net_utils::embassy_test("::1", |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let client = net_utils::tcp_socket(stack);
            let server = net_utils::tcp_socket(stack);

            veecle_osal_api::net::tcp::test_suite::test_accept_all_zero_ip(client, server, "::1")
                .await;
            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
