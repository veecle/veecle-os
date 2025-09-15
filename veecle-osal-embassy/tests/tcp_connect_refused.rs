#![expect(missing_docs, reason = "test")]
pub mod net_utils;

use embassy_net::Stack;

#[test]
#[should_panic(expected = "success")]
fn tcp_connect_refused() {
    net_utils::embassy_test("127.0.0.1", |stack, spawner| {
        #[embassy_executor::task]
        async fn my_test(stack: Stack<'static>) {
            let client = net_utils::tcp_socket(stack);

            veecle_osal_api::net::tcp::test_suite::test_connect_refused(client, "127.0.0.1").await;
            panic!("success");
        }

        spawner.spawn(my_test(stack)).unwrap();
    })
}
